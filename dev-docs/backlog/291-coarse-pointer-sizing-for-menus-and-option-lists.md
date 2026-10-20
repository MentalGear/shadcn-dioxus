---
id: 291
title: "Coarse-pointer sizing for menus and option lists"
status: "PARTLY LANDED 2026-10-20 (theme-touch lane 1ba0b2e5). Select and combobox are the remaining open…"
size: "S–M"
area: theme
updated: 2026-10-20
---

# Row 291: Coarse-pointer sizing for menus and option lists

## Item

Menu rows are about 28 px tall, which is too small for a finger. Add a `pointer: coarse` size step for menus and option lists.

## Status / trigger

**PARTLY LANDED 2026-10-20 (theme-touch lane 1ba0b2e5).** Select and combobox are the remaining open part (see "What is left"). Earlier: OPEN (owner review 2026-10-19). **I concur.** Today the only coarse rules are the 16 px text-entry font floor (row 30) and the resizable handle.

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

## What is left (owner: apply in batch B or a follow-up; these are the files another session owned)

Both are the same two-line change: read the slots instead of the literals.

- `preview/src/components/select/style.css`, `.dx-select-option` (line ~226): `font-size: var(--dx-text-sm);` -> `font-size: var(--dx-menu-item-font-size);` and add `min-height: var(--dx-menu-item-min-height);` (stylelint order: before `align-items`).
- `preview/src/components/combobox/style.css`, `.dx-combobox-option` (line ~174): the same two lines. (The chips input and the 16px input floor already exist.)
- Then add both to `HOSTS` in `menu-touch-targets.spec.ts` (open the select / combobox demo, assert option height >= 44 under touch emulation).
- Not changed on purpose: native select (the OS draws its picker), the dropdown/context triggers (ordinary Buttons), navigation-menu panel links (cards, not rows).
- Visual baselines: none are recorded for these touch sizes here; re-record if the owner keeps a touch baseline set.
- Limit: Playwright can emulate `(any-pointer: coarse) and (hover: none)` (it does for `hasTouch` + `isMobile`) but not a hybrid laptop (coarse AND a hover-capable fine pointer), so "a touchscreen laptop keeps dense menus" rests on the query itself, not a test.
