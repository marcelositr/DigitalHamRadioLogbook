# Digital Ham Radio Logbook v0.8.0

## Better tools for protecting your logbook

Version 0.8.0 adds local maintenance tools so you can verify the health of an important long-term log without sending any data anywhere.

### Check data health

The new **Check data health** action verifies SQLite integrity, foreign keys, schema and migration history, record counts, and consistency between each QSO mode and its specialized metadata. The check is read-only: it reports problems but never repairs or changes records.

### Verify backups before you need them

**Verify backup** opens a selected SQLite backup in read-only mode and reports whether it is:

- healthy and current;
- healthy but from an older supported schema;
- from a newer unsupported schema;
- incomplete, corrupt, invalid, or unreadable.

Creating a backup is also safer: the final filename is published only after the temporary snapshot has passed validation and has been synchronized to storage. Existing files are never overwritten.

### Export the current results

Tools now distinguishes **Export all QSOs** from **Export current results**. The latter exports every QSO matching the current Logbook search or mode filter, across all result pages, while preserving mode metadata and unknown ADIF fields. If there are no matches, no empty file is created.

### Recovery guidance

Database restoration remains an assisted, application-closed procedure rather than a destructive in-app button. The updated recovery guide explains backup verification, active database preservation, WAL/SHM sidecars, older and future schemas, migration failures, and configuration recovery.

See `docs/DATA-RECOVERY.md` for the tested procedure.

## Compatibility

- Database schema remains version 7; no migration is required from v0.7.0.
- Existing SQLite backups remain normal SQLite databases.
- All processing remains local and offline.
- No runtime dependency was added.
