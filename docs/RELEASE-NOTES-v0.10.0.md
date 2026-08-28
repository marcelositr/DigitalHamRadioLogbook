# Digital Ham Radio Logbook v0.10.0

## Pre-1.0 stabilization release

Version 0.10.0 continues the feature freeze. It does not add a digital mode, integration, migration or product feature, and it is not a declaration that the project is ready for `1.0.0`.

### Factual maturity record

The project now maintains `docs/PRE-1.0-READINESS.md`, separating demonstrated guarantees from evidence that still requires prolonged everyday use. A future `1.0.0` decision is based on data integrity, historical upgrades, backup/recovery, ADIF compatibility, distribution discipline and real-world observation—not time, commit count or feature count.

### Clear support boundaries

`docs/SUPPORT-MATRIX.md` distinguishes the primary Linux target, environments that were actually tested, best-effort behavior and platforms that have not been tested. The current official distribution remains a user-local GNU/Linux x86-64 tarball; broad Fedora, openSUSE, Wayland, Windows and macOS support is not claimed.

### Reproducible release process

`docs/RELEASE-CHECKLIST.md` records the complete process from locked Cargo gates and schemas 0–7 through real upgrades, ADIF, recovery, packaging, exact-artifact validation, manual regression and maintainer authorization. An artifact validated for publication must not be silently rebuilt afterward.

### Compatibility and recovery

- Database schema remains version 7.
- Migrations 1–7 and historical schema inputs remain retained.
- Automatic downgrade is not supported.
- SQLite backup remains the native preservation format; ADIF remains the interoperability format.
- Published `APP_DHRL_*` fields and historical aliases remain compatibility contracts.
- No runtime dependency was added or updated for this cycle.

### Performance checkpoint

A deterministic 100,000-QSO release run remained within the historical v0.8.0/v0.9.0 range. No optimization, index or schema change was justified.

## Release-candidate status

`0.10.0-rc.1` begins a period of observation and normal use. Prolonged real-world evidence and repeated stable release/RC cycles are deliberately still marked as not proven. The appropriate next activity after technical RC validation is continued use on the `0.x` line, not an automatic move to `1.0.0`.
