# Pre-1.0 readiness

This is a living, factual record. It does not declare the project ready for `1.0.0` and defines no deadline for that decision.

## Current baseline

- source checkpoint: `0.9.0-rc.1` on `develop`;
- publication state: last public release is `v0.7.0`; `v0.8.0` is integrated in `main` without a tag/release, and `0.9.0-rc.1` is validated only on `develop`;
- SQLite schema: 7, with migrations 1–7 retained;
- test suite: 176 active tests and one manual stress test ignored by default;
- CI: quality/tests, Linux packaging, and migration jobs for schemas 0–7;
- supported mode metadata: Generic, DMR, FT8, D-STAR and YSF/C4FM;
- distribution: user-local GNU/Linux x86-64 tarball with SHA-256;
- open GitHub issues at this checkpoint: none;
- known Critical or High integrity defects: none identified by the v0.9.0 regression.

## Future `1.0.0` criteria

A future `1.0.0` requires evidence for all of the following. Time, commit count and feature count are not criteria.

- [x] No known Critical defect at the current checkpoint.
- [x] No known High data-integrity defect at the current checkpoint.
- [x] Historical migrations 0–7 are covered in CI and local regression.
- [x] Direct and sequential upgrades from published historical binaries have preserved data in isolated tests.
- [x] Native SQLite backup, verification and isolated restore have been demonstrated.
- [x] ADIF corpus, round-trip, unknown fields and published `APP_DHRL_*` contracts are covered.
- [x] Linux package creation, checksum, install, reinstall and uninstall are reproducible in the tested environment.
- [x] Recovery documentation has been exercised in isolated drills.
- [x] Schema 7 and published ADIF/config/path contracts are stable through the current checkpoint.
- [ ] Prolonged everyday use has accumulated enough evidence beyond short regression sessions.
- [ ] Multiple release/RC cycles remain consistently green without recurring severe regressions.
- [ ] Runtime compatibility evidence extends beyond the current primary Debian-family/X11 host where practical.
- [ ] The publication gap after `v0.7.0` is resolved through an explicit maintainer release decision.

## Data integrity

### Confirmed

- SQLite is the source of truth; foreign keys are enabled.
- Startup validates schema objects, `quick_check` and `foreign_key_check`.
- QSO plus mode metadata writes and ADIF import are transactional.
- Mode transitions remove incompatible metadata in the same transaction.
- Health and backup inspection use read-only/query-only connections and do not repair data.
- Duplicate QSO identities are permitted and are not classified as corruption.

### Not yet proven

- Long-term behavior on a growing real-world logbook over an extended operating period.
- Recovery evidence across several independent real-world backup generations.

## Upgrade

### Confirmed

- Schema migrations 0–7 pass and are idempotent.
- Real published v0.4.0–v0.7.0 artifacts were inspected during v0.9.0.
- Sequential and direct upgrades from a real v0.4.0/schema-5 database preserved IDs, timestamps, modes, metadata, ADIF extras and configuration.
- `0.9.0-rc.1` upgraded schema 5 to 7 and reopened successfully.

### Contract

Automatic downgrade is not supported. A database opened by an application with a newer schema must not be forced into an older application; restore a compatible pre-upgrade backup instead.

## Backup and restore

### Confirmed

- Backups are validated before publication and do not overwrite existing destinations.
- Existing backups can be inspected read-only as current, old/migratable, future, incomplete, corrupt or invalid.
- Isolated restore drills preserve Generic, DMR, FT8, D-STAR, YSF/C4FM and ADIF extras.
- Uninstall preserves database and configuration.

### Not yet proven

- Repeated restore drills from backups accumulated during prolonged real-world use.

## ADIF

### Confirmed

- The corpus contains 22 valid and 8 invalid fixtures.
- Generic, DMR, FT8, D-STAR and YSF/C4FM round-trips are covered.
- Unknown/private fields, data types, duplicates and Unicode are preserved according to the documented contract.
- Published `APP_DHRL_*` names and historical aliases remain compatibility contracts.
- Export is deterministic and reports the compiled `PROGRAMVERSION`.

