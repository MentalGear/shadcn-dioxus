---
id: 489
title: "Menubar items select on pointerdown, so a touch scroll that starts on an item selects it"
status: "BACKLOG. Found by row 435 (2026-10-31): Menubar keeps its PressStart selection model; on touch this…"
size: "S"
area: overlays
updated: 2026-10-31
---

# Row 489: Menubar items select on pointerdown, so a touch scroll that starts on an item selects it

## Item

A Menubar item commits on `pointerdown`. On a phone, a finger that lands on an item and then scrolls the page
commits that item: the menu closes and the item's action runs during a gesture the user meant as a scroll. The
ContextMenu, which shares the family's item code, commits on `pointerup` and drops a press that a scroll cancels.

## Status / trigger

BACKLOG. Found by row 435 (2026-10-31): Menubar keeps its PressStart selection model; on touch this conflicts with row 477's rule that a scroll must not act.

## Size

S

## Detail

- **Model today.** `SelectOn::PressStart` (`primitives/src/menu_item.rs`, the `onpointerdown` arm `SelectOn::PressStart => select()`) is Menubar's model, set in `primitives/src/menubar.rs` (the item's `select_on`). Row 435 kept it on purpose (its Decision 1): the commit closes the menu and hands focus back to the trigger inside the one press (`crate::menu_root::keep_focus_on_pointerdown`, row 140). `SelectOn::PressRelease` (ContextMenu) is the other model: `pointerdown` records the press, `pointerup` commits, and `pointercancel` (a scroll takes the pointer) drops it.
- **Conflict.** Row 477 makes a touch or pen press a tap, not a press: a touch that moves into a scroll must not act (its `anchored-placement.spec.ts` "touch scroll vs tap" cases). An item that commits on `pointerdown` breaks that rule for every touch scroll that starts on an item.
- **Proposal.** For a touch pointer, Menubar's item uses the `PressRelease` path (record the press on `pointerdown`, commit on `pointerup`, drop it on `pointercancel`); the mouse keeps `PressStart`. The choice is per pointer type inside the one item, not a new `SelectOn` variant for the host. The focus hand-back moves to the commit for touch: the lane must prove that focus still returns to the Menubar trigger after a tap (row 140's case).
- **Open for the lane.** A pen is not decided here: row 477 treats pen like touch for the tap rule, so the lane decides whether pen follows touch.
- **Spec (written first, on a touch-emulated Menubar):** (a) a touch scroll that starts on a Menubar item selects nothing: the menu stays open, the item's action does not run and focus does not move to the trigger; (b) a tap on an item selects it: the action runs once, the menu closes and focus returns to the trigger; (c) a mouse press on an item still selects on `pointerdown`, as the existing Menubar specs assert. Case (a) must be red on the base build before the change.
- **Parity spec.** Row 435's three-host parity spec (`playwright/oracle/tier1-apg/menu-family-parity.spec.ts`) asserts one selection rule for touch across the hosts, so its Menubar expectation changes with this row, in the same commit.
- **Fits with row 487.** The one dismiss core ([row 487](./487-one-dismiss-core-escape-outside-dismiss-and-focus-return.md)) takes the tap rule for the outside press. This row is the matching rule for the item press inside a menu, so it stays a separate change in `menu_item.rs`.
