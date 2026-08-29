# Reproducible release checklist

This checklist is executable release discipline, not a declaration that a version is ready. Record exact commands, commit, artifact hash and manual approvals in the version-specific regression/readiness document.

The repository automation is documented in [`../operations/CI-CD.md`](../operations/CI-CD.md). The manual `Release candidate` workflow may generate candidate assets, but it never merges, tags, or publishes a GitHub Release.

## 1. Scope and branch

- [ ] Confirm the working branch follows the existing `develop` → `main` process.
- [ ] Confirm the working tree is clean and record the baseline commit.
- [ ] Confirm every change is a bugfix, compatibility, diagnostic, test, documentation or release-engineering change during feature freeze.
- [ ] Review open issues, known blockers and TODO/FIXME findings.

## 2. Version and documentation

- [ ] Set the intended RC/final version in `Cargo.toml`.
- [ ] Synchronize `Cargo.lock` and `fuzz/Cargo.lock` without unrelated dependency updates.
- [ ] Update `CHANGELOG.md` with only factual changes.
- [ ] Update release notes, `../project/PROGRESS.md`, readiness and regression documents.
- [ ] Confirm README, recovery, ADIF, support matrix and limitations match behavior.
- [ ] Confirm `PROGRAMVERSION` will be derived from the intended compiled version.

## 3. Cargo quality gates

The CI uses the Rust toolchain pinned in `rust-toolchain.toml`.

```sh
cargo fmt --check
cargo check --locked
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked
cargo build --locked
```

- [ ] Formatting passes.
- [ ] Check passes.
- [ ] Clippy passes with warnings denied.
- [ ] All active tests pass; record active/ignored counts.
- [ ] Build passes.
- [ ] Repeat the suite when investigating flakiness; never mask a flaky test with retries.

## 4. Database and migrations

For every schema 0–7, use the exact migration preservation test used by `.github/workflows/migrations.yml`:

```sh
MIGRATION_SOURCE_VERSION=N cargo test --locked \
  database::migrations::tests::migrates_every_supported_schema_version_without_losing_data \
  -- --exact
```

- [ ] All historical schemas migrate.
- [ ] Second opening is idempotent.
- [ ] `PRAGMA quick_check` passes.
- [ ] `PRAGMA foreign_key_check` returns no rows.
- [ ] IDs, UTC timestamps, modes, metadata and ADIF extras are preserved.
- [ ] Future schema is rejected without writing.
- [ ] Automatic downgrade remains unsupported and documented.

## 5. Real upgrade paths

- [ ] Select representative published release artifacts, not only synthetic schema numbers.
- [ ] Verify historical artifact checksums before use.
- [ ] Test a direct old-release → current RC upgrade.
- [ ] Test the relevant sequential release chain.
- [ ] Run health check, backup/verify and restart after upgrade.
- [ ] Compare semantic data and configuration.
- [ ] Use only isolated XDG directories and synthetic/safe copies.

## 6. ADIF

- [ ] Valid and invalid corpus passes.
- [ ] Generic, DMR, FT8, D-STAR and YSF/C4FM round-trips pass.
- [ ] Published `APP_DHRL_*` fields and historical aliases remain compatible.
- [ ] Unknown fields, types, duplicates and Unicode remain preserved.
- [ ] Export is deterministic and uses the intended `PROGRAMVERSION`.
- [ ] Import preview/cancel performs no writes.
- [ ] Filtered export includes all matching pages.
- [ ] Destination overwrite and failed-publication cleanup are tested.
- [ ] Run fuzzing if the parser changed; otherwise record whether a smoke run was performed.

## 7. Backup, health and recovery

- [ ] Healthy/current and old/migratable backups pass verification.
- [ ] Future, incomplete, non-SQLite and corrupt backups are rejected without modification.
- [ ] Health inspection is read-only and its report excludes QSO content by default.
- [ ] Create a native backup in an isolated environment.
- [ ] Execute the documented restore procedure with the application closed.
- [ ] Reopen and compare Generic, DMR, FT8, D-STAR, YSF/C4FM and ADIF extras.
- [ ] Create and verify a post-restore backup.

## 8. Performance

- [ ] Run the 10k release baseline.
- [ ] Run the 100k release baseline.
- [ ] Compare startup/open, first/deep pages, search, filters, backup, health and ADIF against historical measurements.
- [ ] Investigate only significant regressions.
- [ ] Do not add indexes, caches, threads or async without evidence.
- [ ] Keep the 1M test manual/ignored unless resources and purpose justify it.

## 9. Packaging dry run

```sh
sh -n packaging/linux/*.sh
packaging/linux/smoke-test.sh
```

- [ ] Shell syntax passes.
- [ ] Packaging smoke test passes.
- [ ] Tarball generation is reproducible with the same inputs/environment.

## 10. Generate the exact RC artifact once

Preferred automated path:

1. open **Actions → Release candidate**;
2. choose the exact branch/commit ref;
3. enter the exact version from `Cargo.toml`;
4. run the workflow;
5. download the resulting Actions artifact.

The workflow compiles the release binary exactly once, creates the tarball, derives `.deb` and AppImage from that same binary, proves the packaged binaries are byte-identical, verifies sidecar hashes, and records `BUILD-METADATA.txt` plus `SHA256SUMS`.

Equivalent manual tarball path remains:

```sh
packaging/linux/make-release.sh /isolated/output
```

After the exact candidate is generated, do not rebuild and publish a different binary.

- [ ] Record commit, artifact filename, size and SHA-256.
- [ ] Preserve the workflow `BUILD-METADATA.txt` when automation was used.
- [ ] Verify aggregate `SHA256SUMS` and individual sidecars.
- [ ] Verify exact archive contents and permissions.
- [ ] Run `ldd` and confirm no `not found` dependency.
- [ ] Install without sudo in isolated HOME/XDG.
- [ ] Reinstall and start twice.
- [ ] Upgrade a historical database using this exact binary.
- [ ] Execute the disaster drill using this exact binary.
- [ ] Uninstall twice and confirm database/config hashes are unchanged.

## 11. Manual regression

- [ ] Validate `1050×680` using `../quality/VISUAL-QA-v0.11.md` for the v0.11 line.
- [ ] Exercise Generic, DMR, FT8, D-STAR and YSF/C4FM.
- [ ] Exercise Save, Save & New, duplicate Review/Save anyway and mode transitions.
- [ ] Exercise edit, delete, search, filters and pagination.
- [ ] Exercise keyboard, mouse, focus, Tab order, clipboard, Escape and close protection.
- [ ] Exercise ADIF import/export, backup verification and health check.
- [ ] Restart and confirm only explicitly saved QSOs persist.
- [ ] Record maintainer approval without personal QSO content.

## 12. CI and publication gate

- [ ] Push the prepared commit to `develop`.
- [ ] Confirm `Quality`, `Tests and build`, `Linux packaging smoke`, `Historical schemas 0-7`, and `Documentation integrity` pass.
- [ ] Confirm the scheduled/dependency-sensitive RustSec audit has no unresolved applicable advisory.
- [ ] Confirm no Critical or High integrity blocker is known.
- [ ] Obtain explicit maintainer authorization before `main`, final tag or GitHub Release.
- [ ] Fast-forward/merge using the established repository policy.
- [ ] Confirm required CI on `main`.
- [ ] Create the authorized annotated tag.
- [ ] Publish the already validated artifact and checksum; do not rebuild.
- [ ] Download release assets and verify checksum and byte equality.
- [ ] Confirm release notes and prerelease/latest flags are correct.
