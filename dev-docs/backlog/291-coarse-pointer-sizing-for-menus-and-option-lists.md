---
id: 291
title: "Coarse-pointer sizing for menus and option lists"
status: "OPEN (owner review 2026-10-19). I concur. Today the only coarse rules are the 16 px text-entry font…"
size: "S–M"
area: theme
updated: 2026-10-19
---

# Row 291: Coarse-pointer sizing for menus and option lists

## Item

Menu rows are about 28 px tall, which is too small for a finger. Add a `pointer: coarse` size step for menus and option lists.

## Status / trigger

OPEN (owner review 2026-10-19). **I concur.** Today the only coarse rules are the 16 px text-entry font floor (row 30) and the resizable handle.

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
