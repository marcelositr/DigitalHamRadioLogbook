use rusqlite::{params, Result};

use crate::domain::{DmrMetadata, Ft8Metadata, Qso};

use super::{
    map_qso, parse_stored_access_type, parse_stored_call_type, DmrFilter, Ft8Filter, QsoListItem,
    QsoPage, QsoRepository,
};

impl QsoRepository {
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
