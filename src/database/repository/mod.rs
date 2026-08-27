mod adif;
mod backup;
mod queries;
#[cfg(test)]
mod stress;

use adif::reconcile_adif_extra_fields;
use std::collections::BTreeMap;
use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension, Result, Transaction};

use crate::adif::ImportedQso;
use crate::domain::{
    DStarMetadata, DmrAccessType, DmrCallType, DmrMetadata, Ft8Metadata, ModeMetadata, NewQso, Qso,
    YsfAccessType, YsfMetadata,
};

use super::migrations;
use backup::{set_private_file_permissions, verify_connection_integrity};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DmrFilter {
    pub dmr_id: Option<u32>,
    pub talkgroup: Option<u32>,
    pub network: Option<String>,
    pub repeater: Option<String>,
    pub hotspot: Option<String>,
    pub timeslot: Option<u8>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DstarFilter {
    pub reflector: Option<String>,
    pub module: Option<String>,
    pub rpt1: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Ft8Filter {
    pub callsign: Option<String>,
    pub grid: Option<String>,
    pub band: Option<String>,
    pub minimum_snr_received_db: Option<i16>,
    pub maximum_snr_received_db: Option<i16>,
    pub start_utc: Option<i64>,
    pub end_utc: Option<i64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct YsfFilter {
    pub room: Option<String>,
    pub wires_x_node: Option<String>,
    pub dg_id: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QsoSelection {
    All,
    General(String),
    Dmr(DmrFilter),
    Ft8(Ft8Filter),
    Dstar(DstarFilter),
    Ysf(YsfFilter),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AdifImportReport {
    pub imported: usize,
    pub duplicates_skipped: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AdifImportPreview {
    pub total: usize,
    pub new_qsos: usize,
    pub duplicates: usize,
    pub invalid: usize,
    pub modes: BTreeMap<String, usize>,
    pub bands: BTreeMap<String, usize>,
    pub earliest_utc: Option<i64>,
    pub latest_utc: Option<i64>,
    pub invalid_details: Vec<String>,
}

pub struct AdifImportPlan {
    preview: AdifImportPreview,
    qsos: Vec<ImportedQso>,
}

impl AdifImportPlan {
    pub fn preview(&self) -> &AdifImportPreview {
        &self.preview
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct QsoIdentity {
    callsign: String,
    datetime_start_utc: i64,
    frequency_hz: i64,
    mode: String,
}

impl From<&NewQso> for QsoIdentity {
    fn from(qso: &NewQso) -> Self {
        Self {
            callsign: qso.callsign.clone(),
            datetime_start_utc: qso.datetime_start_utc,
            frequency_hz: qso.frequency_hz,
            mode: qso.mode.clone(),
        }
    }
}

pub const DEFAULT_PAGE_SIZE: usize = 100;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QsoListItem {
    pub qso: Qso,
    pub metadata: ModeMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QsoPage {
    pub items: Vec<QsoListItem>,
    pub total: usize,
    pub offset: usize,
    pub limit: usize,
}

pub struct QsoRepository {
    connection: Connection,
}

impl QsoRepository {
    pub fn open(path: &Path) -> Result<Self> {
        let mut connection = Connection::open(path)?;
        connection.execute_batch("PRAGMA foreign_keys = ON;")?;
        migrations::run(&mut connection)?;
        verify_connection_integrity(&connection)?;
        set_private_file_permissions(path)?;
        Ok(Self { connection })
    }

    #[cfg(test)]
    fn in_memory() -> Result<Self> {
        let mut connection = Connection::open_in_memory()?;
        connection.execute_batch("PRAGMA foreign_keys = ON;")?;
        migrations::run(&mut connection)?;
        Ok(Self { connection })
    }

    pub fn verify_integrity(&self) -> Result<()> {
        verify_connection_integrity(&self.connection)
    }

    pub fn health(&self) -> super::health::HealthReport {
        super::health::inspect_connection(&self.connection)
    }

    pub fn find_qso_identity_match(
        &self,
        qso: &NewQso,
        excluding_id: Option<i64>,
    ) -> Result<Option<i64>> {
        match excluding_id {
            None => self
                .connection
                .query_row(
                    "SELECT id FROM qsos
                     WHERE callsign = ?1 COLLATE NOCASE
                       AND datetime_start_utc = ?2
                       AND frequency_hz = ?3
                       AND mode = ?4 COLLATE NOCASE
                     LIMIT 1",
                    params![
                        qso.callsign,
                        qso.datetime_start_utc,
                        qso.frequency_hz,
                        qso.mode
                    ],
                    |row| row.get(0),
                )
                .optional(),
            Some(excluding_id) => self
                .connection
                .query_row(
                    "SELECT id FROM qsos
                     WHERE callsign = ?1 COLLATE NOCASE
                       AND datetime_start_utc = ?2
                       AND frequency_hz = ?3
                       AND mode = ?4 COLLATE NOCASE
                       AND id <> ?5
                     LIMIT 1",
                    params![
                        qso.callsign,
                        qso.datetime_start_utc,
                        qso.frequency_hz,
                        qso.mode,
                        excluding_id
                    ],
                    |row| row.get(0),
                )
                .optional(),
        }
    }

    pub fn insert(&self, qso: &NewQso, now_utc: i64) -> Result<i64> {
        insert_qso(&self.connection, qso, now_utc)
    }

    pub fn insert_dmr(&self, qso: &NewQso, metadata: &DmrMetadata, now_utc: i64) -> Result<i64> {
        require_mode(qso, "DMR")?;
        let transaction = self.connection.unchecked_transaction()?;
        let qso_id = insert_qso(&transaction, qso, now_utc)?;
        insert_dmr_metadata(&transaction, qso_id, metadata)?;
        transaction.commit()?;
        Ok(qso_id)
    }

    pub fn insert_ft8(&self, qso: &NewQso, metadata: &Ft8Metadata, now_utc: i64) -> Result<i64> {
        require_mode(qso, "FT8")?;
        let transaction = self.connection.unchecked_transaction()?;
        let qso_id = insert_qso(&transaction, qso, now_utc)?;
        insert_ft8_metadata(&transaction, qso_id, metadata)?;
        transaction.commit()?;
        Ok(qso_id)
    }

    pub fn insert_dstar(
        &self,
        qso: &NewQso,
        metadata: &DStarMetadata,
        now_utc: i64,
    ) -> Result<i64> {
        require_mode(qso, "DSTAR")?;
        let transaction = self.connection.unchecked_transaction()?;
        let qso_id = insert_qso(&transaction, qso, now_utc)?;
        insert_dstar_metadata(&transaction, qso_id, metadata)?;
        transaction.commit()?;
        Ok(qso_id)
    }

    pub fn insert_ysf(&self, qso: &NewQso, metadata: &YsfMetadata, now_utc: i64) -> Result<i64> {
        require_mode(qso, "C4FM")?;
        let transaction = self.connection.unchecked_transaction()?;
        let qso_id = insert_qso(&transaction, qso, now_utc)?;
        insert_ysf_metadata(&transaction, qso_id, metadata)?;
        transaction.commit()?;
        Ok(qso_id)
    }

    pub fn update_ysf(
        &self,
        qso_id: i64,
        qso: &NewQso,
        metadata: &YsfMetadata,
        now_utc: i64,
    ) -> Result<bool> {
        require_mode(qso, "C4FM")?;
        let transaction = self.connection.unchecked_transaction()?;
        let changed = update_qso(&transaction, qso_id, qso, now_utc)?;
        if !changed {
            transaction.rollback()?;
            return Ok(false);
        }
        reconcile_adif_extra_fields(&transaction, qso_id, &qso.mode)?;
        delete_mode_metadata(&transaction, qso_id)?;
        insert_ysf_metadata(&transaction, qso_id, metadata)?;
        transaction.commit()?;
        Ok(true)
    }

    pub fn get_ysf_metadata(&self, qso_id: i64) -> Result<Option<YsfMetadata>> {
        self.connection
            .query_row(
                "SELECT room, wires_x_node, repeater, network, access_type,
                        tx_dg_id, rx_dg_id, notes
                 FROM ysf_metadata WHERE qso_id = ?1",
                params![qso_id],
                map_ysf_metadata,
            )
            .optional()
    }

    pub fn update_dstar(
        &self,
        qso_id: i64,
        qso: &NewQso,
        metadata: &DStarMetadata,
        now_utc: i64,
    ) -> Result<bool> {
        require_mode(qso, "DSTAR")?;
        let transaction = self.connection.unchecked_transaction()?;
        let changed = update_qso(&transaction, qso_id, qso, now_utc)?;
        if !changed {
            transaction.rollback()?;
            return Ok(false);
        }
        reconcile_adif_extra_fields(&transaction, qso_id, &qso.mode)?;
        delete_mode_metadata(&transaction, qso_id)?;
        insert_dstar_metadata(&transaction, qso_id, metadata)?;
        transaction.commit()?;
        Ok(true)
    }

    pub fn get_dstar_metadata(&self, qso_id: i64) -> Result<Option<DStarMetadata>> {
        self.connection
            .query_row(
                "SELECT reflector, module, mycall, urcall, rpt1, rpt2, notes
                 FROM dstar_metadata WHERE qso_id = ?1",
                params![qso_id],
                map_dstar_metadata,
            )
            .optional()
    }

    pub fn update_ft8(
        &self,
        qso_id: i64,
        qso: &NewQso,
        metadata: &Ft8Metadata,
        now_utc: i64,
    ) -> Result<bool> {
        require_mode(qso, "FT8")?;
        let transaction = self.connection.unchecked_transaction()?;
        let changed = update_qso(&transaction, qso_id, qso, now_utc)?;
        if !changed {
            transaction.rollback()?;
            return Ok(false);
        }
        reconcile_adif_extra_fields(&transaction, qso_id, &qso.mode)?;
        delete_mode_metadata(&transaction, qso_id)?;
        insert_ft8_metadata(&transaction, qso_id, metadata)?;
        transaction.commit()?;
        Ok(true)
    }

    pub fn get_ft8_metadata(&self, qso_id: i64) -> Result<Option<Ft8Metadata>> {
        self.connection
            .query_row(
                "SELECT snr_sent_db, snr_received_db, power_watts,
                        audio_frequency_hz, source_software, protocol, final_message
                 FROM ft8_metadata WHERE qso_id = ?1",
                params![qso_id],
                map_ft8_metadata,
            )
            .optional()
    }

    pub fn update_dmr(
        &self,
        qso_id: i64,
        qso: &NewQso,
        metadata: &DmrMetadata,
        now_utc: i64,
    ) -> Result<bool> {
        require_mode(qso, "DMR")?;
        let transaction = self.connection.unchecked_transaction()?;
        let changed = update_qso(&transaction, qso_id, qso, now_utc)?;
        if !changed {
            transaction.rollback()?;
            return Ok(false);
        }
        reconcile_adif_extra_fields(&transaction, qso_id, &qso.mode)?;
        delete_mode_metadata(&transaction, qso_id)?;
        insert_dmr_metadata(&transaction, qso_id, metadata)?;
        transaction.commit()?;
        Ok(true)
    }

    pub fn get_dmr_metadata(&self, qso_id: i64) -> Result<Option<DmrMetadata>> {
        self.connection
            .query_row(
                "SELECT d.remote_dmr_id, d.local_dmr_id, d.talkgroup,
                        d.timeslot, d.color_code, r.network, d.call_type,
                        r.access_type, r.repeater_callsign, r.hotspot,
                        d.rx_frequency_hz, d.tx_frequency_hz, d.notes
                 FROM dmr_metadata d
                 JOIN digital_routes r ON r.qso_id = d.qso_id
                 WHERE d.qso_id = ?1",
                params![qso_id],
                map_dmr_metadata,
            )
            .optional()
    }

    pub fn update(&self, id: i64, qso: &NewQso, now_utc: i64) -> Result<bool> {
        let transaction = self.connection.unchecked_transaction()?;
        let changed = update_qso(&transaction, id, qso, now_utc)?;
        if !changed {
            transaction.rollback()?;
            return Ok(false);
        }
        reconcile_adif_extra_fields(&transaction, id, &qso.mode)?;
        delete_mode_metadata(&transaction, id)?;
        transaction.commit()?;
        Ok(true)
    }

    pub fn delete(&self, id: i64) -> Result<bool> {
        let changed = self
            .connection
            .execute("DELETE FROM qsos WHERE id = ?1", params![id])?;
        Ok(changed > 0)
    }
}

fn require_mode(qso: &NewQso, expected: &str) -> Result<()> {
    if qso.mode == expected {
        Ok(())
    } else {
        Err(rusqlite::Error::InvalidParameterName(format!(
            "specialized {expected} metadata is incompatible with QSO mode {}",
            qso.mode
        )))
    }
}

fn update_qso(connection: &Connection, id: i64, qso: &NewQso, now_utc: i64) -> Result<bool> {
    let changed = connection.execute(
        "UPDATE qsos
         SET callsign = ?1, datetime_start_utc = ?2, frequency_hz = ?3,
             band = ?4, mode = ?5, rst_sent = ?6, rst_received = ?7,
             grid_locator = ?8, name = ?9, qth = ?10, notes = ?11,
             updated_at_utc = ?12
         WHERE id = ?13",
        params![
            qso.callsign,
            qso.datetime_start_utc,
            qso.frequency_hz,
            qso.band,
            qso.mode,
            qso.rst_sent,
            qso.rst_received,
            qso.grid_locator,
            qso.name,
            qso.qth,
            qso.notes,
            now_utc,
            id
        ],
    )?;
    Ok(changed > 0)
}

fn insert_qso(connection: &Connection, qso: &NewQso, now_utc: i64) -> Result<i64> {
    connection.execute(
        "INSERT INTO qsos (
            callsign, datetime_start_utc, frequency_hz, band, mode,
            rst_sent, rst_received, grid_locator, name, qth, notes,
            created_at_utc, updated_at_utc
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?12)",
        params![
            qso.callsign,
            qso.datetime_start_utc,
            qso.frequency_hz,
            qso.band,
            qso.mode,
            qso.rst_sent,
            qso.rst_received,
            qso.grid_locator,
            qso.name,
            qso.qth,
            qso.notes,
            now_utc
        ],
    )?;
    Ok(connection.last_insert_rowid())
}

fn insert_ysf_metadata(
    transaction: &Transaction<'_>,
    qso_id: i64,
    metadata: &YsfMetadata,
) -> Result<()> {
    transaction.execute(
        "INSERT INTO ysf_metadata (
            qso_id, room, wires_x_node, repeater, network, access_type,
            tx_dg_id, rx_dg_id, notes
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            qso_id,
            metadata.room,
            metadata.wires_x_node,
            metadata.repeater,
            metadata.network,
            metadata.access_type.as_str(),
            metadata.tx_dg_id.map(i64::from),
            metadata.rx_dg_id.map(i64::from),
            metadata.notes
        ],
    )?;
    Ok(())
}

fn map_ysf_metadata(row: &rusqlite::Row<'_>) -> Result<YsfMetadata> {
    let access_type: String = row.get(4)?;
    Ok(YsfMetadata {
        room: row.get(0)?,
        wires_x_node: row.get(1)?,
        repeater: row.get(2)?,
        network: row.get(3)?,
        access_type: parse_stored_ysf_access_type(&access_type)?,
        tx_dg_id: row.get(5)?,
        rx_dg_id: row.get(6)?,
        notes: row.get(7)?,
    })
}

fn parse_stored_ysf_access_type(value: &str) -> Result<YsfAccessType> {
    value.parse().map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(error))
    })
}

fn insert_dstar_metadata(
    transaction: &Transaction<'_>,
    qso_id: i64,
    metadata: &DStarMetadata,
) -> Result<()> {
    transaction.execute(
        "INSERT INTO dstar_metadata (
            qso_id, reflector, module, mycall, urcall, rpt1, rpt2, notes
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            qso_id,
            metadata.reflector,
            metadata.module,
            metadata.mycall,
            metadata.urcall,
            metadata.rpt1,
            metadata.rpt2,
            metadata.notes
        ],
    )?;
    Ok(())
}

fn map_dstar_metadata(row: &rusqlite::Row<'_>) -> Result<DStarMetadata> {
    Ok(DStarMetadata {
        reflector: row.get(0)?,
        module: row.get(1)?,
        mycall: row.get(2)?,
        urcall: row.get(3)?,
        rpt1: row.get(4)?,
        rpt2: row.get(5)?,
        notes: row.get(6)?,
    })
}

fn insert_ft8_metadata(
    transaction: &Transaction<'_>,
    qso_id: i64,
    metadata: &Ft8Metadata,
) -> Result<()> {
    transaction.execute(
        "INSERT INTO ft8_metadata (
            qso_id, snr_sent_db, snr_received_db, power_watts,
            audio_frequency_hz, source_software, protocol, final_message
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            qso_id,
            metadata.snr_sent_db,
            metadata.snr_received_db,
            metadata.power_watts.map(i64::from),
            metadata.audio_frequency_hz.map(i64::from),
            metadata.source_software,
            metadata.protocol,
            metadata.final_message
        ],
    )?;
    Ok(())
}

fn map_ft8_metadata(row: &rusqlite::Row<'_>) -> Result<Ft8Metadata> {
    Ok(Ft8Metadata {
        snr_sent_db: row.get(0)?,
        snr_received_db: row.get(1)?,
        power_watts: row.get(2)?,
        audio_frequency_hz: row.get(3)?,
        source_software: row.get(4)?,
        protocol: row.get(5)?,
        final_message: row.get(6)?,
    })
}

fn delete_mode_metadata(transaction: &Transaction<'_>, qso_id: i64) -> Result<()> {
    transaction.execute(
        "DELETE FROM dmr_metadata WHERE qso_id = ?1",
        params![qso_id],
    )?;
    transaction.execute(
        "DELETE FROM digital_routes WHERE qso_id = ?1",
        params![qso_id],
    )?;
    transaction.execute(
        "DELETE FROM ft8_metadata WHERE qso_id = ?1",
        params![qso_id],
    )?;
    transaction.execute(
        "DELETE FROM dstar_metadata WHERE qso_id = ?1",
        params![qso_id],
    )?;
    transaction.execute(
        "DELETE FROM ysf_metadata WHERE qso_id = ?1",
        params![qso_id],
    )?;
    Ok(())
}

fn insert_dmr_metadata(
    transaction: &Transaction<'_>,
    qso_id: i64,
    metadata: &DmrMetadata,
) -> Result<()> {
    transaction.execute(
        "INSERT INTO digital_routes (
            qso_id, access_type, network, repeater_callsign, hotspot
         ) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            qso_id,
            metadata.access_type.as_str(),
            metadata.network,
            metadata.repeater_callsign,
            metadata.hotspot
        ],
    )?;
    transaction.execute(
        "INSERT INTO dmr_metadata (
            qso_id, remote_dmr_id, local_dmr_id, talkgroup, timeslot,
            color_code, call_type, rx_frequency_hz, tx_frequency_hz, notes
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            qso_id,
            metadata.remote_dmr_id.map(i64::from),
            metadata.local_dmr_id.map(i64::from),
            metadata.talkgroup.map(i64::from),
            metadata.timeslot.map(i64::from),
            metadata.color_code.map(i64::from),
            metadata.call_type.as_str(),
            metadata.rx_frequency_hz,
            metadata.tx_frequency_hz,
            metadata.notes
        ],
    )?;
    Ok(())
}

