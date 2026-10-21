---
id: 294
title: "Charts (and leftovers) still allow text selection"
status: "Charts LANDED 2026-10-20 (lane charts-page); the leftover sweep stays OPEN. .dx-chart (the…"
size: "S"
area: preview
updated: 2026-10-20
---

# Row 294: Charts (and leftovers) still allow text selection

## Item

Dragging or double-tapping across a chart selects its axis labels, legend and tooltip text.

## Status / trigger

**Charts LANDED 2026-10-20 (lane `charts-page`); the leftover sweep stays OPEN.** `.dx-chart` (the `ChartContainer` div; legend, tooltip, svg text and arc labels are all inside it) declares `-webkit-touch-callout: none` and `-webkit-user-select`/`user-select: none`: the theme's control rule must not take `[tabindex]` containers (prose, code), so this is a per-sheet declaration with a reasoned `USER_SELECT_ALLOW` entry in `scripts/check-theme-baseline.sh`, not a theme change. Specs: `user-select.spec.ts` "a chart selects no text" (computed style over every chart part on 4 pages, a real drag and double-click, the shipped stylesheet). **Leftovers (static sweep against shadcn new-york-v4):** progress, badge, card header, tabs trigger text and slider track/range need nothing (shadcn leaves badge, progress and card selectable; tab triggers and the slider thumb are `role=tab`/`slider`, already covered). Still selectable where shadcn has `select-none`: `kbd`, `.dx-label`, the input-group addon (and scroll-area); each is a one-line sheet or theme-baseline change outside this lane's files (theme-touch lane / those components' owners). Originally: OPEN, a follow-up to row 229 (landed 2026-10-18).

## Size

S

## Detail

- **Gap:** `chart/style.css` has no `user-select` at all.
  - SVG `text`, the legend (its `role="graphics-symbol"` is not on row 229's list) and the tooltip are selectable.
  - The chart root is a `[tabindex]` container, which row 229 excludes on purpose.
- **Target:** `user-select: none` plus `-webkit-touch-callout: none` on the chart SVG, legend, tooltip and arc labels.
- Then sweep the leftovers I couldn't confirm statically: progress and slider track/range, badge, kbd, tab-list text and card headers. Use `dev-docs/research/user-select-inventory-2026-10-17.md` as the checklist.
- **Files:** `chart/style.css`, the theme baseline list and `check-theme-baseline.sh`.
