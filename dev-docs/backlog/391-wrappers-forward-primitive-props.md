---
id: 391
title: "Themed wrappers forward every primitive prop, enforced by a props-diff gate"
status: "BACKLOG. Found by row 387's first docs-clarity review; the evidence (file:line per finding) is in…"
size: "M"
area: components
updated: 2026-10-24
---

# Row 391: Themed wrappers forward every primitive prop, enforced by a props-diff gate

## Item

Themed wrappers drop primitive props their docs describe; the value falls into the attribute spread and does nothing. `dir` on Navbar, RadioGroup, Menubar, Resizable, Tabs, Slider, Toolbar, Select/SelectMulti (and Calendar has none); `id` on PopoverRoot, MenubarTrigger, NavbarTrigger; `axis` on DragAndDropList; `disable_pointer_dismissal` on Sheet.

## Status / trigger

BACKLOG. Found by row 387's first docs-clarity review; the evidence (file:line per finding) is in the project's shared files, `docs-clarity/2026-10-24-docs-clarity-review.md`.

## Size

M

## Detail

- Forward each dropped prop (or delete the docs sentence where hiding it is deliberate).
- By construction: a gate that reuses `preview/props_scan.rs` to diff each wrapper's props against its primitive's and fails on a primitive prop that is neither forwarded nor listed in an allowlist with a reason.
- Also fix VirtualGrid's `dir` (keys only, no `dir`/`data-direction` emitted) unless row 388 takes it.
