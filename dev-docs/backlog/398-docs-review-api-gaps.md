---
id: 398
title: "API gaps the docs review found"
status: "Owner approved every item as proposed (2026-10-24, backlog thread). LANDED: the DataTable…"
size: "M"
area: components
updated: 2026-10-25
---

# Row 398: API gaps the docs review found

## Item

TagGroup removal fires no callback (no `on_remove`); DataTable has no selection/sort/page callbacks; Toast has no `position`; DropdownMenu placement is fixed; virtual list/grid have no public scroll-to-index; Toolbar has no vertical styles; single Combobox has no `name`/clear; ColorPicker has no alpha.

## Status / trigger

Owner approved every item as proposed (2026-10-24, backlog thread). LANDED: the DataTable selection, sort and page callbacks (with row 397, batch 17) and part A (TagGroup `on_remove`, Toast `position`, virtual scroll-to-index through `use_virtual_scroller`, ColorPicker alpha with `Hsva`; batch 18). Part B (DropdownMenu placement, Toolbar vertical styles, single Combobox `name`/clear) is built on lane/menu-combobox-parts, waiting for its batch. Found by row 387's first docs-clarity review; the evidence (file:line per finding) is in the project's shared files, `docs-clarity/2026-10-24-docs-clarity-review.md`.

## Size

M

## Detail

Owner picks per item; each picked item becomes its own lane. Until built, row 394's table says 'not available'.

### Part A as built

- **TagGroup / TagGroupMulti** `on_remove: Callback<Vec<T>>`: one call per removal action, every removed value in list order (a remove button removes one; Delete/Backspace on a selected tag removes every selected removable tag). The tags still hide themselves; the `multi` demo owns its list and drops them there.
- **ToastProvider** `position: ToastPosition` (`TopStart | TopCenter | TopEnd | BottomStart | BottomCenter | BottomEnd`, logical, default `BottomEnd`), written as `data-toaster-side` and `data-toaster-align`. New `position` demo; `swipe_top` uses the prop. Spec `toast-position.spec.ts`.
- **Virtual** `use_virtual_scroller() -> VirtualScroller` and a `scroller` prop on VirtualList, VirtualGrid, VirtualWrapGrid and VirtualJustifiedGrid; `scroll_to_index(index, ScrollAlign)` and `scroll_to_cell(row, column, ScrollAlign)` (VirtualGrid). `ScrollAlign` is MessageScroller's (`Start | Center | End | Nearest`). Windowed mode uses the engine's measured-or-estimated offsets (an unmounted row can be the target); content-visibility uses the DOM. A call before mount is kept and run on attach (last wins). Known limit: with variable row heights, `Center` can be off by the estimate error of the rows between (no re-aim after measurement). Spec `virtual-scroll-to.spec.ts`; the wrap and justified grids have unit tests but no browser test.
- **ColorPicker** value type `Hsv` -> `Hsva` (breaking, pre-1.0), `ColorPickerContext::set_alpha`, opt-in `alpha` prop on `ColorPicker`/`ColorPickerSelect`: an Alpha slider and an `#RRGGBBAA` field. With `alpha` off the UI is unchanged and an incoming alpha is kept. `preview/src/main.rs`'s `BlockColorPalette` follows the type change.

### Part B as built

The three part-B items are built (lane menu-combobox-parts).
(1) `DropdownMenuContent { side, align, side_offset }` (defaults `Bottom`, `Start`, 4, the Radix values; the
themed wrapper forwards them). The measured placement (`use_anchor_position_fallback`) is the one authority and
publishes the resolved side as `data-placed-side`; the content deliberately carries no `data-side`, because the
engine's `[data-side]` anchor rules use an 8px gap. Demo `dropdown_menu/variants/placement`, docs "Placement".
(2) Toolbar vertical: `toolbar/style.css` now has `[data-orientation="vertical"]` rules (a column, groups
stacked, full-width buttons) and a horizontal-separator rule keyed on the separator's own `data-orientation`
(it also fixes a separator that sets `horizontal: true` in a horizontal bar). Demo `toolbar/variants/vertical`,
docs "Orientation". (3) Single `Combobox`: `name` submits the selected value as `item_to_value`'s text
(Base UI's `itemToStringValue`, default the label, so `T` needs no `ToString`), `<form reset>` restores
`default_value`, and `ComboboxClear` now works in a single root too (`ComboboxModel::Single` carries `clear`;
the shared `use_single_selectable_value` returns a `SingleSelectableValue` with a `clear` callback). Wrapper
`show_clear`, demo `combobox/variants/clear_form`. Spec: `playwright/demos-menu-toolbar-combobox-parts.spec.ts`.
