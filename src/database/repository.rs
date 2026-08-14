use std::collections::{BTreeMap, HashSet};
use std::error::Error;
use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension, Result, Transaction};

use crate::adif::{
    domain_to_record, record_to_domain, AdifDocument, AdifField, AdifRecord, ImportedModeMetadata,
    ImportedQso,
};
use crate::domain::{DmrAccessType, DmrCallType, DmrMetadata, Ft8Metadata, NewQso, Qso};

use super::migrations;

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
pub struct Ft8Filter {
    pub callsign: Option<String>,
    pub grid: Option<String>,
    pub band: Option<String>,
    pub minimum_snr_received_db: Option<i16>,
    pub maximum_snr_received_db: Option<i16>,
    pub start_utc: Option<i64>,
    pub end_utc: Option<i64>,
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
    pub dmr: Option<DmrMetadata>,
    pub ft8: Option<Ft8Metadata>,
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

    pub fn backup_to(&self, destination: &Path) -> std::result::Result<(), Box<dyn Error>> {
        if destination.exists() {
            return Err("backup destination already exists".into());
        }
        let parent = destination
            .parent()
            .ok_or("backup destination has no parent directory")?;
        if !parent.is_dir() {
            return Err("backup destination directory does not exist".into());
        }

        self.connection
            .execute("VACUUM INTO ?1", params![destination.to_string_lossy()])?;
        let result = (|| -> std::result::Result<(), Box<dyn Error>> {
            let backup = Connection::open(destination)?;
            verify_connection_integrity(&backup)?;
            drop(backup);
            let file = std::fs::OpenOptions::new().read(true).open(destination)?;
            file.sync_all()?;
            set_private_file_permissions(destination)?;
            Ok(())
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(destination);
        }
        result
    }

    pub fn verify_integrity(&self) -> Result<()> {
        verify_connection_integrity(&self.connection)
    }

    pub fn export_adif(&self) -> std::result::Result<AdifDocument, Box<dyn Error>> {
        let mut records = Vec::new();
        for qso in self.list()? {
            let mode_metadata = if let Some(metadata) = self.get_dmr_metadata(qso.id)? {
                ImportedModeMetadata::Dmr(metadata)
            } else if let Some(metadata) = self.get_ft8_metadata(qso.id)? {
                ImportedModeMetadata::Ft8(metadata)
            } else {
                ImportedModeMetadata::Generic
            };
            let imported = ImportedQso {
                qso: new_qso_from_stored(&qso),
                mode_metadata,
                extra_fields: self.get_adif_extra_fields(qso.id)?,
            };
            records.push(domain_to_record(&imported)?);
        }

        Ok(AdifDocument {
            header: Some(AdifRecord {
                fields: vec![
                    AdifField {
                        name: "ADIF_VER".into(),
                        value: "3.1.4".into(),
                        data_type: None,
                    },
                    AdifField {
                        name: "PROGRAMID".into(),
                        value: "Digital Ham Radio Logbook".into(),
                        data_type: None,
                    },
                ],
            }),
            records,
        })
    }

    pub fn prepare_adif_import(
        &self,
        document: &AdifDocument,
    ) -> std::result::Result<AdifImportPlan, Box<dyn Error>> {
        let mut identities = existing_qso_identities(&self.connection)?;
        let mut preview = AdifImportPreview {
            total: document.records.len(),
            ..Default::default()
        };
        let mut qsos = Vec::new();

        for (index, record) in document.records.iter().enumerate() {
            let imported_qso = match record_to_domain(record) {
                Ok(imported_qso) => imported_qso,
                Err(error) => {
                    preview.invalid += 1;
                    if preview.invalid_details.len() < 20 {
                        preview
                            .invalid_details
                            .push(format!("Record {} — {error}", index + 1));
                    }
                    continue;
                }
            };
            *preview
                .modes
                .entry(imported_qso.qso.mode.clone())
                .or_default() += 1;
            *preview
                .bands
                .entry(
                    imported_qso
                        .qso
                        .band
                        .clone()
                        .unwrap_or_else(|| "Unknown".to_owned()),
                )
                .or_default() += 1;
            preview.earliest_utc = Some(
                preview
                    .earliest_utc
                    .map_or(imported_qso.qso.datetime_start_utc, |current| {
                        current.min(imported_qso.qso.datetime_start_utc)
                    }),
            );
            preview.latest_utc = Some(
                preview
                    .latest_utc
                    .map_or(imported_qso.qso.datetime_start_utc, |current| {
                        current.max(imported_qso.qso.datetime_start_utc)
                    }),
            );
            if !identities.insert(QsoIdentity::from(&imported_qso.qso)) {
                preview.duplicates += 1;
                continue;
            }
            preview.new_qsos += 1;
            qsos.push(imported_qso);
        }

        if preview.invalid > preview.invalid_details.len() {
            preview.invalid_details.push(format!(
                "… {} additional invalid record(s) omitted",
                preview.invalid - preview.invalid_details.len()
            ));
        }

        Ok(AdifImportPlan { preview, qsos })
    }

    pub fn import_adif_plan(
        &self,
        plan: AdifImportPlan,
        now_utc: i64,
    ) -> std::result::Result<AdifImportReport, Box<dyn Error>> {
        let transaction = self.connection.unchecked_transaction()?;
        for imported_qso in &plan.qsos {
            let qso_id = insert_qso(&transaction, &imported_qso.qso, now_utc)?;
            match &imported_qso.mode_metadata {
                ImportedModeMetadata::Dmr(metadata) => {
                    insert_dmr_metadata(&transaction, qso_id, metadata)?;
                }
                ImportedModeMetadata::Ft8(metadata) => {
                    insert_ft8_metadata(&transaction, qso_id, metadata)?;
                }
                ImportedModeMetadata::Generic => {}
            }
            insert_adif_extra_fields(&transaction, qso_id, &imported_qso.extra_fields)?;
        }
        transaction.commit()?;
        Ok(AdifImportReport {
            imported: plan.preview.new_qsos,
            duplicates_skipped: plan.preview.duplicates,
        })
    }

    pub fn import_adif(
        &self,
        document: &AdifDocument,
        now_utc: i64,
    ) -> std::result::Result<AdifImportReport, Box<dyn Error>> {
        for (index, record) in document.records.iter().enumerate() {
            record_to_domain(record)
                .map_err(|error| format!("ADIF record {}: {error}", index + 1))?;
        }
        let plan = self.prepare_adif_import(document)?;
        self.import_adif_plan(plan, now_utc)
    }

    pub fn get_adif_extra_fields(&self, qso_id: i64) -> Result<Vec<AdifField>> {
        let mut statement = self.connection.prepare(
            "SELECT name, value, data_type
             FROM adif_extra_fields
             WHERE qso_id = ?1
             ORDER BY field_order",
        )?;
        let rows = statement.query_map(params![qso_id], |row| {
            Ok(AdifField {
                name: row.get(0)?,
                value: row.get(1)?,
                data_type: row.get(2)?,
            })
        })?;
        rows.collect()
    }

    pub fn insert(&self, qso: &NewQso, now_utc: i64) -> Result<i64> {
        insert_qso(&self.connection, qso, now_utc)
    }

    pub fn insert_dmr(&self, qso: &NewQso, metadata: &DmrMetadata, now_utc: i64) -> Result<i64> {
        let transaction = self.connection.unchecked_transaction()?;
        let qso_id = insert_qso(&transaction, qso, now_utc)?;
        insert_dmr_metadata(&transaction, qso_id, metadata)?;
        transaction.commit()?;
        Ok(qso_id)
    }

    pub fn insert_ft8(&self, qso: &NewQso, metadata: &Ft8Metadata, now_utc: i64) -> Result<i64> {
        let transaction = self.connection.unchecked_transaction()?;
        let qso_id = insert_qso(&transaction, qso, now_utc)?;
        insert_ft8_metadata(&transaction, qso_id, metadata)?;
        transaction.commit()?;
        Ok(qso_id)
    }

    pub fn update_ft8(
        &self,
        qso_id: i64,
        qso: &NewQso,
        metadata: &Ft8Metadata,
        now_utc: i64,
    ) -> Result<bool> {
        let transaction = self.connection.unchecked_transaction()?;
        let changed = update_qso(&transaction, qso_id, qso, now_utc)?;
        if !changed {
            transaction.rollback()?;
            return Ok(false);
        }
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
        let transaction = self.connection.unchecked_transaction()?;
        let changed = update_qso(&transaction, qso_id, qso, now_utc)?;
        if !changed {
            transaction.rollback()?;
            return Ok(false);
        }
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

    pub fn search_page(&self, query: &str, offset: usize, limit: usize) -> Result<QsoPage> {
        let pattern = trimmed_pattern(Some(query));
        let total: i64 = self.connection.query_row(
            "SELECT COUNT(*) FROM qsos q
             WHERE (?1 IS NULL OR q.callsign LIKE ?1 COLLATE NOCASE
                    OR q.mode LIKE ?1 COLLATE NOCASE)",
            params![pattern],
            |row| row.get(0),
        )?;
        let (offset, limit, sql_offset, sql_limit) = normalize_page(offset, limit);
        let mut statement = self.connection.prepare(&format!(
            "{LIST_ITEM_SELECT}
             WHERE (?1 IS NULL OR q.callsign LIKE ?1 COLLATE NOCASE
                    OR q.mode LIKE ?1 COLLATE NOCASE)
             ORDER BY q.datetime_start_utc DESC, q.id DESC
             LIMIT ?2 OFFSET ?3"
        ))?;
        let items = statement
            .query_map(params![pattern, sql_limit, sql_offset], map_qso_list_item)?
            .collect::<Result<Vec<_>>>()?;
        Ok(QsoPage {
            items,
            total: count_to_usize(total),
            offset,
            limit,
        })
    }

    pub fn search_dmr_page(
        &self,
        filter: &DmrFilter,
        offset: usize,
        limit: usize,
    ) -> Result<QsoPage> {
        let network = trimmed_value(filter.network.as_deref()).map(contains_pattern);
        let repeater = trimmed_value(filter.repeater.as_deref()).map(contains_pattern);
        let hotspot = trimmed_value(filter.hotspot.as_deref()).map(contains_pattern);
        let filter_params = params![
            filter.dmr_id.map(i64::from),
            filter.talkgroup.map(i64::from),
            network,
            repeater,
            hotspot,
            filter.timeslot.map(i64::from),
        ];
        let total: i64 = self.connection.query_row(
            "SELECT COUNT(*) FROM qsos q
             JOIN dmr_metadata d ON d.qso_id = q.id
             JOIN digital_routes r ON r.qso_id = q.id
             WHERE (?1 IS NULL OR d.remote_dmr_id = ?1 OR d.local_dmr_id = ?1)
               AND (?2 IS NULL OR d.talkgroup = ?2)
               AND (?3 IS NULL OR r.network LIKE ?3 COLLATE NOCASE)
               AND (?4 IS NULL OR r.repeater_callsign LIKE ?4 COLLATE NOCASE)
               AND (?5 IS NULL OR r.hotspot LIKE ?5 COLLATE NOCASE)
               AND (?6 IS NULL OR d.timeslot = ?6)",
            filter_params,
            |row| row.get(0),
        )?;
        let (offset, limit, sql_offset, sql_limit) = normalize_page(offset, limit);
        let mut statement = self.connection.prepare(&format!(
            "{LIST_ITEM_SELECT}
             WHERE d.qso_id IS NOT NULL
               AND (?1 IS NULL OR d.remote_dmr_id = ?1 OR d.local_dmr_id = ?1)
               AND (?2 IS NULL OR d.talkgroup = ?2)
               AND (?3 IS NULL OR r.network LIKE ?3 COLLATE NOCASE)
               AND (?4 IS NULL OR r.repeater_callsign LIKE ?4 COLLATE NOCASE)
               AND (?5 IS NULL OR r.hotspot LIKE ?5 COLLATE NOCASE)
               AND (?6 IS NULL OR d.timeslot = ?6)
             ORDER BY q.datetime_start_utc DESC, q.id DESC
             LIMIT ?7 OFFSET ?8"
        ))?;
        let items = statement
            .query_map(
                params![
                    filter.dmr_id.map(i64::from),
                    filter.talkgroup.map(i64::from),
                    network,
                    repeater,
                    hotspot,
                    filter.timeslot.map(i64::from),
                    sql_limit,
                    sql_offset,
                ],
                map_qso_list_item,
            )?
            .collect::<Result<Vec<_>>>()?;
        Ok(QsoPage {
            items,
            total: count_to_usize(total),
            offset,
            limit,
        })
    }

    pub fn search_ft8_page(
        &self,
        filter: &Ft8Filter,
        offset: usize,
        limit: usize,
    ) -> Result<QsoPage> {
        let callsign = trimmed_pattern(filter.callsign.as_deref());
        let grid = trimmed_pattern(filter.grid.as_deref());
        let band = trimmed_pattern(filter.band.as_deref());
        let total: i64 = self.connection.query_row(
            "SELECT COUNT(*) FROM qsos q
             JOIN ft8_metadata f ON f.qso_id = q.id
             WHERE (?1 IS NULL OR q.callsign LIKE ?1 COLLATE NOCASE)
               AND (?2 IS NULL OR q.grid_locator LIKE ?2 COLLATE NOCASE)
               AND (?3 IS NULL OR q.band LIKE ?3 COLLATE NOCASE)
               AND (?4 IS NULL OR f.snr_received_db >= ?4)
               AND (?5 IS NULL OR f.snr_received_db <= ?5)
               AND (?6 IS NULL OR q.datetime_start_utc >= ?6)
               AND (?7 IS NULL OR q.datetime_start_utc <= ?7)",
            params![
                callsign,
                grid,
                band,
                filter.minimum_snr_received_db,
                filter.maximum_snr_received_db,
                filter.start_utc,
                filter.end_utc,
            ],
            |row| row.get(0),
        )?;
        let (offset, limit, sql_offset, sql_limit) = normalize_page(offset, limit);
        let mut statement = self.connection.prepare(&format!(
            "{LIST_ITEM_SELECT}
             WHERE f.qso_id IS NOT NULL
               AND (?1 IS NULL OR q.callsign LIKE ?1 COLLATE NOCASE)
               AND (?2 IS NULL OR q.grid_locator LIKE ?2 COLLATE NOCASE)
               AND (?3 IS NULL OR q.band LIKE ?3 COLLATE NOCASE)
               AND (?4 IS NULL OR f.snr_received_db >= ?4)
               AND (?5 IS NULL OR f.snr_received_db <= ?5)
               AND (?6 IS NULL OR q.datetime_start_utc >= ?6)
               AND (?7 IS NULL OR q.datetime_start_utc <= ?7)
             ORDER BY q.datetime_start_utc DESC, q.id DESC
             LIMIT ?8 OFFSET ?9"
        ))?;
        let items = statement
            .query_map(
                params![
                    callsign,
                    grid,
                    band,
                    filter.minimum_snr_received_db,
                    filter.maximum_snr_received_db,
                    filter.start_utc,
                    filter.end_utc,
                    sql_limit,
                    sql_offset,
                ],
                map_qso_list_item,
            )?
            .collect::<Result<Vec<_>>>()?;
        Ok(QsoPage {
            items,
            total: count_to_usize(total),
            offset,
            limit,
        })
    }

    pub fn search_ft8(&self, filter: &Ft8Filter) -> Result<Vec<Qso>> {
        let callsign = trimmed_pattern(filter.callsign.as_deref());
        let grid = trimmed_pattern(filter.grid.as_deref());
        let band = trimmed_pattern(filter.band.as_deref());
        let mut statement = self.connection.prepare(
            "SELECT q.id, q.callsign, q.datetime_start_utc, q.datetime_end_utc,
                    q.frequency_hz, q.band, q.mode, q.submode, q.rst_sent,
                    q.rst_received, q.grid_locator, q.name, q.qth, q.notes,
                    q.created_at_utc, q.updated_at_utc
             FROM qsos q
             JOIN ft8_metadata f ON f.qso_id = q.id
             WHERE (?1 IS NULL OR q.callsign LIKE ?1 COLLATE NOCASE)
               AND (?2 IS NULL OR q.grid_locator LIKE ?2 COLLATE NOCASE)
               AND (?3 IS NULL OR q.band LIKE ?3 COLLATE NOCASE)
               AND (?4 IS NULL OR f.snr_received_db >= ?4)
               AND (?5 IS NULL OR f.snr_received_db <= ?5)
               AND (?6 IS NULL OR q.datetime_start_utc >= ?6)
               AND (?7 IS NULL OR q.datetime_start_utc <= ?7)
             ORDER BY q.datetime_start_utc DESC, q.id DESC",
        )?;
        let rows = statement.query_map(
            params![
                callsign,
                grid,
                band,
                filter.minimum_snr_received_db,
                filter.maximum_snr_received_db,
                filter.start_utc,
                filter.end_utc,
            ],
            map_qso,
        )?;
        rows.collect()
    }

    pub fn search_dmr(&self, filter: &DmrFilter) -> Result<Vec<Qso>> {
        let network = filter
            .network
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let repeater = filter
            .repeater
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let hotspot = filter
            .hotspot
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());

        let mut statement = self.connection.prepare(
            "SELECT q.id, q.callsign, q.datetime_start_utc, q.datetime_end_utc,
                    q.frequency_hz, q.band, q.mode, q.submode, q.rst_sent,
                    q.rst_received, q.grid_locator, q.name, q.qth, q.notes,
                    q.created_at_utc, q.updated_at_utc
             FROM qsos q
             JOIN dmr_metadata d ON d.qso_id = q.id
             JOIN digital_routes r ON r.qso_id = q.id
             WHERE (?1 IS NULL OR d.remote_dmr_id = ?1 OR d.local_dmr_id = ?1)
               AND (?2 IS NULL OR d.talkgroup = ?2)
               AND (?3 IS NULL OR r.network LIKE ?3 COLLATE NOCASE)
               AND (?4 IS NULL OR r.repeater_callsign LIKE ?4 COLLATE NOCASE)
               AND (?5 IS NULL OR r.hotspot LIKE ?5 COLLATE NOCASE)
               AND (?6 IS NULL OR d.timeslot = ?6)
             ORDER BY q.datetime_start_utc DESC, q.id DESC",
        )?;

        let rows = statement.query_map(
            params![
                filter.dmr_id.map(i64::from),
                filter.talkgroup.map(i64::from),
                network.map(contains_pattern),
                repeater.map(contains_pattern),
                hotspot.map(contains_pattern),
                filter.timeslot.map(i64::from),
            ],
            map_qso,
        )?;
        rows.collect()
    }

    pub fn search(&self, query: &str) -> Result<Vec<Qso>> {
        let query = query.trim();
        if query.is_empty() {
            return self.list();
        }

        let pattern = format!("%{query}%");
        let mut statement = self.connection.prepare(
            "SELECT id, callsign, datetime_start_utc, datetime_end_utc,
                    frequency_hz, band, mode, submode, rst_sent, rst_received,
                    grid_locator, name, qth, notes, created_at_utc, updated_at_utc
             FROM qsos
             WHERE callsign LIKE ?1 COLLATE NOCASE OR mode LIKE ?1 COLLATE NOCASE
             ORDER BY datetime_start_utc DESC, id DESC",
        )?;
        let rows = statement.query_map(params![pattern], map_qso)?;
        rows.collect()
    }

    pub fn list(&self) -> Result<Vec<Qso>> {
        let mut statement = self.connection.prepare(
            "SELECT id, callsign, datetime_start_utc, datetime_end_utc,
                    frequency_hz, band, mode, submode, rst_sent, rst_received,
                    grid_locator, name, qth, notes, created_at_utc, updated_at_utc
             FROM qsos
             ORDER BY datetime_start_utc DESC, id DESC",
        )?;

        let rows = statement.query_map([], map_qso)?;

        rows.collect()
    }
}

fn existing_qso_identities(connection: &Connection) -> Result<HashSet<QsoIdentity>> {
    let mut statement =
        connection.prepare("SELECT callsign, datetime_start_utc, frequency_hz, mode FROM qsos")?;
    let rows = statement.query_map([], |row| {
        let callsign: String = row.get(0)?;
        let mode: String = row.get(3)?;
        Ok(QsoIdentity {
            callsign: callsign.trim().to_uppercase(),
            datetime_start_utc: row.get(1)?,
            frequency_hz: row.get(2)?,
            mode: mode.trim().to_uppercase(),
        })
    })?;
    rows.collect()
}

#[cfg(unix)]
fn set_private_file_permissions(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = std::fs::metadata(path)
        .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?
        .permissions();
    permissions.set_mode(0o600);
    std::fs::set_permissions(path, permissions)
        .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
}

#[cfg(not(unix))]
fn set_private_file_permissions(_path: &Path) -> Result<()> {
    Ok(())
}

fn verify_connection_integrity(connection: &Connection) -> Result<()> {
    let quick_check: String = connection.query_row("PRAGMA quick_check", [], |row| row.get(0))?;
    if quick_check != "ok" {
        return Err(rusqlite::Error::InvalidParameterName(format!(
            "database integrity check failed: {quick_check}"
        )));
    }

    let foreign_key_violation: Option<String> = connection
        .query_row(
            "SELECT printf('%s row %s references %s', \"table\", rowid, parent) FROM pragma_foreign_key_check LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()?;
    if let Some(violation) = foreign_key_violation {
        return Err(rusqlite::Error::InvalidParameterName(format!(
            "database foreign key check failed: {violation}"
        )));
    }
    Ok(())
}

fn new_qso_from_stored(qso: &Qso) -> NewQso {
    NewQso {
        callsign: qso.callsign.clone(),
        datetime_start_utc: qso.datetime_start_utc,
        frequency_hz: qso.frequency_hz,
        band: qso.band.clone(),
        mode: qso.mode.clone(),
        rst_sent: qso.rst_sent.clone(),
        rst_received: qso.rst_received.clone(),
        grid_locator: qso.grid_locator.clone(),
        name: qso.name.clone(),
        qth: qso.qth.clone(),
        notes: qso.notes.clone(),
    }
}

fn trimmed_value(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

fn trimmed_pattern(value: Option<&str>) -> Option<String> {
    trimmed_value(value).map(contains_pattern)
}

fn normalize_page(offset: usize, limit: usize) -> (usize, usize, i64, i64) {
    let maximum = i64::MAX as usize;
    let offset = offset.min(maximum);
    let limit = limit.max(1).min(maximum);
    (offset, limit, offset as i64, limit as i64)
}

fn count_to_usize(count: i64) -> usize {
    usize::try_from(count).unwrap_or(usize::MAX)
}

fn contains_pattern(value: &str) -> String {
    format!("%{value}%")
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

fn insert_adif_extra_fields(
    transaction: &Transaction<'_>,
    qso_id: i64,
    fields: &[AdifField],
) -> Result<()> {
    for (index, field) in fields.iter().enumerate() {
        transaction.execute(
            "INSERT INTO adif_extra_fields(qso_id, field_order, name, value, data_type)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                qso_id,
                index as i64,
                field.name,
                field.value,
                field.data_type
            ],
        )?;
    }
    Ok(())
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

const LIST_ITEM_SELECT: &str = "
    SELECT q.id, q.callsign, q.datetime_start_utc, q.datetime_end_utc,
           q.frequency_hz, q.band, q.mode, q.submode, q.rst_sent,
           q.rst_received, q.grid_locator, q.name, q.qth, q.notes,
           q.created_at_utc, q.updated_at_utc,
           d.qso_id, d.remote_dmr_id, d.local_dmr_id, d.talkgroup,
           d.timeslot, d.color_code, r.network, d.call_type, r.access_type,
           r.repeater_callsign, r.hotspot, d.rx_frequency_hz,
           d.tx_frequency_hz, d.notes,
           f.qso_id, f.snr_sent_db, f.snr_received_db, f.power_watts,
           f.audio_frequency_hz, f.source_software, f.protocol, f.final_message
    FROM qsos q
    LEFT JOIN dmr_metadata d ON d.qso_id = q.id
    LEFT JOIN digital_routes r ON r.qso_id = q.id
    LEFT JOIN ft8_metadata f ON f.qso_id = q.id";

fn map_qso_list_item(row: &rusqlite::Row<'_>) -> Result<QsoListItem> {
    let dmr = if row.get::<_, Option<i64>>(16)?.is_some() {
        let call_type: String = row.get(23)?;
        let access_type: String = row.get(24)?;
        Some(DmrMetadata {
            remote_dmr_id: row.get(17)?,
            local_dmr_id: row.get(18)?,
            talkgroup: row.get(19)?,
            timeslot: row.get(20)?,
            color_code: row.get(21)?,
            network: row.get(22)?,
            call_type: parse_stored_call_type(&call_type)?,
            access_type: parse_stored_access_type(&access_type)?,
            repeater_callsign: row.get(25)?,
            hotspot: row.get(26)?,
            rx_frequency_hz: row.get(27)?,
            tx_frequency_hz: row.get(28)?,
            notes: row.get(29)?,
        })
    } else {
        None
    };
    let ft8 = if row.get::<_, Option<i64>>(30)?.is_some() {
        Some(Ft8Metadata {
            snr_sent_db: row.get(31)?,
            snr_received_db: row.get(32)?,
            power_watts: row.get(33)?,
            audio_frequency_hz: row.get(34)?,
            source_software: row.get(35)?,
            protocol: row.get(36)?,
            final_message: row.get(37)?,
        })
    } else {
        None
    };
    Ok(QsoListItem {
        qso: map_qso(row)?,
        dmr,
        ft8,
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
    use crate::adif::{export, parse};
    use crate::domain::{CommonQsoFields, DmrMetadataInput, Ft8MetadataInput};

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
        assert!(repository.backup_to(&destination).is_err());
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
    fn exports_database_to_parseable_adif_with_metadata_and_extras() {
        let repository = QsoRepository::in_memory().unwrap();
        let input = parse(
            "<CALL:6>PY2ABC<QSO_DATE:8>20231114<TIME_ON:6>221320<FREQ:6>14.074\
             <MODE:3>FT8<SNR:3>-18<APP_VENDOR_FIELD:5:S>value<EOR>\
             <CALL:6>PU2XYZ<QSO_DATE:8>20231114<TIME_ON:6>221321<FREQ:7>438.500\
             <MODE:3>DMR<APP_DHRL_TALKGROUP:3>724<APP_DHRL_CALL_TYPE:5>group\
             <APP_DHRL_ACCESS_TYPE:7>simplex<EOR>",
        )
        .unwrap();
        repository.import_adif(&input, 1_700_000_100).unwrap();

        let document = repository.export_adif().unwrap();
        assert_eq!(
            document.header.as_ref().unwrap().get("PROGRAMID"),
            Some("Digital Ham Radio Logbook")
        );
        let encoded = export(&document);
        let reparsed = parse(&encoded).unwrap();
        assert_eq!(reparsed.records.len(), 2);
        let ft8 = reparsed
            .records
            .iter()
            .find(|record| record.get("MODE") == Some("FT8"))
            .unwrap();
        assert_eq!(ft8.get("SNR"), Some("-18"));
        assert_eq!(ft8.get("APP_VENDOR_FIELD"), Some("value"));
        let dmr = reparsed
            .records
            .iter()
            .find(|record| record.get("MODE") == Some("DMR"))
            .unwrap();
        assert_eq!(dmr.get("APP_DHRL_TALKGROUP"), Some("724"));
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
    fn rejects_a_non_sqlite_database_without_replacing_it() {
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("dhrl-corrupt-test-{suffix}.sqlite3"));
        std::fs::write(&path, b"not a sqlite database").unwrap();

        assert!(QsoRepository::open(&path).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"not a sqlite database");
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn reports_integrity_for_a_healthy_database() {
        let repository = QsoRepository::in_memory().unwrap();
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
        assert_eq!(all.items[0].ft8, Some(ft8.clone()));
        assert!(all.items[0].dmr.is_none());
        assert_eq!(all.items[1].dmr, Some(dmr.clone()));
        assert!(all.items[1].ft8.is_none());

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
        assert_eq!(dmr_page.items[0].dmr, Some(dmr));

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
        assert_eq!(ft8_page.items[0].ft8, Some(ft8));
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
