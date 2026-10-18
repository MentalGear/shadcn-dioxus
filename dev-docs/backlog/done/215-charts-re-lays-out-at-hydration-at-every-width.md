---
id: 215
title: "/charts/ re-lays out at hydration at every width"
status: "LANDED 2026-10-18 (lane charts 9c876631), merged on the session branch; deploy pending -- was…"
size: "M"
area: charts
updated: 2026-10-18
---

# Row 215: /charts/ re-lays out at hydration at every width

## Item

`/charts/` re-lays out at hydration at every width

## Status / trigger

**LANDED 2026-10-18 (lane charts 9c876631), merged on the session branch; deploy pending** -- was: **Scheduled 2026-10-15 in `dev-docs/research/next-round-plan-2026-10-15.md`** (set up, waiting for the owner's go). **OPEN 2026-10-14 (found by the row 204 lane)**

## Size

M

## Detail

Plot and axes are re-measured, the header select is resized: 120-630 boxes change at every width. A chart-primitive problem (row 211 fixed only the entry animation, not the layout); `/charts/` is excluded from `layout-stability.spec.ts` until it is fixed, then the exclusion goes.
