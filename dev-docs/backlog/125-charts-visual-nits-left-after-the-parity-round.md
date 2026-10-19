---
id: 125
title: "Charts visual nits left after the parity round: radar legend ~12px lower than shadcn; pie-interactive Select…"
status: "LANDED 2026-10-19 (radar legend overhang, pie-interactive swatch, area icons glyph color)"
size: "S"
area: charts
updated: 2026-10-03
---

# Row 125: Charts visual nits left after the parity round: radar legend ~12px lower than shadcn; pie-interactive Select…

## Item

Charts visual nits left after the parity round: radar legend ~12px lower than shadcn; pie-interactive Select lacks the color swatch; area `icons` legend glyphs muted, not foreground

## Status / trigger

**OPEN** -- cosmetic, listed in `dev-docs/research/chart-parity-2026-10-03/visual-parity.md`; not covered by the parity tests

## Size

S

## Detail

(a) **Radar legend / icons:** geometry matches (r 96, centre 125) but ours sits below the 250px square (box 250x278) while shadcn's overhangs the box bottom by 10px (`margin bottom -10`, `mt-8`) -- Recharts centres Radar on the whole box (`formatAxisMap`), so Radar/RadialBar deliberately keep the full square; matching needs a negative legend margin option. (b) **Pie interactive:** shadcn's select trigger shows the selected slice's color swatch, ours does not. (c) **Area `icons` demo:** our legend icon glyphs are muted grey, shadcn's are foreground. Take these together in one small pass; extend the demo-level parity tests (row 119) rather than adding screenshot checks.

## Update 2026-10-19

(a) `ChartLegendProps::overhang: Option<f64>` (`primitives/src/chart/components/legend.rs`): the legend hangs that many px past the chart box's end edge with zero net layout room (two block margins that cancel, from the legend's measured or default height), so the radar and radar-icons demos use `overhang: 10.0` and the container stays the 250 px square (measured: svg 250, legend bottom 10 px below the box). (b) pie-interactive's Select now has a custom trigger with the selected slice's 12 px swatch (and swatches on the options); it follows the selection. (c) legend `chart-icon` glyphs are `--dx-foreground` (tooltip ones stay muted). Evidence: `playwright/chart-parity-nits.spec.ts` (4 tests, demo-level, no screenshots). Remaining: nothing.
