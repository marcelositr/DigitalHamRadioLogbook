# Digital Ham Radio Logbook v0.11.0

> **Status:** draft / unreleased stable version. The v0.11 line is integrated into `main`, and `v0.11.0-RC1` is published separately as a GitHub **Pre-release** for evaluation. This document does not declare a final `v0.11.0` release or final visual approval.

## Slint-native desktop UI release

Version 0.11.0 is a product-interface reconstruction. Its purpose is to replace the previous visual layer with a simpler, native Slint desktop architecture while preserving the established Rust, SQLite, ADIF, backup, migration, filtering, and QSO behavior.

The release is intentionally not a domain-feature cycle. It changes how the product is presented and documented, not what a QSO means or how the database contract works.

## Highlights

### Fluent becomes the product style

The application now uses **Fluent** as its fixed Slint style. The previous experiment comparing Fluent, Material, Cupertino, and Cosmic is closed: style-family switching is not exposed as an end-user preference.

The style is selected by the Slint compiler configuration in `build.rs`, making Fluent part of the product identity rather than an environment-dependent choice.

### System, Light, and Dark appearance

Settings now provides a dedicated **Appearance** group with three color-scheme choices:

- **System** — default; follows the light/dark preference reported by the desktop;
- **Light** — forces the application into light mode;
- **Dark** — forces the application into dark mode.

The change is applied immediately through `Palette.color-scheme` and persisted in `config.toml`. Existing configuration files without the new appearance section remain valid and default to `System`.

### Native application shell

The main window was rebuilt around Slint-native controls and layout behavior:

- real `MenuBar`, `Menu`, `MenuItem`, and `MenuSeparator`;
- simplified collapsible sidebar;
- one central workspace;
- global status bar;
- content-driven sizing with `preferred-*`, `min-*`, stretch, `Palette`, and `StyleMetrics`;
- minimal custom components only where the standard widget set does not provide the required semantic primitive.

The previous simulated top menu, contextual bar, custom surface system, and decorative navigation categories were removed.

### Logbook becomes a data workspace

The Logbook is no longer presented as a stack of cards. It now uses a compact desktop data layout with aligned columns for UTC, callsign, mode, frequency, band, route/signal, grid, and actions.

Existing behavior remains available:

- search;
- DMR, FT8, D-STAR, and YSF/C4FM filters;
- pagination;
- external callsign/grid lookups;
- edit and confirmed delete actions;
- export of the complete current result set across pages.

### QSO editor rebuilt as a native form

The creation/editing flow now uses a scrollable form with native `GroupBox` sections for:

- Contact;
- Station and report;
- DMR metadata;
- FT8 metadata;
- D-STAR metadata;
- YSF/C4FM metadata;
- Notes.

The functional contract is preserved, including:

- `Save QSO`;
- `Save & New` during creation;
- possible-duplicate warning with `Review` and `Save anyway`;
- unsaved-change confirmation;
- initial callsign focus;
- keyboard shortcuts and Escape behavior.

### Tools and Settings simplified

Tools now groups the existing functionality into native sections for:

1. ADIF import and export;
2. Data health;
3. Database backup.

Settings is organized into:

1. Appearance;
2. Local station;
3. External lookup links.

No service account, cloud, telemetry, or automatic synchronization was introduced.

## Documentation restructuring

The repository documentation was reorganized by responsibility so the root remains focused and the technical corpus is easier to navigate.

```text
docs/
├── README.md
├── architecture/
├── data/
├── operations/
├── quality/
├── releases/
└── project/
```

Key effects:

- `docs/README.md` is now the technical documentation index;
- architecture and UI architecture are separated from historical engineering records;
- ADIF and recovery material live under `docs/data/`;
- Linux distribution/support live under `docs/operations/`;
- QA, regression, hardening, and performance evidence live under `docs/quality/`;
- changelog, release checklist, readiness records, and release notes live under `docs/releases/`;
- `SPEC.md` and `PROGRESS.md` were moved from the repository root to `docs/project/`.

The Linux packaging scripts were updated to consume the reorganized source path while preserving the tarball-facing documentation filename expected by users.

## GitHub Wiki

A dedicated GitHub Wiki was prepared as the user-facing manual, separate from the engineering documentation stored in `docs/`.

The documentation model is now:

- repository `README.md` — concise product entry point;
- GitHub Wiki — navigable user manual and operational guidance;
- `docs/` — engineering architecture, data contracts, QA, project records, and release discipline.

The Wiki covers installation, first run, interface navigation, Logbook, QSO creation/editing, DMR, FT8, D-STAR, YSF/System Fusion, ADIF, backup, recovery, configuration, shortcuts, troubleshooting, architecture, development, and release workflow.

## Compatibility

### Database

- SQLite remains the source of truth.
- Schema remains **version 7**.
- No migration was added for the UI reconstruction.
- Historical schema inputs and migration behavior remain unchanged.
- Automatic downgrade remains unsupported.

### ADIF

- No published `APP_DHRL_*` compatibility contract was removed or renamed.
- DMR, FT8, D-STAR, and YSF/C4FM mapping remains unchanged.
- Unknown-field preservation behavior remains unchanged.
- SQLite backup remains the native recovery format; ADIF remains the interoperability format.

### Runtime and dependencies

- Slint remains the GUI toolkit.
- Rust remains the application language.
- SQLite/rusqlite remains the persistence layer.
- No replacement with Tauri, Electron, Qt, or GTK was performed.
- The UI work does not introduce a new database or network dependency.

### Configuration

The only new persisted product preference in this cycle is the appearance color scheme. It is stored retrocompatibly in `config.toml`; older files remain valid.

## Validation performed during the development cycle

The v0.11 line has repeatedly passed the repository CI while the UI and documentation were being rebuilt. The gate includes:

- `cargo fmt --check`;
- Clippy with warnings denied;
- all active tests;
- application build;
- Linux packaging smoke tests;
- migration validation for historical schemas 0 through 7.

The documentation-path migration also passed the Linux packaging smoke test after `LINUX-DISTRIBUTION.md` moved under `docs/operations/`.

## Manual validation still required before release approval

Technical CI does not replace visual acceptance.

Before v0.11.0 can be considered approved for stable release, the real application must complete the current manual visual gate at the reference window size `1050×680` using Fluent in:

- System;
- Light;
- Dark.

Immediate visual failures include:

- unintended clipped text;
- overlapping labels or controls;
- separators/borders crossing inputs;
- truncated essential buttons;
- inaccessible required content due to missing scroll;
- broken focus/navigation;
- Light or Dark making an essential state unreadable.

The authoritative checklist is `docs/quality/VISUAL-QA-v0.11.md`.

## Release boundaries

This cycle deliberately does **not**:

- add another digital mode;
- change QSO domain semantics;
- alter the SQLite schema;
- replace migrations;
- change duplicate identity rules;
- change backup/restore contracts;
- redefine ADIF mappings;
- add accounts, cloud sync, telemetry, or automatic updates;
- claim Windows or macOS distribution support.

## Release status

`v0.11.0-RC1` is the current public candidate and is intentionally marked as a GitHub **Pre-release**. It may contain regressions, may be replaced by a later release candidate, and does not represent stable or production-ready status.

The final `v0.11.0` release has **not** been published. Final publication still requires the normal project release discipline: successful technical gates, completed manual visual QA, exact-artifact validation, maintainer approval, creation of a new immutable final tag, and explicit stable GitHub Release publication. None of those final publication actions are implied by this draft.
