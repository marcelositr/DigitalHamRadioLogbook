use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::{Connection, OpenFlags};

use super::migrations::CURRENT_SCHEMA_VERSION;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    HealthyCurrent,
    HealthyMigratableOld,
    FutureIncompatible,
    InvalidOrCorrupt,
    Unreadable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthFinding {
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthReport {
    pub path: Option<PathBuf>,
    pub status: HealthStatus,
    pub file_size: Option<u64>,
    pub schema_version: Option<i64>,
    pub migration_versions: Vec<i64>,
    pub quick_check_messages: Vec<String>,
    pub foreign_key_violations: Option<u64>,
    pub qso_count: Option<u64>,
    pub adif_extra_count: Option<u64>,
    pub counts_by_mode: BTreeMap<String, u64>,
    pub findings: Vec<HealthFinding>,
}

impl HealthReport {
    fn new(path: Option<PathBuf>, file_size: Option<u64>) -> Self {
        Self {
            path,
            status: HealthStatus::Unreadable,
            file_size,
            schema_version: None,
            migration_versions: Vec::new(),
            quick_check_messages: Vec::new(),
            foreign_key_violations: None,
            qso_count: None,
            adif_extra_count: None,
            counts_by_mode: BTreeMap::new(),
            findings: Vec::new(),
        }
    }

    pub fn diagnostic_text(&self) -> String {
        let database = self
            .path
            .as_deref()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            .unwrap_or("database");
        let mut lines = vec![format!("Database health: {:?} ({database})", self.status)];
        if let Some(size) = self.file_size {
            lines.push(format!("File size: {size} bytes"));
        }
        if let Some(version) = self.schema_version {
            lines.push(format!(
                "Schema version: {version} (supported: {CURRENT_SCHEMA_VERSION})"
            ));
        }
        if let Some(count) = self.qso_count {
            lines.push(format!("QSOs: {count}"));
        }
        if let Some(count) = self.adif_extra_count {
            lines.push(format!("ADIF extra fields: {count}"));
        }
        if !self.counts_by_mode.is_empty() {
            let counts = self
                .counts_by_mode
                .iter()
                .map(|(mode, count)| format!("{mode}={count}"))
                .collect::<Vec<_>>()
                .join(", ");
            lines.push(format!("Modes: {counts}"));
        }
        for finding in &self.findings {
            lines.push(format!("[{}] {}", finding.code, finding.message));
        }
        lines.join("\n")
    }

    fn finding(&mut self, code: &'static str, message: impl Into<String>) {
        self.findings.push(HealthFinding {
            code,
            message: message.into(),
        });
    }
}

impl fmt::Display for HealthReport {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.diagnostic_text())
    }
}

pub fn inspect_database(path: &Path) -> HealthReport {
    let size = fs::metadata(path).ok().map(|metadata| metadata.len());
    let mut report = HealthReport::new(Some(path.to_path_buf()), size);
    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI;
    match Connection::open_with_flags(path, flags) {
        Ok(connection) => {
            if let Err(error) = connection.execute_batch("PRAGMA query_only = ON;") {
                report.finding(
                    "unreadable",
                    format!("Could not enable query-only mode: {error}"),
                );
                return report;
            }
            inspect_connection_into(&connection, &mut report);
        }
        Err(error) => report.finding("unreadable", format!("Could not open database: {error}")),
    }
    report
}

pub(crate) fn inspect_connection(connection: &Connection) -> HealthReport {
    let mut report = HealthReport::new(None, None);
    inspect_connection_into(connection, &mut report);
    report
}

