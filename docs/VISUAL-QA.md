# Visual QA checklist

Last full visual approval: 2026-08-14 — i3/tiled mode at the default `1050×680` window size.

All items below were approved after Marco 10. No control may require fullscreen to become accessible.

Marco 20 extracted the four pages into separate Slint components without intentional visual changes. The detailed historical checklist and focused regression below were approved on 2026-08-13 in i3/tiled mode.

## Marco 26 — Logbook redesign pilot

Approved on 2026-08-14 in i3/tiled mode at `1050×680`, including the redesigned shell and operational Logbook.

### Shell and sizing

- [x] The unified 66px operational bar is fully visible and readable.
- [x] Logbook, New QSO, Tools and Settings activate with one click and remain keyboard-accessible.
- [x] `+ New QSO` is visually primary without overpowering the current-page context.
- [x] Station callsign and `LOCAL` remain readable with configured, empty and long values.
- [x] No control requires fullscreen and no content overlaps at `1050×680`.

### Search and filters

- [x] QSO total, `LOCAL / UTC`, search field and actions align without clipping.
- [x] Enter submits search and Clear preserves its previous behavior.
- [x] Filters expand and collapse without covering or compressing the QSO list excessively.
- [x] All, DMR and FT8 selectors activate with one click, Enter and Space.
- [x] Active-filter chip is readable and appears only when a filter is active.
- [x] DMR/FT8 validation errors remain visible and keep the filter panel usable.

### Operational QSO list

- [x] The list is the dominant visual element and no longer reads as a spreadsheet.
- [x] Timestamp, callsign, mode, frequency and band are scannable at a glance.
- [x] Route/details, grid and actions form a clear secondary information level.
- [x] Generic, DMR and FT8 entries remain distinguishable without saturated colors.
- [x] Long callsigns, routes and technical details elide without displacing actions.
- [x] Callsign and non-empty grid open with one click and through keyboard activation.
- [x] Empty grid and missing route remain clear without adding unusable focus stops.
- [x] Edit and Delete remain discoverable but visually secondary.

### States, pagination and feedback

- [x] Empty database and no-results states are centered, concise and readable.
- [x] Range, total, current page and page count remain correct on empty, partial and full pages.
- [x] Previous and Next show clear enabled, disabled, hover and focus states.
- [x] Delete confirmation appears only when requested and reserves no space while closed.
- [x] Cancel and Confirm delete are unambiguous by mouse and keyboard.
- [x] Info, success, warning and error messages remain readable and unobtrusive.

### Compatibility boundary

- [x] New/Edit QSO still exposes every generic, DMR and FT8 field through scrolling.
- [x] Tools file selectors, ADIF actions and backup remain fully usable.
- [x] Settings station and external-link controls remain fully usable.
- [x] Global Escape behavior and existing confirmation flows remain unchanged.
- [x] No functional regression is observed in search, filters, pagination, links, editing or deletion.

## Marco 27 — Full visual-language propagation

Approved on 2026-08-14 in i3/tiled mode at `1050×680`, including final dimensional corrections to QSO rows and Local Station.

### New/Edit QSO

- [x] Header clearly distinguishes New QSO from Edit QSO and shows mode/UTC context without clipping.
- [x] Contact and report panels remain compact, aligned and readable.
- [x] Generic mode shows no DMR/FT8 panel and leaves no reserved blank space.
- [x] DMR exposes IDs, talkgroup, timeslot, color code, network, call/access type, repeater, hotspot and notes.
- [x] FT8 exposes SNR values, power, audio frequency, software, protocol and final message.
- [x] Scrolling reaches every conditional field and Notes at `1050×680`.
- [x] Footer remains fixed and Cancel/Save activate with one click, Enter and Space.
- [x] Enter from Notes saves and global Escape retains the established behavior.
- [x] Unsaved-change confirmation is fully visible and keyboard-accessible.

### Tools

- [x] ADIF path, import selector, export selector, preview and export actions align without clipping.
- [x] Long paths elide or remain contained without moving actions off-screen.
- [x] Preview clearly shows total, new, duplicates, invalid, modes, bands and UTC range.
- [x] Duplicate rule and invalid-record details wrap inside their surfaces.
- [x] Cancel and Import actions remain available at the bottom of the preview.
- [x] Backup destination, selector and Create backup remain visible and usable.

### Settings

- [x] Local station identity is visually primary and Save station is fully visible.
- [x] Empty, normal and long callsigns remain contained.
- [x] Callsign and grid URL templates remain readable with long values.
- [x] Restore defaults and Save links remain visible and keyboard-accessible.
- [x] Privacy/offline explanation is readable but visually secondary.

### Cross-page consistency

- [x] All four pages share the same surface, typography, spacing and action hierarchy.
- [x] Navigation still activates with one click and visible keyboard focus.
- [x] No page requires fullscreen or overlaps shell/status at `1050×680`.
- [x] Tab order follows the visual order on every page.
- [x] Info, success, warning and error feedback retain adequate contrast.

## Marco 20 focused regression

