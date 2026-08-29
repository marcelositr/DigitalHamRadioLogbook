# CI/CD architecture

This document defines the continuous integration, security, documentation, migration, and release-candidate automation for Digital Ham Radio Logbook.

The repository uses a lightweight trunk-based workflow: `main` is the only permanent branch. Work is performed in short-lived branches, reviewed through pull requests, validated by CI, merged into `main`, and then deleted.

## Principles

- `main` is the single permanent integration branch;
- feature, fix, documentation and maintenance branches are short-lived;
- no development work is committed directly to `main` as part of the normal workflow;
- pull requests target `main`;
- branches are deleted after successful integration;
- workflows have one clear responsibility;
- permissions are minimal and read-only unless publishing explicitly requires write access;
- external GitHub Actions are pinned to full commit SHAs;
- the Rust toolchain is pinned in `rust-toolchain.toml`;
- shared Linux/Rust setup lives in `.github/actions/setup-rust-linux/action.yml`;
- CI validates source; normal CI does not create tags or publish GitHub Releases;
- release-candidate artifacts are built from an exact validated commit;
- release tags are immutable historical references;
- GitHub Releases use explicit pre-release/stable status and are never used as source backups.

## Branch workflow

Normal development follows this model:

```text
main
  ↑
Pull Request
  ↑
feat/* | fix/* | docs/* | chore/*
```

A branch exists only for the lifetime of one coherent change. After the pull request is merged, the branch is removed. The next task starts from the current `main` in a new branch.

Dependabot follows the same model and opens dependency-update pull requests directly against `main`.

## Workflows

### `ci.yml` — pull-request gate

Runs on pushes to `main`, pull requests targeting `main`, and manual dispatch.

Jobs:

- **Quality**
  - repository whitespace check;
  - `cargo fmt --check`;
  - `cargo check --locked`;
  - Clippy with warnings denied.
- **Tests and build**
  - `cargo test --locked`;
  - `cargo build --locked`.
- **Linux packaging smoke**
  - POSIX shell syntax;
  - `packaging/linux/smoke-test.sh`.

These names are intentionally stable so they can be used as required status checks.

### `migrations.yml` — historical database compatibility

Runs the migration-preservation test for schemas 0 through 7 on pushes to `main`, pull requests targeting `main`, and manual dispatch.

Linux dependencies and Rust are installed once. Each source schema remains a separate named step so failures identify the historical version immediately.

### `docs.yml` — documentation integrity

Runs `scripts/check-doc-links.py` on pushes to `main`, pull requests targeting `main`, and manual dispatch.

The validator enforces:

- only `README.md` remains as Markdown in the repository root;
- the professional `docs/` category structure exists;
- live documentation files exist;
- repository-relative links in live documentation resolve.

External URLs are intentionally not tested by CI because third-party availability would create false negatives.

Historical release notes, old QA snapshots, `SPEC.md`, and `PROGRESS.md` are preserved as historical context and are excluded from link-freshness enforcement.

### `security.yml` — dependency advisories

Runs RustSec against `Cargo.lock`:

- weekly;
- manually;
- on dependency-sensitive pushes to `main`;
- on dependency-sensitive pull requests targeting `main`.

The workflow uses a full-SHA-pinned RustSec action and read-only repository permissions.

### `release.yml` — exact release-candidate artifacts

This workflow is manual only.

It requires the operator to enter the exact version expected in `Cargo.toml`. It does not merge, tag, or publish a GitHub Release by itself.

The workflow:

1. checks out the exact selected ref with full history;
2. verifies the requested version and release-note document;
3. runs locked formatting, compile, Clippy, and test gates;
4. validates schemas 0–7;
5. runs packaging syntax and smoke checks;
6. downloads pinned AppImage tooling and verifies SHA-256 hashes;
7. calls `make-release.sh` once, producing the release binary and tarball;
8. derives `.deb` and AppImage from that exact already-built binary;
9. verifies every package contains byte-identical application binary content;
10. verifies all sidecar checksums;
11. writes `BUILD-METADATA.txt` and aggregate `SHA256SUMS`;
12. uploads the entire `dist/` directory as a GitHub Actions artifact.

The uploaded artifact is a release candidate for manual QA and explicit maintainer approval. It is not automatically a published release.

## Toolchain and supply-chain pins

`rust-toolchain.toml` currently pins Rust `1.98.0`.

External Actions are referenced by full commit SHA instead of floating tags.

Current pins introduced with this architecture:

- `actions/checkout` v5 commit `fbc6f3992d24b796d5a048ff273f7fcc4a7b6c09`;
- `dtolnay/rust-toolchain` stable action commit `4360b52568e2003a75bf9bc1d59f33a8e3fc893c`;
- `Swatinem/rust-cache` v2.9.2 commit `6323deb102c322ba6fcbdcafc7e3dddab59af2b6`;
- `rustsec/audit-check` v2.0.0 commit `69366f33c96575abad1ee0dba8212993eecbe998`;
- `actions/upload-artifact` v7 commit `043fb46d1a93c77aae656e7c1c64a875d1fc6a0a`.

The release-candidate workflow also pins and hashes:

- AppImage `appimagetool` 1.9.1, x86-64;
- AppImage type-2 runtime release `20251108`, x86-64.

Dependabot can propose updates to GitHub Actions and Cargo dependencies, but updates still go through normal review and CI against `main`.

## Recommended `main` ruleset

Repository governance should protect `main` with a ruleset that:

- requires pull requests for normal changes;
- blocks force pushes;
- requires the following status checks:
  - `Quality`;
  - `Tests and build`;
  - `Linux packaging smoke`;
  - `Historical schemas 0-7`;
  - `Documentation integrity`;
- optionally requires the branch to be up to date before merge.

The automated security workflow is intentionally not a universal required PR check because it only runs on dependency-sensitive changes and on schedule.

## Release discipline

`main` is the source line; releases are identified by immutable tags, not release branches.

For an RC:

1. prepare and validate the candidate through a short-lived branch and PR to `main`;
2. run the exact release-candidate workflow from the approved commit;
3. perform manual QA on those exact artifacts;
4. create the authorized RC tag;
5. publish GitHub Release with **Pre-release** enabled;
6. never move an existing release tag; use RC2/RC3 or the next patch version when changes are required.

GitHub Releases are distribution records, not backups. Source history remains in Git commits and tags.

## What CI does not replace

Automation does not replace:

- manual visual QA at `1050×680`;
- real-world UI inspection in System, Light, and Dark;
- representative upgrade drills using released artifacts;
- release authorization;
- tag creation;
- final GitHub Release publication.

Those remain governed by `docs/releases/RELEASE-CHECKLIST.md`.
