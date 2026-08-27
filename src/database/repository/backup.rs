use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use rusqlite::{params, Connection, OpenFlags, OptionalExtension, Result};

static NEXT_BACKUP_TEMP: AtomicU64 = AtomicU64::new(0);

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

        let temporary = unique_temporary_path(destination)?;
        let cleanup = TemporaryBackup::new(temporary);
        self.connection
            .execute("VACUUM INTO ?1", params![cleanup.path().to_string_lossy()])?;

        let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI;
        let backup = Connection::open_with_flags(cleanup.path(), flags)?;
        backup.execute_batch("PRAGMA query_only = ON;")?;
        verify_connection_integrity(&backup)?;
        migrations::validate_current_schema(&backup)?;
        drop(backup);

        set_private_file_permissions(cleanup.path())?;
        fs::OpenOptions::new()
            .read(true)
            .open(cleanup.path())?
            .sync_all()?;

        fs::hard_link(cleanup.path(), destination).map_err(|error| {
            if destination.exists() {
                "backup destination already exists".into()
            } else {
                Box::new(error) as Box<dyn Error>
            }
        })?;
        if let Err(error) = cleanup.remove() {
            let _ = fs::remove_file(destination);
            let _ = sync_directory(parent);
            return Err(error.into());
        }
        sync_directory(parent)?;
        Ok(())
    }
}

struct TemporaryBackup {
    path: PathBuf,
}

impl TemporaryBackup {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn remove(mut self) -> std::io::Result<()> {
        fs::remove_file(&self.path)?;
        self.path.clear();
        Ok(())
    }
}

impl Drop for TemporaryBackup {
    fn drop(&mut self) {
        if !self.path.as_os_str().is_empty() {
            let _ = fs::remove_file(&self.path);
        }
    }
}

fn unique_temporary_path(destination: &Path) -> std::result::Result<PathBuf, Box<dyn Error>> {
    let parent = destination
        .parent()
        .ok_or("backup destination has no parent directory")?;
    let name = destination
        .file_name()
        .ok_or("backup destination has no file name")?
        .to_string_lossy();
    for _ in 0..100 {
        let nonce = NEXT_BACKUP_TEMP.fetch_add(1, Ordering::Relaxed);
        let candidate = parent.join(format!(".{name}.{}.{}.tmp", std::process::id(), nonce));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err("could not allocate a temporary backup path".into())
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> std::io::Result<()> {
    fs::File::open(path)?.sync_all()
}

#[cfg(not(unix))]
fn sync_directory(_path: &Path) -> std::io::Result<()> {
    Ok(())
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
