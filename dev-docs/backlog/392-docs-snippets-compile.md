---
id: 392
title: "Compile-check the docs.md code samples; samples use the uncontrolled props"
status: "BACKLOG. Found by row 387's first docs-clarity review; the evidence (file:line per finding) is in…"
size: "M"
area: docs
updated: 2026-10-24
---

# Row 392: Compile-check the docs.md code samples; samples use the uncontrolled props

## Item

Docs samples pass a literal to the controlled prop and freeze the component (RadioGroup `value`, Slider `value`, Switch `checked`, Select `value`, Dialog/Drawer `open` with no `on_open_change`, so the dialog cannot close), or do not type-check (Calendar's nonexistent `CalendarDate`, Navbar items without `index`/`to`, Menubar items without `index`, NavigationMenu links without `content_index`).

## Status / trigger

BACKLOG. Found by row 387's first docs-clarity review; the evidence (file:line per finding) is in the project's shared files, `docs-clarity/2026-10-24-docs-clarity-review.md`.

## Size

M

## Detail

- Rewrite the samples with `default_*` and add one shared 'Controlled vs uncontrolled' paragraph that every page links to.
- By construction: extract every rust block in `docs.md` into a compile check under the cargo gates (blocks that are fragments get a wrapper; a marker opts a block out with a reason).
