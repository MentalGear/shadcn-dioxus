---
id: 124
title: "Charts: a small pre-hydration layout jump on the legend cards, and on pie-interactive's Select (SSR renders…"
status: "OPEN -- measured 2026-10-03 in dev-docs/research/chart-parity-2026-10-03/visual-parity.md (SSR…"
size: "S-M"
area: charts
updated: 2026-10-03
---

# Row 124: Charts: a small pre-hydration layout jump on the legend cards, and on pie-interactive's Select (SSR renders…

## Item

Charts: a small pre-hydration layout jump on the legend cards, and on pie-interactive's Select (SSR renders one `<option>`)

## Status / trigger

**OPEN** -- measured 2026-10-03 in `dev-docs/research/chart-parity-2026-10-03/visual-parity.md` (SSR no-JS vs hydrated + measured); cosmetic, actionable when the owner minds the jump or Select SSR is touched

## Size

S-M

## Detail

Legend cards (area legend/icons, bar stacked + legend, pie legend): **1.7px at 1440, 6px at 390**, before wasm only -- the legend's 28px is constant while the pre-measure svg (369x180) scales with the card; fix needs container-query units on `.dx-chart`. **Pie interactive:** **4px at 1440, 20px at 390** -- pre-hydration the `Select` renders a single `<option>`, so the description wraps and the card is taller until wasm loads; that is a `Select` component SSR issue (render all options server-side), not the chart. Everything else is within 0.3px (aspect-height rounding) or does not move (line, radar, radial, tooltip tabs, fixed-250px heroes).
