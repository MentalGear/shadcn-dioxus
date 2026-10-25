---
id: 398
title: "API gaps the docs review found"
status: "Owner approved every item as proposed (2026-10-24, backlog thread). The DataTable callbacks landed…"
size: "M"
area: components
updated: 2026-10-25
---

# Row 398: API gaps the docs review found

## Item

TagGroup removal fires no callback (no `on_remove`); DataTable has no selection/sort/page callbacks; Toast has no `position`; DropdownMenu placement is fixed; virtual list/grid have no public scroll-to-index; Toolbar has no vertical styles; single Combobox has no `name`/clear; ColorPicker has no alpha.

## Status / trigger

Owner approved every item as proposed (2026-10-24, backlog thread). The DataTable callbacks landed with row 397 (batch 17); part A (TagGroup `on_remove`, Toast `position`, virtual scroll-to-index, ColorPicker alpha) is in a thread; part B (DropdownMenu placement, Toolbar vertical, single Combobox `name`/clear) waits for row 396.

BACKLOG. Found by row 387's first docs-clarity review; the evidence (file:line per finding) is in the project's shared files, `docs-clarity/2026-10-24-docs-clarity-review.md`.

## Size

M

## Detail

Owner picks per item; each picked item becomes its own lane. Until built, row 394's table says 'not available'.

**Update 2026-10-25 (lane menu-combobox-parts, part B):** the three part-B items are built.
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
