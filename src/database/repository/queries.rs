use rusqlite::{params, Result};

use crate::domain::{
    DStarMetadata, DmrMetadata, Ft8Metadata, ModeMetadata, Qso, YsfAccessType, YsfMetadata,
};

use super::{
    map_qso, parse_stored_access_type, parse_stored_call_type, DmrFilter, DstarFilter, Ft8Filter,
    QsoListItem, QsoPage, QsoRepository, YsfFilter,
};

impl QsoRepository {
    pub(super) fn list_items(&self) -> Result<Vec<QsoListItem>> {
        let mut statement = self.connection.prepare(&format!(
            "{LIST_ITEM_SELECT}\n             ORDER BY q.datetime_start_utc DESC, q.id DESC"
        ))?;
        let items = statement
            .query_map([], map_qso_list_item)?
            .collect::<Result<Vec<_>>>()?;
        Ok(items)
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

    pub fn search_ysf_page(
        &self,
        filter: &YsfFilter,
        offset: usize,
        limit: usize,
    ) -> Result<QsoPage> {
        let room = trimmed_pattern(filter.room.as_deref());
        let wires_x_node = trimmed_pattern(filter.wires_x_node.as_deref());
        let dg_id = filter.dg_id.map(i64::from);
        let total: i64 = self.connection.query_row(
            "SELECT COUNT(*) FROM qsos q
             JOIN ysf_metadata y ON y.qso_id = q.id
             WHERE (?1 IS NULL OR y.room LIKE ?1 COLLATE NOCASE)
               AND (?2 IS NULL OR y.wires_x_node LIKE ?2 COLLATE NOCASE)
               AND (?3 IS NULL OR y.tx_dg_id = ?3 OR y.rx_dg_id = ?3)",
            params![room, wires_x_node, dg_id],
            |row| row.get(0),
        )?;
        let (offset, limit, sql_offset, sql_limit) = normalize_page(offset, limit);
        let mut statement = self.connection.prepare(&format!(
            "{LIST_ITEM_SELECT}
             WHERE y.qso_id IS NOT NULL
               AND (?1 IS NULL OR y.room LIKE ?1 COLLATE NOCASE)
               AND (?2 IS NULL OR y.wires_x_node LIKE ?2 COLLATE NOCASE)
               AND (?3 IS NULL OR y.tx_dg_id = ?3 OR y.rx_dg_id = ?3)
             ORDER BY q.datetime_start_utc DESC, q.id DESC
             LIMIT ?4 OFFSET ?5"
        ))?;
        let items = statement
            .query_map(
                params![room, wires_x_node, dg_id, sql_limit, sql_offset],
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

    pub fn search_dstar_page(
        &self,
        filter: &DstarFilter,
        offset: usize,
        limit: usize,
    ) -> Result<QsoPage> {
        let reflector = trimmed_pattern(filter.reflector.as_deref());
        let module = trimmed_pattern(filter.module.as_deref());
        let rpt1 = trimmed_pattern(filter.rpt1.as_deref());
        let total: i64 = self.connection.query_row(
            "SELECT COUNT(*) FROM qsos q
             JOIN dstar_metadata ds ON ds.qso_id = q.id
             WHERE (?1 IS NULL OR ds.reflector LIKE ?1 COLLATE NOCASE)
               AND (?2 IS NULL OR ds.module LIKE ?2 COLLATE NOCASE)
               AND (?3 IS NULL OR ds.rpt1 LIKE ?3 COLLATE NOCASE)",
            params![reflector, module, rpt1],
            |row| row.get(0),
        )?;
        let (offset, limit, sql_offset, sql_limit) = normalize_page(offset, limit);
        let mut statement = self.connection.prepare(&format!(
            "{LIST_ITEM_SELECT}
             WHERE ds.qso_id IS NOT NULL
               AND (?1 IS NULL OR ds.reflector LIKE ?1 COLLATE NOCASE)
               AND (?2 IS NULL OR ds.module LIKE ?2 COLLATE NOCASE)
               AND (?3 IS NULL OR ds.rpt1 LIKE ?3 COLLATE NOCASE)
             ORDER BY q.datetime_start_utc DESC, q.id DESC
             LIMIT ?4 OFFSET ?5"
        ))?;
        let items = statement
            .query_map(
                params![reflector, module, rpt1, sql_limit, sql_offset],
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

    pub fn search_ysf(&self, filter: &YsfFilter) -> Result<Vec<Qso>> {
        let room = trimmed_pattern(filter.room.as_deref());
        let wires_x_node = trimmed_pattern(filter.wires_x_node.as_deref());
        let mut statement = self.connection.prepare(
            "SELECT q.id, q.callsign, q.datetime_start_utc, q.datetime_end_utc,
                    q.frequency_hz, q.band, q.mode, q.submode, q.rst_sent,
                    q.rst_received, q.grid_locator, q.name, q.qth, q.notes,
                    q.created_at_utc, q.updated_at_utc
             FROM qsos q
             JOIN ysf_metadata y ON y.qso_id = q.id
             WHERE (?1 IS NULL OR y.room LIKE ?1 COLLATE NOCASE)
               AND (?2 IS NULL OR y.wires_x_node LIKE ?2 COLLATE NOCASE)
               AND (?3 IS NULL OR y.tx_dg_id = ?3 OR y.rx_dg_id = ?3)
             ORDER BY q.datetime_start_utc DESC, q.id DESC",
        )?;
        let rows = statement.query_map(
            params![room, wires_x_node, filter.dg_id.map(i64::from)],
            map_qso,
        )?;
        rows.collect()
    }

    pub fn search_dstar(&self, filter: &DstarFilter) -> Result<Vec<Qso>> {
        let reflector = trimmed_pattern(filter.reflector.as_deref());
        let module = trimmed_pattern(filter.module.as_deref());
        let rpt1 = trimmed_pattern(filter.rpt1.as_deref());
        let mut statement = self.connection.prepare(
            "SELECT q.id, q.callsign, q.datetime_start_utc, q.datetime_end_utc,
                    q.frequency_hz, q.band, q.mode, q.submode, q.rst_sent,
                    q.rst_received, q.grid_locator, q.name, q.qth, q.notes,
                    q.created_at_utc, q.updated_at_utc
             FROM qsos q
             JOIN dstar_metadata ds ON ds.qso_id = q.id
             WHERE (?1 IS NULL OR ds.reflector LIKE ?1 COLLATE NOCASE)
               AND (?2 IS NULL OR ds.module LIKE ?2 COLLATE NOCASE)
               AND (?3 IS NULL OR ds.rpt1 LIKE ?3 COLLATE NOCASE)
             ORDER BY q.datetime_start_utc DESC, q.id DESC",
        )?;
        let rows = statement.query_map(params![reflector, module, rpt1], map_qso)?;
        rows.collect()
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
           f.audio_frequency_hz, f.source_software, f.protocol, f.final_message,
           ds.qso_id, ds.reflector, ds.module, ds.mycall, ds.urcall,
           ds.rpt1, ds.rpt2, ds.notes,
           y.qso_id, y.room, y.wires_x_node, y.repeater, y.network,
           y.access_type, y.tx_dg_id, y.rx_dg_id, y.notes
    FROM qsos q
    LEFT JOIN dmr_metadata d ON d.qso_id = q.id
    LEFT JOIN digital_routes r ON r.qso_id = q.id
    LEFT JOIN ft8_metadata f ON f.qso_id = q.id
    LEFT JOIN dstar_metadata ds ON ds.qso_id = q.id
    LEFT JOIN ysf_metadata y ON y.qso_id = q.id";

const DMR_OFFSET: usize = 16;
const FT8_OFFSET: usize = 30;
const DSTAR_OFFSET: usize = 38;
const YSF_OFFSET: usize = 46;

fn map_qso_list_item(row: &rusqlite::Row<'_>) -> Result<QsoListItem> {
    let qso = map_qso(row)?;
    let has_dmr = row.get::<_, Option<i64>>(DMR_OFFSET)?.is_some();
    let has_ft8 = row.get::<_, Option<i64>>(FT8_OFFSET)?.is_some();
    let has_dstar = row.get::<_, Option<i64>>(DSTAR_OFFSET)?.is_some();
    let has_ysf = row.get::<_, Option<i64>>(YSF_OFFSET)?.is_some();
    let metadata_count =
        usize::from(has_dmr) + usize::from(has_ft8) + usize::from(has_dstar) + usize::from(has_ysf);
    if metadata_count > 1 {
        return Err(invalid_metadata(&qso, "multiple specialized metadata rows"));
    }

    let metadata = if has_dmr {
        let call_type: String = row.get(23)?;
        let access_type: String = row.get(24)?;
        ModeMetadata::Dmr(DmrMetadata {
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
    } else if has_ft8 {
        ModeMetadata::Ft8(Ft8Metadata {
            snr_sent_db: row.get(31)?,
            snr_received_db: row.get(32)?,
            power_watts: row.get(33)?,
            audio_frequency_hz: row.get(34)?,
            source_software: row.get(35)?,
            protocol: row.get(36)?,
            final_message: row.get(37)?,
        })
    } else if has_dstar {
        ModeMetadata::Dstar(DStarMetadata {
            reflector: row.get(39)?,
            module: row.get(40)?,
            mycall: row.get(41)?,
            urcall: row.get(42)?,
            rpt1: row.get(43)?,
            rpt2: row.get(44)?,
            notes: row.get(45)?,
        })
    } else if has_ysf {
        let access_type: String = row.get(YSF_OFFSET + 5)?;
        ModeMetadata::Ysf(YsfMetadata {
            room: row.get(YSF_OFFSET + 1)?,
            wires_x_node: row.get(YSF_OFFSET + 2)?,
            repeater: row.get(YSF_OFFSET + 3)?,
            network: row.get(YSF_OFFSET + 4)?,
            access_type: parse_ysf_access_type(&access_type)?,
            tx_dg_id: row.get(YSF_OFFSET + 6)?,
            rx_dg_id: row.get(YSF_OFFSET + 7)?,
            notes: row.get(YSF_OFFSET + 8)?,
        })
    } else {
        ModeMetadata::Generic
    };
    if !metadata.is_compatible_with(&qso.mode) {
        return Err(invalid_metadata(&qso, "mode and metadata are incompatible"));
    }
    Ok(QsoListItem { qso, metadata })
}

fn parse_ysf_access_type(value: &str) -> Result<YsfAccessType> {
    value.parse().map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(
            YSF_OFFSET + 5,
            rusqlite::types::Type::Text,
            Box::new(error),
        )
    })
}

fn invalid_metadata(qso: &Qso, reason: &str) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        6,
        rusqlite::types::Type::Text,
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("QSO {} ({}) has {reason}", qso.id, qso.mode),
        )
        .into(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::NewQso;

    fn metadata(reflector: &str, module: &str, rpt1: &str) -> DStarMetadata {
        DStarMetadata {
            reflector: Some(reflector.into()),
            module: Some(module.into()),
            mycall: Some("PY2LOCAL G".into()),
            urcall: Some("CQCQCQ".into()),
            rpt1: Some(rpt1.into()),
            rpt2: Some("PY2RPT G".into()),
            notes: "D-STAR test".into(),
        }
    }

    fn ysf_metadata(room: &str, node: &str, tx_dg_id: u8, rx_dg_id: u8) -> YsfMetadata {
        YsfMetadata {
            room: Some(room.into()),
            wires_x_node: Some(node.into()),
            repeater: Some("PY2RPT".into()),
            network: Some("YSF".into()),
            access_type: YsfAccessType::Repeater,
            tx_dg_id: Some(tx_dg_id),
            rx_dg_id: Some(rx_dg_id),
            notes: "YSF test".into(),
        }
    }

    #[test]
    fn list_items_hydrates_dstar_metadata_without_affecting_generic_qsos() {
        let repository = QsoRepository::in_memory().unwrap();
        let dstar_qso = NewQso::new("PY2DSTAR", 1_700_000_001, 145_670_000, "DSTAR").unwrap();
        let expected = metadata("REF001 C", "C", "PY2RPT B");
        repository
            .insert_dstar(&dstar_qso, &expected, 1_700_000_010)
            .unwrap();
        let generic = NewQso::new("PY2FM", 1_700_000_000, 145_500_000, "FM").unwrap();
        repository.insert(&generic, 1_700_000_010).unwrap();

        let items = repository.list_items().unwrap();

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].qso.callsign, "PY2DSTAR");
        assert_eq!(items[0].metadata, ModeMetadata::Dstar(expected));
        assert_eq!(items[1].qso.callsign, "PY2FM");
        assert_eq!(items[1].metadata, ModeMetadata::Generic);
    }

    #[test]
    fn mapper_rejects_missing_mismatched_and_multiple_specialized_metadata() {
        let repository = QsoRepository::in_memory().unwrap();
        let missing = NewQso::new("PY2MISS", 1_700_000_001, 14_074_000, "FT8").unwrap();
        repository.insert(&missing, 1_700_000_010).unwrap();
        assert!(repository.list_items().is_err());

        repository.delete(repository.list().unwrap()[0].id).unwrap();
        let generic = NewQso::new("PY2BAD", 1_700_000_002, 145_500_000, "FM").unwrap();
        let qso_id = repository.insert(&generic, 1_700_000_010).unwrap();
        repository
            .connection
            .execute("INSERT INTO ft8_metadata(qso_id) VALUES (?1)", [qso_id])
            .unwrap();
        assert!(repository.list_items().is_err());

        repository
            .connection
            .execute("UPDATE qsos SET mode = 'FT8' WHERE id = ?1", [qso_id])
            .unwrap();
        repository
            .connection
            .execute("INSERT INTO dstar_metadata(qso_id) VALUES (?1)", [qso_id])
            .unwrap();
        assert!(repository.list_items().is_err());
    }

    #[test]
    fn list_items_hydrates_ysf_metadata_and_preserves_generic_qsos() {
        let repository = QsoRepository::in_memory().unwrap();
        let ysf = NewQso::new("PY2YSF", 1_700_000_001, 145_500_000, "C4FM").unwrap();
        let expected = ysf_metadata("Brazil", "PY2NODE", 10, 20);
        repository
            .insert_ysf(&ysf, &expected, 1_700_000_010)
            .unwrap();
        repository
            .insert(
                &NewQso::new("PY2FM", 1_700_000_000, 145_500_000, "FM").unwrap(),
                1_700_000_010,
            )
            .unwrap();

        let items = repository.list_items().unwrap();

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].metadata, ModeMetadata::Ysf(expected));
        assert_eq!(items[1].metadata, ModeMetadata::Generic);
    }

    #[test]
    fn searches_ysf_with_combined_case_insensitive_filters_and_either_dg_id() {
        let repository = QsoRepository::in_memory().unwrap();
        for (callsign, datetime, room, node, tx, rx) in [
            ("PY2TX", 1_700_000_001, "Brazil Room", "PY2NODE", 10, 20),
            ("PY2RX", 1_700_000_002, "Brazil Room", "PY2NODE", 30, 10),
            ("PY2OTHER", 1_700_000_003, "America", "OTHER", 10, 40),
        ] {
            let qso = NewQso::new(callsign, datetime, 145_500_000, "C4FM").unwrap();
            repository
                .insert_ysf(&qso, &ysf_metadata(room, node, tx, rx), datetime + 10)
                .unwrap();
        }

        let result = repository
            .search_ysf(&YsfFilter {
                room: Some(" brazil ".into()),
                wires_x_node: Some(" py2node ".into()),
                dg_id: Some(10),
            })
            .unwrap();

        assert_eq!(
            result
                .iter()
                .map(|qso| qso.callsign.as_str())
                .collect::<Vec<_>>(),
            ["PY2RX", "PY2TX"]
        );
        assert_eq!(
            repository.search_ysf(&YsfFilter::default()).unwrap().len(),
            3
        );
    }

    #[test]
    fn paginates_ysf_in_order_with_metadata_and_reports_absence() {
        let repository = QsoRepository::in_memory().unwrap();
        for (callsign, datetime) in [("PY2OLD", 1_700_000_001), ("PY2NEW", 1_700_000_002)] {
            let qso = NewQso::new(callsign, datetime, 145_500_000, "C4FM").unwrap();
            repository
                .insert_ysf(&qso, &ysf_metadata("Brazil", "NODE", 10, 20), datetime + 10)
                .unwrap();
        }

        let page = repository
            .search_ysf_page(&YsfFilter::default(), 1, 1)
            .unwrap();
        assert_eq!((page.total, page.offset, page.limit), (2, 1, 1));
        assert_eq!(page.items[0].qso.callsign, "PY2OLD");
        assert!(matches!(page.items[0].metadata, ModeMetadata::Ysf(_)));

        let absent = repository
            .search_ysf_page(
                &YsfFilter {
                    room: Some("missing".into()),
                    ..Default::default()
                },
                0,
                10,
            )
            .unwrap();
        assert_eq!(absent.total, 0);
        assert!(absent.items.is_empty());
    }

    #[test]
    fn searches_dstar_with_combined_filters_and_preserves_ordering() {
        let repository = QsoRepository::in_memory().unwrap();
        for (callsign, datetime, reflector, module, rpt1) in [
            ("PY2OLD", 1_700_000_000, "REF001 C", "C", "PY2RPT B"),
            ("PY2NEW", 1_700_000_002, "REF001 C", "C", "PY2RPT B"),
            ("PY2OTHER", 1_700_000_003, "REF002 A", "A", "PY2ALT C"),
        ] {
            let qso = NewQso::new(callsign, datetime, 145_670_000, "DSTAR").unwrap();
            repository
                .insert_dstar(&qso, &metadata(reflector, module, rpt1), datetime + 10)
                .unwrap();
        }
        let filter = DstarFilter {
            reflector: Some(" ref001 ".into()),
            module: Some(" c ".into()),
            rpt1: Some(" py2rpt ".into()),
        };

        let qsos = repository.search_dstar(&filter).unwrap();

        assert_eq!(
            qsos.iter()
                .map(|qso| qso.callsign.as_str())
                .collect::<Vec<_>>(),
            ["PY2NEW", "PY2OLD"]
        );
    }

    #[test]
    fn paginates_dstar_results_and_returns_joined_metadata() {
        let repository = QsoRepository::in_memory().unwrap();
        for (callsign, datetime) in [("PY2ONE", 1_700_000_001), ("PY2TWO", 1_700_000_002)] {
            let qso = NewQso::new(callsign, datetime, 145_670_000, "DSTAR").unwrap();
            repository
                .insert_dstar(&qso, &metadata("REF001 C", "C", "PY2RPT B"), datetime + 10)
                .unwrap();
        }

        let page = repository
            .search_dstar_page(&DstarFilter::default(), 1, 1)
            .unwrap();

        assert_eq!(page.total, 2);
        assert_eq!(page.offset, 1);
        assert_eq!(page.limit, 1);
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].qso.callsign, "PY2ONE");
        let ModeMetadata::Dstar(metadata) = &page.items[0].metadata else {
            panic!("expected D-STAR metadata");
        };
        assert_eq!(metadata.reflector.as_deref(), Some("REF001 C"));
    }
}
