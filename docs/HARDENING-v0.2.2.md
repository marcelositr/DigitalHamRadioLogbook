# Hardening checkpoint — v0.2.2

## Scope

Cycle focused exclusively on reliability, integrity, recoverable failures and permanent regression coverage. No new features, UI redesign, modes or integrations belong to this cycle.

Baseline analyzed:

- branch: `develop`;
- source version: `0.2.1`;
- target version: `0.2.2`;
- SQLite schema: version 5;
- baseline suite: 73 tests;
- CI: fmt, strict Clippy, tests, build and migration matrix for schemas 0–5.

## Initial risk inventory

### Existing protections

- transactional migrations and DMR/FT8 writes;
- future-schema and non-SQLite rejection;
- SQLite `quick_check` and foreign-key checks;
- atomic configuration and ADIF file publication;
- validated SQLite backups without silent overwrite;
- transactional ADIF preview/import flow;
- duplicate handling and unknown ADIF-field preservation;
- safe pending-edit and application-close flows.

### Priority gaps

- incompatible specialized metadata survived QSO mode changes;
- backup validation did not prove full application-schema compatibility;
- ADIF plans could become stale between preview and confirmation;
- corrupt/truncated SQLite and zero-byte database contracts lacked file-based regression tests;
- invalid/truncated configuration preserved the file but prevented startup without a recovery-oriented path;
- XDG relative paths were accepted;
- filesystem error paths for ADIF export and configuration lacked permanent tests;
- a durable save/delete followed by refresh failure could be reported as if the mutation failed.

## Bugs

### H-001 — Specialized metadata survived mode changes

- Severity: High
- Cause: each repository update replaced only metadata for the destination mode; generic updates changed only `qsos`.
- Impact: one QSO could retain both DMR and FT8 metadata, appear in the wrong filters and export metadata inconsistent with its current mode.
- Correction: all update paths now remove every specialized metadata set inside the same transaction before inserting only the destination mode metadata.
- Regression tests:
  - `changing_dmr_to_ft8_removes_dmr_metadata_and_route`
  - `changing_ft8_to_dmr_removes_ft8_metadata`
  - `changing_specialized_mode_to_generic_removes_all_mode_metadata`
- Rollback verification: existing DMR/FT8 update-failure tests remain green and confirm that failed replacement restores the previous QSO and metadata.

## Tested cases

### Mode transitions

- [x] DMR → FT8 removes DMR metadata and digital route.
- [x] FT8 → DMR removes FT8 metadata.
- [x] Specialized → generic removes all specialized metadata.
- [x] Previous metadata survives when destination metadata insertion fails.

## Remaining risks

- Full schema validation and backup restorability.
- Real SQLite corruption/truncation fixtures.
- Stale ADIF preview confirmation.
- Duplicate known ADIF fields.
- Configuration/XDG adverse environments.
- Filesystem permission and atomic-publication failure paths.
- Durable mutation followed by presentation refresh failure.

## Deferred ideas

None. Feature ideas discovered during hardening must be recorded separately and not implemented in this cycle.
