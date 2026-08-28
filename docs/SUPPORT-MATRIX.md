# Support and test matrix

This document distinguishes tested environments from support intent. “Tested” means a concrete check was executed; it is not a promise for every system in that family.

## Runtime environments

| Environment | Status | Evidence / limitation |
|---|---|---|
| GNU/Linux x86-64 | Primary | Official user-local tarball format and CI build target |
| Ubuntu 24.04 build environment | Tested | GitHub Actions quality and migration jobs |
| Debian-family local host | Tested | Build, package, install, startup, upgrade, restore and uninstall drills |
| X11 / i3 at `1050×680` | Tested | Repeated visual and keyboard QA through v0.9.0 |
| Wayland | Best effort | Native dependencies are installed in CI, but no formal visual/runtime QA recorded |
| Fedora | Not yet tested | Do not claim official support |
| openSUSE | Not yet tested | Do not claim official support |
| Other Linux architectures | Not tested | Artifact architecture is encoded in its filename |
| Windows | Not supported by current distribution | No package or QA |
| macOS | Not supported by current distribution | No package or QA |

The Linux artifact dynamically links to native system libraries. Passing `ldd` on the build/test host does not guarantee compatibility with every glibc or desktop stack.

## Display and interaction

| Area | Status |
|---|---|
| Mouse | Tested |
| Keyboard-first navigation | Tested |
| Clipboard shortcuts in text fields | Tested |
| Visible focus and Tab order | Tested |
| Slint accessibility semantics | Implemented and manually regressed |
| Multiple screen readers | Not yet broadly tested |
| Baseline window `1050×680` | Tested |
| Smaller windows | Best effort; no formal support promise |

## Data and compatibility

| Contract | Status |
|---|---|
| SQLite schema | Version 7 |
| Historical migration inputs | Schemas 0–7 retained and tested |
| Automatic downgrade | Not supported |
| Native backup | SQLite snapshot, supported |
| Restore | Assisted/documented with application closed |
| Config | TOML/XDG, backwards-compatible defaults for absent fields |
| Invalid config | Preserved and reported; not silently overwritten |
| Uninstall data preservation | Supported and tested |

## Mode metadata

| Mode | Domain/database/UI | Filters | ADIF |
|---|---:|---:|---:|
| Generic | Tested | General search | Tested |
| DMR | Tested | Tested | Tested with private fields/aliases |
| FT8 | Tested | Tested | Tested with standard/private fields |
| D-STAR | Tested | Tested | Canonical and historical forms tested |
| YSF/C4FM | Tested | Tested | Canonical and historical forms tested |
| Other modes | Generic QSO only | General search | Subject to generic ADIF contract |

No claim is made for complete protocol/equipment coverage within DMR, D-STAR or YSF.

## ADIF

| Capability | Status |
|---|---|
| Import preview and cancel | Tested, no writes before confirmation |
| Transactional import | Tested |
| Export all | Tested |
| Export current results across pages | Tested |
| Unknown fields/types/duplicates | Preserved within documented contract |
| UTF-8, LF/CRLF, BOM input | Tested |
| External software certification | Not claimed |

## Offline and external behavior

Core database, logging, search, ADIF, backup, health and configuration functionality is local and does not require a network service. Callsign and grid links use the configured browser only after explicit activation. There is no telemetry, account, cloud synchronization or automatic crash reporting.
