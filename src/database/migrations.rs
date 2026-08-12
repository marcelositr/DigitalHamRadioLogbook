use rusqlite::{Connection, Error, Result, Transaction};

const CURRENT_SCHEMA_VERSION: i64 = 4;

const INITIAL_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS schema_migrations (
    version INTEGER PRIMARY KEY,
    applied_at_utc INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS qsos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    callsign TEXT NOT NULL,
    datetime_start_utc INTEGER NOT NULL,
    datetime_end_utc INTEGER,
    frequency_hz INTEGER NOT NULL CHECK (frequency_hz > 0),
    band TEXT,
    mode TEXT NOT NULL,
    submode TEXT,
    rst_sent TEXT,
    rst_received TEXT,
    grid_locator TEXT,
    name TEXT,
    qth TEXT,
    notes TEXT NOT NULL DEFAULT '',
    created_at_utc INTEGER NOT NULL,
    updated_at_utc INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_qsos_callsign ON qsos(callsign);
CREATE INDEX IF NOT EXISTS idx_qsos_datetime_start ON qsos(datetime_start_utc);
CREATE INDEX IF NOT EXISTS idx_qsos_mode ON qsos(mode);

INSERT OR IGNORE INTO schema_migrations(version, applied_at_utc)
VALUES (1, CAST(strftime('%s', 'now') AS INTEGER));
"#;

const DMR_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS digital_routes (
    qso_id INTEGER PRIMARY KEY,
    access_type TEXT NOT NULL,
    network TEXT,
    repeater_callsign TEXT,
    hotspot TEXT,
    FOREIGN KEY (qso_id) REFERENCES qsos(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS dmr_metadata (
    qso_id INTEGER PRIMARY KEY,
    remote_dmr_id INTEGER CHECK (remote_dmr_id > 0),
    local_dmr_id INTEGER CHECK (local_dmr_id > 0),
    talkgroup INTEGER CHECK (talkgroup > 0),
    timeslot INTEGER CHECK (timeslot BETWEEN 1 AND 2),
    color_code INTEGER CHECK (color_code BETWEEN 0 AND 15),
    call_type TEXT NOT NULL CHECK (call_type IN ('group', 'private')),
    rx_frequency_hz INTEGER CHECK (rx_frequency_hz > 0),
    tx_frequency_hz INTEGER CHECK (tx_frequency_hz > 0),
    notes TEXT NOT NULL DEFAULT '',
    FOREIGN KEY (qso_id) REFERENCES qsos(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_dmr_metadata_remote_id ON dmr_metadata(remote_dmr_id);
CREATE INDEX IF NOT EXISTS idx_dmr_metadata_talkgroup ON dmr_metadata(talkgroup);

INSERT INTO schema_migrations(version, applied_at_utc)
VALUES (2, CAST(strftime('%s', 'now') AS INTEGER));
"#;

const ADIF_EXTRA_FIELDS_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS adif_extra_fields (
    qso_id INTEGER NOT NULL,
    field_order INTEGER NOT NULL,
    name TEXT NOT NULL,
    value TEXT NOT NULL,
    data_type TEXT,
    PRIMARY KEY (qso_id, field_order),
    FOREIGN KEY (qso_id) REFERENCES qsos(id) ON DELETE CASCADE
);

INSERT INTO schema_migrations(version, applied_at_utc)
VALUES (4, CAST(strftime('%s', 'now') AS INTEGER));
"#;

const FT8_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS ft8_metadata (
    qso_id INTEGER PRIMARY KEY,
    snr_sent_db INTEGER CHECK (snr_sent_db BETWEEN -50 AND 50),
    snr_received_db INTEGER CHECK (snr_received_db BETWEEN -50 AND 50),
    power_watts INTEGER CHECK (power_watts > 0),
    audio_frequency_hz INTEGER CHECK (audio_frequency_hz > 0),
    source_software TEXT,
    protocol TEXT,
    final_message TEXT,
    FOREIGN KEY (qso_id) REFERENCES qsos(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_ft8_metadata_snr_received ON ft8_metadata(snr_received_db);

INSERT INTO schema_migrations(version, applied_at_utc)
VALUES (3, CAST(strftime('%s', 'now') AS INTEGER));
"#;

pub fn run(connection: &mut Connection) -> Result<()> {
    reject_future_schema(connection)?;
    let transaction = connection.transaction()?;
    transaction.execute_batch(INITIAL_SCHEMA)?;

    let has_dmr_schema: bool = transaction.query_row(
        "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 2)",
        [],
        |row| row.get(0),
    )?;
    if !has_dmr_schema {
        transaction.execute_batch(DMR_SCHEMA)?;
    }

    let has_ft8_schema: bool = transaction.query_row(
        "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 3)",
        [],
        |row| row.get(0),
    )?;
    if !has_ft8_schema {
        transaction.execute_batch(FT8_SCHEMA)?;
    }

    let has_adif_extra_fields: bool = transaction.query_row(
        "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 4)",
        [],
        |row| row.get(0),
    )?;
    if !has_adif_extra_fields {
        transaction.execute_batch(ADIF_EXTRA_FIELDS_SCHEMA)?;
    }

    validate_schema(&transaction)?;
    transaction.commit()
}

fn reject_future_schema(connection: &Connection) -> Result<()> {
    let has_migrations: bool = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'schema_migrations')",
        [],
        |row| row.get(0),
    )?;
    if !has_migrations {
        return Ok(());
    }
    let version: Option<i64> =
        connection.query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
            row.get(0)
        })?;
    if version.is_some_and(|version| version > CURRENT_SCHEMA_VERSION) {
        return Err(Error::InvalidParameterName(format!(
            "database schema version is newer than supported version {CURRENT_SCHEMA_VERSION}"
        )));
    }
    Ok(())
}

fn validate_schema(transaction: &Transaction<'_>) -> Result<()> {
    for table in [
        "schema_migrations",
        "qsos",
        "digital_routes",
        "dmr_metadata",
        "ft8_metadata",
        "adif_extra_fields",
    ] {
        let exists: bool = transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)",
            [table],
            |row| row.get(0),
        )?;
        if !exists {
            return Err(Error::InvalidParameterName(format!(
                "database schema is inconsistent: missing table {table}"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_is_idempotent_and_enables_schema_version() {
        let mut connection = Connection::open_in_memory().unwrap();

        run(&mut connection).unwrap();
        run(&mut connection).unwrap();

        let version: i64 = connection
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(version, 4);

        let dmr_table_exists: bool = connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'dmr_metadata')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(dmr_table_exists);

        let ft8_table_exists: bool = connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'ft8_metadata')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(ft8_table_exists);

        let extra_fields_table_exists: bool = connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'adif_extra_fields')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(extra_fields_table_exists);
    }

    #[test]
    fn rejects_a_database_from_a_future_schema_version() {
        let mut connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch(
                "CREATE TABLE schema_migrations (
                    version INTEGER PRIMARY KEY,
                    applied_at_utc INTEGER NOT NULL
                 );
                 INSERT INTO schema_migrations(version, applied_at_utc) VALUES (999, 0);",
            )
            .unwrap();

        let error = run(&mut connection).unwrap_err().to_string();
        assert!(error.contains("newer than supported"));
    }

    #[test]
    fn rejects_a_version_marker_with_missing_schema_objects() {
        let mut connection = Connection::open_in_memory().unwrap();
        run(&mut connection).unwrap();
        connection
            .execute("DROP TABLE adif_extra_fields", [])
            .unwrap();

        let error = run(&mut connection).unwrap_err().to_string();
        assert!(error.contains("missing table adif_extra_fields"));
    }

    #[test]
    fn ft8_metadata_is_deleted_with_its_qso() {
        let mut connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch("PRAGMA foreign_keys = ON;")
            .unwrap();
        run(&mut connection).unwrap();
        connection
            .execute(
                "INSERT INTO qsos (
                    callsign, datetime_start_utc, frequency_hz, mode,
                    created_at_utc, updated_at_utc
                 ) VALUES ('PY2ABC', 1700000000, 14074000, 'FT8', 1700000000, 1700000000)",
                [],
            )
            .unwrap();
        let qso_id = connection.last_insert_rowid();
        connection
            .execute(
                "INSERT INTO ft8_metadata(qso_id, snr_received_db) VALUES (?1, -12)",
                [qso_id],
            )
            .unwrap();

        connection
            .execute("DELETE FROM qsos WHERE id = ?1", [qso_id])
            .unwrap();
        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM ft8_metadata", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn dmr_metadata_is_deleted_with_its_qso() {
        let mut connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch("PRAGMA foreign_keys = ON;")
            .unwrap();
        run(&mut connection).unwrap();
        connection
            .execute(
                "INSERT INTO qsos (
                    callsign, datetime_start_utc, frequency_hz, mode,
                    created_at_utc, updated_at_utc
                 ) VALUES ('PU2XYZ', 1700000000, 438500000, 'DMR', 1700000000, 1700000000)",
                [],
            )
            .unwrap();
        let qso_id = connection.last_insert_rowid();
        connection
            .execute(
                "INSERT INTO dmr_metadata(qso_id, call_type) VALUES (?1, 'group')",
                [qso_id],
            )
            .unwrap();

        connection
            .execute("DELETE FROM qsos WHERE id = ?1", [qso_id])
            .unwrap();
        let metadata_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM dmr_metadata", [], |row| row.get(0))
            .unwrap();
        assert_eq!(metadata_count, 0);
    }
}
