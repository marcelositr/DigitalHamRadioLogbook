# Digital Ham Radio Logbook v0.11.0

> **Status:** draft / unreleased stable version. `v0.11.0-RC1` remains published as a GitHub **Pre-release**; `v0.11.0-RC2` is being prepared from the approved post-RC1 source state. This document does not declare the final `v0.11.0` release approved or published.

## Slint-native desktop UI release

Version 0.11.0 is a product-interface reconstruction. Its purpose is to replace the previous visual layer with a simpler, native Slint desktop architecture while preserving the established Rust, SQLite, ADIF, backup, migration, filtering and QSO behavior.

The release is intentionally not a domain-feature cycle. It changes how the product is presented and documented, not what a QSO means or how the database contract works.

## Highlights

### Fluent becomes the product style

The application uses **Fluent** as its fixed Slint style. Style-family switching is not exposed as an end-user preference. The compiler configuration in `build.rs` makes Fluent part of the product identity rather than an environment-dependent choice.

### System, Light and Dark appearance

Settings provides a dedicated **Appearance** group with three color-scheme choices:

- **System** — default; follows the preference reported by the desktop;
- **Light** — forces light mode;
- **Dark** — forces dark mode.

The change is applied immediately through `Palette.color-scheme` and persisted in `config.toml`. Existing configuration files without the appearance section remain valid and default to `System`.

### Native application shell

The main window was rebuilt around Slint-native controls and layout behavior:

- real `MenuBar`, `Menu`, `MenuItem` and `MenuSeparator`;
- simplified collapsible sidebar;
- one central workspace;
- global status bar;
- content-driven sizing using native layout metrics;
- minimal custom components only where the standard widget set does not provide the required semantic primitive.

The previous simulated top menu, contextual bar, custom surface system and decorative navigation categories were removed.

### Logbook becomes a data workspace

The Logbook is presented as a compact desktop data layout with aligned columns for UTC, callsign, mode, frequency, band, route/signal, grid and actions.

Existing behavior remains available: search, DMR/FT8/D-STAR/YSF filters, pagination, external callsign/grid lookups, edit/delete and export of the complete current result set.

### QSO editor rebuilt as a native form

The creation/editing flow uses a scrollable form with native `GroupBox` sections for contact, station/report, mode metadata and notes. Save, Save & New, duplicate review, unsaved-change protection, focus and keyboard contracts remain preserved.

### Tools and Settings simplified

Tools groups ADIF, data health and database backup. Settings groups Appearance, Local station and External lookup links. No account, cloud, telemetry or automatic synchronization was introduced.

## Post-RC1 correction promoted by RC2

After RC1 publication, the Logbook still exposed clipping/compression at the documented `1050×680` reference size, including the advanced-filter workspace. Commit `c3ffd3dd49d2dc18ef3b7cf227e77217b47cc7c4` corrected that layout without changing backend, SQLite, migrations, ADIF, configuration or domain behavior.

The maintainer then ran the corrected real application locally and reported the test as approved on 2026-08-29. That approval authorizes promotion of the corrected source state to RC2. The exact packaged RC2 artifact remains subject to the release-candidate artifact gate before publication.

The same post-RC1 period also consolidated `main` as the single permanent branch and hardened repository release/security governance. Those changes are administrative/release-engineering changes rather than product features.

## Documentation restructuring

Repository engineering documentation is organized under `docs/architecture/`, `docs/data/`, `docs/operations/`, `docs/quality/`, `docs/releases/` and `docs/project/`, with `docs/README.md` as the technical index. Packaging references were updated to consume those paths while preserving user-facing package documentation contracts.

## Compatibility

### Database

- SQLite remains the source of truth.
- Schema remains **version 7**.
- No migration was added for the UI reconstruction or RC2 promotion.
- Historical schema inputs and migration behavior remain unchanged.
- Automatic downgrade remains unsupported.

### ADIF

- No published `APP_DHRL_*` compatibility contract was removed or renamed.
- DMR, FT8, D-STAR and YSF/C4FM mappings remain unchanged.
- Unknown-field preservation behavior remains unchanged.
- SQLite backup remains the native recovery format; ADIF remains the interoperability format.

### Runtime and dependencies

- Slint remains the GUI toolkit.
- Rust remains the application language.
- SQLite/rusqlite remains the persistence layer.
- RC2 preparation does not introduce a runtime dependency update.
- No replacement with Tauri, Electron, Qt or GTK was performed.

### Configuration

The appearance color scheme is stored retrocompatibly in `config.toml`; older files remain valid.

## Validation state

The v0.11 line is protected by repository CI covering formatting/Clippy, active tests/build, Linux packaging smoke, historical schemas 0–7 and documentation integrity. Dependency-sensitive RustSec checks are reviewed separately and the current audit has no known vulnerability blocking RC2; informational unmaintained transitive dependencies are tracked in issue #10.

The corrected post-RC1 source state passed the maintainer's local application test. RC2 still requires validation of the exact generated package set before its annotated tag and GitHub prerelease are created.

## Manual validation before stable release

Technical CI and RC2 acceptance do not replace the formal stable-release visual evidence. Before final `v0.11.0` is declared stable, the authoritative checklist in `docs/quality/VISUAL-QA-v0.11.md` should explicitly record the required System, Light and Dark visual matrix at `1050×680` rather than infer unchecked items from the RC2 promotion test.

Immediate visual failures remain unintended clipping, overlapping labels/controls, borders crossing inputs, truncated essential buttons, inaccessible content, broken focus/navigation or unreadable essential states.

## Release boundaries

This cycle deliberately does **not** add another mode, alter QSO semantics, change schema/migrations, redefine duplicate rules, change backup/restore or ADIF contracts, add cloud/accounts/telemetry/automatic updates, or claim Windows/macOS distribution support.

## Release status

`v0.11.0-RC1` remains an immutable historical prerelease. `v0.11.0-RC2` is the next candidate and must be built once from the exact approved merged commit, validated, then tagged and published as a GitHub **Pre-release** without rebuilding or moving RC1.

The final `v0.11.0` release has **not** been published. Final publication still requires the normal project release discipline and explicit maintainer approval.
