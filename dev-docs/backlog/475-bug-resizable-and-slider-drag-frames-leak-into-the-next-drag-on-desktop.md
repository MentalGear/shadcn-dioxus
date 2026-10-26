---
id: 475
title: "Bug: Resizable and Slider on desktop: a previous drag's queued per-move frames can land in the next drag"
status: "APPROVED 2026-10-26 by the owner (\"Ok for the rest\", backlog thread); a batch 30 lane. BACKLOG.…"
size: "S"
area: components
updated: 2026-10-26
---

# Row 475: Bug: Resizable and Slider on desktop: a previous drag's queued per-move frames can land in the next drag

## Item

Slider and Resizable start a drag in rsx (`move_interaction.rs:119`) and receive its per-move frames from a `window` listener keyed by `pointerId` (`pointer.rs:28-38` sends `[kind, [pointerId, x, y]]`; `update_pointer` writes by id at `:101`; `pointer_move` takes its delta from the last stored position, `move_interaction.rs:148-160`). A mouse's `pointerId` is the same on every press (Pointer Events 3). On desktop a frame is a bridge fact that waits behind any unacknowledged render, while the next drag's `pointerdown` is a Dioxus event handled inline, so a frame of drag k that is still queued is delivered after drag k+1's `pointerdown`. It overwrites pointer 1's stored position, and the first real move of drag k+1 is measured from drag k's last position: the handle jumps by the distance between the two. On web the frames run before the next `pointerdown`, so the renderers diverge. Inferred from the sources, not yet reproduced.
**Where:** Slider and Resizable (and any other user of `use_move_interaction`, such as the drawer's drag) on desktop and mobile; web is unaffected.
**Reproduce first:** drag a Resizable handle, release and re-grab within one render acknowledgement (a quick double drag), on the desktop-cadence web build of [row 442](./442-desktop-cadence-harness-and-desktop-pass.md) or with `__dxBridgeDelayFrames = 2`, and compare the handle position at the second press with the pointer; on a real desktop webview do the same on a busy Rust side (`?dx-bridge-spin` of [row 443](./443-owner-measurement-kit-perf-recorder-and-impl-switch.md)).
**Source:** [`board-2026-10-26-r7/whole-core.md`](../research/board-2026-10-26-r7/whole-core.md) finding 4 (carried by [`board-2026-10-26-r7/SYNTHESIS.md`](../research/board-2026-10-26-r7/SYNTHESIS.md), "Live defect found in today's code").
Related to [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) but not part of it: the plan's gesture epoch (a stream epoch minted at the drag's start by the rsx `pointerdown` and carried by every per-move frame, root doc section 2.2, order contract) subsumes this defect, and a local fix does not wait for the plan.

## Status / trigger

APPROVED 2026-10-26 by the owner ("Ok for the rest", backlog thread); a batch 30 lane.

BACKLOG. Found by board round 7; reproduce first. Independent of plan approval.

## Size

S

## Detail

- **Files owned:** To be found by the reproduction; expect `primitives/src/pointer.rs` and `primitives/src/move_interaction.rs`.
- **Depends on:** None. The plan's gesture epoch ([row 447](./447-phase-0-7-the-order-runtime.md) builds it, [row 455](./455-port-3-gs-gestures-slider-resizable-drawer-toast-pan.md) applies it to Slider, Resizable and the drawer) fixes it by construction; a local fix is to tag each drag with a counter incremented at `pointerdown` and to drop a frame whose tag is not the current drag's, or to clear the stored position at the start of a drag.
- **Proof:** A Playwright spec that fails before the fix and passes after (a re-grab at `__dxBridgeDelayFrames = 2` keeps the handle under the pointer; Resizable reads the container rect on the same channel as its deltas); run on the web build with the delay hook and, where the bug is renderer-dependent, the desktop-cadence build of [row 442](./442-desktop-cadence-harness-and-desktop-pass.md). A desktop webview run is not possible in the lane container and stays "web-verified; desktop inferred".
- **Hours (inferred):** not estimated (size guess S).
