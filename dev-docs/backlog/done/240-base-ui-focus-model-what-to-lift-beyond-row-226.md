---
id: 240
title: "Base UI focus model: what to lift beyond row 226"
status: "LANDED 2026-10-19 (lane focus; focusVisible not adopted, see Update)"
size: "S-M each"
area: a11y
updated: 2026-10-19
---

# Row 240: Base UI focus model: what to lift beyond row 226

## Item

Base UI focus model: what to lift beyond row 226

## Status / trigger

**LANDED 2026-10-19 (lane focus; focusVisible not adopted, see Update)**

## Size

S-M each

## Detail

Research `research/baseui-focus-2026-10-18.md`. Decided now (owner 2026-10-18): open focus = Base UI default (touch/pen -> panel; mouse and keyboard -> first focusable incl. fields, Drawer included -- a deliberate deviation from Base UI's always-popup Drawer, whose reasons (mobile keyboard, swipe) our touch rule and preventScroll already cover), lane mousefocus; text-field rings stay as shadcn (any focus rings a text field; option B, a quieter pointer state, rejected). Remaining: LIFT function-form `initial_focus`/`final_focus` props that receive the open modality (dialog, popover); LIFT a programmatic open returns focus to the previously focused element, not only the trigger (extend `__dxOpener` beyond dialogs); LEARN Select highlights the selected item on a pointer open too, Menu highlights the first item only on a keyboard open (check ours); LEARN `click.detail === 0` as the keyboard fallback in `MODALITY_JS`; LEARN the `focus({ focusVisible })` option (Chrome 144+, feature-detect) for script focus; SKIP `alignItemWithTrigger`, tabindex juggling, Safari blur-on-unmount, `openOnHover` parity.


## Update 2026-10-19 (lane focus)

Landed on the single policy, `primitives/src/modality.rs` (`dxOpenFocus` stays the one open-focus decision; nothing decided on 2026-10-18 changed):

- **Function-form `initial_focus` / `final_focus`** on `DialogContent`, `AlertDialogContent`, `PopoverContent` (new optional props, Dialog/Drawer/Sheet API unchanged). `Option<Callback<OpenModality, FocusTarget>>`; `FocusTarget::{Default, Panel, Selector, None}`. Constructed as inputs, not a second path: the callback is called for the four modalities at render, encoded by `focus_attributes` as `data-dx-initial-focus` / `data-dx-final-focus`, read only by `dxFocusSpec` inside `dxOpenFocus` / `dxCloseFocus`. A selector that matches nothing falls back to the built-in policy. The preview `Dialog`, `AlertDialog` (new `AlertDialogProps`) and `PopoverContent` wrappers forward them; `focus_target` demo variants for all three. Drawer and Sheet compose the primitives directly and are untouched.
- **One opener mechanism.** `dxOpenFocus` assigns `panel.__dxOpener` (the lib.rs copy in `use_dialog_open_driver` is deleted), so the modal Dialog, AlertDialog, modal Popover and the non-modal Popover all remember the element focused before a (programmatic or click) open. `dxCloseFocus(panel)` is the close half: the modal `<dialog>` runs it right after its native `close()` (which already restores to the previously focused element), the non-modal Popover, which leaves the DOM by state, runs it from `use_close_focus` when `open` goes false and focus is still inside the panel. Focus the user moved elsewhere is never taken back.
- **Select / Menu.** Ours already matched Menu (keyboard open: first item; pointer open: none). Select differed: a click open left focus on the listbox. A single Select now opens on its selected option for a pointer open too (`SelectContext::open_for_pointer`); nothing selected, or `SelectMulti`, stays as before (existing multi spec encodes ArrowDown-to-first).
- **`click.detail === 0`** is the keyboard fallback in `MODALITY_JS` (all three copies identical, gate green): a click with no press (screen reader, voice control, script `click()`) writes `keyboard`.
- **`focus({ focusVisible })` not adopted.** The non-field rings are decided by `data-dx-modality`, not by `:focus-visible`, so the option adds nothing there; and `focusVisible: false` on a mouse-opened text field would remove its `--dx-ring-field` ring, contradicting the owner decision that text fields ring on any focus (shadcn parity). `focusVisible: true` after a keyboard open is already what the browser does. Spec asserts a mouse-open field still matches `:focus-visible`.
- **Gate** `scripts/check-open-focus.sh`: `__dxOpener` may be assigned only in modality.rs; `data-dx-initial/final-focus` only encoded/read there; `dxFocusSpec`/`dxCloseFocus` must exist. Files outside the lane list touched: `primitives/src/lib.rs` (use_dialog_open_driver: opener assignment removed, `dxCloseFocus` after the native close).
- **Specs** `playwright/open-focus-lift.spec.ts` (15): initial focus per modality (keyboard, mouse, touch, pen) for Dialog, AlertDialog, Popover; final focus; programmatic open returns to X (Dialog, non-modal Popover closed by state, modal Popover opener); Select/Menu pointer vs keyboard open; detail-0 fallback; field `:focus-visible`. SKIP list unchanged (alignItemWithTrigger, tabindex juggling, Safari blur-on-unmount, openOnHover parity).
