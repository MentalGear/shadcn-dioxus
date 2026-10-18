---
id: 211
title: "Charts flicker on hydration: no entry animation for charts already painted"
status: "LANDED 2026-10-14 (charts lane 3591bd8)"
size: "S"
area: theme
updated: 2026-10-14
---

# Row 211: Charts flicker on hydration: no entry animation for charts already painted

## Item

Charts flicker on hydration: no entry animation for charts already painted

## Status / trigger

**LANDED 2026-10-14 (charts lane 3591bd8)**

## Size

S

## Detail

Cause confirmed: SSG paints the final chart, hydration sets `data-animate` and restarts from zero. Above-the-fold charts never animate; below-fold ones animate once on first entering view via `use_motion`; no JS / reduced motion = final drawing. Item 9. Plan: `dev-docs/research/next-round-plan-2026-10-14.md`. **Landed:** the trigger was in `primitives/src/chart/components/chart.rs` (`use_entered_view_when` latched for an already-visible chart, so hydration set `data-animate` and restarted the SSG-painted chart). A hydrated chart now reads once at mount whether it was in view (`PAINTED_IN_VIEW_JS` via `document::eval`, wrapper `data-chart-key`): in view -> never animates; below the fold -> animates once at >=40% visible; client-created charts animate once in view; no JS / reduced motion -> the final drawing. `chart-entry-animation.spec` 25 tests; existing chart specs 289 passed; the pie_chart load-animation test now scrolls a below-fold demo. The lane also found the shared-target mtime hazard (row 216). **Open:** row 215 (`/charts/` still re-lays out at hydration).
