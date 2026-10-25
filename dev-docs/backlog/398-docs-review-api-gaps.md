---
id: 398
title: "API gaps the docs review found"
status: "Owner approved every item as proposed (2026-10-24, backlog thread). The DataTable callbacks landed…"
size: "M"
area: components
updated: 2026-10-24
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
