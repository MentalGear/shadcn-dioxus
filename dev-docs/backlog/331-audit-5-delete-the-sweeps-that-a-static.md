---
id: 331
title: "Audit 5: delete the sweeps that a static gate or another spec already covers"
status: "BACKLOG (owner 2026-10-21: do it if it holds up; overlaps row 152b)"
size: "S"
area: tests
updated: 2026-10-21
---

# Row 331: Audit 5: delete the sweeps that a static gate or another spec already covers

## Item

Audit 5: delete the sweeps that a static gate or another spec already covers.

## Status / trigger

**BACKLOG (owner 2026-10-21: do it if it holds up; overlaps row 152b)**

## Size

S

## Detail

Audit section 2 #5, ~445 s: `demo-forms-no-navigation` (covered by `check-demo-forms.sh`), `dx-class-migration` and `stylesheets-present` (covered by the CSS bundle and `check-css-delivery.sh`), the `rtl-toggle` per-component overflow check (reads `scrollWidth`, which `all-routes.spec.ts:17-20` says proves nothing under `overflow-x: clip` -- unverified), and the `user-select`, `touch-double-tap` and `theme-all-reset` sweeps once row 330 lands. Verify each overlap with evidence before deleting; durations.json entries go with them.
