---
id: 394
title: "A 'Coming from shadcn / Radix / TanStack' table on every component page"
status: "BACKLOG. Found by row 387's first docs-clarity review; the evidence (file:line per finding) is in…"
size: "M"
area: docs
updated: 2026-10-24
---

# Row 394: A 'Coming from shadcn / Radix / TanStack' table on every component page

## Item

Readers search by shadcn, Radix, TanStack and Sonner names and assume their defaults. Names: `asChild`=`as`, `orientation`=`horizontal`, `loop`=`roving_loop`, `onOpenChange`, `defaultValue`, missing Trigger parts, TanStack `getRowId`/`overscan`/`estimateSize`, Sonner `position`. Defaults that differ and are unstated: ScrollArea direction Both, Accordion collapsible true, Separator decorative false, HoverCard side Top, Select position Popper plus scroll lock, Tabs manual activation and empty default, menus modal by default, date clamps 1925-2050 and Sunday week start, Toast cap 10 and fixed position, Button with no default `type`, Sidebar's Sheet below 768px.

## Status / trigger

BACKLOG. Found by row 387's first docs-clarity review; the evidence (file:line per finding) is in the project's shared files, `docs-clarity/2026-10-24-docs-clarity-review.md`.

## Size

M

## Detail

- Per component a small data file (their name, our name, default difference, not available), rendered by `preview/build.rs` as a section like the props reference.
- A gate fails a component without one, so every new component gets the table.
