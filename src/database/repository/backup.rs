use std::error::Error;
use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension, Result};

use super::QsoRepository;
use crate::database::migrations;

impl QsoRepository {
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
            migrations::validate_current_schema(&backup)?;
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
}

#[cfg(unix)]
pub(super) fn set_private_file_permissions(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = std::fs::metadata(path)
        .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))?
        .permissions();
    permissions.set_mode(0o600);
    std::fs::set_permissions(path, permissions)
        .map_err(|error| rusqlite::Error::ToSqlConversionFailure(Box::new(error)))
}

#[cfg(not(unix))]
pub(super) fn set_private_file_permissions(_path: &Path) -> Result<()> {
    Ok(())
}

pub(super) fn verify_connection_integrity(connection: &Connection) -> Result<()> {
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