### Limitations

- ADIF is interoperability, not a complete native backup.
- Documents are currently materialized in memory.
- No claim of certification against the entire ADIF software ecosystem is made.

## Performance

### Confirmed

- 10k and 100k release baselines remained stable during v0.9.0; the v0.10.0 100k checkpoint remained within the same historical range.
- 100k remains comfortable for normal operations on the measured host.
- No index or optimization is justified without a measured regression.

### Known limit

- One million QSOs is an extreme case: deep OFFSET pagination exceeds one second and full ADIF export historically took about 10.6 minutes with high memory pressure on a 7.7 GiB host.

## Distribution

### Confirmed

- The package is a user-local Linux x86-64 tarball with SHA-256.
- Packaging checks content, checksum publication, install/reinstall and idempotent uninstall.
- The exact `0.9.0-rc.1` artifact passed install, real upgrade, startup, restore and uninstall checks.
- The exact `0.10.0-rc.1` artifact (`SHA-256 2ee764dd25358da91a7c6b33c42ceeae4614dfc8a66ad751e7a81188c514d9a0`) passed checksum/content/`ldd`, install, schema-5→7 upgrade, repeated startup, five-mode restore and double uninstall without data/config hash changes.

### Not yet proven

- Broad runtime coverage on Fedora and openSUSE.
- Wayland behavior beyond natural/best-effort support.
- Compatibility with every glibc/native-library combination.

## Accessibility

### Confirmed

- Keyboard navigation, focus, mouse operation, clipboard preservation and `1050×680` layout were approved in the v0.9.0 regression.
- Native controls and custom actions expose the documented semantics where supported by Slint.

### Not yet proven

- Broad testing with multiple screen readers and desktop environments.

## Offline and privacy

### Confirmed

- No telemetry, analytics, account, server or automatic crash reporting exists.
- Core logging, database, ADIF, backup and health operations are local.
- External callsign/grid links require explicit user activation.
- Diagnostic reports omit QSO content by default.

### Limitation

A physical network namespace block could not be reproduced in the existing environment; source/dependency audits found no HTTP client used by core functionality.

## Dependency and security audit

### Confirmed

- Runtime dependencies remain unchanged for `0.10.0-rc.1`.
- `Cargo.lock` and the fuzz lockfile are versioned and resolve offline/locked.
- No HTTP client, telemetry, token or credential storage was identified in the v0.9.0 source/dependency audit.

### Limitation

- `cargo-audit` is not installed and no trusted local RustSec database is available, so this checkpoint cannot claim a current RustSec vulnerability scan. The dependency tree was reviewed without updating crates; duplicate transitive versions originate primarily from the existing Slint/desktop stack.
- Slint licensing terms applicable to distribution still require maintainer confirmation before a future maturity declaration.

## Known issues and blockers

### Current release blockers

No Critical, High integrity, broken migration, unsafe restore, promised ADIF data-loss or normal-flow crash is currently known.

### Future `1.0.0` blockers

- Prolonged real-world usage evidence is insufficient.
- The project needs repeated stable release/RC observations, not only one green checkpoint.
- The release-history gap after public `v0.7.0` requires an explicit maintainer decision and accurate documentation.

## Real-world usage log

Do not record callsigns, QTH, notes or other personal QSO content here. For each checkpoint, record only:

- application version and schema;
- approximate QSO count/range, rounded if appropriate;
- database size;
- whether health, backup verification, ADIF export and restart passed;
- backup checksum kept privately, not committed when it identifies a real dataset;
- reproduced bugs or operational friction.

### Evidence accumulated

- v0.9.0: mixed-mode manual regression at `1050×680`, including keyboard, mouse, focus, clipboard and operational flows, approved by the maintainer on 2026-08-28.

### Evidence still needed

- Multiple naturally occurring logging sessions over the `0.x` line.
- Periodic verified backups and isolated restore drills using safe copies.
- Observation of database growth, pagination and exports during normal use.

## Future ideas

Feature requests do not block maturity. Record them separately from defects and do not implement them during feature freeze. No new feature backlog item was identified during this baseline audit.
