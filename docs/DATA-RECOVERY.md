# Data integrity, backup, and recovery

Digital Ham Radio Logbook stores all mutable data locally. This guide describes safe recovery procedures; it never requires deleting the current database first.

## Data locations

With XDG variables:

- database: `$XDG_DATA_HOME/digital-ham-log/logbook.sqlite3`
- configuration: `$XDG_CONFIG_HOME/digital-ham-log/config.toml`

With standard GNU/Linux fallbacks:

- database: `~/.local/share/digital-ham-log/logbook.sqlite3`
- configuration: `~/.config/digital-ham-log/config.toml`

A backup created in Tools is a consistent SQLite snapshot. ADIF exports are exchange files, not complete substitutes for database backups because application-specific state and future metadata may not be represented identically.

## Built-in safeguards

At startup the application:

1. opens SQLite without replacing an existing file;
2. rejects schema versions newer than the application supports;
3. runs migrations transactionally;
4. verifies required schema objects;
5. runs SQLite `quick_check` and `foreign_key_check`.

A backup is reported as successful only after a temporary snapshot passes SQLite integrity, foreign-key, supported-version, required-table and required-index checks in read-only mode, receives private permissions and is synchronized to storage. It is then published without overwriting an existing destination. A failed validation removes the temporary snapshot and leaves the final destination absent.

Configuration and ADIF exports use a temporary file in the destination directory, synchronize it, and then publish it by rename. Existing ADIF/export destinations are never overwritten.

## Check health and verify a backup

In **Tools**:

- **Check data health** checks the active logbook's SQLite integrity, foreign keys, schema objects, migration sequence and mode/metadata invariants. It does not repair or modify data.
- **Verify backup** opens a selected SQLite file read-only. Current backups pass normally; an older supported schema is reported as valid and migratable; future, incomplete, corrupt or unreadable files are rejected without modification.

If a check reports corruption, stop editing, preserve the database and its `-wal`/`-shm` sidecars, verify known backups and follow the procedure below. Duplicated QSO identities are not corruption and are not removed.

## Restore a database backup safely

Restore remains assisted/documented rather than an in-application button. Replacing an active SQLite database while callbacks and an open connection exist would make WAL/sidecar handling and rollback unsafe.

1. Close Digital Ham Radio Logbook completely.
2. Locate the active database using the XDG paths above.
3. Do not overwrite or delete the active database immediately.
4. Copy the active database and any adjacent `-wal`/`-shm` files to a separate recovery directory.
5. Use **Tools → Verify backup** before closing the application; proceed only with a healthy current backup or an explicitly supported older schema.
6. Copy the backup to a temporary filename in the active database directory.
7. Rename the active `logbook.sqlite3` to a dated recovery name.
8. Rename the temporary restored file to `logbook.sqlite3`.
9. Ensure stale `logbook.sqlite3-wal` and `logbook.sqlite3-shm` files from the previous database are not left beside the restored database; preserve them in the recovery directory instead of deleting them.
10. Start the application. It will validate integrity and apply supported migrations.
11. Verify representative generic, DMR, FT8, D-STAR, and YSF/C4FM records before removing any recovery copies.

Example with default paths, while the application is closed:

```sh
mkdir -p "$HOME/dhrl-recovery"
cp -a "$HOME/.local/share/digital-ham-log/logbook.sqlite3"* "$HOME/dhrl-recovery/"
cp "/path/to/logbook-backup.sqlite3" "$HOME/.local/share/digital-ham-log/logbook.restore.tmp"
mv "$HOME/.local/share/digital-ham-log/logbook.sqlite3" "$HOME/dhrl-recovery/logbook-before-restore.sqlite3"
mv "$HOME/.local/share/digital-ham-log/logbook.restore.tmp" "$HOME/.local/share/digital-ham-log/logbook.sqlite3"
```

Move any old `-wal`/`-shm` files to the recovery directory before restarting. Adjust paths when XDG variables are set.

## If the database does not open or reports corruption/incompatible schema

- Do not repeatedly modify, truncate, or recreate the original file.
- Preserve the exact database and related `-wal`/`-shm` files.
- Check filesystem permissions and available disk space.
- Try a known-good backup by following the restore procedure above.
- A “newer schema version” error means the database was opened by a newer application. Upgrade the application instead of forcing an older version to write it.
- A missing-table/schema inconsistency error indicates an incomplete or damaged schema; preserve it for diagnosis and restore a backup.
- If a migration fails, preserve the pre-migration database and sidecars. Migrations are transactional; do not manually insert migration version markers.
- If configuration is invalid, preserve `config.toml`, move it aside while the application is closed, and let defaults be recreated. Database backup/restore does not include configuration.

## Permission and path failures

The application creates its own XDG data/configuration directories. Backup and export destination directories must already exist. If an operation fails:

- confirm the parent is a directory, not a file;
- confirm the current user can write to it;
- confirm the destination does not already exist;
- confirm enough disk space is available;
- retry with a new destination selected through the graphical chooser.

## Verify recovery

After restoration:

- open and search the Logbook;
- inspect one generic, one DMR, one FT8, one D-STAR, and one YSF/C4FM QSO when available;
- create a new backup to a new filename;
- export ADIF to a new filename;
- close and restart the application to confirm persistence.

Never use an uninstall script to remove data. Linux uninstall preserves the entire `digital-ham-log` data and configuration directories by design.
