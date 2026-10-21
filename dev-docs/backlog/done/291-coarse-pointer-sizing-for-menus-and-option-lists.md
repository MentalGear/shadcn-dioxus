---
id: 291
title: "Coarse-pointer sizing for menus and option lists"
status: "LANDED 2026-10-21 (lane leftovers, sha 1025fd76; theme part by theme-touch lane 1ba0b2e5). The…"
size: "S–M"
area: theme
updated: 2026-10-21
---

# Row 291: Coarse-pointer sizing for menus and option lists

## Item

Menu rows are about 28 px tall, which is too small for a finger. Add a `pointer: coarse` size step for menus and option lists.

## Status / trigger

**LANDED 2026-10-21 (lane leftovers, sha 1025fd76; theme part by theme-touch lane 1ba0b2e5).** The select and combobox part left in "What was left" is done. Earlier: OPEN (owner review 2026-10-19). **I concur.** Today the only coarse rules are the 16 px text-entry font floor (row 30) and the resizable handle.

## Size

S–M

## Detail

- **Today's sizes:**
  - Shared menu item (context menu, dropdown and menubar): `:where([data-dx-menu-item])` (`dx-components-theme.css:1242`), padding `--dx-menu-item-padding` 4px 6px plus 14 px text, about 28 px tall.
  - Command item: about 32 px.
  - Select option: about 28 px.
  - Menubar trigger: about 24 px.
- **Target:** one coarse override of tokens in the theme: `--dx-menu-item-min-height: 44px`, padding about 10px 12px, text `--dx-text-base`. It applies to context menu, dropdown menu, menubar triggers and items, select and combobox options, and command items. Desktop sizes stay unchanged.
- Use the same media query as `modality.rs`, `(any-pointer: coarse) and (hover: none)`, so a laptop with a touchscreen keeps dense menus.
- Upstream shadcn and Base UI don't ship this as far as I know; I did not check their source. It would be our own addition, kept as tokens so apps can override it.
- **Files:** the theme, plus `select`, `command`, `menubar`, `combobox` and `navigation_menu` `style.css`. Re-record the visual baselines.

## Landed (theme-touch lane, 2026-10-20)

One construction, no per-sheet `@media`: `dx-components-theme.css` declares the slots on `:root` and replaces them in ONE block under `(any-pointer: coarse) and (hover: none)` (the query `modality.rs` uses, so a laptop with a touchscreen keeps dense menus):

| slot | desktop | coarse |
|---|---|---|
| `--dx-menu-item-min-height` | `0px` | `2.75rem` (44px) |
| `--dx-menu-item-font-size` | `--dx-text-sm` | `--dx-text-base` |
| `--dx-menu-item-padding-inline` (and `-block`, composed into `--dx-menu-item-padding`) | `--dx-space-1-5` | `--dx-space-3` |
| `--dx-menu-trigger-min-height` | `0px` | `2.75rem` |
| `--dx-menu-trigger-padding` | `0.5 / 1.5` space | `--dx-space-2 --dx-space-3` |

Readers: the theme's `[data-dx-menu-item]` row (dropdown menu, context menu, menubar items and sub-triggers) and `[data-dx-menu-label]` (inline padding only, so labels stay aligned with item text); `.dx-command-item`, `.dx-navbar-item` (now `display: flex; align-items: center` so a taller row centres its text), the bar triggers `.dx-menubar-trigger`, `.dx-navbar-trigger`, `.dx-navigation-menu-trigger` (the menubar's `height` became `min-height` so the bar grows with its triggers; at rest it is the same 32px). Apps retune one value (`--dx-menu-item-min-height`).

Spec: `playwright/menu-touch-targets.spec.ts` (touch emulation: every item and trigger >= 44px; desktop rows stay 24-34px; the slot is the single knob). Screenshots: `/home/user/tgt/shots-theme-touch/291-*-{before,after}.png`.

## Landed (leftovers lane, 2026-10-21): select and combobox

`.dx-select-option` and `.dx-combobox-option` now read the slots instead of literals: `min-height: var(--dx-menu-item-min-height)`, `font-size: var(--dx-menu-item-font-size)`, `padding-block`/`padding-inline-start` from `--dx-menu-item-padding-block`/`-inline` (desktop values are the old literals: `space-1`, `space-1-5`, `--dx-text-sm`, so the dense rows are unchanged; the inline-end padding is the indicator slot and is untouched). Under touch emulation both measure >= 44px with 16px text.

Spec: `playwright/menu-touch-targets.spec.ts` gained `Select options` and `Combobox options` hosts (>= 44px, 16px text under `hasTouch` + `isMobile`; desktop keeps 24-34px rows and 14px text; the retuned slot moves both). 16 tests, all pass.

Still not changed on purpose: native select (the OS draws its picker), the dropdown/context triggers (ordinary Buttons), navigation-menu panel links (cards, not rows). Visual baselines: none are recorded for the touch sizes. Limit: Playwright cannot emulate a hybrid laptop (coarse AND hover-capable), so "a touchscreen laptop keeps dense menus" rests on the query itself.
