---
id: 395
title: "`as` keeps the component's children and handlers (Radix asChild semantics)"
status: "BACKLOG. Found by row 387's first docs-clarity review; the evidence (file:line per finding) is in…"
size: "M"
area: components
updated: 2026-10-24
---

# Row 395: `as` keeps the component's children and handlers (Radix asChild semantics)

## Item

With `as` set, Item, Marker, the Collapsible trigger, Attachment and others render only the closure; children and `onclick`/`onkeydown` given to the component vanish. Radix `asChild` keeps them.

## Status / trigger

BACKLOG. Found by row 387's first docs-clarity review; the evidence (file:line per finding) is in the project's shared files, `docs-clarity/2026-10-24-docs-clarity-review.md`.

## Size

M

## Detail

Owner decision before building: merge children and handlers into the closure's element, or keep the current shape, document it once and warn when both are passed.
