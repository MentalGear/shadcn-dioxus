---
id: 343
title: "One virtualize mode in the engine, and a data body for Table"
status: "BACKLOG (owner 2026-10-21). Comes after 340 and before 341 and 342."
size: "M (about 2 to 2.5 days)"
area: primitives
updated: 2026-10-21
---

# Row 343: One virtualize mode in the engine, and a data body for Table

## Item

Move `VirtualListMode` into the shared engine as `VirtualizeMode`, and give Table a data-driven body (`TableRows`) with the same `virtualize: Auto | ContentVisibility | Windowed` and `auto_threshold` API, windowing rows inside a real `<table>`.

## Status / trigger

BACKLOG (owner 2026-10-21). Comes after 340 and before 341 and 342.

## Size

M (about 2 to 2.5 days)

## Detail

New order (owner 2026-10-21, 09:35): 340, then 343, then 341, then 342. The shared design is in row 340's "Amendment" section.

- **Today:** the mode type and `resolve` live in `primitives/src/virtual_list.rs:66-117` (default threshold 1000, cv chunk size at `:74`). Only VirtualList has it. Table renders whatever children it gets and has no virtualization.
- **Target:**
  - `VirtualizeMode` and `resolve` in `primitives/src/virtual/`, with `VirtualListMode` kept as a deprecated alias for one release, so existing callers keep compiling.
  - `TableRows` (themed in `table/component.rs`, with the logic in a primitive): content-visibility chunks using `cv_chunks.rs` as VirtualList does, and Windowed using the one-axis `Engine` and `use_virtual_bridge`, with a top and bottom spacer `<tr>` (one cell spanning all columns, `aria-hidden`) and `aria-rowcount` and `aria-rowindex` on the real rows.
  - Hand-written `TableBody` gets an opt-in content-visibility class only.
  - The docs give one table of the three modes and their costs (find-in-page, server render, screen readers), shared by VirtualList, Table and VirtualGrid.
- **Done when:** specs cover each mode on `TableRows` (row count in the DOM, a sticky `<thead>` while scrolling, Auto switching at the threshold, find-in-page working in content-visibility mode), and the VirtualList specs pass unchanged after the move.
