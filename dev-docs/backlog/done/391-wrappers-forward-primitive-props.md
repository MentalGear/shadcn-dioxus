---
id: 391
title: "Themed wrappers forward every primitive prop, enforced by a props-diff gate"
status: "LANDED (merge queue batch 18, lane/wrapper-props-forward): every primitive prop a wrapper drops is…"
size: "M"
area: components
updated: 2026-10-24
---

# Row 391: Themed wrappers forward every primitive prop, enforced by a props-diff gate

## Item

Themed wrappers drop primitive props their docs describe; the value falls into the attribute spread and does nothing. `dir` on Navbar, RadioGroup, Menubar, Resizable, Tabs, Slider, Toolbar, Select/SelectMulti (and Calendar has none); `id` on PopoverRoot, MenubarTrigger, NavbarTrigger; `axis` on DragAndDropList; `disable_pointer_dismissal` on Sheet.

## Status / trigger

LANDED (merge queue batch 18, lane/wrapper-props-forward): every primitive prop a wrapper drops is forwarded or allowlisted with a reason, enforced by `check-wrapper-props`. Trigger for reopening: a new primitive prop that no wrapper forwards. Found by row 387's first docs-clarity review; the evidence (file:line per finding) is in the project's shared files, `docs-clarity/2026-10-24-docs-clarity-review.md`.

## Size

M

## Detail

- Forward each dropped prop (or delete the docs sentence where hiding it is deliberate).
- By construction: a gate that reuses `preview/props_scan.rs` to diff each wrapper's props against its primitive's and fails on a primitive prop that is neither forwarded nor listed in an allowlist with a reason.
- Also fix VirtualGrid's `dir` (keys only, no `dir`/`data-direction` emitted) unless row 388 takes it.

Built:
- Gate `scripts/check-wrapper-props.sh` (scanner `scripts/wrapper-props/main.rs`, sharing `preview/props_scan.rs` with the docs props tables): every call of a primitive component in a `component.rs` (a `module::Name { .. }` call, a name imported from a primitive module, or a local component that takes a primitive's props struct, such as `CalendarRoot`) must pass each of the primitive's props by name or through a `..props` spread, or name it in `scripts/check-wrapper-props.allow.tsv` with a reason. Stale allowlist entries fail, so the list only shrinks. The first scan found 56 dropped props in 19 wrapper files, plus the top_layer fixture page (allowlisted whole).
- Forwarded: `dir` on Navbar, RadioGroup, Menubar, ResizablePanelGroup, Tabs, Slider, RangeSlider, Toolbar, Select, SelectMulti, Calendar, RangeCalendar (new wrapper field + through `CalendarRoot`/`RangeCalendarRoot`), ContextMenu and DropdownMenu; `modal` on ContextMenu and DropdownMenu; `id` on PopoverRoot, MenubarTrigger, NavbarTrigger, ContextMenuTrigger and DropdownMenuTrigger; `axis` on DragAndDropList; `disable_pointer_dismissal`, `initial_focus` and `final_focus` on Sheet; `is_modal` on CommandDialog; `gap` on CarouselContent; `onclick_only` and `rel` on NavbarItem; `remove_label` on ComboboxMulti.
- Allowlisted (24, each with its reason): internal ids and parts of the composite Select, Combobox, Command and Calendar; the scroller inside AttachmentGroup; the popover inside ColorPicker; Slider's single thumb index; SelectValue's default placeholder.
- Not covered: VirtualGrid's `dir` (left to row 388).
