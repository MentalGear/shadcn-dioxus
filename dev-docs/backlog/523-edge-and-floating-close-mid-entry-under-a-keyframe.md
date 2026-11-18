---
id: 523
title: "Sheet, Drawer and the modal Popover still enter with a keyframe: a close in the middle of that entry cancels it and the exit keyframe starts from the fully open pose"
status: "BACKLOG (found 2026-11-15). Not started. Small; it belongs with the HoverSurface / anchored-panel…"
size: "S."
area: overlays
updated: 2026-11-18
---

# Row 523: an edge or floating overlay closed in the middle of its keyframe entry

## Item

The modal dialog family's entry is a transition again (row 516, changed 2026-11-14), so a close mid-entry reverses it.
Sheet and Drawer (`--dx-edge-enter` / `--dx-edge-exit`) and the modal Popover and the other floating panels
(`--dx-floating-enter` / `--dx-floating-exit`) still enter and exit with `@keyframes` pairs in
`preview/assets/dx-components-theme.css`. A close while the entry keyframe is still playing removes `data-open`,
which cancels the entry; the exit keyframe then starts from its own 0% (the fully open pose), so a half-slid sheet
jumps to the open position and slides out from there. It does not snap shut (the exit is a keyframe of its own, so
`use_dialog_open_driver` finds a running animation), but the jump is visible for a close in the first part of the
slide. Found 2026-11-15 by the dialog lane, which did not touch it.

## Status / trigger

BACKLOG (found 2026-11-15). Not started. Small; it belongs with the HoverSurface / anchored-panel generalisation
(rows 517, 519), which touches the same recipes, and goes in row 528's lane: it waits for the owner's go with that row
(`STATUS.md`, "Next").

## Size

S.

## Detail

- **The construction.** The same as the dialog's: the overlay's own `data-open` lands a frame after it starts being
  rendered (`use_dialog_open_driver` already returns that state for every `showModal()` consumer; the non-modal
  floating panels need the same delayed state from `use_animated_open`), and the entry becomes a `transition` from
  the rendered base pose that a close reverses. The keyframe pairs go, the side tokens stay as the transition's start
  values. No `@starting-style`. `data-entering` (`state_attrs::OpenState`) is the state its transition start keys on.
- **Proof.** `overlay-entry-motion.spec.ts`'s "a close in the middle of the entry" case runs for one Sheet side, the
  Drawer and the modal Popover; the per-frame opacity recordings keep the entry itself pinned.
- **Files.** `preview/assets/dx-components-theme.css`, `primitives/src/lib.rs` (`use_animated_open`),
  `primitives/src/popover.rs`, `primitives/src/sheet.rs`, `primitives/src/drawer.rs`, `playwright/overlay-entry-motion.spec.ts`, CHANGELOG.
