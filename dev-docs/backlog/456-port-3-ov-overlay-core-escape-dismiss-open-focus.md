---
id: 456
title: "Port 3.OV: overlay core (L1-L3, L5-L9, L16-L17, L22, L26) with openFocus"
status: "APPROVED 2026-10-26 (scope 2): after phase 0, taken as an ordinary row in order of JavaScript…"
size: "L"
area: overlays
js-grant: js_file=124 (relocated lines, root doc section 5.4; the main loop's grant row at the merge base is the number the gate reads)
adds: [openFocus]
updated: 2026-10-31
---

# Row 456: Port 3.OV: overlay core (L1-L3, L5-L9, L16-L17, L22, L26) with openFocus

## Item

Port the overlay core: L6 (Escape: a bridge `listen` plus an rsx `prevent_default` on the panel, or `hotkey` if its site-relative witness is red, in which case this row builds it unless 3.GS did), L1 and L2 (rsx `onkeydown` and `oncancel`, which is also the owner's carrier of the dialog's dismiss transition, with a fact fallback for a dialog that closes without `cancel` on Chrome before 132, Firefox before 133 and Safari 18.x; and the modal driver as a consumer of the executor, `close` only through its report), L3, L5 (`use_animated_open`, two placements: the executor's exit wait for a modal close, and R for the unmount hold of the twelve non-reconciled users, a `getAnimations` read plus a `finished` fact), L7, L8, L9, L26, L16 (`openFocus`, with L18's close arm, through the focus owner), L17 (Select arm and the rest of the id, through the focus owner) and L22 (a plain rsx handler: `onpointerover` and `onpointerdown` on the host elements with `PointerData::pointer_type()`, since a bridge fact consumed by a vdom decision would break the order contract).

Part of the [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.4 (candidate rows) and the ledger of section 3.3.

## Status / trigger

APPROVED 2026-10-26 (scope 2): after phase 0, taken as an ordinary row in order of JavaScript removed per hour (yield to be re-measured when taken). Row 487 (one dismiss core) and row 435 (one menu family) land first, so this row ports one core.

## Size

L

## Detail

Re-scoped 2026-10-31 after [row 477](./477-anchored-overlays-placed-in-document-coordinates.md) (PR #156), [row 478](./478-overlay-defaults-to-base-ui.md) and [row 479](./479-scroll-lock-method-and-the-touch-carve-out.md). This row **ports JavaScript to Rust or TS; it does not unify**. [Row 487](./487-one-dismiss-core-escape-outside-dismiss-and-focus-return.md) (one dismiss core: Escape, outside dismiss, focus return) and [row 435](./435-one-menu-family-behind-three-hosts.md) land first (owner 2026-10-31), so this row ports one dismiss core and one menu family instead of the scattered sites.

- **Rewritten by 477/478, no longer as the ledger says:** the dismiss parts. L3 and L7 are `use_outside_dismiss` (now `primitives/src/lib.rs:391`; the old anchors `:463`/`:886` are stale), which carries the touch tap rule (a mouse press dismisses on `pointerdown`; a touch or pen press is a tap, `pointerup` within 10 px, `pointercancel` drops it). L8 is `use_sub_outside_dismiss` (`menu_sub.rs:420`). L9 is NavigationMenu's own document `pointerdown` listener (`navigation_menu.rs:~947-955`) until row 487 brings it under the tap rule. L22 (`hover_intent.rs`) is a pointer-type handler after row 478 item 3 (Tooltip and HoverCard ignore touch).
- **Not this row:** the scroll lock (L10, L11) is [row 459](./459-port-3-sl-scroll-lock-l10-l11-l13.md); `scroll_lock.rs` is 55 JS lines after row 479 (it was 82), and `CONTEXT_MENU_SCROLL_BLOCK_JS` is removed by row 479, not ported. Close-on-scroll is Tooltip-only (`tooltip.rs:497`).
- **Still valid as written in the Item:** Dialog L1/L2/L5/L6 (the `onkeydown`/`oncancel` handlers, the modal driver as an executor consumer, `use_animated_open`'s two placements, Escape on the panel) and the open-focus parts L16/L17 (`openFocus`, with L18's close arm and Select's arm, through the focus owner), plus L26.
- **Withdrawn:** "removes about 253 lines, relocates 124 (9 per hour)" predates 477, 478, 479 and the unification of 435/487. Re-measure the removed and relocated lines, and the yield, when the row is taken.
- **Files owned:** `lib.rs` (after phase 0), `dialog.rs` (L6's `DialogContentNonModal` caller only), `menu_sub.rs`, `navigation_menu.rs`, `navbar.rs`, `hover_intent.rs`, `date_picker/context.rs`, Select's trigger file (one line), `modality.rs` (the `dxOpenFocus`/`dxCloseFocus` lines; 3.PW edits the writer after this lane), `ts/behaviours/open_focus.ts`, `bridge/behaviours/open_focus.rs`, `primitives/conformance/behaviours/open_focus.json`, `ts/behaviours/hotkey.ts`, `bridge/behaviours/hotkey.rs` and `primitives/conformance/behaviours/hotkey.json` (only if L6's witness is red and 3.GS has not built them). Must not touch `top_layer/*`.
- **Depends on:** [row 487](./487-one-dismiss-core-escape-outside-dismiss-and-focus-return.md) and [row 435](./435-one-menu-family-behind-three-hosts.md) (unify first), [row 449](./449-phase-0-4-proof-port-b-the-executor-and-popover-spreads.md) (modal executor), [row 448](./448-phase-0-3-proof-port-a-focus-trap-and-otpfilter.md) (`lib.rs` order), [row 447](./447-phase-0-7-the-order-runtime.md) (focus owner), [row 444](./444-gate-half-of-lane-3-0b-grants-bans-and-witness-presence.md); an `adds: [openFocus]` row (and `[hotkey]` if L6's witness is red and this row builds it first).
- **Proof:** The witness per `openFocus` effect; nested-layer and Escape specs, the dismiss parity spec of row 487, `touch-open-focus.spec.ts`, native-dialog specs on a release build.
- **Hours:** the plan's 29 (inferred) predates these rows; re-estimate when taken.
