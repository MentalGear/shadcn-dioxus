---
id: 393
title: "Fix the docs statements the code contradicts"
status: "BACKLOG. Found by row 387's first docs-clarity review; the evidence (file:line per finding) is in…"
size: "S"
area: docs
updated: 2026-10-24
---

# Row 393: Fix the docs statements the code contradicts

## Item

Docs claims the code contradicts: AlertDialog 'click-outside dismissal' (it has none); Checkbox sample with indicator children the themed Checkbox ignores, and its indeterminate note vs the code; SidebarRail 'draggable resize handle' (click-to-toggle); DatePicker `read_only` doc inverted; HoverCard `force_mount` doc; Questionnaire 'cannot happen' (logs and disables); Progress 'no reduced-motion rule' (there is one); Field 'label/description/error wiring' (none); Combobox `query` doc; Command `value` 'highlighted'; menus and RadioGroup `index` 'display order' (keyboard order); Accordion `index` 'controls placement' (unread); Calendar `dir` on the themed component.

## Status / trigger

BACKLOG. Found by row 387's first docs-clarity review; the evidence (file:line per finding) is in the project's shared files, `docs-clarity/2026-10-24-docs-clarity-review.md`.

## Size

S

## Detail

One docs lane over the list; the full list with file:line is in section 1 #3 of the report. Row 392's compile check covers the type-level ones from then on.
