---
id: 264
title: "Non-modal Dialog/Drawer: aria-modal and an outside-press switch"
status: "BACKLOG (left open by row 156, 2026-10-19)"
size: "S"
area: primitives
updated: 2026-10-19
---

# Row 264: Non-modal Dialog/Drawer: aria-modal and an outside-press switch

## Item

The non-modal Dialog arm still renders `aria-modal="true"`, and a non-modal Drawer has no way to ignore outside presses.

## Status / trigger

**BACKLOG (left open by row 156, 2026-10-19)**

## Size

S

## Detail

Row 156 landed the non-modal Drawer (`is_modal: false`: no scrim, page interactive and scrollable, no trap, no scroll lock),
composed on `primitives/src/dialog.rs`. Two gaps live in `dialog.rs`: (1) the non-modal arm still sets `aria-modal="true"`,
which tells assistive tech the page behind is inert when it is not -- derive `aria-modal` from `is_modal` in the one place
the arm is chosen; (2) Base UI's `disablePointerDismissal`: today a press or focus outside a non-modal drawer dismisses it,
so the first page click both acts and closes. Add an opt-in prop on Dialog that Drawer forwards; spec both, plus an axe
check that a non-modal panel reports no `aria-modal`.
