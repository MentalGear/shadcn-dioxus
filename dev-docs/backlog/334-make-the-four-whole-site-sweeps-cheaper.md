---
id: 334
title: "Make the four whole-site sweeps cheaper (all-routes, first-paint-styles, layout-stability, theme-picker-phone)"
status: "BACKLOG (owner approved 2026-10-21, option (c) of row 323)"
size: "M"
area: tests
updated: 2026-10-21
---

# Row 334: Make the four whole-site sweeps cheaper (all-routes, first-paint-styles, layout-stability, theme-picker-phone)

## Item

Make the four whole-site sweeps cheaper (all-routes, first-paint-styles, layout-stability, theme-picker-phone).

## Status / trigger

**BACKLOG (owner approved 2026-10-21, option (c) of row 323)**

## Size

M

## Detail

Row 323's measurement (lane selector, 2026-10-21): any docs-shell-wide or token CSS change must run four sweeps on every page, and they are 4,659 of the 6,570 worker-seconds a stylesheet change selects -- `all-routes` 2,308 s (axe at ~4.3 s per test is the biggest line), `first-paint-styles` 1,086 s, `layout-stability` 819 s, `theme-picker-phone` 446 s. Row 327 (Round 4) already attacks `all-routes`' six loads per route; this row covers the rest: profile each sweep's per-test cost (navigation vs probe vs axe), share one navigation across probe families, run axe once per (route, scheme) with only the rules that can change, and check whether `layout-stability` and `theme-picker-phone` can run on a page sample when no shell CSS changed. Build only what a measurement shows holds up (owner condition, rows 320-331). Option (b) of row 323 -- a 10-page sample for shell-wide CSS in lanes (-35%) -- stays unbuilt: unsound in principle, revisit only if the lock queue still hurts after this row.
