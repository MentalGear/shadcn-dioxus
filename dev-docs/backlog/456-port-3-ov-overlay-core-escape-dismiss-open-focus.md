---
id: 456
title: "Port 3.OV: overlay core (L1-L3, L5-L9, L16-L17, L22, L26) with openFocus"
status: "BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan."
size: "L"
area: overlays
updated: 2026-10-26
---

# Row 456: Port 3.OV: overlay core (L1-L3, L5-L9, L16-L17, L22, L26) with openFocus

## Item

Port the overlay core: L6 (Escape: a bridge `listen` plus an rsx `prevent_default` on the panel, or `hotkey` if its site-relative witness is red, in which case this row builds it unless 3.GS did), L1 and L2 (rsx `onkeydown` and `oncancel`, and the modal driver as a consumer of the executor, `close` only through its report), L3, L5 (consumes the executor's exit wait), L7, L8, L9, L26, L16 (`openFocus`, with L18's close arm, through the focus owner), L17 (Select arm and the rest of the id, through the focus owner) and L22 (a plain rsx handler: `onpointerover` and `onpointerdown` on the host elements with `PointerData::pointer_type()`, since a bridge fact consumed by a vdom decision would break the order contract).

Part of the [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.4 (candidate rows) and the ledger of section 3.3.

## Status / trigger

BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan.

## Size

L

## Detail

- **Files owned:** `lib.rs` (after phase 0), `dialog.rs` (L6's `DialogContentNonModal` caller only), `menu_sub.rs`, `navigation_menu.rs`, `navbar.rs`, `hover_intent.rs`, `date_picker/context.rs`, Select's trigger file (one line), `modality.rs` (the `dxOpenFocus`/`dxCloseFocus` lines; 3.PW edits the writer after this lane), `ts/behaviours/open_focus.ts`, `bridge/behaviours/open_focus.rs`, `primitives/conformance/behaviours/open_focus.json`, `ts/behaviours/hotkey.ts`, `bridge/behaviours/hotkey.rs` and `primitives/conformance/behaviours/hotkey.json` (only if L6's witness is red and 3.GS has not built them). Must not touch `top_layer/*`.
- **Depends on:** [row 449](./449-phase-0-4-proof-port-b-the-executor-and-popover-spreads.md) (modal executor), [row 448](./448-phase-0-3-proof-port-a-focus-trap-and-otpfilter.md) (`lib.rs` order), [row 447](./447-phase-0-7-the-order-runtime.md) (focus owner), [row 444](./444-gate-half-of-lane-3-0b-grants-bans-and-witness-presence.md); an `adds: [open_focus]` row.
- **Proof:** The witness per `openFocus` effect; nested-layer and Escape specs, `touch-open-focus.spec.ts`, native-dialog specs on a release build.
- **Hours (inferred):** 28 (27 after round 5, 26 before it; round 6 +1: L6's decomposition and, if it opens first, the `hotkey` build; the focus owner is item 0.7's); removes about 253 lines, relocates 124 (9 per hour).