fn inspect_connection_into(connection: &Connection, report: &mut HealthReport) {
    let quick_check = match collect_strings(connection, "PRAGMA quick_check") {
        Ok(messages) => messages,
        Err(error) => {
            report.status = HealthStatus::InvalidOrCorrupt;
            report.finding(
                "quick-check-failed",
                format!("Quick check could not run: {error}"),
            );
            return;
        }
    };
    let quick_ok = quick_check.len() == 1 && quick_check[0].eq_ignore_ascii_case("ok");
    report.quick_check_messages = quick_check;
    if !quick_ok {
        report.finding("quick-check", "SQLite quick check reported a problem");
    }

    let objects = match schema_objects(connection) {
        Ok(objects) => objects,
        Err(error) => {
            report.status = HealthStatus::InvalidOrCorrupt;
            report.finding(
                "schema-unreadable",
                format!("Schema could not be read: {error}"),
            );
            return;
        }
    };

    if !objects.contains("table:schema_migrations") {
        report.status = HealthStatus::InvalidOrCorrupt;
        report.finding("missing-migrations", "Migration metadata table is missing");
        return;
    }

    match collect_i64(
        connection,
        "SELECT version FROM schema_migrations ORDER BY version",
    ) {
        Ok(versions) => {
            report.schema_version = versions.last().copied();
            report.migration_versions = versions;
        }
        Err(error) => {
            report.finding(
                "migration-unreadable",
                format!("Migration metadata is invalid: {error}"),
            );
            report.status = HealthStatus::InvalidOrCorrupt;
            return;
        }
    }

    let sequence_ok = report.schema_version.is_some_and(|maximum| {
        maximum >= 1 && report.migration_versions == (1..=maximum).collect::<Vec<_>>()
    });
    if !sequence_ok {
        report.finding(
            "migration-sequence",
            "Migration versions are missing, duplicated, or out of sequence",
        );
    }

    if let Some(version) = report.schema_version {
        let expected_version = version.clamp(1, CURRENT_SCHEMA_VERSION);
        for object in expected_objects(expected_version) {
            if !objects.contains(object) {
                report.finding(
                    "missing-object",
                    format!("Expected schema object is missing: {object}"),
                );
            }
        }
    }

    report.foreign_key_violations =
        scalar_u64(connection, "SELECT COUNT(*) FROM pragma_foreign_key_check").ok();
    if report.foreign_key_violations.is_some_and(|count| count > 0) {
        report.finding("foreign-key", "Foreign key violations were found");
    }

    if objects.contains("table:qsos") {
        report.qso_count = scalar_u64(connection, "SELECT COUNT(*) FROM qsos").ok();
        if let Ok(mut statement) =
            connection.prepare("SELECT mode, COUNT(*) FROM qsos GROUP BY mode ORDER BY mode")
        {
            if let Ok(rows) = statement.query_map([], |row| Ok((row.get(0)?, row.get(1)?))) {
                for row in rows.flatten() {
                    report.counts_by_mode.insert(row.0, row.1);
                }
            }
        }
        check_metadata_invariants(connection, &objects, report);
    }
    if objects.contains("table:adif_extra_fields") {
        report.adif_extra_count =
            scalar_u64(connection, "SELECT COUNT(*) FROM adif_extra_fields").ok();
    }

    let invalid = !quick_ok
        || !sequence_ok
        || report.findings.iter().any(|finding| {
            matches!(
                finding.code,
                "missing-object" | "foreign-key" | "metadata" | "dmr-route"
            )
        });
    report.status = if invalid {
        HealthStatus::InvalidOrCorrupt
    } else if report
        .schema_version
        .is_some_and(|v| v > CURRENT_SCHEMA_VERSION)
    {
        HealthStatus::FutureIncompatible
    } else if report
        .schema_version
        .is_some_and(|v| v < CURRENT_SCHEMA_VERSION)
    {
        HealthStatus::HealthyMigratableOld
    } else {
        HealthStatus::HealthyCurrent
    };
}

