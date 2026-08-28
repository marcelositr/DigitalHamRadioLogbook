# Regression matrix — v0.9.0 feature freeze

This is an executable stabilization checklist. It does not authorize new features. Every code change must reference a reproduced bug or a concrete compatibility risk.

## Baseline

- source version: `0.8.0` at merge commit `5b47ade`;
- publication state: integrated in `main`, without a `v0.8.0` tag/release;
- SQLite schema: 7, migrations 1–7;
- supported metadata: Generic, DMR, FT8, D-STAR and YSF/C4FM;
- tests: 175 active + 1 manual stress test ignored by default;
- CI jobs: quality, Linux packaging and migrations from schemas 0–7;
- distribution: user-local Linux x86-64 tarball + SHA-256.

## Automated gates

```sh
cargo fmt --check
cargo check --locked
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked
cargo build --locked
```

Expected: all pass without warnings.

## Database and migrations

- [x] Missing database creates schema 7.
- [x] Zero-byte database initializes safely.
- [x] Non-SQLite and truncated SQLite are rejected without byte changes.
- [x] Future schema is rejected.
- [x] Migration matrix schemas 0–7 passes twice/idempotently.
- [x] `quick_check` and `foreign_key_check` pass after migrations.
- [x] Generic, DMR, FT8, D-STAR, YSF and ADIF extras survive the synthetic matrix.
- [x] Real sequential upgrade `v0.4.0 → v0.5.0 → v0.6.0 → v0.7.0 → 0.8.0` passes in isolated XDG directories.
- [x] Real direct upgrades from a v0.4.0/schema-5 database to v0.6.0, v0.7.0 and 0.8.0 pass.
- [ ] Repeat the real upgrade matrix against the eventual `0.9.0-rc.1` artifact.

## ADIF

- [x] Valid/invalid corpus passes.
- [x] Generic/DMR/FT8/D-STAR/YSF round-trips pass.
- [x] Unknown/private fields, types, duplicates and Unicode survive the documented contract.
- [x] Import preview/cancel performs no writes.
- [x] Existing and intra-file duplicates are handled without merge.
- [x] Filtered export crosses page boundaries and preserves metadata/extras.
- [x] Destination overwrite is refused and failed publication cleans temporary files.
- [x] Parser fuzzing: 60 seconds, 3,622,542 executions, no crash.
- [ ] Repeat corpus and round-trip against the eventual RC artifact.

## Backup, health and recovery

- [x] Healthy, old, future, incomplete, non-SQLite and corrupt databases are classified.
- [x] Read-only inspection preserves bytes/mtime and excludes QSO content from reports.
- [x] Backup is published only after validation and never overwrites an existing destination.
- [x] Restore drill preserves all modes and ADIF extras in an isolated directory.
- [ ] Execute the final disaster drill with the exact RC artifact.

## Editor and operational state

- [x] Save produces one insert; double-action guard rejects reentry.
- [x] Save & New persists only the completed QSO and resets the next form.
- [x] Validation/database failure preserves the form.
- [x] Duplicate create/edit-self/edit-collision/review/save-anyway paths are covered.
- [x] Mode-transition invariant removes incompatible metadata.
- [x] Dirty state includes common and all specialized fields.
- [ ] Manual 50-QSO mixed-mode session in `1050×680`.
- [ ] Manual keyboard, mouse, focus, clipboard, close and pending-preview regression.

## Time, numeric and input boundaries

- [x] UTC parsing/formatting and invalid format rejection.
- [x] Leap day, midnight boundary and invalid date/time rejection.
- [x] Frequency parsing is integer-based and checks overflow/negative/zero.
- [x] Domain limits for DMR, FT8, D-STAR and YSF are covered.
- [x] Unicode and long ADIF notes are covered.

## Performance

Run in release mode:

```sh
DHRL_STRESS_QSOS=10000 cargo test --release --locked \
  database::repository::stress::benchmarks_deterministic_large_database \
  -- --ignored --exact --nocapture

DHRL_STRESS_QSOS=100000 cargo test --release --locked \
  database::repository::stress::benchmarks_deterministic_large_database \
  -- --ignored --exact --nocapture
```

- [ ] 10k baseline recorded for v0.9.0.
- [ ] 100k baseline recorded for v0.9.0.
- [ ] 1M manual stress executed if resources permit.

## Distribution

- [x] Published v0.4.0–v0.7.0 assets match their SHA-256 files.
- [x] Published assets are Linux x86-64 ELF and have no unresolved libraries on the test host.
- [x] Current real tarball builds and checksum matches.
- [x] Clean user-local install, second startup, uninstall and double uninstall pass.
- [x] Database/config hashes remain unchanged by uninstall.
- [ ] Generate `0.9.0-rc.1` once and test that exact artifact without rebuilding.

## Security and privacy

- [x] No runtime `panic!`, `unwrap()` or `expect()` reachable from external input was identified.
- [x] Dynamic SQL uses fixed predicates and bound user values.
- [x] No HTTP client, telemetry, token or credential storage was found.
- [x] Operational logs avoid QSO content.
- [ ] RustSec audit unavailable locally; run when `cargo-audit` or equivalent trusted tooling is available.
- [ ] Confirm the applicable Slint licensing terms for the distributed artifact.

## Manual release blockers

The RC/final release is blocked by any known Critical issue, High integrity issue, broken migration, unsafe restore, supported ADIF data loss or reproducible normal-flow crash.
