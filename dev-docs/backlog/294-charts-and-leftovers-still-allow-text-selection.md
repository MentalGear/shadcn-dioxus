---
id: 294
title: "Charts (and leftovers) still allow text selection"
status: "OPEN, a follow-up to row 229 (landed 2026-10-18), which covered buttons and ARIA control roles but…"
size: "S"
area: preview
updated: 2026-10-19
---

# Row 294: Charts (and leftovers) still allow text selection

## Item

Dragging or double-tapping across a chart selects its axis labels, legend and tooltip text.

## Status / trigger

OPEN, a follow-up to row 229 (landed 2026-10-18), which covered buttons and ARIA control roles but not charts.

## Size

S

## Detail

- **Gap:** `chart/style.css` has no `user-select` at all.
  - SVG `text`, the legend (its `role="graphics-symbol"` is not on row 229's list) and the tooltip are selectable.
  - The chart root is a `[tabindex]` container, which row 229 excludes on purpose.
- **Target:** `user-select: none` plus `-webkit-touch-callout: none` on the chart SVG, legend, tooltip and arc labels.
- Then sweep the leftovers I couldn't confirm statically: progress and slider track/range, badge, kbd, tab-list text and card headers. Use `dev-docs/research/user-select-inventory-2026-10-17.md` as the checklist.
- **Files:** `chart/style.css`, the theme baseline list and `check-theme-baseline.sh`.
