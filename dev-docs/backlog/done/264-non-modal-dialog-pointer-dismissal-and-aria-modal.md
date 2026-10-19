---
id: 264
title: "Non-modal Dialog/Drawer: aria-modal and an outside-press switch"
status: "LANDED 2026-10-19 (aria-modal derived per arm; disable_pointer_dismissal on Dialog, forwarded by…"
size: "S"
area: primitives
updated: 2026-10-19
---

# Row 264: Non-modal Dialog/Drawer: aria-modal and an outside-press switch

## Item

The non-modal Dialog arm still renders `aria-modal="true"`, and a non-modal Drawer has no way to ignore outside presses.

## Status / trigger

**LANDED 2026-10-19 (aria-modal derived per arm; disable_pointer_dismissal on Dialog, forwarded by Drawer).** Was BACKLOG (left open by row 156, 2026-10-19).

## Size

S

## Detail

Row 156 landed the non-modal Drawer (`is_modal: false`: no scrim, page interactive and scrollable, no trap, no scroll lock),
composed on `primitives/src/dialog.rs`. Two gaps live in `dialog.rs`: (1) the non-modal arm still sets `aria-modal="true"`,
which tells assistive tech the page behind is inert when it is not -- derive `aria-modal` from `is_modal` in the one place
the arm is chosen; (2) Base UI's `disablePointerDismissal`: today a press or focus outside a non-modal drawer dismisses it,
so the first page click both acts and closes. Add an opt-in prop on Dialog that Drawer forwards; spec both, plus an axe
check that a non-modal panel reports no `aria-modal`.

## Update 2026-10-19

Landed. `primitives/src/dialog.rs`: `aria_modal(is_modal)` is the one helper all three content arms build their attribute
from, so the non-modal panel carries no `aria-modal` and only modal arms say `"true"`. New `DialogRootProps::disable_pointer_dismissal`
(Base UI `disablePointerDismissal`, default `false`, carried in `DialogCtx`): the non-modal and native-modal `use_outside_dismiss`
callbacks and the web-modal backdrop dismissal skip `set_open(false)` when it is set; Escape and close controls are untouched.
`DrawerRootProps` and the preview `Dialog`/`Drawer` forward it; the drawer `non_modal` demo gains a second drawer using it.
Spec: `playwright/dialog-non-modal-aria-and-dismissal.spec.ts`. Docs: dialog and drawer `docs.md`. The class removed: any arm
claiming modality it does not have (the attribute now follows the arm), and "outside press always dismisses" (now a switch).
Not covered: popover/alert-dialog keep their own `aria-modal` literals (separate primitives).