fn map_dmr_metadata(row: &rusqlite::Row<'_>) -> Result<DmrMetadata> {
    let call_type: String = row.get(6)?;
    let access_type: String = row.get(7)?;
    Ok(DmrMetadata {
        remote_dmr_id: row.get(0)?,
        local_dmr_id: row.get(1)?,
        talkgroup: row.get(2)?,
        timeslot: row.get(3)?,
        color_code: row.get(4)?,
        network: row.get(5)?,
        call_type: parse_stored_call_type(&call_type)?,
        access_type: parse_stored_access_type(&access_type)?,
        repeater_callsign: row.get(8)?,
        hotspot: row.get(9)?,
        rx_frequency_hz: row.get(10)?,
        tx_frequency_hz: row.get(11)?,
        notes: row.get(12)?,
    })
}

fn parse_stored_call_type(value: &str) -> Result<DmrCallType> {
    value.parse().map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(6, rusqlite::types::Type::Text, Box::new(error))
    })
}

fn parse_stored_access_type(value: &str) -> Result<DmrAccessType> {
    value.parse().map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(7, rusqlite::types::Type::Text, Box::new(error))
    })
}
fn map_qso(row: &rusqlite::Row<'_>) -> Result<Qso> {
    Ok(Qso {
        id: row.get(0)?,
        callsign: row.get(1)?,
        datetime_start_utc: row.get(2)?,
        datetime_end_utc: row.get(3)?,
        frequency_hz: row.get(4)?,
        band: row.get(5)?,
        mode: row.get(6)?,
        submode: row.get(7)?,
        rst_sent: row.get(8)?,
        rst_received: row.get(9)?,
        grid_locator: row.get(10)?,
        name: row.get(11)?,
        qth: row.get(12)?,
        notes: row.get(13)?,
        created_at_utc: row.get(14)?,
        updated_at_utc: row.get(15)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adif::{export, parse, AdifField};
    use crate::domain::{
        CommonQsoFields, DStarMetadataInput, DmrMetadataInput, Ft8MetadataInput, YsfMetadataInput,
    };

    fn assert_metadata_invariant(repository: &QsoRepository, qso_id: i64, expected_mode: &str) {
        let (mode, dmr, ft8, dstar, ysf): (String, i64, i64, i64, i64) = repository
            .connection
            .query_row(
                "SELECT q.mode,
                        (SELECT COUNT(*) FROM dmr_metadata WHERE qso_id = q.id),
                        (SELECT COUNT(*) FROM ft8_metadata WHERE qso_id = q.id),
                        (SELECT COUNT(*) FROM dstar_metadata WHERE qso_id = q.id),
                        (SELECT COUNT(*) FROM ysf_metadata WHERE qso_id = q.id)
                 FROM qsos q WHERE q.id = ?1",
                [qso_id],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                },
            )
            .unwrap();
        assert_eq!(mode, expected_mode);
        let expected = match expected_mode {
            "DMR" => (1, 0, 0, 0),
            "FT8" => (0, 1, 0, 0),
            "DSTAR" => (0, 0, 1, 0),
            "C4FM" => (0, 0, 0, 1),
            _ => (0, 0, 0, 0),
        };
        assert_eq!((dmr, ft8, dstar, ysf), expected);
    }

    fn identity_qso() -> NewQso {
        NewQso::new("PY2ABC", 1_700_000_000, 145_500_000, "FM").unwrap()
    }

    #[test]
    fn finds_exact_identity_case_insensitively() {
        let repository = QsoRepository::in_memory().unwrap();
        let qso = identity_qso();
        let id = repository.insert(&qso, 1).unwrap();
        let differently_cased = NewQso::new("py2abc", 1_700_000_000, 145_500_000, "fm").unwrap();

        assert_eq!(
            repository
                .find_qso_identity_match(&differently_cased, None)
                .unwrap(),
            Some(id)
        );
    }

    #[test]
    fn identity_requires_every_field_to_match() {
        let repository = QsoRepository::in_memory().unwrap();
        repository.insert(&identity_qso(), 1).unwrap();

        let variants = [
            NewQso::new("PY2XYZ", 1_700_000_000, 145_500_000, "FM").unwrap(),
            NewQso::new("PY2ABC", 1_700_000_001, 145_500_000, "FM").unwrap(),
            NewQso::new("PY2ABC", 1_700_000_000, 145_500_001, "FM").unwrap(),
            NewQso::new("PY2ABC", 1_700_000_000, 145_500_000, "SSB").unwrap(),
        ];

        for variant in variants {
            assert_eq!(
                repository.find_qso_identity_match(&variant, None).unwrap(),
                None
            );
        }
    }

    #[test]
    fn editing_excludes_self_but_finds_another_collision() {
        let repository = QsoRepository::in_memory().unwrap();
        let qso = identity_qso();
        let first_id = repository.insert(&qso, 1).unwrap();

        assert_eq!(
            repository
                .find_qso_identity_match(&qso, Some(first_id))
                .unwrap(),
            None
        );

        let duplicate_id = repository.insert(&qso, 2).unwrap();
        assert_eq!(
            repository
                .find_qso_identity_match(&qso, Some(first_id))
                .unwrap(),
            Some(duplicate_id)
        );
    }

    #[test]
    fn identity_lookup_does_not_block_duplicate_inserts() {
        let repository = QsoRepository::in_memory().unwrap();
        let qso = identity_qso();

        let first_id = repository.insert(&qso, 1).unwrap();
        assert_eq!(
            repository.find_qso_identity_match(&qso, None).unwrap(),
            Some(first_id)
        );
        let duplicate_id = repository.insert(&qso, 2).unwrap();

        assert_ne!(first_id, duplicate_id);
        assert_eq!(repository.list().unwrap().len(), 2);
    }

    #[test]
    fn editing_with_nonexistent_id_still_finds_a_collision() {
        let repository = QsoRepository::in_memory().unwrap();
        let qso = identity_qso();
        let id = repository.insert(&qso, 1).unwrap();

        assert_eq!(
            repository
                .find_qso_identity_match(&qso, Some(i64::MAX))
                .unwrap(),
            Some(id)
        );
    }

    fn ysf_metadata() -> YsfMetadata {
        YsfMetadata::from_input(YsfMetadataInput {
            room: "America-Link".into(),
            wires_x_node: "Node 724".into(),
            repeater: "PY2YSF-RPT".into(),
            network: "WIRES-X".into(),
            access_type: "repeater".into(),
            tx_dg_id: "1".into(),
            rx_dg_id: "99".into(),
            notes: "Clear audio".into(),
        })
        .unwrap()
    }

    #[test]
    fn specialized_writes_reject_mode_mismatch_without_partial_changes() {
        let repository = QsoRepository::in_memory().unwrap();
        let ft8_metadata = Ft8Metadata::from_input(Ft8MetadataInput::default()).unwrap();
        let fm = NewQso::new("PY2BAD", 1_700_000_000, 145_500_000, "FM").unwrap();

        assert!(repository
            .insert_ft8(&fm, &ft8_metadata, 1_700_000_001)
            .is_err());
        assert!(repository.list().unwrap().is_empty());

        let valid = NewQso::new("PY2GOOD", 1_700_000_002, 14_074_000, "FT8").unwrap();
        let id = repository
            .insert_ft8(&valid, &ft8_metadata, 1_700_000_003)
            .unwrap();
        assert!(repository
            .update_ft8(id, &fm, &ft8_metadata, 1_700_000_004)
            .is_err());
        let stored = repository.list().unwrap().remove(0);
        assert_eq!(stored.mode, "FT8");
        assert_eq!(repository.get_ft8_metadata(id).unwrap(), Some(ft8_metadata));
    }

    #[test]
    fn inserts_reads_updates_and_deletes_ysf_atomically() {
        let repository = QsoRepository::in_memory().unwrap();
        let qso = NewQso::new("PY2YSF", 1_700_000_000, 145_562_500, "C4FM").unwrap();
        let metadata = ysf_metadata();
        let qso_id = repository.insert_ysf(&qso, &metadata, 1).unwrap();
        assert_eq!(repository.get_ysf_metadata(qso_id).unwrap(), Some(metadata));
        assert_metadata_invariant(&repository, qso_id, "C4FM");

        let updated_qso = NewQso::new("PU2YSF", 1_700_000_010, 439_600_000, "C4FM").unwrap();
        let updated_metadata = YsfMetadata::from_input(YsfMetadataInput {
            room: "Brazil".into(),
            access_type: "hotspot".into(),
            tx_dg_id: "10".into(),
            notes: "Updated".into(),
            ..Default::default()
        })
        .unwrap();
        assert!(repository
            .update_ysf(qso_id, &updated_qso, &updated_metadata, 2)
            .unwrap());
        assert_eq!(
            repository.get_ysf_metadata(qso_id).unwrap(),
            Some(updated_metadata)
        );
        assert_metadata_invariant(&repository, qso_id, "C4FM");

        assert!(repository.delete(qso_id).unwrap());
        assert_eq!(repository.get_ysf_metadata(qso_id).unwrap(), None);
    }

    #[test]
    fn rolls_back_ysf_insert_and_update_failures() {
        let repository = QsoRepository::in_memory().unwrap();
        repository
            .connection
            .execute_batch(
                "CREATE TRIGGER reject_ysf_insert BEFORE INSERT ON ysf_metadata
                 BEGIN SELECT RAISE(ABORT, 'rejected YSF metadata'); END;",
            )
            .unwrap();
        let qso = NewQso::new("PY2YSF", 1_700_000_000, 145_562_500, "C4FM").unwrap();
        let metadata = ysf_metadata();
        assert!(repository.insert_ysf(&qso, &metadata, 1).is_err());
        assert!(repository.list().unwrap().is_empty());

        repository
            .connection
            .execute_batch("DROP TRIGGER reject_ysf_insert;")
            .unwrap();
        let qso_id = repository.insert_ysf(&qso, &metadata, 2).unwrap();
        repository
            .connection
            .execute_batch(
                "CREATE TRIGGER reject_ysf_update BEFORE INSERT ON ysf_metadata
                 BEGIN SELECT RAISE(ABORT, 'rejected YSF metadata'); END;",
            )
            .unwrap();
        let changed = NewQso::new("PU2BAD", 1_700_000_010, 439_600_000, "C4FM").unwrap();
        assert!(repository
            .update_ysf(qso_id, &changed, &metadata, 3)
            .is_err());
        assert_eq!(repository.list().unwrap()[0].callsign, "PY2YSF");
        assert_eq!(repository.get_ysf_metadata(qso_id).unwrap(), Some(metadata));
        assert_metadata_invariant(&repository, qso_id, "C4FM");
    }

    #[test]
    fn ysf_writes_require_canonical_c4fm_mode_without_partial_changes() {
        let repository = QsoRepository::in_memory().unwrap();
        let metadata = ysf_metadata();
        for mode in ["YSF", "SYSTEM FUSION"] {
            let mismatch = NewQso::new("PY2BAD", 1_700_000_000, 145_562_500, mode).unwrap();
            assert!(repository.insert_ysf(&mismatch, &metadata, 1).is_err());
        }
        assert!(repository.list().unwrap().is_empty());

        let valid = NewQso::new("PY2YSF", 1_700_000_000, 145_562_500, "C4FM").unwrap();
        let qso_id = repository.insert_ysf(&valid, &metadata, 2).unwrap();
        let mismatch = NewQso::new("PU2BAD", 1_700_000_010, 439_600_000, "YSF").unwrap();
        assert!(repository
            .update_ysf(qso_id, &mismatch, &metadata, 3)
            .is_err());
        assert_eq!(repository.list().unwrap()[0].callsign, "PY2YSF");
        assert_metadata_invariant(&repository, qso_id, "C4FM");
    }

    #[test]
    fn creates_consistent_backup_without_overwriting() {
        let repository = QsoRepository::in_memory().unwrap();
        let qso = NewQso::new("PU2XYZ", 1_700_000_000, 438_500_000, "DMR").unwrap();
        repository.insert(&qso, 1_700_000_001).unwrap();
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!("dhrl-backup-test-{suffix}"));
        std::fs::create_dir_all(&directory).unwrap();
        let destination = directory.join("backup.sqlite3");

        repository.backup_to(&destination).unwrap();
        let backup = QsoRepository::open(&destination).unwrap();
        assert_eq!(backup.list().unwrap().len(), 1);
        drop(backup);
        let original = std::fs::read(&destination).unwrap();
        assert!(repository.backup_to(&destination).is_err());
        assert_eq!(std::fs::read(&destination).unwrap(), original);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn backup_rejects_a_missing_destination_directory_without_creating_a_file() {
        let repository = QsoRepository::in_memory().unwrap();
        let directory = temporary_database_path("backup-missing-parent");
        let destination = directory.join("missing").join("backup.sqlite3");

        let error = repository.backup_to(&destination).unwrap_err().to_string();

        assert!(error.contains("directory does not exist"));
        assert!(!destination.exists());
    }

    #[cfg(unix)]
    #[test]
    fn backup_uses_private_file_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let repository = QsoRepository::in_memory().unwrap();
        let directory = temporary_database_path("backup-permissions");
        std::fs::create_dir_all(&directory).unwrap();
        let destination = directory.join("backup.sqlite3");

        repository.backup_to(&destination).unwrap();

        let mode = std::fs::metadata(&destination)
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn backup_rejects_and_removes_an_incomplete_application_schema() {
        let repository = QsoRepository::in_memory().unwrap();
        repository
            .connection
            .execute("DROP TABLE adif_extra_fields", [])
            .unwrap();
        let directory = temporary_database_path("backup-incomplete");
        std::fs::create_dir_all(&directory).unwrap();
        let destination = directory.join("backup.sqlite3");

        let error = repository.backup_to(&destination).unwrap_err().to_string();
        assert!(error.contains("missing table adif_extra_fields"));
        assert!(!destination.exists());
        assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 0);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn backup_rejects_and_removes_a_future_schema_snapshot() {
        let repository = QsoRepository::in_memory().unwrap();
        repository
            .connection
            .execute(
                "INSERT INTO schema_migrations(version, applied_at_utc) VALUES (999, 0)",
                [],
            )
            .unwrap();
        let directory = temporary_database_path("backup-future");
        std::fs::create_dir_all(&directory).unwrap();
        let destination = directory.join("backup.sqlite3");

        let error = repository.backup_to(&destination).unwrap_err().to_string();
        assert!(error.contains("newer than supported"));
        assert!(!destination.exists());
        assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 0);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn backup_restores_generic_dmr_ft8_dstar_ysf_and_adif_extra_data() {
        let repository = QsoRepository::in_memory().unwrap();
        let document = parse(
            "<CALL:6>PU2GEN<QSO_DATE:8>20231114<TIME_ON:6>221320<FREQ:7>145.500<MODE:3>M17<EOR>\
             <CALL:6>PU2DMR<QSO_DATE:8>20231114<TIME_ON:6>221321<FREQ:7>438.500<MODE:3>DMR\
             <APP_DHRL_CALL_TYPE:5>group<APP_DHRL_ACCESS_TYPE:7>simplex<APP_DHRL_TALKGROUP:3>724<EOR>\
             <CALL:6>PY2FT8<QSO_DATE:8>20231114<TIME_ON:6>221322<FREQ:6>14.074<MODE:3>FT8\
             <SNR:3>-18<APP_VENDOR_FIELD:5:S>value<EOR>\
             <CALL:6>PY2DST<QSO_DATE:8>20231114<TIME_ON:6>221323<FREQ:7>145.670\
             <MODE:12>DIGITALVOICE<SUBMODE:5>DSTAR<APP_DHRL_DSTAR_REFLECTOR:8>REF001 C\
             <APP_DHRL_DSTAR_MODULE:1>C<STATION_CALLSIGN:8>PY2OWN G\
             <APP_DHRL_DSTAR_URCALL:6>CQCQCQ<APP_DHRL_DSTAR_RPT1:8>PY2RPT B\
             <APP_DHRL_DSTAR_RPT2:8>PY2RPT G<APP_DHRL_DSTAR_NOTES:19>Backup D-STAR route<EOR>",
        )
        .unwrap();
        repository.import_adif(&document, 1_700_000_100).unwrap();
        let ysf_qso = NewQso::new("PY2YSF", 1_700_000_004, 145_562_500, "C4FM").unwrap();
        let ysf = ysf_metadata();
        repository
            .insert_ysf(&ysf_qso, &ysf, 1_700_000_100)
            .unwrap();
        let directory = temporary_database_path("backup-restore");
        std::fs::create_dir_all(&directory).unwrap();
        let backup = directory.join("backup.sqlite3");
        let restored = directory.join("restored.sqlite3");

        repository.backup_to(&backup).unwrap();
        std::fs::copy(&backup, &restored).unwrap();
        let restored_repository = QsoRepository::open(&restored).unwrap();
        let qsos = restored_repository.list().unwrap();
        assert_eq!(qsos.len(), 5);
        let dmr_id = qsos.iter().find(|qso| qso.mode == "DMR").unwrap().id;
        let ft8_id = qsos.iter().find(|qso| qso.mode == "FT8").unwrap().id;
        let dstar_id = qsos.iter().find(|qso| qso.mode == "DSTAR").unwrap().id;
        let ysf_id = qsos.iter().find(|qso| qso.mode == "C4FM").unwrap().id;
        assert_eq!(
            restored_repository
                .get_dmr_metadata(dmr_id)
                .unwrap()
                .unwrap()
                .talkgroup,
            Some(724)
        );
        assert_eq!(
            restored_repository
                .get_ft8_metadata(ft8_id)
                .unwrap()
                .unwrap()
                .snr_received_db,
            Some(-18)
        );
        assert_eq!(
            restored_repository.get_dstar_metadata(dstar_id).unwrap(),
            Some(DStarMetadata {
                reflector: Some("REF001 C".into()),
                module: Some("C".into()),
                mycall: Some("PY2OWN G".into()),
                urcall: Some("CQCQCQ".into()),
                rpt1: Some("PY2RPT B".into()),
                rpt2: Some("PY2RPT G".into()),
                notes: "Backup D-STAR route".into(),
            })
        );
        assert_eq!(
            restored_repository.get_ysf_metadata(ysf_id).unwrap(),
            Some(ysf)
        );
        assert_metadata_invariant(&restored_repository, ysf_id, "C4FM");
        assert_eq!(
            restored_repository.get_adif_extra_fields(ft8_id).unwrap(),
            vec![AdifField {
                name: "APP_VENDOR_FIELD".into(),
                value: "value".into(),
                data_type: Some("S".into()),
            }]
        );
        restored_repository.verify_integrity().unwrap();
        drop(restored_repository);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn imports_adif_document_atomically_and_preserves_unknown_fields() {
        let repository = QsoRepository::in_memory().unwrap();
        let document = parse(
            "<CALL:6>PY2ABC<QSO_DATE:8>20231114<TIME_ON:6>221320<FREQ:6>14.074\
             <MODE:3>FT8<SNR:3>-18<APP_VENDOR_FIELD:5:S>value<EOR>\
             <CALL:6>PU2XYZ<QSO_DATE:8>20231114<TIME_ON:6>221321<FREQ:7>438.500\
             <MODE:3>DMR<APP_DHRL_CALL_TYPE:5>group<APP_DHRL_ACCESS_TYPE:7>simplex<EOR>",
        )
        .unwrap();

        assert_eq!(
            repository.import_adif(&document, 1_700_000_100).unwrap(),
            AdifImportReport {
                imported: 2,
                duplicates_skipped: 0,
            }
        );
        let qsos = repository.list().unwrap();
        assert_eq!(qsos.len(), 2);
        let ft8 = qsos.iter().find(|qso| qso.mode == "FT8").unwrap();
        assert_eq!(
            repository
                .get_ft8_metadata(ft8.id)
                .unwrap()
                .unwrap()
                .snr_received_db,
            Some(-18)
        );
        assert_eq!(
            repository.get_adif_extra_fields(ft8.id).unwrap(),
            vec![AdifField {
                name: "APP_VENDOR_FIELD".into(),
                value: "value".into(),
                data_type: Some("S".into()),
            }]
        );
    }

    #[test]
    fn previews_adif_without_writing_and_imports_only_after_confirmation() {
        let repository = QsoRepository::in_memory().unwrap();
        let existing = NewQso::new("PY2ABC", 1_700_000_000, 14_074_000, "FT8").unwrap();
        repository.insert(&existing, 1_700_000_001).unwrap();
        let document = parse(
            "<CALL:6>PY2ABC<QSO_DATE:8>20231114<TIME_ON:6>221320<FREQ:6>14.074<MODE:3>FT8<EOR>\
             <CALL:6>PU2XYZ<QSO_DATE:8>20231114<TIME_ON:6>221321<FREQ:7>438.500<MODE:3>DMR<EOR>\
             <CALL:6>PY2BAD<MODE:3>FT8<EOR>",
        )
        .unwrap();

        let plan = repository.prepare_adif_import(&document).unwrap();
        assert_eq!(
            plan.preview(),
            &AdifImportPreview {
                total: 3,
                new_qsos: 1,
                duplicates: 1,
                invalid: 1,
                modes: BTreeMap::from([("DMR".into(), 1), ("FT8".into(), 1)]),
                bands: BTreeMap::from([("20m".into(), 1), ("70cm".into(), 1)]),
                earliest_utc: Some(1_700_000_000),
                latest_utc: Some(1_700_000_001),
                invalid_details: vec!["Record 3 — missing ADIF field QSO_DATE".into()],
            }
        );
        assert_eq!(repository.list().unwrap().len(), 1);

        assert_eq!(
            repository.import_adif_plan(plan, 1_700_000_100).unwrap(),
            AdifImportReport {
                imported: 1,
                duplicates_skipped: 1,
            }
        );
        assert_eq!(repository.list().unwrap().len(), 2);
    }

    #[test]
    fn confirmation_skips_duplicates_created_after_adif_preview() {
        let repository = QsoRepository::in_memory().unwrap();
        let document = parse(
            "<CALL:6>PY2ABC<QSO_DATE:8>20231114<TIME_ON:6>221320<FREQ:6>14.074<MODE:3>FT8<EOR>",
        )
        .unwrap();
        let plan = repository.prepare_adif_import(&document).unwrap();
        assert_eq!(plan.preview().new_qsos, 1);

        let same_qso = NewQso::new("PY2ABC", 1_700_000_000, 14_074_000, "FT8").unwrap();
        repository.insert(&same_qso, 1_700_000_050).unwrap();
        let report = repository.import_adif_plan(plan, 1_700_000_100).unwrap();

        assert_eq!(
            report,
            AdifImportReport {
                imported: 0,
                duplicates_skipped: 1,
            }
        );
        assert_eq!(repository.list().unwrap().len(), 1);
    }

    #[test]
    fn dropping_an_adif_plan_cancels_without_writing() {
        let repository = QsoRepository::in_memory().unwrap();
        let document = parse(
            "<CALL:6>PU2XYZ<QSO_DATE:8>20231114<TIME_ON:6>221321<FREQ:7>438.500<MODE:3>DMR<EOR>",
        )
        .unwrap();

        let plan = repository.prepare_adif_import(&document).unwrap();
        assert_eq!(plan.preview().new_qsos, 1);
        drop(plan);
        assert!(repository.list().unwrap().is_empty());
    }

    #[test]
    fn skips_exact_duplicates_when_reimporting_adif() {
        let repository = QsoRepository::in_memory().unwrap();
        let document = parse(
            "<CALL:6>PY2ABC<QSO_DATE:8>20231114<TIME_ON:6>221320<FREQ:6>14.074<MODE:3>FT8<EOR>\
             <CALL:6>PU2XYZ<QSO_DATE:8>20231114<TIME_ON:6>221321<FREQ:7>438.500<MODE:3>DMR<EOR>",
        )
        .unwrap();

        assert_eq!(
            repository.import_adif(&document, 1_700_000_100).unwrap(),
            AdifImportReport {
                imported: 2,
                duplicates_skipped: 0,
            }
        );
        assert_eq!(
            repository.import_adif(&document, 1_700_000_200).unwrap(),
            AdifImportReport {
                imported: 0,
                duplicates_skipped: 2,
            }
        );
        assert_eq!(repository.list().unwrap().len(), 2);
    }

    #[test]
    fn skips_duplicates_within_one_adif_without_merging_fields() {
        let repository = QsoRepository::in_memory().unwrap();
        let document = parse(
            "<CALL:6>PY2ABC<QSO_DATE:8>20231114<TIME_ON:6>221320<FREQ:6>14.074<MODE:3>FT8<COMMENT:5>first<EOR>\
             <CALL:6>PY2ABC<QSO_DATE:8>20231114<TIME_ON:6>221320<FREQ:6>14.074<MODE:3>FT8<COMMENT:6>second<EOR>",
        )
        .unwrap();

        assert_eq!(
            repository.import_adif(&document, 1_700_000_100).unwrap(),
            AdifImportReport {
                imported: 1,
                duplicates_skipped: 1,
            }
        );
        let qsos = repository.list().unwrap();
        assert_eq!(qsos.len(), 1);
        assert_eq!(qsos[0].notes, "first");
    }

    #[test]
    fn skips_adif_duplicate_of_a_manually_created_qso() {
        let repository = QsoRepository::in_memory().unwrap();
        let qso = NewQso::new("py2abc", 1_700_000_000, 14_074_000, "ft8").unwrap();
        repository.insert(&qso, 1_700_000_001).unwrap();
        let document = parse(
            "<CALL:6>PY2ABC<QSO_DATE:8>20231114<TIME_ON:6>221320<FREQ:6>14.074<MODE:3>FT8<EOR>",
        )
        .unwrap();

        assert_eq!(
            repository.import_adif(&document, 1_700_000_100).unwrap(),
            AdifImportReport {
                imported: 0,
                duplicates_skipped: 1,
            }
        );
        assert_eq!(repository.list().unwrap().len(), 1);
    }

    #[test]
    fn imports_qsos_that_differ_in_any_identity_field() {
        let repository = QsoRepository::in_memory().unwrap();
        let existing = NewQso::new("PY2ABC", 1_700_000_000, 14_074_000, "FT8").unwrap();
        repository.insert(&existing, 1_700_000_001).unwrap();
        let document = parse(
            "<CALL:6>PY2ABD<QSO_DATE:8>20231114<TIME_ON:6>221320<FREQ:6>14.074<MODE:3>FT8<EOR>\
             <CALL:6>PY2ABC<QSO_DATE:8>20231114<TIME_ON:6>221321<FREQ:6>14.074<MODE:3>FT8<EOR>\
             <CALL:6>PY2ABC<QSO_DATE:8>20231114<TIME_ON:6>221320<FREQ:6>14.075<MODE:3>FT8<EOR>\
             <CALL:6>PY2ABC<QSO_DATE:8>20231114<TIME_ON:6>221320<FREQ:6>14.074<MODE:4>MFSK<EOR>",
        )
        .unwrap();

        assert_eq!(
            repository.import_adif(&document, 1_700_000_100).unwrap(),
            AdifImportReport {
                imported: 4,
                duplicates_skipped: 0,
            }
        );
        assert_eq!(repository.list().unwrap().len(), 5);
    }

    #[test]
    fn round_trips_complete_adif_semantics_through_two_sqlite_repositories() {
        let input = parse(
            "<CALL:6>PU2GEN<QSO_DATE:8>20231114<TIME_ON:6>221320<FREQ:7>145.500\
             <MODE:3>M17<RST_SENT:3>599<RST_RCVD:3>579<GRIDSQUARE:6>GG66AA\
             <NAME:5>José<QTH:10>São Paulo<COMMENT:11>ação 🚀\
             <APP_ZETA:9:S>café ☕<APP_ALPHA:2:N>42<EOR>\
             <CALL:6>PU2DMR<QSO_DATE:8>20231114<TIME_ON:6>221321<FREQ:7>438.500\
             <MODE:3>DMR<APP_DHRL_REMOTE_DMR_ID:7>7241234\
             <APP_DHRL_LOCAL_DMR_ID:7>7245678<APP_DHRL_TALKGROUP:3>724\
             <APP_DHRL_TIMESLOT:1>2<APP_DHRL_COLOR_CODE:1>1\
             <APP_DHRL_NETWORK:12>BrandMeister<APP_DHRL_CALL_TYPE:5>group\
             <APP_DHRL_ACCESS_TYPE:8>repeater<APP_DHRL_REPEATER:6>PY2ABC\
             <APP_DHRL_RX_FREQUENCY_HZ:9>438500125\
             <APP_DHRL_TX_FREQUENCY_HZ:9>430900625\
             <APP_DHRL_DMR_NOTES:12>áudio claro<EOR>\
             <CALL:6>PY2FT8<QSO_DATE:8>20231114<TIME_ON:6>221322<FREQ:6>14.074\
             <MODE:3>FT8<APP_DHRL_SNR_SENT:3>-10<SNR:3>-18<TX_PWR:2>25\
             <APP_DHRL_AUDIO_FREQUENCY:4>1500<APP_DHRL_SOURCE_SOFTWARE:6>WSJT-X\
             <APP_DHRL_PROTOCOL:3>FT8<APP_DHRL_FINAL_MESSAGE:4>RR73<EOR>",
        )
        .unwrap();
        let first = QsoRepository::in_memory().unwrap();
        assert_eq!(
            first.import_adif(&input, 1_700_000_100).unwrap(),
            AdifImportReport {
                imported: 3,
                duplicates_skipped: 0,
            }
        );

        let exported = first.export_adif().unwrap();
        assert_eq!(
            exported.header.as_ref().unwrap().get("PROGRAMID"),
            Some("Digital Ham Radio Logbook")
        );
        assert_eq!(
            exported.header.as_ref().unwrap().get("PROGRAMVERSION"),
            Some(env!("CARGO_PKG_VERSION"))
        );
        let reparsed = parse(&export(&exported)).unwrap();
        let second = QsoRepository::in_memory().unwrap();
        assert_eq!(
            second.import_adif(&reparsed, 1_700_000_200).unwrap(),
            AdifImportReport {
                imported: 3,
                duplicates_skipped: 0,
            }
        );

        let qsos = second.list().unwrap();
        assert_eq!(qsos.len(), 3);
        let generic = qsos.iter().find(|qso| qso.callsign == "PU2GEN").unwrap();
        assert_eq!(generic.mode, "M17");
        assert_eq!(generic.rst_sent.as_deref(), Some("599"));
        assert_eq!(generic.rst_received.as_deref(), Some("579"));
        assert_eq!(generic.grid_locator.as_deref(), Some("GG66AA"));
        assert_eq!(generic.name.as_deref(), Some("José"));
        assert_eq!(generic.qth.as_deref(), Some("São Paulo"));
        assert_eq!(generic.notes, "ação 🚀");
        assert_eq!(
            second.get_adif_extra_fields(generic.id).unwrap(),
            vec![
                AdifField {
                    name: "APP_ZETA".into(),
                    value: "café ☕".into(),
                    data_type: Some("S".into()),
                },
                AdifField {
                    name: "APP_ALPHA".into(),
                    value: "42".into(),
                    data_type: Some("N".into()),
                },
            ]
        );

        let dmr = qsos.iter().find(|qso| qso.callsign == "PU2DMR").unwrap();
        assert_eq!(
            second.get_dmr_metadata(dmr.id).unwrap().unwrap(),
            DmrMetadata {
                remote_dmr_id: Some(7_241_234),
                local_dmr_id: Some(7_245_678),
                talkgroup: Some(724),
                timeslot: Some(2),
                color_code: Some(1),
                network: Some("BrandMeister".into()),
                call_type: DmrCallType::Group,
                access_type: DmrAccessType::Repeater,
                repeater_callsign: Some("PY2ABC".into()),
                hotspot: None,
                rx_frequency_hz: Some(438_500_125),
                tx_frequency_hz: Some(430_900_625),
                notes: "áudio claro".into(),
            }
        );

        let ft8 = qsos.iter().find(|qso| qso.callsign == "PY2FT8").unwrap();
        assert_eq!(
            second.get_ft8_metadata(ft8.id).unwrap().unwrap(),
            Ft8Metadata {
                snr_sent_db: Some(-10),
                snr_received_db: Some(-18),
                power_watts: Some(25),
                audio_frequency_hz: Some(1_500),
                source_software: Some("WSJT-X".into()),
                protocol: Some("FT8".into()),
                final_message: Some("RR73".into()),
            }
        );

        let second_export = parse(&export(&second.export_adif().unwrap())).unwrap();
        assert_eq!(second_export.records, reparsed.records);
        assert_eq!(
            second.import_adif(&reparsed, 1_700_000_300).unwrap(),
            AdifImportReport {
                imported: 0,
                duplicates_skipped: 3,
            }
        );
        assert_eq!(second.list().unwrap().len(), 3);
    }

    #[test]
    fn rejects_invalid_adif_before_writing_any_qso() {
        let repository = QsoRepository::in_memory().unwrap();
        let document = parse(
            "<CALL:6>PY2ABC<QSO_DATE:8>20231114<TIME_ON:6>221320<FREQ:6>14.074<MODE:3>FT8<EOR>\
             <CALL:6>PU2XYZ<MODE:3>DMR<EOR>",
        )
        .unwrap();

        let error = repository
            .import_adif(&document, 1_700_000_100)
            .unwrap_err();
        assert!(error.to_string().contains("record 2"));
        assert!(repository.list().unwrap().is_empty());
    }

    #[test]
    fn opens_and_initializes_a_missing_database_file() {
        let path = temporary_database_path("missing");
        assert!(!path.exists());

        let repository = QsoRepository::open(&path).unwrap();
        repository.verify_integrity().unwrap();
        drop(repository);

        assert!(path.is_file());
        let reopened = QsoRepository::open(&path).unwrap();
        reopened.verify_integrity().unwrap();
        drop(reopened);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn opens_and_initializes_an_existing_zero_byte_database_file() {
        let path = temporary_database_path("zero-byte");
        std::fs::write(&path, []).unwrap();

        let repository = QsoRepository::open(&path).unwrap();
        repository.verify_integrity().unwrap();
        drop(repository);

        assert!(std::fs::metadata(&path).unwrap().len() > 0);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn rejects_a_truncated_sqlite_database_without_changing_its_bytes() {
        let path = temporary_database_path("truncated");
        {
            let repository = QsoRepository::open(&path).unwrap();
            for index in 0..200 {
                let qso = NewQso::new(
                    format!("PU2{index:04}"),
                    1_700_000_000 + index,
                    438_500_000,
                    "DMR",
                )
                .unwrap();
                repository.insert(&qso, 1_700_000_000 + index).unwrap();
            }
        }
        let mut corrupted = std::fs::read(&path).unwrap();
        corrupted.truncate(corrupted.len() / 2);
        std::fs::write(&path, &corrupted).unwrap();

        assert!(QsoRepository::open(&path).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), corrupted);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn rejects_a_non_sqlite_database_without_replacing_it() {
        let path = temporary_database_path("not-sqlite");
        std::fs::write(&path, b"not a sqlite database").unwrap();

        assert!(QsoRepository::open(&path).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"not a sqlite database");
        std::fs::remove_file(path).unwrap();
    }

    fn temporary_database_path(label: &str) -> std::path::PathBuf {
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "dhrl-{label}-{}-{suffix}.sqlite3",
            std::process::id()
        ))
    }

    #[test]
    fn reports_integrity_for_a_healthy_database() {
        let repository = QsoRepository::in_memory().unwrap();
        repository.verify_integrity().unwrap();
    }

    #[test]
    fn repository_delete_cascades_dmr_routes_and_ft8_metadata() {
        let repository = QsoRepository::in_memory().unwrap();
        let dmr_qso = NewQso::new("PU2DMR", 1_700_000_000, 438_500_000, "DMR").unwrap();
        let dmr = DmrMetadata::from_input(DmrMetadataInput {
            call_type: "group".into(),
            access_type: "simplex".into(),
            ..Default::default()
        })
        .unwrap();
        let dmr_id = repository
            .insert_dmr(&dmr_qso, &dmr, 1_700_000_001)
            .unwrap();
        let ft8_qso = NewQso::new("PY2FT8", 1_700_000_002, 14_074_000, "FT8").unwrap();
        let ft8 = Ft8Metadata::from_input(Ft8MetadataInput::default()).unwrap();
        let ft8_id = repository
            .insert_ft8(&ft8_qso, &ft8, 1_700_000_003)
            .unwrap();

        assert!(repository.delete(dmr_id).unwrap());
        assert!(repository.delete(ft8_id).unwrap());
        assert_eq!(repository.get_dmr_metadata(dmr_id).unwrap(), None);
        assert_eq!(repository.get_ft8_metadata(ft8_id).unwrap(), None);
        let dmr_routes: i64 = repository
            .connection
            .query_row(
                "SELECT COUNT(*) FROM digital_routes WHERE qso_id = ?1",
                [dmr_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(dmr_routes, 0);
        repository.verify_integrity().unwrap();
    }

    #[test]
    fn inserts_and_lists_a_qso() {
        let repository = QsoRepository::in_memory().unwrap();
        let qso = NewQso::new("PU2XYZ", 1_700_000_000, 438_500_000, "DMR").unwrap();

        let id = repository.insert(&qso, 1_700_000_001).unwrap();
        let saved = repository.list().unwrap();

        assert_eq!(saved.len(), 1);
        assert_eq!(saved[0].id, id);
        assert_eq!(saved[0].callsign, "PU2XYZ");
        assert_eq!(saved[0].frequency_hz, 438_500_000);
    }

    #[test]
    fn updates_and_deletes_a_qso() {
        let repository = QsoRepository::in_memory().unwrap();
        let original = NewQso::new("PU2XYZ", 1_700_000_000, 438_500_000, "DMR").unwrap();
        let id = repository.insert(&original, 1_700_000_001).unwrap();
        let updated = NewQso::new("PY2ABC", 1_700_000_000, 145_500_000, "M17")
            .unwrap()
            .with_common_fields(CommonQsoFields {
                rst_sent: "59".into(),
                rst_received: "57".into(),
                grid_locator: "GG66AA".into(),
                name: "Operator".into(),
                qth: "SP".into(),
                notes: "Updated".into(),
                ..Default::default()
            })
            .unwrap();

        assert!(repository.update(id, &updated, 1_700_000_010).unwrap());
        let saved = repository.list().unwrap();
        assert_eq!(saved[0].callsign, "PY2ABC");
        assert_eq!(saved[0].mode, "M17");
        assert_eq!(saved[0].datetime_start_utc, 1_700_000_000);
        assert_eq!(saved[0].updated_at_utc, 1_700_000_010);
        assert_eq!(saved[0].band.as_deref(), Some("2m"));
        assert_eq!(saved[0].grid_locator.as_deref(), Some("GG66AA"));
        assert_eq!(saved[0].notes, "Updated");

        assert!(repository.delete(id).unwrap());
        assert!(repository.list().unwrap().is_empty());
        assert!(!repository.delete(id).unwrap());
    }

    #[test]
    fn inserts_reads_and_updates_ft8_atomically() {
        let repository = QsoRepository::in_memory().unwrap();
        let qso = NewQso::new("PY2ABC", 1_700_000_000, 14_074_000, "FT8").unwrap();
        let metadata = Ft8Metadata::from_input(Ft8MetadataInput {
            snr_sent_db: "-12".into(),
            snr_received_db: "-18".into(),
            power_watts: "25".into(),
            source_software: "WSJT-X".into(),
            final_message: "RR73".into(),
            ..Default::default()
        })
        .unwrap();
        let qso_id = repository
            .insert_ft8(&qso, &metadata, 1_700_000_001)
            .unwrap();
        assert_eq!(repository.get_ft8_metadata(qso_id).unwrap(), Some(metadata));

        let updated_qso = NewQso::new("PY2XYZ", 1_700_000_010, 7_074_000, "FT8").unwrap();
        let updated_metadata = Ft8Metadata::from_input(Ft8MetadataInput {
            snr_received_db: "-9".into(),
            power_watts: "50".into(),
            protocol: "FT8".into(),
            ..Default::default()
        })
        .unwrap();
        assert!(repository
            .update_ft8(qso_id, &updated_qso, &updated_metadata, 1_700_000_020)
            .unwrap());
        assert_eq!(repository.list().unwrap()[0].callsign, "PY2XYZ");
        assert_eq!(
            repository.get_ft8_metadata(qso_id).unwrap(),
            Some(updated_metadata)
        );
    }

    #[test]
    fn changing_dmr_to_ft8_removes_dmr_metadata_and_route() {
        let repository = QsoRepository::in_memory().unwrap();
        let dmr_qso = NewQso::new("PU2DMR", 1_700_000_000, 438_500_000, "DMR").unwrap();
        let dmr = DmrMetadata::from_input(DmrMetadataInput {
            talkgroup: "724".into(),
            network: "BrandMeister".into(),
            call_type: "group".into(),
            access_type: "simplex".into(),
            ..Default::default()
        })
        .unwrap();
        let qso_id = repository
            .insert_dmr(&dmr_qso, &dmr, 1_700_000_001)
            .unwrap();

        let ft8_qso = NewQso::new("PU2DMR", 1_700_000_000, 14_074_000, "FT8").unwrap();
        let ft8 = Ft8Metadata::from_input(Ft8MetadataInput {
            snr_received_db: "-12".into(),
            ..Default::default()
        })
        .unwrap();
        assert!(repository
            .update_ft8(qso_id, &ft8_qso, &ft8, 1_700_000_002)
            .unwrap());

        assert_eq!(repository.get_dmr_metadata(qso_id).unwrap(), None);
        assert_eq!(repository.get_ft8_metadata(qso_id).unwrap(), Some(ft8));
        assert_eq!(
            repository
                .search_dmr_page(&Default::default(), 0, 100)
                .unwrap()
                .total,
            0
        );
    }

    #[test]
    fn changing_ft8_to_dmr_removes_ft8_metadata() {
        let repository = QsoRepository::in_memory().unwrap();
        let ft8_qso = NewQso::new("PY2FT8", 1_700_000_000, 14_074_000, "FT8").unwrap();
        let ft8 = Ft8Metadata::from_input(Ft8MetadataInput {
            snr_received_db: "-18".into(),
            ..Default::default()
        })
        .unwrap();
        let qso_id = repository
            .insert_ft8(&ft8_qso, &ft8, 1_700_000_001)
            .unwrap();

        let dmr_qso = NewQso::new("PY2FT8", 1_700_000_000, 438_500_000, "DMR").unwrap();
        let dmr = DmrMetadata::from_input(DmrMetadataInput {
            talkgroup: "91".into(),
            call_type: "group".into(),
            access_type: "simplex".into(),
            ..Default::default()
        })
        .unwrap();
        assert!(repository
            .update_dmr(qso_id, &dmr_qso, &dmr, 1_700_000_002)
            .unwrap());

        assert_eq!(repository.get_ft8_metadata(qso_id).unwrap(), None);
        assert_eq!(repository.get_dmr_metadata(qso_id).unwrap(), Some(dmr));
        assert_eq!(
            repository
                .search_ft8_page(&Default::default(), 0, 100)
                .unwrap()
                .total,
            0
        );
    }

    #[test]
    fn changing_specialized_mode_to_generic_removes_all_mode_metadata() {
        let repository = QsoRepository::in_memory().unwrap();
        let dmr_qso = NewQso::new("PU2GEN", 1_700_000_000, 438_500_000, "DMR").unwrap();
        let dmr = DmrMetadata::from_input(DmrMetadataInput {
            talkgroup: "724".into(),
            call_type: "group".into(),
            access_type: "simplex".into(),
            ..Default::default()
        })
        .unwrap();
        let qso_id = repository
            .insert_dmr(&dmr_qso, &dmr, 1_700_000_001)
            .unwrap();

        let generic_qso = NewQso::new("PU2GEN", 1_700_000_000, 145_500_000, "M17").unwrap();
        assert!(repository
            .update(qso_id, &generic_qso, 1_700_000_002)
            .unwrap());

        assert_eq!(repository.get_dmr_metadata(qso_id).unwrap(), None);
        assert_eq!(repository.get_ft8_metadata(qso_id).unwrap(), None);
        assert_eq!(
            repository
                .search_dmr_page(&Default::default(), 0, 100)
                .unwrap()
                .total,
            0
        );
        assert_eq!(
            repository
                .search_ft8_page(&Default::default(), 0, 100)
                .unwrap()
                .total,
            0
        );
    }

    #[test]
    fn rolls_back_ft8_insert_and_update_failures() {
        let repository = QsoRepository::in_memory().unwrap();
        let qso = NewQso::new("PY2ABC", 1_700_000_000, 14_074_000, "FT8").unwrap();
        let invalid = Ft8Metadata {
            snr_sent_db: Some(-60),
            snr_received_db: None,
            power_watts: None,
            audio_frequency_hz: None,
            source_software: None,
            protocol: None,
            final_message: None,
        };
        assert!(repository
            .insert_ft8(&qso, &invalid, 1_700_000_001)
            .is_err());
        assert!(repository.list().unwrap().is_empty());

        let valid = Ft8Metadata::from_input(Ft8MetadataInput::default()).unwrap();
        let qso_id = repository.insert_ft8(&qso, &valid, 1_700_000_001).unwrap();
        let changed_qso = NewQso::new("PY2XYZ", 1_700_000_010, 7_074_000, "FT8").unwrap();
        assert!(repository
            .update_ft8(qso_id, &changed_qso, &invalid, 1_700_000_020)
            .is_err());
        assert_eq!(repository.list().unwrap()[0].callsign, "PY2ABC");
        assert_eq!(repository.get_ft8_metadata(qso_id).unwrap(), Some(valid));
    }

    #[test]
    fn inserts_and_reads_dmr_qso_atomically() {
        let repository = QsoRepository::in_memory().unwrap();
        let qso = NewQso::new("PU2XYZ", 1_700_000_000, 438_500_000, "DMR").unwrap();
        let metadata = DmrMetadata::from_input(DmrMetadataInput {
            remote_dmr_id: "7241234".into(),
            talkgroup: "724".into(),
            timeslot: "1".into(),
            color_code: "1".into(),
            network: "BrandMeister".into(),
            call_type: "group".into(),
            access_type: "repeater".into(),
            repeater_callsign: "PY2XYZ".into(),
            ..Default::default()
        })
        .unwrap();

        let qso_id = repository
            .insert_dmr(&qso, &metadata, 1_700_000_001)
            .unwrap();
        let saved = repository.get_dmr_metadata(qso_id).unwrap().unwrap();

        assert_eq!(saved, metadata);
        assert_eq!(repository.list().unwrap().len(), 1);
    }

    #[test]
    fn updates_dmr_qso_and_metadata_atomically() {
        let repository = QsoRepository::in_memory().unwrap();
        let original_qso = NewQso::new("PU2XYZ", 1_700_000_000, 438_500_000, "DMR").unwrap();
        let original_metadata = DmrMetadata::from_input(DmrMetadataInput {
            talkgroup: "724".into(),
            call_type: "group".into(),
            access_type: "simplex".into(),
            ..Default::default()
        })
        .unwrap();
        let qso_id = repository
            .insert_dmr(&original_qso, &original_metadata, 1_700_000_001)
            .unwrap();

        let updated_qso = NewQso::new("PY2ABC", 1_700_000_010, 439_000_000, "DMR").unwrap();
        let updated_metadata = DmrMetadata::from_input(DmrMetadataInput {
            remote_dmr_id: "7249999".into(),
            talkgroup: "91".into(),
            timeslot: "2".into(),
            color_code: "3".into(),
            network: "TGIF".into(),
            call_type: "private".into(),
            access_type: "hotspot".into(),
            hotspot: "Portable hotspot".into(),
            ..Default::default()
        })
        .unwrap();

        assert!(repository
            .update_dmr(qso_id, &updated_qso, &updated_metadata, 1_700_000_020)
            .unwrap());
        assert_eq!(repository.list().unwrap()[0].callsign, "PY2ABC");
        assert_eq!(
            repository.get_dmr_metadata(qso_id).unwrap(),
            Some(updated_metadata)
        );
    }

    #[test]
    fn rolls_back_dmr_update_when_metadata_is_invalid() {
        let repository = QsoRepository::in_memory().unwrap();
        let original_qso = NewQso::new("PU2XYZ", 1_700_000_000, 438_500_000, "DMR").unwrap();
        let original_metadata = DmrMetadata::from_input(DmrMetadataInput {
            talkgroup: "724".into(),
            call_type: "group".into(),
            access_type: "simplex".into(),
            ..Default::default()
        })
        .unwrap();
        let qso_id = repository
            .insert_dmr(&original_qso, &original_metadata, 1_700_000_001)
            .unwrap();
        let changed_qso = NewQso::new("PY2ABC", 1_700_000_010, 439_000_000, "DMR").unwrap();
        let mut invalid_metadata = original_metadata.clone();
        invalid_metadata.color_code = Some(16);

        assert!(repository
            .update_dmr(qso_id, &changed_qso, &invalid_metadata, 1_700_000_020)
            .is_err());
        assert_eq!(repository.list().unwrap()[0].callsign, "PU2XYZ");
        assert_eq!(
            repository.get_dmr_metadata(qso_id).unwrap(),
            Some(original_metadata)
        );
    }

    #[test]
    fn rolls_back_qso_when_dmr_insert_fails() {
        let repository = QsoRepository::in_memory().unwrap();
        let qso = NewQso::new("PU2XYZ", 1_700_000_000, 438_500_000, "DMR").unwrap();
        let invalid_metadata = DmrMetadata {
            remote_dmr_id: None,
            local_dmr_id: None,
            talkgroup: None,
            timeslot: Some(3),
            color_code: None,
            network: None,
            call_type: DmrCallType::Group,
            access_type: DmrAccessType::Simplex,
            repeater_callsign: None,
            hotspot: None,
            rx_frequency_hz: None,
            tx_frequency_hz: None,
            notes: String::new(),
        };

        assert!(repository
            .insert_dmr(&qso, &invalid_metadata, 1_700_000_001)
            .is_err());
        assert!(repository.list().unwrap().is_empty());
    }

    #[test]
    fn inserts_reads_updates_and_deletes_dstar_atomically() {
        let repository = QsoRepository::in_memory().unwrap();
        let qso = NewQso::new("PY2ABC", 1_700_000_000, 145_670_000, "DSTAR").unwrap();
        let metadata = DStarMetadata::from_input(DStarMetadataInput {
            reflector: "REF001 C".into(),
            module: "C".into(),
            mycall: "PY2ABC G".into(),
            urcall: "CQCQCQ".into(),
            rpt1: "PY2XYZ B".into(),
            rpt2: "PY2XYZ G".into(),
            notes: "First contact".into(),
        })
        .unwrap();
        let qso_id = repository
            .insert_dstar(&qso, &metadata, 1_700_000_001)
            .unwrap();
        assert_eq!(
            repository.get_dstar_metadata(qso_id).unwrap(),
            Some(metadata)
        );

        let updated_qso = NewQso::new("PY2XYZ", 1_700_000_010, 438_800_000, "DSTAR").unwrap();
        let updated_metadata = DStarMetadata::from_input(DStarMetadataInput {
            reflector: "XLX724 A".into(),
            module: "A".into(),
            notes: "Updated".into(),
            ..Default::default()
        })
        .unwrap();
        assert!(repository
            .update_dstar(qso_id, &updated_qso, &updated_metadata, 1_700_000_020)
            .unwrap());
        assert_eq!(repository.list().unwrap()[0].callsign, "PY2XYZ");
        assert_eq!(
            repository.get_dstar_metadata(qso_id).unwrap(),
            Some(updated_metadata)
        );

        assert!(repository.delete(qso_id).unwrap());
        assert_eq!(repository.get_dstar_metadata(qso_id).unwrap(), None);
        let metadata_count: i64 = repository
            .connection
            .query_row("SELECT COUNT(*) FROM dstar_metadata", [], |row| row.get(0))
            .unwrap();
        assert_eq!(metadata_count, 0);
    }

    #[test]
    fn rolls_back_dstar_insert_and_update_failures() {
        let repository = QsoRepository::in_memory().unwrap();
        repository
            .connection
            .execute_batch(
                "CREATE TRIGGER reject_dstar_insert
                 BEFORE INSERT ON dstar_metadata
                 BEGIN SELECT RAISE(ABORT, 'rejected D-STAR metadata'); END;",
            )
            .unwrap();
        let qso = NewQso::new("PY2ABC", 1_700_000_000, 145_670_000, "DSTAR").unwrap();
        let metadata = DStarMetadata::from_input(DStarMetadataInput::default()).unwrap();
        assert!(repository
            .insert_dstar(&qso, &metadata, 1_700_000_001)
            .is_err());
        assert!(repository.list().unwrap().is_empty());

        repository
            .connection
            .execute_batch("DROP TRIGGER reject_dstar_insert;")
            .unwrap();
        let qso_id = repository
            .insert_dstar(&qso, &metadata, 1_700_000_001)
            .unwrap();
        repository
            .connection
            .execute_batch(
                "CREATE TRIGGER reject_dstar_update
                 BEFORE INSERT ON dstar_metadata
                 BEGIN SELECT RAISE(ABORT, 'rejected D-STAR metadata'); END;",
            )
            .unwrap();
        let changed_qso = NewQso::new("PY2XYZ", 1_700_000_010, 438_800_000, "DSTAR").unwrap();
        assert!(repository
            .update_dstar(qso_id, &changed_qso, &metadata, 1_700_000_020)
            .is_err());
        assert_eq!(repository.list().unwrap()[0].callsign, "PY2ABC");
        assert_eq!(
            repository.get_dstar_metadata(qso_id).unwrap(),
            Some(metadata)
        );
    }

    #[test]
    fn transitions_between_dmr_ft8_and_dstar_without_orphaned_metadata() {
        let repository = QsoRepository::in_memory().unwrap();
        let dmr_qso = NewQso::new("PU2DMR", 1_700_000_000, 438_500_000, "DMR").unwrap();
        let dmr = DmrMetadata::from_input(DmrMetadataInput {
            call_type: "group".into(),
            access_type: "simplex".into(),
            ..Default::default()
        })
        .unwrap();
        let ft8_qso = NewQso::new("PY2FT8", 1_700_000_000, 14_074_000, "FT8").unwrap();
        let ft8 = Ft8Metadata::from_input(Ft8MetadataInput::default()).unwrap();
        let dstar_qso = NewQso::new("PY2DST", 1_700_000_000, 145_670_000, "DSTAR").unwrap();
        let dstar = DStarMetadata::from_input(DStarMetadataInput {
            reflector: "REF001 C".into(),
            module: "C".into(),
            ..Default::default()
        })
        .unwrap();

        let dmr_id = repository.insert_dmr(&dmr_qso, &dmr, 1).unwrap();
        assert!(repository
            .update_dstar(dmr_id, &dstar_qso, &dstar, 2)
            .unwrap());
        assert_eq!(repository.get_dmr_metadata(dmr_id).unwrap(), None);
        assert_eq!(
            repository.get_dstar_metadata(dmr_id).unwrap(),
            Some(dstar.clone())
        );
        assert!(repository.update_dmr(dmr_id, &dmr_qso, &dmr, 3).unwrap());
        assert_eq!(repository.get_dstar_metadata(dmr_id).unwrap(), None);
        assert_eq!(
            repository.get_dmr_metadata(dmr_id).unwrap(),
            Some(dmr.clone())
        );

        let ft8_id = repository.insert_ft8(&ft8_qso, &ft8, 4).unwrap();
        assert!(repository
            .update_dstar(ft8_id, &dstar_qso, &dstar, 5)
            .unwrap());
        assert_eq!(repository.get_ft8_metadata(ft8_id).unwrap(), None);
        assert_eq!(
            repository.get_dstar_metadata(ft8_id).unwrap(),
            Some(dstar.clone())
        );
        assert!(repository.update_ft8(ft8_id, &ft8_qso, &ft8, 6).unwrap());
        assert_eq!(repository.get_dstar_metadata(ft8_id).unwrap(), None);
        assert_eq!(repository.get_ft8_metadata(ft8_id).unwrap(), Some(ft8));

        let orphaned: i64 = repository
            .connection
            .query_row(
                "SELECT COUNT(*) FROM qsos q
                 WHERE (SELECT COUNT(*) FROM dmr_metadata d WHERE d.qso_id = q.id)
                     + (SELECT COUNT(*) FROM ft8_metadata f WHERE f.qso_id = q.id)
                     + (SELECT COUNT(*) FROM dstar_metadata s WHERE s.qso_id = q.id) > 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(orphaned, 0);
    }

    #[test]
    fn transitions_between_generic_and_dstar_without_orphaned_metadata() {
        let repository = QsoRepository::in_memory().unwrap();
        let generic_qso = NewQso::new("PY2GEN", 1_700_000_000, 145_500_000, "M17").unwrap();
        let qso_id = repository.insert(&generic_qso, 1).unwrap();
        let dstar_qso = NewQso::new("PY2GEN", 1_700_000_000, 145_670_000, "DSTAR").unwrap();
        let dstar = DStarMetadata::from_input(DStarMetadataInput::default()).unwrap();

        assert!(repository
            .update_dstar(qso_id, &dstar_qso, &dstar, 2)
            .unwrap());
        assert_eq!(repository.get_dstar_metadata(qso_id).unwrap(), Some(dstar));
        assert!(repository.update(qso_id, &generic_qso, 3).unwrap());
        assert_eq!(repository.get_dstar_metadata(qso_id).unwrap(), None);
        let metadata_count: i64 = repository
            .connection
            .query_row(
                "SELECT (SELECT COUNT(*) FROM dmr_metadata WHERE qso_id = ?1)
                      + (SELECT COUNT(*) FROM ft8_metadata WHERE qso_id = ?1)
                      + (SELECT COUNT(*) FROM dstar_metadata WHERE qso_id = ?1)",
                [qso_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(metadata_count, 0);
    }

    #[test]
    fn transitions_between_generic_dmr_ft8_dstar_and_ysf_keep_exact_metadata() {
        let repository = QsoRepository::in_memory().unwrap();
        let ysf_qso = NewQso::new("PY2YSF", 1_700_000_000, 145_562_500, "C4FM").unwrap();
        let ysf = ysf_metadata();
        let generic_qso = NewQso::new("PY2GEN", 1_700_000_000, 145_500_000, "M17").unwrap();
        let dmr_qso = NewQso::new("PU2DMR", 1_700_000_000, 438_500_000, "DMR").unwrap();
        let dmr = DmrMetadata::from_input(DmrMetadataInput {
            call_type: "group".into(),
            access_type: "simplex".into(),
            ..Default::default()
        })
        .unwrap();
        let ft8_qso = NewQso::new("PY2FT8", 1_700_000_000, 14_074_000, "FT8").unwrap();
        let ft8 = Ft8Metadata::from_input(Ft8MetadataInput::default()).unwrap();
        let dstar_qso = NewQso::new("PY2DST", 1_700_000_000, 145_670_000, "DSTAR").unwrap();
        let dstar = DStarMetadata::from_input(DStarMetadataInput::default()).unwrap();

        let generic_id = repository.insert(&generic_qso, 1).unwrap();
        repository
            .update_ysf(generic_id, &ysf_qso, &ysf, 2)
            .unwrap();
        assert_metadata_invariant(&repository, generic_id, "C4FM");
        repository.update(generic_id, &generic_qso, 3).unwrap();
        assert_metadata_invariant(&repository, generic_id, "M17");

        let dmr_id = repository.insert_dmr(&dmr_qso, &dmr, 4).unwrap();
        repository.update_ysf(dmr_id, &ysf_qso, &ysf, 5).unwrap();
        assert_metadata_invariant(&repository, dmr_id, "C4FM");
        repository.update_dmr(dmr_id, &dmr_qso, &dmr, 6).unwrap();
        assert_metadata_invariant(&repository, dmr_id, "DMR");

        let ft8_id = repository.insert_ft8(&ft8_qso, &ft8, 7).unwrap();
        repository.update_ysf(ft8_id, &ysf_qso, &ysf, 8).unwrap();
        assert_metadata_invariant(&repository, ft8_id, "C4FM");
        repository.update_ft8(ft8_id, &ft8_qso, &ft8, 9).unwrap();
        assert_metadata_invariant(&repository, ft8_id, "FT8");

        let dstar_id = repository.insert_dstar(&dstar_qso, &dstar, 10).unwrap();
        repository.update_ysf(dstar_id, &ysf_qso, &ysf, 11).unwrap();
        assert_metadata_invariant(&repository, dstar_id, "C4FM");
        repository
            .update_dstar(dstar_id, &dstar_qso, &dstar, 12)
            .unwrap();
        assert_metadata_invariant(&repository, dstar_id, "DSTAR");
    }

    #[test]
    fn filters_ft8_qsos_by_common_fields_snr_and_period() {
        let repository = QsoRepository::in_memory().unwrap();
        let first_qso = NewQso::new("PY2AAA", 1_700_000_000, 14_074_000, "FT8")
            .unwrap()
            .with_common_fields(CommonQsoFields {
                grid_locator: "GG66AA".into(),
                ..Default::default()
            })
            .unwrap();
        let first_metadata = Ft8Metadata::from_input(Ft8MetadataInput {
            snr_received_db: "-18".into(),
            ..Default::default()
        })
        .unwrap();
        repository
            .insert_ft8(&first_qso, &first_metadata, 1_700_000_001)
            .unwrap();

        let second_qso = NewQso::new("PY2BBB", 1_700_001_000, 7_074_000, "FT8")
            .unwrap()
            .with_common_fields(CommonQsoFields {
                grid_locator: "GG67BB".into(),
                ..Default::default()
            })
            .unwrap();
        let second_metadata = Ft8Metadata::from_input(Ft8MetadataInput {
            snr_received_db: "-5".into(),
            ..Default::default()
        })
        .unwrap();
        repository
            .insert_ft8(&second_qso, &second_metadata, 1_700_001_001)
            .unwrap();

        let result = repository
            .search_ft8(&Ft8Filter {
                callsign: Some("py2a".into()),
                grid: Some("gg66".into()),
                band: Some("20m".into()),
                minimum_snr_received_db: Some(-20),
                maximum_snr_received_db: Some(-10),
                start_utc: Some(1_699_999_999),
                end_utc: Some(1_700_000_001),
            })
            .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].callsign, "PY2AAA");
        assert_eq!(
            repository.search_ft8(&Ft8Filter::default()).unwrap().len(),
            2
        );
    }

    #[test]
    fn filters_dmr_qsos_by_metadata_and_route() {
        let repository = QsoRepository::in_memory().unwrap();
        let first_qso = NewQso::new("PU2AAA", 1_700_000_000, 438_500_000, "DMR").unwrap();
        let first_metadata = DmrMetadata::from_input(DmrMetadataInput {
            remote_dmr_id: "7241111".into(),
            local_dmr_id: "7240001".into(),
            talkgroup: "724".into(),
            timeslot: "1".into(),
            network: "BrandMeister".into(),
            call_type: "group".into(),
            access_type: "repeater".into(),
            repeater_callsign: "PY2XYZ".into(),
            ..Default::default()
        })
        .unwrap();
        repository
            .insert_dmr(&first_qso, &first_metadata, 1_700_000_001)
            .unwrap();

        let second_qso = NewQso::new("PU2BBB", 1_700_000_010, 439_000_000, "DMR").unwrap();
        let second_metadata = DmrMetadata::from_input(DmrMetadataInput {
            remote_dmr_id: "7242222".into(),
            talkgroup: "91".into(),
            timeslot: "2".into(),
            network: "TGIF".into(),
            call_type: "group".into(),
            access_type: "hotspot".into(),
            hotspot: "Portable node".into(),
            ..Default::default()
        })
        .unwrap();
        repository
            .insert_dmr(&second_qso, &second_metadata, 1_700_000_011)
            .unwrap();

        let by_id = repository
            .search_dmr(&DmrFilter {
                dmr_id: Some(7_241_111),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(by_id.len(), 1);
        assert_eq!(by_id[0].callsign, "PU2AAA");

        let by_route = repository
            .search_dmr(&DmrFilter {
                network: Some("tgif".into()),
                hotspot: Some("portable".into()),
                timeslot: Some(2),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(by_route.len(), 1);
        assert_eq!(by_route[0].callsign, "PU2BBB");

        let no_match = repository
            .search_dmr(&DmrFilter {
                talkgroup: Some(724),
                repeater: Some("UNKNOWN".into()),
                ..Default::default()
            })
            .unwrap();
        assert!(no_match.is_empty());
        assert_eq!(
            repository.search_dmr(&DmrFilter::default()).unwrap().len(),
            2
        );
    }

    #[test]
    fn paginates_search_with_total_stable_order_and_safe_limits() {
        let repository = QsoRepository::in_memory().unwrap();
        for (callsign, timestamp) in [
            ("PU2AAA", 1_700_000_000),
            ("PU2BBB", 1_700_000_010),
            ("PU2CCC", 1_700_000_010),
        ] {
            let qso = NewQso::new(callsign, timestamp, 145_500_000, "FM").unwrap();
            repository.insert(&qso, timestamp).unwrap();
        }

        let first = repository.search_page("pu2", 0, 2).unwrap();
        assert_eq!(first.total, 3);
        assert_eq!(first.offset, 0);
        assert_eq!(first.limit, 2);
        assert_eq!(first.items.len(), 2);
        assert_eq!(first.items[0].qso.callsign, "PU2CCC");
        assert_eq!(first.items[1].qso.callsign, "PU2BBB");

        let second = repository.search_page("pu2", 2, 2).unwrap();
        assert_eq!(second.total, 3);
        assert_eq!(second.items.len(), 1);
        assert_eq!(second.items[0].qso.callsign, "PU2AAA");

        let minimum = repository.search_page("", 0, 0).unwrap();
        assert_eq!(minimum.limit, 1);
        assert_eq!(minimum.items.len(), 1);
        let safe = repository.search_page("", usize::MAX, usize::MAX).unwrap();
        assert_eq!(safe.offset, i64::MAX as usize);
        assert_eq!(safe.limit, i64::MAX as usize);
        assert!(safe.items.is_empty());
    }

    #[test]
    fn paged_queries_join_mode_metadata() {
        let repository = QsoRepository::in_memory().unwrap();
        let dmr_qso = NewQso::new("PU2DMR", 1_700_000_000, 438_500_000, "DMR").unwrap();
        let dmr = DmrMetadata::from_input(DmrMetadataInput {
            talkgroup: "724".into(),
            network: "BrandMeister".into(),
            call_type: "group".into(),
            access_type: "simplex".into(),
            ..Default::default()
        })
        .unwrap();
        repository
            .insert_dmr(&dmr_qso, &dmr, 1_700_000_001)
            .unwrap();
        let ft8_qso = NewQso::new("PY2FT8", 1_700_000_010, 14_074_000, "FT8").unwrap();
        let ft8 = Ft8Metadata::from_input(Ft8MetadataInput {
            snr_received_db: "-12".into(),
            ..Default::default()
        })
        .unwrap();
        repository
            .insert_ft8(&ft8_qso, &ft8, 1_700_000_011)
            .unwrap();

        let all = repository.search_page("", 0, DEFAULT_PAGE_SIZE).unwrap();
        assert_eq!(all.total, 2);
        assert_eq!(all.items[0].metadata, ModeMetadata::Ft8(ft8.clone()));
        assert_eq!(all.items[1].metadata, ModeMetadata::Dmr(dmr.clone()));

        let dmr_page = repository
            .search_dmr_page(
                &DmrFilter {
                    talkgroup: Some(724),
                    ..Default::default()
                },
                0,
                10,
            )
            .unwrap();
        assert_eq!(dmr_page.total, 1);
        assert_eq!(dmr_page.items[0].metadata, ModeMetadata::Dmr(dmr));

        let ft8_page = repository
            .search_ft8_page(
                &Ft8Filter {
                    minimum_snr_received_db: Some(-15),
                    ..Default::default()
                },
                0,
                10,
            )
            .unwrap();
        assert_eq!(ft8_page.total, 1);
        assert_eq!(ft8_page.items[0].metadata, ModeMetadata::Ft8(ft8));
    }

    #[test]
    fn searches_by_callsign_or_mode() {
        let repository = QsoRepository::in_memory().unwrap();
        let dmr = NewQso::new("PU2XYZ", 1_700_000_000, 438_500_000, "DMR").unwrap();
        let ft8 = NewQso::new("PY2ABC", 1_700_000_010, 14_074_000, "FT8").unwrap();
        repository.insert(&dmr, 1_700_000_001).unwrap();
        repository.insert(&ft8, 1_700_000_011).unwrap();

        let callsign_results = repository.search("pu2").unwrap();
        assert_eq!(callsign_results.len(), 1);
        assert_eq!(callsign_results[0].callsign, "PU2XYZ");

        let mode_results = repository.search("ft8").unwrap();
        assert_eq!(mode_results.len(), 1);
        assert_eq!(mode_results[0].mode, "FT8");
        assert_eq!(repository.search("").unwrap().len(), 2);
    }
}
