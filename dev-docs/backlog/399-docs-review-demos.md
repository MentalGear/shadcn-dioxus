---
id: 399
title: "Demo batch from the docs review"
status: "BACKLOG. Found by row 387's first docs-clarity review; the evidence (file:line per finding) is in…"
size: "S"
area: docs
updated: 2026-10-25
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

**Added 2026-10-25 (owner, backlog thread):**
- Demo names: rename the Accordion "Until found" demo to "Find in Accordion". More generally, many demo names could be
  more fitting and obvious. Audit every demo title and propose a rename list to the owner, then rename what is approved.
  Specs address a demo's frame by its variant directory (`#component-preview-frame-<variant>`), so change the display
  title rather than the directory where the two are separate.
- "Focus target" demos (Dialog, Alert Dialog and Popover `variants/focus_target`): a mouse open does move focus to
  Cancel, but rings are keyboard-only by design, so a mouse user sees nothing happen. Keep the policy and make the demo
  explicit: a live line inside the panel ("Opened with: mouse, focus on: Cancel") and notes that spell out each
  modality's result.
