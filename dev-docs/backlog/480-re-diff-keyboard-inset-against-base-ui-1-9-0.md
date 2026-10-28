---
id: 480
title: "Re-diff primitives/src/keyboard_inset.rs against Base UI 1.9.0"
status: "BACKLOG. From the scan of 2026-10-28."
size: "M"
area: overlays
updated: 2026-10-28
---

# Row 480: Re-diff keyboard_inset.rs against Base UI 1.9.0

## Item

`primitives/src/keyboard_inset.rs` (the Drawer's virtual-keyboard handling) is lifted from Base UI 1.6.0. Upstream has moved; bring it up to 1.9.0 or record why a difference stays. Source: [`radix-vs-base-ui-2026-10-28.md`](../research/radix-vs-base-ui-2026-10-28.md), appendix A (drawer row).

## Status / trigger

BACKLOG. From the scan of 2026-10-28.

## Size

M

## Detail

- **Ours (1.6.0):** `KEYBOARD_RESIZE_THRESHOLD = 60`, `KEYBOARD_SCROLL_SLACK = 48` (`keyboard_inset.rs:34-60`).
- **Base UI 1.9.0** (`DrawerVirtualKeyboardProvider.mjs:18-41`): no slack constant; adds `KEYBOARD_VISIBILITY_MARGIN = 16`, `KEYBOARD_REALIGN_INTERVAL = 150`, `KEYBOARD_SETTLE_FRAME_LIMIT = 60` and a `KEYBOARD_TAP_BLOCKED` state.
- **Method:** diff the two files function by function, port what changes behaviour (the realign interval and the settle limit look like iOS keyboard-animation fixes), keep what the Dioxus port needs differently, and cite the pinned upstream revision in the file's provenance comment.
- **Interaction with row 477:** the Drawer's own placement is not an anchored overlay, so rule P does not change it; the `visualViewport` reads here stay.
- **Files owned:** `primitives/src/keyboard_inset.rs`, its tests, the Drawer keyboard specs.
- **Proof:** the keyboard-inset unit tests updated to the 1.9.0 constants; a device check on the iPhone (Drawer with an input, keyboard up and down).
