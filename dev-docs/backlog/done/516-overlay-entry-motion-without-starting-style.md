---
id: 516
title: "Overlay entry motion depends on @starting-style from display: none: dialogs, the scrim and every popover-based overlay snap in where an engine does not animate from it"
status: "LANDED 2026-11-14 with the batch of that date (approved by the owner the same day; built on…"
size: "M (inferred): one shared recipe, the…"
area: components
updated: 2026-11-14
---

# Row 516: overlay entry motion without `@starting-style`

## Item

The entry fade of Dialog, AlertDialog, the Command dialog, the modal scrim and the popover-based overlays is a
transition whose start value comes from `@starting-style` on an element entering from `display: none` or into the top
layer. An engine that does not animate from `display: none` runs the exit and snaps the entry. The owner saw this on
chart tooltips ("fade out but not fade in"); the chart tooltip is always mounted and uses `visibility` and `opacity`
(its own fix), which a top-layer element cannot.

## Status / trigger

LANDED 2026-11-14 with the batch of that date (approved by the owner the same day; built on `lane/overlay-entry-fade`).

## Size

M (inferred): one shared recipe, the component sheets that name it, one spec, one gate rule.

## Detail

- **The construction.** Entry is a keyframe animation bound to the open state (an animation starts when the element
  starts being rendered, in every engine), from the start values the `@starting-style` blocks declare, on the floating
  motion tokens, in one shared recipe of the theme. Exits stay as they are.
- **What changed (2026-11-14, after the batch).** The modal dialog family's entry is no longer a keyframe animation.
  A close while the entry animation was still playing removed `data-open`, which cancelled the animation; Chromium
  starts no transition from a cancelled animation's value, so `use_dialog_open_driver` found no running animation and
  called `close()` at once (the panel snapped shut for any close in the first ~200 ms, and the release suite's
  "the lock holds through the exit" failed). The recipe is now a delayed open state plus a transition: the `<dialog>`
  carries `data-closed` while `showModal()` renders it at its base rule, `data-open` lands about two frames later
  (`use_dialog_open_driver` returns the state the pair follows), and the panel's and the scrim's own `transition`
  runs from that rendered value. A close mid-entry reverses the transition. The open state has three values, `data-closed`, `data-entering` (the rendered first frame, which takes pointer events) and `data-open`; only `data-closed` is inert. `dx-modal-in`, `dx-scrim-in` and
  `--dx-modal-enter` are gone; floating and edge overlays keep their keyframe recipes.
- **Proof.** A spec that samples opacity per frame on open for one overlay of each kind and pins the mechanism (no
  `@starting-style` rule for the element, a CSS animation of the recipe's name on open); the mechanism check stands in
  for the engines the container cannot run. A gate refuses a new `@starting-style` rule outside a reviewed list.
- **Residue.** Firefox, Safari and the native webview are unverified here (row 153 owns those runs).
