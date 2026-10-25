---
id: 396
title: "Derive item `index` from registration order"
status: "Owner approved as proposed (2026-10-24, backlog thread): order from registration, index only an…"
size: "L"
area: components
updated: 2026-10-24
---

# Row 396: Derive item `index` from registration order

## Item

Accordion (unread), RadioGroup, DropdownMenu, ContextMenu, Menubar, Select (continuing across groups), Tabs, Combobox and NavigationMenu (`content_index`) take a hand-numbered index that must be unique and ascending; duplicates collide silently.

## Status / trigger

Owner approved as proposed (2026-10-24, backlog thread): order from registration, `index` only an optional override. In a thread.

BACKLOG. Found by row 387's first docs-clarity review; the evidence (file:line per finding) is in the project's shared files, `docs-clarity/2026-10-24-docs-clarity-review.md`.

## Size

L

## Detail

Owner decision before building (public API change). Derive the order from registration (the collection module already orders items) and keep `index` only as an optional override.
