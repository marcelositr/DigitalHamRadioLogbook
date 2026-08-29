# Digital Ham Radio Logbook v0.9.0

## Stabilization release

Version 0.9.0 is a feature-freeze release focused on compatibility, regression testing and confidence in existing logbook data. It does not add a new mode or change the database schema.

### Upgrade and data preservation

Real published binaries from v0.4.0 through v0.7.0 were exercised in isolated environments. Sequential and direct upgrades preserved QSO IDs, UTC timestamps, Generic, DMR, FT8, D-STAR and YSF/C4FM records, specialized metadata, unknown ADIF fields and application configuration. SQLite integrity and foreign-key checks passed after migration and on the second opening.

### ADIF hardening

The ADIF parser corpus and round-trip suite were revalidated. The fuzzing workflow is reproducible with its locked dependency graph and now uses a dedicated mutable corpus, keeping permanent fixtures unchanged. A 60-second run completed 3,622,542 executions without a crash.

### Backup and recovery regression

The backup/restore drill covers all supported mode families and ADIF extra fields. Backup classification, read-only health checks, migration compatibility and packaging workflows were re-exercised without changing the active schema.

### Time boundaries

Regression coverage now explicitly includes the UTC midnight boundary, leap day, invalid calendar dates and invalid hour values.

### Performance

Release measurements with 10,000 and 100,000 deterministic QSOs remained consistent with previous releases. No new index, migration or optimization was justified during the feature freeze.

## Compatibility

- Database schema remains version 7.
- No runtime dependency was added.
- Existing native SQLite backups and ADIF contracts remain unchanged.
- All core processing remains local and offline.

## Release-candidate status

`0.9.0-rc.1` is intended for final manual regression. Its exact Linux artifact (`SHA-256 2ac5ffd8585981eafc60a07eb94c1ee4e4967706cedeed170fc20e595dad73e0`) passed installation, real schema-5 upgrade, repeated startup, recovery and uninstall checks without a rebuild. The release remains blocked on completing the mixed-mode keyboard/mouse workflow at `1050×680` before a final tag or GitHub Release is authorized.
