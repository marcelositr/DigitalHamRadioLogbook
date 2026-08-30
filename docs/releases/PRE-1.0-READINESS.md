# Pre-1.0 readiness

This is a living, factual record. It does not declare the project ready for `1.0.0` and defines no deadline for that decision.

## Current baseline

- source baseline before RC2 preparation: `c3ffd3dd49d2dc18ef3b7cf227e77217b47cc7c4` on `main`;
- release preparation: `0.11.0-rc.2` on short-lived branch `release/v0.11.0-rc2` through PR #13;
- current preserved public release: `v0.11.0-RC1`, explicitly marked **Pre-release**;
- SQLite schema: 7, with migrations 1–7 retained and historical schemas 0–7 covered by CI;
- supported mode metadata: Generic, DMR, FT8, D-STAR and YSF/C4FM;
- distribution targets: GNU/Linux x86-64 tarball, Debian package and AppImage with checksums derived from one validated binary;
- repository model: `main` is the single permanent branch; release work uses short-lived PR branches;
- open follow-up: issue #10 tracks informational `unmaintained` transitive Rust dependencies;
- known Critical or High integrity defects: none currently identified;
- most recent RustSec review: 0 known vulnerabilities; informational maintenance advisories are tracked separately.

## v0.11 RC2 evidence

The post-RC1 state adds no feature or data-contract change. It contains repository/release hardening plus the Logbook layout correction at the documented `1050×680` reference size.

The maintainer ran the corrected real application locally after commit `c3ffd3dd49d2dc18ef3b7cf227e77217b47cc7c4` and reported the local test as approved. This is sufficient to promote the corrected source state to a second release candidate, while the exact packaged RC2 artifact still requires generation and validation before publication.

RC2 preparation must not move or rewrite the existing RC1 tag. No SQLite migration, schema change, ADIF contract change, runtime dependency change or feature is part of the promotion.

## Future `1.0.0` criteria

A future `1.0.0` requires evidence for all of the following. Time, commit count and feature count are not criteria.

- [x] No known Critical defect at the current checkpoint.
- [x] No known High data-integrity defect at the current checkpoint.
- [x] Historical migrations 0–7 are covered in CI and local regression.
- [x] Direct and sequential upgrades from retained historical artifacts have preserved data in isolated tests.
- [x] Native SQLite backup, verification and isolated restore have been demonstrated.
- [x] ADIF corpus, round-trip, unknown fields and published `APP_DHRL_*` contracts are covered.
- [x] Linux package creation, checksum, install, reinstall and uninstall are reproducible in the tested environment.
- [x] Recovery documentation has been exercised in isolated drills.
- [x] Schema 7 and published ADIF/config/path contracts are stable through the current checkpoint.
- [ ] Prolonged everyday use has accumulated enough evidence beyond short regression sessions.
- [ ] Multiple public RC/stable cycles remain consistently green without recurring severe regressions.
- [ ] Runtime compatibility evidence extends beyond the current primary GNU/Linux environments where practical.
- [ ] The full v0.11 stable visual matrix is explicitly recorded for System, Light and Dark using the authoritative QA checklist.

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
- Historical upgrade exercises preserved IDs, timestamps, modes, metadata, ADIF extras and configuration.
- Future schema versions are rejected rather than automatically downgraded.

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

- The permanent corpus contains valid and invalid fixtures and is covered by active regression tests.
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

- 10k and 100k release baselines have remained within the established historical range.
- 100k remains comfortable for normal operations on the measured host.
- No index or optimization is justified without a measured regression.

### Known limit

- One million QSOs remains an extreme/manual case; deep OFFSET pagination and complete ADIF export are intentionally not treated as normal interactive workloads.

## Distribution

### Confirmed

- Release engineering can produce tarball, Debian package and AppImage from one exact release binary.
- Sidecar and aggregate SHA-256 checksums are produced and verified.
- Packaging checks install/reinstall/startup and idempotent uninstall in isolated HOME/XDG environments.
- Published RC1 assets remain preserved and are not rewritten by RC2 preparation.

### RC2 still required

- generate the exact `0.11.0-rc.2` Actions artifact from the approved merged commit;
- verify checksums, archive contents, binary identity and native dependencies;
- exercise installation/startup and the documented package checks using that exact artifact;
- only after approval create the immutable annotated `v0.11.0-RC2` tag and GitHub **Pre-release** without rebuilding.

### Not yet proven broadly

- broad runtime coverage across Fedora/openSUSE and diverse Wayland environments;
- compatibility with every glibc/native-library combination.

## Accessibility and visual behavior

### Confirmed

- Earlier release checkpoints exercised keyboard navigation, focus, mouse operation, clipboard preservation and the `1050×680` reference size.
- The post-RC1 Logbook clipping/filter-layout correction was run locally and accepted by the maintainer before RC2 promotion.

### Still required before stable v0.11.0

- preserve explicit evidence for the full System/Light/Dark visual matrix described in `../quality/VISUAL-QA-v0.11.md`;
- do not infer unchecked items solely from the RC2 promotion acceptance.

## Offline and privacy

### Confirmed

- No telemetry, analytics, account, server or automatic crash reporting exists.
- Core logging, database, ADIF, backup and health operations are local.
- External callsign/grid links require explicit user activation.
- Diagnostic reports omit QSO content by default.

## Dependency and security audit

### Confirmed

- RC2 preparation does not change runtime dependency versions.
- `Cargo.lock` and `fuzz/Cargo.lock` are versioned and synchronized for the RC2 package version without unrelated dependency updates.
- The current RustSec workflow reports no known vulnerability blocking the candidate.

### Follow-up

Issue #10 tracks four informational `unmaintained` advisories in transitive dependencies. They are not confirmed vulnerabilities and must not be force-fixed with incompatible direct overrides merely to silence the warning.

## Known issues and blockers

### Current RC2 blockers

Before publication, RC2 still requires successful final PR/main CI plus validation of the exact generated release-candidate artifact. No Critical/High data-integrity blocker is currently known.

### Future stable / `1.0.0` blockers

- prolonged real-world usage evidence remains insufficient for a maturity declaration;
- repeated stable/public release observations are still needed;
- broad environment/accessibility evidence remains limited;
- final stable v0.11 visual evidence must be explicitly recorded rather than inferred.

## Real-world usage log

Do not record callsigns, QTH, notes or other personal QSO content here. For each checkpoint, record only application version/schema, approximate dataset scale, database size where useful, health/backup/export/restart outcome and reproduced operational friction.

### Evidence accumulated

- v0.9.0: mixed-mode manual regression at `1050×680`, including keyboard, mouse, focus, clipboard and operational flows, approved by the maintainer on 2026-08-28.
- v0.10.0-rc.1: exact isolated artifact exercised with synthetic data during maintainer inspection on 2026-08-28 without a reported problem.
- v0.11 post-RC1: corrected `main` state at `c3ffd3dd49d2dc18ef3b7cf227e77217b47cc7c4` was run locally and accepted by the maintainer on 2026-08-29 before RC2 preparation.

### Evidence still needed

- multiple naturally occurring logging sessions over the `0.x` line;
- periodic verified backups and isolated restore drills using safe copies;
- observation of database growth, pagination and exports during normal use;
- exact packaged RC2 validation before its publication.

## Future ideas

Feature requests do not block maturity. Record them separately from defects and do not implement them during feature freeze.
