---
id: 294
title: "Charts (and leftovers) still allow text selection"
status: "LANDED. Charts 2026-10-20 (lane charts-page); the leftover sweep 2026-10-21 (lane leftovers, sha…"
size: "S"
area: preview
updated: 2026-10-21
---

# Row 294: Charts (and leftovers) still allow text selection

## Item

Dragging or double-tapping across a chart selects its axis labels, legend and tooltip text.

## Status / trigger

**LANDED. Charts 2026-10-20 (lane `charts-page`); the leftover sweep 2026-10-21 (lane `leftovers`, sha 1025fd76).** `.dx-chart` (the `ChartContainer` div; legend, tooltip, svg text and arc labels are all inside it) declares `-webkit-touch-callout: none` and `-webkit-user-select`/`user-select: none`: the theme's control rule must not take `[tabindex]` containers (prose, code), so this is a per-sheet declaration with a reasoned `USER_SELECT_ALLOW` entry in `scripts/check-theme-baseline.sh`, not a theme change. Specs: `user-select.spec.ts` "a chart selects no text" (computed style over every chart part on 4 pages, a real drag and double-click, the shipped stylesheet). **Leftovers (static sweep against shadcn new-york-v4):** progress, badge, card header, tabs trigger text and slider track/range need nothing (shadcn leaves badge, progress and card selectable; tab triggers and the slider thumb are `role=tab`/`slider`, already covered). Still selectable where shadcn has `select-none`: `kbd`, `.dx-label`, the input-group addon (and scroll-area); each is a one-line sheet or theme-baseline change outside this lane's files (theme-touch lane / those components' owners). Originally: OPEN, a follow-up to row 229 (landed 2026-10-18).

## Size

S

## Detail

- **Gap:** `chart/style.css` has no `user-select` at all.
  - SVG `text`, the legend (its `role="graphics-symbol"` is not on row 229's list) and the tooltip are selectable.
  - The chart root is a `[tabindex]` container, which row 229 excludes on purpose.
- **Target:** `user-select: none` plus `-webkit-touch-callout: none` on the chart SVG, legend, tooltip and arc labels.
- Then sweep the leftovers I couldn't confirm statically: progress and slider track/range, badge, kbd, tab-list text and card headers. Use `dev-docs/archive/research/user-select-inventory-2026-10-17.md` as the checklist.
- **Files:** `chart/style.css`, the theme baseline list and `check-theme-baseline.sh`.

## Leftover sweep landed (leftovers lane, 2026-10-21)

Checked against `dev-docs/archive/research/user-select-inventory-2026-10-17.md`, which recorded shadcn new-york-v4 `select-none` in `kbd`, `label` and `input-group` (the other vendored shadcn clone is not in this container, so the inventory is the evidence). None is a button, `summary` or widget role, so the theme's shared control rule does not reach them and the theme-baseline construction asks for a per-sheet declaration with a reasoned `USER_SELECT_ALLOW` entry:

| element | sheet | selector | shadcn |
|---|---|---|---|
| key cap | `kbd/style.css` | `.dx-kbd` | `select-none` |
| form label | `label/style.css` | `.dx-label` (Field's label composes it through slot variables, so it is covered too) | `select-none` |
| input-group addon | `input_group/style.css` | `.dx-input-group-addon` (text and icons inside inherit `none` through `auto`) | `select-none` |

Each gets `-webkit-user-select: none; user-select: none` (the prefixed line carries the scoped `property-no-vendor-prefix` disable). A text field in an input group stays selectable. Left alone, as the sweep said: progress, badge, card header, tab text and slider (shadcn leaves them selectable or the role is already covered). The scroll-area `select-none` in shadcn is on the scrollbar track, which holds no text, so there is nothing to select and no declaration was added.

Spec: `playwright/user-select-kbd-label-addon.spec.ts` (a new file, not `user-select.spec.ts`, which row 331 is shrinking): computed style on `/component/kbd/`, `label/`, `field/`, `input_group/`, a double-click on a label selects nothing, the input beside an addon keeps a selectable value. 6 tests, all pass.