fn check_metadata_invariants(
    connection: &Connection,
    objects: &BTreeSet<String>,
    report: &mut HealthReport,
) {
    let tables = [
        ("dmr_metadata", "DMR"),
        ("ft8_metadata", "FT8"),
        ("dstar_metadata", "DSTAR"),
        ("ysf_metadata", "C4FM"),
    ];
    let available = tables
        .iter()
        .filter(|(table, _)| objects.contains(&format!("table:{table}")))
        .copied()
        .collect::<Vec<_>>();

    for (table, mode) in &available {
        let others = available
            .iter()
            .filter(|(other, _)| other != table)
            .map(|(other, _)| format!("(SELECT COUNT(*) FROM {other} m WHERE m.qso_id = q.id)"))
            .collect::<Vec<_>>();
        let own = format!("(SELECT COUNT(*) FROM {table} m WHERE m.qso_id = q.id)");
        let no_others = if others.is_empty() {
            String::new()
        } else {
            format!(" AND ({}) = 0", others.join(" + "))
        };
        let route = if *mode == "DMR" && objects.contains("table:digital_routes") {
            " AND (SELECT COUNT(*) FROM digital_routes r WHERE r.qso_id = q.id) = 1"
        } else {
            ""
        };
        let sql = format!(
            "SELECT COUNT(*) FROM qsos q WHERE UPPER(q.mode) = '{mode}' AND NOT ({own} = 1{no_others}{route})"
        );
        if scalar_u64(connection, &sql).is_ok_and(|count| count > 0) {
            report.finding(
                "metadata",
                format!("{mode} metadata is missing, wrong, or multiple"),
            );
        }
    }

    let sums = available
        .iter()
        .map(|(table, _)| format!("(SELECT COUNT(*) FROM {table} m WHERE m.qso_id = q.id)"))
        .collect::<Vec<_>>()
        .join(" + ");
    if !sums.is_empty() {
        let generic = format!(
            "SELECT COUNT(*) FROM qsos q WHERE UPPER(q.mode) NOT IN ('DMR','FT8','DSTAR','C4FM') AND ({sums}) <> 0"
        );
        if scalar_u64(connection, &generic).is_ok_and(|count| count > 0) {
            report.finding("metadata", "Generic-mode QSOs have specialized metadata");
        }
    }
    if objects.contains("table:digital_routes") {
        if scalar_u64(
            connection,
            "SELECT COUNT(*) FROM qsos q WHERE UPPER(q.mode) <> 'DMR' AND EXISTS (SELECT 1 FROM digital_routes r WHERE r.qso_id = q.id)",
        )
        .is_ok_and(|count| count > 0)
        {
            report.finding("dmr-route", "A non-DMR QSO has a DMR route");
        }
        if objects.contains("table:dmr_metadata")
            && scalar_u64(
                connection,
                "SELECT COUNT(*) FROM qsos q WHERE UPPER(q.mode) = 'DMR' AND ((SELECT COUNT(*) FROM dmr_metadata m WHERE m.qso_id = q.id) <> (SELECT COUNT(*) FROM digital_routes r WHERE r.qso_id = q.id))",
            )
            .is_ok_and(|count| count > 0)
        {
            report.finding("dmr-route", "DMR metadata and route counts do not match");
        }
    }
}

fn schema_objects(connection: &Connection) -> rusqlite::Result<BTreeSet<String>> {
    let mut statement = connection.prepare(
        "SELECT type, name FROM sqlite_master WHERE type IN ('table', 'index') AND name NOT LIKE 'sqlite_%'",
    )?;
    let rows = statement.query_map([], |row| {
        Ok(format!(
            "{}:{}",
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?
        ))
    })?;
    rows.collect()
}

fn expected_objects(version: i64) -> Vec<&'static str> {
    const V1: &[&str] = &[
        "table:schema_migrations",
        "table:qsos",
        "index:idx_qsos_callsign",
        "index:idx_qsos_datetime_start",
        "index:idx_qsos_mode",
    ];
    const V2: &[&str] = &[
        "table:schema_migrations",
        "table:qsos",
        "table:digital_routes",
        "table:dmr_metadata",
        "index:idx_qsos_callsign",
        "index:idx_qsos_datetime_start",
        "index:idx_qsos_mode",
        "index:idx_dmr_metadata_remote_id",
        "index:idx_dmr_metadata_talkgroup",
    ];
    const V3: &[&str] = &[
        "table:schema_migrations",
        "table:qsos",
        "table:digital_routes",
        "table:dmr_metadata",
        "table:ft8_metadata",
        "index:idx_qsos_callsign",
        "index:idx_qsos_datetime_start",
        "index:idx_qsos_mode",
        "index:idx_dmr_metadata_remote_id",
        "index:idx_dmr_metadata_talkgroup",
        "index:idx_ft8_metadata_snr_received",
    ];
    const V4: &[&str] = &[
        "table:schema_migrations",
        "table:qsos",
        "table:digital_routes",
        "table:dmr_metadata",
        "table:ft8_metadata",
        "table:adif_extra_fields",
        "index:idx_qsos_callsign",
        "index:idx_qsos_datetime_start",
        "index:idx_qsos_mode",
        "index:idx_dmr_metadata_remote_id",
        "index:idx_dmr_metadata_talkgroup",
        "index:idx_ft8_metadata_snr_received",
    ];
    const V5_PLUS: &[&str] = &[
        "table:schema_migrations",
        "table:qsos",
        "table:digital_routes",
        "table:dmr_metadata",
        "table:ft8_metadata",
        "table:adif_extra_fields",
        "index:idx_qsos_callsign",
        "index:idx_qsos_datetime_start",
        "index:idx_qsos_mode",
        "index:idx_dmr_metadata_remote_id",
        "index:idx_dmr_metadata_talkgroup",
        "index:idx_ft8_metadata_snr_received",
        "index:idx_qsos_datetime_start_id_desc",
        "index:idx_qsos_mode_datetime_start_id_desc",
        "index:idx_qsos_callsign_nocase",
        "index:idx_qsos_grid_locator_nocase",
        "index:idx_qsos_band_nocase",
        "index:idx_dmr_metadata_local_id",
        "index:idx_dmr_metadata_timeslot",
        "index:idx_digital_routes_network_nocase",
        "index:idx_digital_routes_repeater_nocase",
        "index:idx_digital_routes_hotspot_nocase",
    ];
    const V6: &[&str] = &[
        "table:dstar_metadata",
        "index:idx_dstar_metadata_reflector_nocase",
        "index:idx_dstar_metadata_module_nocase",
        "index:idx_dstar_metadata_rpt1_nocase",
    ];
    const V7: &[&str] = &[
        "table:ysf_metadata",
        "index:idx_ysf_metadata_tx_dg_id",
        "index:idx_ysf_metadata_rx_dg_id",
    ];
    match version {
        1 => V1.to_vec(),
        2 => V2.to_vec(),
        3 => V3.to_vec(),
        4 => V4.to_vec(),
        5 => V5_PLUS.to_vec(),
        6 => V5_PLUS.iter().chain(V6).copied().collect(),
        _ => V5_PLUS.iter().chain(V6).chain(V7).copied().collect(),
    }
}

