---
id: 489
title: "Menubar items select on pointerdown, so a touch scroll that starts on an item selects it"
status: "DONE 2026-11-06, landed in PR #165 (lane lane/menu-dismiss-489). Found by row 435 (2026-10-31)…"
size: "S"
area: overlays
updated: 2026-11-06
---

# Row 489: Menubar items select on pointerdown, so a touch scroll that starts on an item selects it

## Item

A Menubar item commits on `pointerdown`. On a phone, a finger that lands on an item and then scrolls the page
commits that item: the menu closes and the item's action runs during a gesture the user meant as a scroll. The
ContextMenu, which shares the family's item code, commits on `pointerup` and drops a press that a scroll cancels.

## Status / trigger

DONE 2026-11-06, landed in PR #165 (lane `lane/menu-dismiss-489`). Found by row 435 (2026-10-31): Menubar keeps its PressStart selection model; on touch this conflicts with row 477's rule that a scroll must not act.

**What changed.** `SelectOn::commits_on_press(pointer_type)` / `records_press(pointer_type)` (`primitives/src/menu_item.rs`) pick the path per pointer type inside the one item: a mouse press on a `PressStart` host still commits on `pointerdown`; a touch or pen press records the press on `pointerdown`, commits on the release (the `click` after it) and drops it on `pointercancel`, exactly as `ContextMenu`'s items do. There is no new `SelectOn` variant and no host change (`menubar.rs` still says `PressStart`).

**Pen follows touch (the lane's decision on "Open for the lane").** Row 477 treats a pen like a touch for every tap rule, and a pen on a touch screen scrolls the page the same way a finger does (the browser takes the gesture and sends `pointercancel`), so a pen item press that committed on `pointerdown` would have the same bug. `ContextMenu` already treats pen like touch for its items.

**Focus hand-back (row 140).** The commit for a touch moves from `pointerdown` to the `click`; the hand-back is still `MenuHost::close` (`ctx.focus.set_focus(index)`, the collection's own focus, not the touch-never-refocus `RefocusTrigger` rule), so a tap ends with focus on the Menubar trigger. The item's `pointerdown` still cancels the browser's default (`pointer_select_start`), so the compat `mousedown` that follows a touch release cannot focus the unmounting item. Proven by the tap case below.

**Proof.** `menubar.spec.ts` "An item press": (a) a CDP touch drag that starts on the "Pinned" item scrolls the page and selects nothing (menu open, `aria-checked` unchanged, focus where it was), (b) a tap selects once, the menu closes and focus is on the trigger, (c) a mouse press selects on `pointerdown`, before the release. On the base build `pub-menu489base` (`0de04d77`): (a) red (the menu closed), (b) and (c) green. Parity: `menu-family-parity.spec.ts` gains "a touch pointerdown on an item, with no release, selects nothing" for the three hosts (red for Menubar on base, green on the change), and its header names the new Menubar rule. On the change build: `pnpm run test:changed` 993 passed, 0 failed (48 spec files).

## Size

S

## Detail

- **Model before this row.** `SelectOn::PressStart` (`primitives/src/menu_item.rs`, the `onpointerdown` arm `SelectOn::PressStart => select()`) is Menubar's model, set in `primitives/src/menubar.rs` (the item's `select_on`). Row 435 kept it on purpose (its Decision 1): the commit closes the menu and hands focus back to the trigger inside the one press (`crate::menu_root::keep_focus_on_pointerdown`, row 140). `SelectOn::PressRelease` (ContextMenu) is the other model: `pointerdown` records the press, `pointerup` commits, and `pointercancel` (a scroll takes the pointer) drops it.
- **Conflict.** Row 477 makes a touch or pen press a tap, not a press: a touch that moves into a scroll must not act (its `anchored-placement.spec.ts` "touch scroll vs tap" cases). An item that commits on `pointerdown` breaks that rule for every touch scroll that starts on an item.
- **Proposal.** For a touch pointer, Menubar's item uses the `PressRelease` path (record the press on `pointerdown`, commit on `pointerup`, drop it on `pointercancel`); the mouse keeps `PressStart`. The choice is per pointer type inside the one item, not a new `SelectOn` variant for the host. The focus hand-back moves to the commit for touch: the lane must prove that focus still returns to the Menubar trigger after a tap (row 140's case).
- **Open for the lane.** A pen is not decided here: row 477 treats pen like touch for the tap rule, so the lane decides whether pen follows touch.
- **Spec (written first, on a touch-emulated Menubar):** (a) a touch scroll that starts on a Menubar item selects nothing: the menu stays open, the item's action does not run and focus does not move to the trigger; (b) a tap on an item selects it: the action runs once, the menu closes and focus returns to the trigger; (c) a mouse press on an item still selects on `pointerdown`, as the existing Menubar specs assert. Case (a) must be red on the base build before the change.
- **Parity spec.** Row 435's three-host parity spec (`playwright/oracle/tier1-apg/menu-family-parity.spec.ts`) asserts one selection rule for touch across the hosts, so its Menubar expectation changes with this row, in the same commit.
- **Fits with row 487.** The one dismiss core ([row 487](./487-one-dismiss-core-escape-outside-dismiss-and-focus-return.md)) takes the tap rule for the outside press. This row is the matching rule for the item press inside a menu, so it stays a separate change in `menu_item.rs`.
