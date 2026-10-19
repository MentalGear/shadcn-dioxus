---
id: 124
title: "Charts: a small pre-hydration layout jump on the legend cards, and on pie-interactive's Select (SSR renders…"
status: "LANDED 2026-10-19 (already gone since row 215; guarded by chart-prehydration.spec.ts)"
size: "S-M"
area: charts
updated: 2026-10-03
---

# Row 124: Charts: a small pre-hydration layout jump on the legend cards, and on pie-interactive's Select (SSR renders…

## Item

Charts: a small pre-hydration layout jump on the legend cards, and on pie-interactive's Select (SSR renders one `<option>`)

## Status / trigger

**LANDED 2026-10-19 (already gone since row 215; guarded by chart-prehydration.spec.ts)**

## Size

S-M

## Detail

Legend cards (area legend/icons, bar stacked + legend, pie legend): **1.7px at 1440, 6px at 390**, before wasm only -- the legend's 28px is constant while the pre-measure svg (369x180) scales with the card; fix needs container-query units on `.dx-chart`. **Pie interactive:** **4px at 1440, 20px at 390** -- pre-hydration the `Select` renders a single `<option>`, so the description wraps and the card is taller until wasm loads; that is a `Select` component SSR issue (render all options server-side), not the chart. Everything else is within 0.3px (aspect-height rounding) or does not move (line, radar, radial, tooltip tabs, fixed-250px heroes).

## Update 2026-10-19

Measured first, JavaScript-disabled context (the static HTML) vs hydrated, at 1440 and 390 px, on `/charts/` and the bar, line, pie, radar, radial and tooltip tabs (every chart card: card, `.dx-chart`, svg root, legend, header, content, footer, Select trigger; positions relative to the card). Result: **0 differences above 0.05 px** (was 1.7 px / 6 px on the legend cards and 4 px / 20 px on pie-interactive). Row 215 had already landed the construction: the svg's box is CSS alone (container-query units in `aspect_height_css`, `chart.rs`) and the legend reserve is the same default row before and after measuring, so (a) is gone; (b) is gone because the demo's trigger has a fixed width and no longer reflows the header. No Select change was needed, so none was made (no Select-wide risk taken).

What landed: `playwright/chart-prehydration.spec.ts`, which fails the whole class (any chart card whose pre-hydration geometry differs from the hydrated one by more than 0.5 px, at both widths, on every chart route). Run with `CHP_DUMP=1` to print the numbers. Remaining: nothing.
