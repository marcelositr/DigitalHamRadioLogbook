# Visual QA checklist

Last full visual approval: 2026-08-12 — i3/tiled mode at the default `1050×680` window size.

All items below were approved after Marco 10. Reopen the checkboxes after future UI changes. No control may require fullscreen to become accessible.

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
- [x] Empty database and no-results states are centered and informative.
- [x] Delete confirmation shows title, callsign, warning, Cancel, and Confirm delete.

## New/Edit QSO

- [x] Required-field notice is visible.
- [x] Callsign, UTC date/time, mode, frequency, band, and grid proportions are balanced.
- [x] Report/station fields align consistently.
- [x] DMR card appears only for DMR and all rows remain inside the card.
- [x] FT8 card appears only for FT8 and all rows remain inside the card.
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
- [x] Saving updates the header badge without layout shift or clipping.

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