fn collect_strings(connection: &Connection, sql: &str) -> rusqlite::Result<Vec<String>> {
    let mut statement = connection.prepare(sql)?;
    let values = statement.query_map([], |row| row.get(0))?.collect();
    values
}

fn collect_i64(connection: &Connection, sql: &str) -> rusqlite::Result<Vec<i64>> {
    let mut statement = connection.prepare(sql)?;
    let values = statement.query_map([], |row| row.get(0))?.collect();
    values
}

fn scalar_u64(connection: &Connection, sql: &str) -> rusqlite::Result<u64> {
    connection.query_row(sql, [], |row| row.get(0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::migrations;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static NEXT_FILE: AtomicU64 = AtomicU64::new(0);

    struct TestDb(PathBuf);

    impl TestDb {
        fn new() -> Self {
            let nonce = NEXT_FILE.fetch_add(1, Ordering::Relaxed);
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            Self(std::env::temp_dir().join(format!(
                "digital-ham-radio-health-{}-{nanos}-{nonce}.sqlite",
                std::process::id()
            )))
        }

        fn current() -> Self {
            let database = Self::new();
            let mut connection = Connection::open(&database.0).unwrap();
            migrations::run(&mut connection).unwrap();
            drop(connection);
            database
        }

        fn connect(&self) -> Connection {
            Connection::open(&self.0).unwrap()
        }
    }

    impl Drop for TestDb {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
            let _ = fs::remove_file(self.0.with_extension("sqlite-wal"));
            let _ = fs::remove_file(self.0.with_extension("sqlite-shm"));
        }
    }

    #[test]
    fn reports_healthy_current_counts_and_preserves_file() {
        let database = TestDb::current();
        let connection = database.connect();
        connection
            .execute(
                "INSERT INTO qsos (callsign, datetime_start_utc, frequency_hz, mode, notes, created_at_utc, updated_at_utc)
                 VALUES ('PRIVATE1', 1, 100, 'OTHER', 'private note', 1, 1)",
                [],
            )
            .unwrap();
        drop(connection);
        let bytes_before = fs::read(&database.0).unwrap();
        let modified_before = fs::metadata(&database.0).unwrap().modified().unwrap();

        let report = inspect_database(&database.0);

        assert_eq!(report.status, HealthStatus::HealthyCurrent);
        assert_eq!(report.qso_count, Some(1));
        assert_eq!(report.counts_by_mode.get("OTHER"), Some(&1));
        assert_eq!(fs::read(&database.0).unwrap(), bytes_before);
        assert_eq!(
            fs::metadata(&database.0).unwrap().modified().unwrap(),
            modified_before
        );
        let text = report.diagnostic_text();
        assert!(!text.contains("PRIVATE1"));
        assert!(!text.contains("private note"));
        assert!(!text.contains(database.0.parent().unwrap().to_str().unwrap()));
    }

    #[test]
    fn classifies_supported_old_schemas_five_and_six() {
        for version in [5, 6] {
            let database = TestDb::current();
            let connection = database.connect();
            if version == 5 {
                connection
                    .execute_batch("DROP TABLE dstar_metadata;")
                    .unwrap();
            }
            connection
                .execute_batch("DROP TABLE ysf_metadata;")
                .unwrap();
            connection
                .execute(
                    "DELETE FROM schema_migrations WHERE version > ?1",
                    [version],
                )
                .unwrap();
            drop(connection);

            let report = inspect_database(&database.0);
            assert_eq!(report.status, HealthStatus::HealthyMigratableOld);
            assert_eq!(report.schema_version, Some(version));
        }
    }

    #[test]
    fn classifies_future_schema_without_running_migrations() {
        let database = TestDb::current();
        database
            .connect()
            .execute("INSERT INTO schema_migrations VALUES (8, 1)", [])
            .unwrap();
        let report = inspect_database(&database.0);
        assert_eq!(report.status, HealthStatus::FutureIncompatible);
        assert_eq!(report.schema_version, Some(8));
    }

    #[test]
    fn detects_missing_object_and_migration_inconsistency() {
        let database = TestDb::current();
        let connection = database.connect();
        connection
            .execute_batch("DROP TABLE ft8_metadata;")
            .unwrap();
        connection
            .execute("DELETE FROM schema_migrations WHERE version = 4", [])
            .unwrap();
        drop(connection);
        let report = inspect_database(&database.0);
        assert_eq!(report.status, HealthStatus::InvalidOrCorrupt);
        assert!(report
            .findings
            .iter()
            .any(|finding| finding.code == "missing-object"));
        assert!(report
            .findings
            .iter()
            .any(|finding| finding.code == "migration-sequence"));
    }

    #[test]
    fn handles_non_sqlite_and_truncated_files() {
        for contents in [
            b"not sqlite".as_slice(),
            b"SQLite format 3\0short".as_slice(),
        ] {
            let database = TestDb::new();
            fs::write(&database.0, contents).unwrap();
            let report = inspect_database(&database.0);
            assert!(matches!(
                report.status,
                HealthStatus::InvalidOrCorrupt | HealthStatus::Unreadable
            ));
            assert!(!report.findings.is_empty());
        }
    }

    #[test]
    fn detects_controlled_foreign_key_violation() {
        let database = TestDb::current();
        let connection = database.connect();
        connection
            .execute_batch("PRAGMA foreign_keys = OFF;")
            .unwrap();
        connection
            .execute(
                "INSERT INTO dmr_metadata(qso_id, call_type) VALUES (999, 'group')",
                [],
            )
            .unwrap();
        drop(connection);
        let report = inspect_database(&database.0);
        assert_eq!(report.foreign_key_violations, Some(1));
        assert_eq!(report.status, HealthStatus::InvalidOrCorrupt);
    }

    #[test]
    fn detects_missing_wrong_multiple_metadata_and_dmr_route_mismatch() {
        let cases = [
            ("DMR", ""),
            ("FT8", "INSERT INTO dstar_metadata(qso_id) VALUES (1);"),
            ("DSTAR", "INSERT INTO dstar_metadata(qso_id) VALUES (1); INSERT INTO ft8_metadata(qso_id) VALUES (1);"),
            ("DMR", "INSERT INTO dmr_metadata(qso_id, call_type) VALUES (1, 'group');"),
        ];
        for (mode, metadata) in cases {
            let database = TestDb::current();
            let connection = database.connect();
            connection
                .execute_batch(&format!(
                    "INSERT INTO qsos (id, callsign, datetime_start_utc, frequency_hz, mode, created_at_utc, updated_at_utc)
                     VALUES (1, 'SECRET', 1, 100, '{mode}', 1, 1); {metadata}"
                ))
                .unwrap();
            drop(connection);
            let report = inspect_database(&database.0);
            assert_eq!(
                report.status,
                HealthStatus::InvalidOrCorrupt,
                "{mode}: {report:?}"
            );
            assert!(report
                .findings
                .iter()
                .any(|finding| matches!(finding.code, "metadata" | "dmr-route")));
        }
    }

    #[test]
    fn detects_metadata_and_routes_on_generic_modes() {
        let database = TestDb::current();
        database
            .connect()
            .execute_batch(
                "INSERT INTO qsos (id, callsign, datetime_start_utc, frequency_hz, mode, created_at_utc, updated_at_utc)
                 VALUES (1, 'SECRET', 1, 100, 'OTHER', 1, 1);
                 INSERT INTO digital_routes(qso_id, access_type) VALUES (1, 'simplex');
                 INSERT INTO ft8_metadata(qso_id) VALUES (1);",
            )
            .unwrap();
        let report = inspect_database(&database.0);
        assert_eq!(report.status, HealthStatus::InvalidOrCorrupt);
        assert!(report
            .findings
            .iter()
            .any(|finding| finding.code == "metadata"));
        assert!(report
            .findings
            .iter()
            .any(|finding| finding.code == "dmr-route"));
    }
}
