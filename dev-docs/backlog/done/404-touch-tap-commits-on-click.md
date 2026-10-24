---
id: 404
title: "A touch tap on a Select, Combobox, Command or context-menu item commits on the tap's click"
status: "LANDED (pending merge) -- owner iOS feedback 2026-10-24: selecting an option in the overview's…"
size: "S"
area: primitives
updated: 2026-10-24
---

# Row 404: a touch tap on a listbox or menu item commits on the tap's click

## Item

On iOS, tapping an option in the library Select on the overview form opened the native select underneath. The
shared item-press helper in `primitives/src/selectable.rs` committed a touch press on `pointerup`, which closed the
popup before the browser fired the tap's `click`, so the click landed on the page below the closed panel. Select,
Combobox, Command and context-menu items share that helper.

## Status / trigger

**LANDED (pending merge)** -- owner iOS feedback 2026-10-24: selecting an option in the overview's custom Select,
then Apple again, opened the native select below it. Fix: `fix/select-touch-tap-through` (23adc231). A touch or pen
press commits on the tap's `click`; a mouse press still commits on `pointerup`. `playwright/select.spec.ts` has the
regression test (on a build without the fix the click lands on the `SECTION` below).

## Size

S

## Detail

The press helper reads the pointer type of the press: `touch` and `pen` defer the commit to the `click` that ends the tap, so the popup stays open until the browser has delivered it; `mouse` keeps the `pointerup` commit, which is what makes press-drag-release selection work.
