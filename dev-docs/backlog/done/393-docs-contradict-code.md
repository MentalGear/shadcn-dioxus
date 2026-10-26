---
id: 393
title: "Fix the docs statements the code contradicts"
status: "DONE 2026-10-25. Found by row 387's first docs-clarity review; the evidence (file:line per finding)…"
size: "S"
area: docs
updated: 2026-10-25
---

# Row 393: Fix the docs statements the code contradicts

## Item

Docs claims the code contradicts: AlertDialog 'click-outside dismissal' (it has none); Checkbox sample with indicator children the themed Checkbox ignores, and its indeterminate note vs the code; SidebarRail 'draggable resize handle' (click-to-toggle); DatePicker `read_only` doc inverted; HoverCard `force_mount` doc; Questionnaire 'cannot happen' (logs and disables); Progress 'no reduced-motion rule' (there is one); Field 'label/description/error wiring' (none); Combobox `query` doc; Command `value` 'highlighted'; menus and RadioGroup `index` 'display order' (keyboard order); Accordion `index` 'controls placement' (unread); Calendar `dir` on the themed component.

## Status / trigger

DONE 2026-10-25. Found by row 387's first docs-clarity review; the evidence (file:line per finding) is in the project's shared files, `docs-clarity/2026-10-24-docs-clarity-review.md`.

## Size

S

## Detail

One docs lane over the list; the full list with file:line is in section 1 #3 of the report. Row 392's compile check covers the type-level ones from then on.

## Outcome (2026-10-25)

Each claim was re-verified against the tree first. Fixed in docs: AlertDialog (no click-outside dismissal; Escape and its own buttons only), Checkbox (snippet no longer shows indicator children, and the indeterminate note and variant note both say a click unchecks), SidebarRail (a click-to-toggle button, not draggable), DatePicker `read_only` (it only sets `contenteditable=false`), HoverCard `force_mount` (does not keep the card mounted), Questionnaire (a mismatched part logs a warning and renders disabled), Progress (the indeterminate bar has a reduced-motion rule; the determinate transition has none), Field (wires nothing; `html_for` is manual, `FieldError` is `role=alert`), Combobox `query` (the filter text, not the input's displayed text), Command `value` (picked, not highlighted), and the stale "must define an index" in the ContextMenu and Menubar primitive docs plus Navbar's snippet comments. Already correct on this tree: the menu items and RadioGroup `index` docs (optional keyboard order), Accordion `index` (the prop is gone), Calendar `dir` (the themed prop exists and is forwarded).

Code defects the docs now describe honestly, left for a code row: DatePicker `read_only` does not stop typing or the arrow keys from changing the value, nor the popover from picking; HoverCard `force_mount` has no effect on what is rendered.