- [x] Global header, navigation, exit confirmation and status bar are unchanged.
- [x] Logbook search, filters, table links, edit and delete behave unchanged.
- [x] Generic, DMR and FT8 editors show every field with the same spacing and scrolling.
- [x] Tools selectors, ADIF preview/import/export and backup behave unchanged.
- [x] Settings station and external-link controls behave unchanged.
- [x] `Tab`, `Enter` and `Escape` retain their previous behavior.
- [x] Unsaved-edit and safe-exit confirmations retain their previous behavior.

## Marco 21 focused regression

Approved on 2026-08-13 in i3/tiled mode, including the isolated 10,000-QSO database.

- [x] Pagination bar fits at `1050×680` without clipping or compressing the table excessively.
- [x] Showing range, total and page count remain aligned for empty, partial and full pages.
- [x] Previous and Next enable only when their destination page exists.
- [x] Search criteria remain active while navigating pages.
- [x] DMR and FT8 filter criteria remain active while navigating pages.
- [x] Deleting the last item on the final page returns to the preceding valid page.
- [x] Edit, links and route/details metadata remain correct on later pages.

## Global shell

- [x] Header title and subtitle are fully visible.
- [x] Station badge is readable when configured and when empty.
- [x] Navigation labels are readable; active and hover states are distinct.
- [x] Status bar remains visible and long messages elide instead of overflowing.
- [x] `INFO`, `DONE`, `NOTICE`, and `ERROR` states have readable contrast.
- [x] No page overlaps the header, navigation, or status bar.

## Logbook

- [x] Search field and toolbar buttons are vertically aligned.
- [x] Closed filters leave most of the page available to the table.
- [x] DMR and FT8 panels use natural height and never overlap the table.
- [x] Active-filter summary appears only while a filter is active.
- [x] Table header aligns with data rows.
- [x] Long timestamps, callsigns, frequencies, grids, and route summaries elide cleanly.
- [x] Empty route/details displays `—`.
- [x] Alternating rows and mode badges remain readable.
- [x] Callsign and grid links have clear hover states and open only after a click.
- [x] Empty grid displays `—` and is not clickable.
- [x] Empty database and no-results states are centered and informative.
- [x] Delete confirmation shows title, callsign, warning, Cancel, and Confirm delete.

## New/Edit QSO

- [x] Required-field notice is visible.
- [x] Callsign, UTC date/time, mode, frequency, band, and grid proportions are balanced.
- [x] Report/station fields align consistently.
- [x] DMR card appears only for DMR and all rows remain inside the card.
- [x] FT8 card appears only for FT8 and all rows remain inside the card.
- [x] DMR/FT8 card appears immediately after common fields, with Notes immediately after the active card and no reserved blank space.
- [x] Form scroll reaches Notes and every mode-specific field.
- [x] Footer remains fixed with keyboard hint, Cancel, and primary Save action.
- [x] Validation errors keep the form open.

## Tools

- [x] ADIF and backup cards size naturally around their contents.
- [x] Long paths remain within the fields.
- [x] Import ADIF is secondary and Export ADIF is primary.
- [x] Create backup is primary.
- [x] Descriptions remain readable without clipping.

## Settings

- [x] Local-station card sizes naturally around its contents.
- [x] Callsign field and Save station button align.
- [x] Privacy/offline message is readable.
- [x] External-link templates, privacy notice, Restore defaults, and Save links are fully visible.
- [x] Saving updates the header badge without layout shift or clipping.

## Marco 24 accessibility and keyboard regression

Approved on 2026-08-13 in i3 at `1050×680`; the initial double-click regression caused by the overlapping focus scope was corrected and click-once behavior was re-approved.

- [x] `Tab` reaches Logbook, New QSO, Tools, and Settings in visual order.
- [x] Focused navigation and external lookup actions show a high-contrast Frost outline.
- [x] `Enter` and `Space` activate every focused navigation item.
- [x] Callsign and non-empty grid links are reachable by `Tab` and activate with `Enter` or `Space`.
- [x] Empty grids remain plain text and do not add an unusable Tab stop.
- [x] Native fields and buttons remain in visible order after the custom keyboard actions.
- [x] Search `Enter`, Notes `Enter`, and global `Escape` behavior remain unchanged.
- [x] Accessible semantics are exposed for banner, navigation, one main region per page, search/form regions, labeled key fields, actionable links, and polite status updates.
- [x] All controls remain visible and usable at `1050×680` in i3.

## Keyboard and focus

- [x] `Tab` follows the visible control order only.
- [x] Focus indicator has sufficient contrast on Nord surfaces.
- [x] `Enter` runs search in the search field.
- [x] `Enter` saves from the Notes field.
- [x] `Escape` cancels the QSO form and clears delete confirmation.
- [x] Disabled/selected filter controls remain understandable.

## Content stress cases

- [x] Empty database.
- [x] One generic QSO.
- [x] Multiple generic, DMR, and FT8 QSOs.
- [x] Long DMR route summary.
- [x] Search with no result.
- [x] DMR filter with validation error.
- [x] FT8 filter with validation error.
- [x] Long ADIF and backup paths.
- [x] Long error message in the status bar.
- [x] Station configured and station not configured.
