---
id: 399
title: "Demo batch from the docs review"
status: "BACKLOG. Found by row 387's first docs-clarity review; the evidence (file:line per finding) is in…"
size: "S"
area: docs
updated: 2026-10-24
---

# Row 399: Demo batch from the docs review

## Item

Supported modes with no demo: ScrollArea Both (the default) and `scroll_type`; Progress indeterminate; Dialog non-modal; DataTable reorder with a sortable column; Pie half-pie and padding; Line gaps; Input OTP letters and `on_complete`; Pagination with disabled ends; Resizable controlled sizes; menus with `modal: false`; Collapsible `keep_mounted`; ButtonGroupText; Avatar Rounded.

## Status / trigger

BACKLOG. Found by row 387's first docs-clarity review; the evidence (file:line per finding) is in the project's shared files, `docs-clarity/2026-10-24-docs-clarity-review.md`.

## Size

S

## Detail

One or two lanes, each verified with `dx serve` and `pnpm run test:changed`.
