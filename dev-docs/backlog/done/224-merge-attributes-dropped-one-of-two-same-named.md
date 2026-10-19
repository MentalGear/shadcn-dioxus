---
id: 224
title: "merge_attributes dropped one of two same-named event listeners"
status: "LANDED 2026-10-15 (d40091b, merged in 83758d5)"
size: "M (done)"
area: process
updated: 2026-10-15
---

# Row 224: merge_attributes dropped one of two same-named event listeners

## Item

`merge_attributes` dropped one of two same-named event listeners

## Status / trigger

**LANDED 2026-10-15** (d40091b, merged in 83758d5)

## Size

M (done)

## Detail

Found only by the full-suite validation of c3fafe0 (34 failures: dialogs, sheets and drawers did not close, Button-triggered menus did not open), never by a lane alone: row 155 group 6 (3b81edf) moved Button's `onclick`/`onmousedown`/`onmouseup`/`onkeydown` into an always-present attribute list merged LAST, and `merge_attributes` let the later same-named attribute win, so callers' listeners passed through the spread (DialogClose, SheetClose, DrawerClose, `as: Button` triggers) were replaced by Button's no-op. The same flaw worked in reverse at every `[base, attributes]` site (a caller's `onclick` replaced a primitive's own). The main loop approved the Button change in review on the false premise that callers never pass handlers through the spread. **Construction:** same-named listeners now chain in `merge_attributes` (primitives/src/lib.rs): the later list (the caller's) runs first, the earlier (the component's own) runs after it unless the caller called `prevent_default` (Radix `composeEventHandlers`); an identical callback runs once; `AttributeValue::None` still removes. Button emits a handler only when its prop is `Some`. Unit tests (791 lib tests) + `playwright/attribute-listeners.spec.ts` (7). Re-validation: no double-fire anywhere in the suite.
