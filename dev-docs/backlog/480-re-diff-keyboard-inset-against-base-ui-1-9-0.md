---
id: 480
title: "Re-diff primitives/src/keyboard_inset.rs against Base UI 1.9.0"
status: "PARTLY LANDED 2026-10-28 (lane keyboard-inset-480; device check pending). Two 1.6.0 -> 1.9.0…"
size: "M"
area: overlays
updated: 2026-10-28
---

# Row 480: Re-diff keyboard_inset.rs against Base UI 1.9.0

## Item

`primitives/src/keyboard_inset.rs` (the Drawer's virtual-keyboard handling) is lifted from Base UI 1.6.0. Upstream has moved; bring it up to 1.9.0 or record why a difference stays. Source: [`radix-vs-base-ui-2026-10-28.md`](../research/radix-vs-base-ui-2026-10-28.md), appendix A (drawer row).

## Status / trigger

PARTLY LANDED 2026-10-28 (lane keyboard-inset-480; device check pending). Two 1.6.0 -> 1.9.0 changes are ported with no net JS (the `keyboard_inset.rs` baseline row fell 90 -> 88); the rest are not ported and listed below with the reason. Needs one iPhone pass (checklist at the end of Detail).

## Size

M

## Detail

- **Ours (1.6.0):** `KEYBOARD_RESIZE_THRESHOLD = 60`, `KEYBOARD_SCROLL_SLACK = 48` (`keyboard_inset.rs:34-60`).
- **Base UI 1.9.0** (`DrawerVirtualKeyboardProvider.mjs:18-41`): no slack constant; adds `KEYBOARD_VISIBILITY_MARGIN = 16`, `KEYBOARD_REALIGN_INTERVAL = 150`, `KEYBOARD_SETTLE_FRAME_LIMIT = 60` and a `KEYBOARD_TAP_BLOCKED` state.
- **Method:** diff the two files function by function, port what changes behaviour (the realign interval and the settle limit look like iOS keyboard-animation fixes), keep what the Dioxus port needs differently, and cite the pinned upstream revision in the file's provenance comment.
- **Interaction with row 477:** the Drawer's own placement is not an anchored overlay, so rule P does not change it; the `visualViewport` reads here stay.
- **Files owned:** `primitives/src/keyboard_inset.rs`, its tests, the Drawer keyboard specs.
- **Proof:** the keyboard-inset unit tests updated to the 1.9.0 constants; a device check on the iPhone (Drawer with an input, keyboard up and down).

## Findings: Base UI `@base-ui/react` 1.6.0 -> 1.9.0, `drawer/virtual-keyboard-provider/` (read from the npm tarballs)

The scan's constant list was partly wrong. `KEYBOARD_VISIBILITY_MARGIN = 16` and `KEYBOARD_TAP_BLOCKED` are already in 1.6.0 (ours: `FIELD_MARGIN_PX`, now `KEYBOARD_VISIBILITY_MARGIN_PX`; the tap trick is deliberately not ported). The real constant changes: `KEYBOARD_SCROLL_SLACK = 48` is gone; `KEYBOARD_REALIGN_INTERVAL = 150`, `KEYBOARD_REALIGN_MAX_PASSES = 4` and `KEYBOARD_SETTLE_FRAME_LIMIT = 60` are new. The context file is unchanged; `scrollable.mjs`, `getElementAtPoint.mjs` only refactor (shadow root and SVG ancestors) and take a root node.

| 1.9.0 change (changelog) | Ported | Why |
|---|---|---|
| Scroll room is the overlap itself, none when the scroller ends above the keyboard; the padding floor under the content is 16px (#5768 "Remove excess virtual keyboard scroll space") | yes | Ours added `overlap + 48px` to the scroller whenever the field was not already inside the band, even with overlap 0, so every panel whose stylesheet pulls it above the keyboard (all shipped ones) grew 48px of blank scroll space under its last field. Now `overlap > 0 ? addSlack(overlap) : restoreSlack()`, floor `max(computed padding, 16)`. Net -2 JS lines. |
| Layout viewport read again on window `resize` ("Chrome can keep resizing the layout viewport after visual viewport events stop"; #5774 Chrome on iOS) | yes | One `listen(window, 'resize', update)`. Without it a layout viewport that follows the keyboard after the visual viewport events leaves a stale `--dx-keyboard-inset` on the panel (lifted twice). |
| Keyboard stays detected after the layout viewport shrank with it (`keyboardVisualHeight` memory, `100svh` probe, `resizing` flag; #5774) | no | Needs a probe element appended to `body` and a memory across events: more script. In that browser `innerHeight - vv.height` is below 60, so the contract is absent and the browser places the fixed panel inside the shrunk layout viewport itself; what is lost is only the helper's own field centring there. |
| Settle-watching realign: 4 passes every 150ms and a 60-frame destination settle (`scheduleDelayedKeyboardRealign`, scroll bookkeeping; focus moves with the keyboard already up, WebKit cancels an in-flight smooth scroll when its native reveal resolves, layout reacting to `:focus-within`) | no | JS timers or a frame loop with progress bookkeeping, about 20 lines; the file may not grow. Our reveal recomputes from live rects, so a later `focusin` or inset change re-centres; what is not covered is a scroll cancelled by WebKit with no further event. |
| Page scroll pinned while a modal's keyboard is up (`restoreWindowScroll`, `handleWindowScroll`, `baseScrollX/Y`) | no | A window `scroll` listener plus `scrollTo`: more script. Matters only if WebKit's reveal moves the page behind the modal (the keyboard's previous/next arrows); needs the device. |
| Focus pre-emption for native focus moves (`preemptFocusReveal`, `overrideGeometryDuringFocus`, `handleFocus`, iOS 27 `preventScroll` re-apply; #5769) | no | Writes opacity/transform on the incoming field in `focusout`; tied to the same focus-geometry trick as the tap path. |
| Tap-to-focus path: `KEYBOARD_TAP_BLOCKED`, `programmaticKeyboardFocusRef`, label click `preventScroll` re-apply, root-node hit tests, inert while closed (#5864), item taps while dismissing (#5882) | no | The tap trick stays out (row 227: only if a real iPhone shows the page jumping). |
| Release of inset and slack on `focusout` deferred one frame (a tap's compat `mousedown` moves focus before `click`) | n/a | Ours follows the real visual viewport, not focus, so nothing is released on blur; the panel moves when the keyboard actually closes. |
| `findScrollableAncestor` refactor, `clamp` import | n/a | Same selection as 1.6.0. |

Pre-existing differences from both versions, left alone: no `scroll-padding-bottom += 16px` on the scroller; the scroller is the nearest `overflow-y: auto|scroll|overlay` ancestor, not the nearest one that already overflows; the inset is rounded, upstream ceils it.

Proof: `cargo test -p dioxus-primitives keyboard_inset` (script text pins the new contract); `playwright/keyboard-inset.spec.ts` (the far-below-the-fold case now asserts no extra padding; a new case asserts a covered scroller gets `covered + 16px` and gets it back on close; Chromium, stubbed visual viewport).

iPhone checklist (not run in the lane): Drawer/Sheet/Dialog with a field, keyboard up and down: no blank band under the last field; the last field is centred or at least 16px above the keyboard; the keyboard's previous/next arrows between two fields (the realign and page-pin gaps); a footer button tapped with the keyboard up; Chrome on iOS (the window `resize` path).
