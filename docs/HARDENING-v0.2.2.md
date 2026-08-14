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

### M-001 — Current schema accepted missing specialized indexes

- Severity: Medium
- Cause: final schema validation listed only indexes introduced in migration 5; published DMR/FT8 indexes from migrations 2–3 were omitted.
- Impact: a database marked as schema 5 could open while missing expected query indexes, silently degrading specialized searches and violating the declared schema contract.
- Correction: validation now requires every published index from migrations 1–5. Initial indexes may still be repaired idempotently by the initial schema; specialized missing indexes cause safe refusal.
- Regression test: `rejects_a_current_schema_with_missing_published_indexes`.

### H-002 — Backup validation did not prove application compatibility

- Severity: High
- Cause: backup post-validation checked SQLite integrity and foreign keys, but not application schema version, tables or indexes.
- Impact: a structurally healthy SQLite snapshot could be announced as valid while being incompatible with the current application.
- Correction: snapshots now pass the same current-schema validation used during normal database opening; uncertain destinations are removed.
- Regression tests:
  - `backup_rejects_and_removes_an_incomplete_application_schema`
  - `backup_rejects_and_removes_a_future_schema_snapshot`
  - `backup_restores_generic_dmr_ft8_and_adif_extra_data`

### H-003 — ADIF preview could become stale before confirmation

- Severity: High
- Cause: duplicate identities were calculated only during preview and trusted during confirmation.
- Impact: a matching QSO created after preview could be imported again.
- Correction: confirmation reloads identities inside its transaction, skips newly conflicting records and reports actual committed counts. Manual duplicate creation remains allowed.
- Regression test: `confirmation_skips_duplicates_created_after_adif_preview`.

### M-002 — Repeated known ADIF fields lost data silently

- Severity: Medium
- Cause: conversion used the first known field and filtered every occurrence from extras.
- Impact: later values disappeared without warning.
- Correction: repeated known fields make the record invalid; repeated unknown application fields remain preserved.
- Regression test: `rejects_duplicate_known_fields_without_rejecting_repeated_unknown_fields`.

### M-003 — Relative XDG paths depended on the launch directory

- Severity: Medium
- Cause: non-empty XDG variables were accepted without requiring absolute paths; empty/relative HOME also produced relative fallbacks.
- Impact: terminal and desktop launches could use different databases/configurations or write into unexpected directories.
- Correction: only absolute XDG values are accepted; relative values fall back to an absolute HOME, and missing/empty/relative HOME is rejected.
- Regression tests are in `app::paths::tests`.

### L-001 — Negative fractional frequency was accepted as positive

- Severity: Low
- Cause: parsing `-0.5` converted the integer part `-0` to zero before adding the fraction.
- Impact: invalid input was normalized to 500 kHz.
- Correction: any explicit negative sign is rejected before numeric decomposition.
- Regression coverage: `rejects_invalid_frequency`.

## Tested cases

### Database opening and schema

- [x] Missing database file is created, migrated, integrity-checked and reopenable.
- [x] Existing zero-byte file is initialized as a valid current database.
- [x] Non-SQLite text file is refused without replacement.
- [x] Truncated real SQLite file is refused without changing its bytes.
- [x] Future schema is refused.
- [x] Missing tables and every published index are detected or safely repaired where idempotent.
- [x] Schemas 0–5 migrate to schema 5 with representative data preserved.

### Backup and restoration

- [x] Valid snapshot creation and overwrite refusal.
- [x] Future and incomplete application schemas are rejected and removed.
- [x] Controlled restore opens successfully and preserves generic, DMR, FT8 and unknown ADIF data.
- [x] Restored database passes integrity and foreign-key checks.

### ADIF robustness

- [x] Empty, whitespace and header-only documents are handled without panic.
- [x] Missing EOR, malformed/truncated tags, oversized declared lengths and UTF-8 boundary cuts return controlled errors.
- [x] Repeated known fields are rejected; repeated unknown fields are preserved.
- [x] Preview cancellation writes nothing.
- [x] Duplicates existing before or created after preview are skipped.
- [x] Existing export destination and missing parent are refused without partial final files.
- [x] Exported ADIF uses mode `0600` on Unix.

### Configuration and XDG

- [x] Missing configuration uses defaults.
- [x] Invalid and truncated TOML is rejected without modifying the file.
- [x] Configuration round-trip works in paths containing spaces and Unicode.
- [x] Saved configuration uses mode `0600` on Unix.
- [x] Absolute XDG paths work; relative XDG paths use absolute HOME fallbacks.
- [x] Missing, empty or relative HOME without valid XDG paths is rejected.
- [x] File in place of the application data directory returns a controlled error.

### Inputs and deletion

- [x] Explicit negative frequencies, including `-0.5`, are rejected.
- [x] Inverted FT8 UTC ranges are rejected.
- [x] Public repository deletion cascades DMR routes and FT8 metadata and leaves foreign keys valid.

### Mode transitions

- [x] DMR → FT8 removes DMR metadata and digital route.
- [x] FT8 → DMR removes FT8 metadata.
- [x] Specialized → generic removes all specialized metadata.
- [x] Previous metadata survives when destination metadata insertion fails.

## Remaining risks

- Full table/column/constraint-definition validation beyond required object names.
- Additional SQLite corruption forms beyond controlled truncation.
- Atomic no-replace publication race for ADIF destinations requires a platform primitive not added in this cycle.
- Deterministic injection of post-rename directory-sync failures is not available without a larger filesystem abstraction.
- Durable mutation followed by presentation refresh failure remains a presentation-layer risk.
- Resource limits for very large ADIF/text inputs remain undefined; no arbitrary limits were introduced without product policy.

## Deferred ideas

None. Feature ideas discovered during hardening must be recorded separately and not implemented in this cycle.
