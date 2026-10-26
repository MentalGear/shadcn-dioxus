---
id: 464
title: "Measured port 1: 3.CA carousel (C1-C4, C6, L20) as Rust arms next to the frozen legacy arm"
status: "BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan."
size: "L"
area: components
js-grant: js_file=60 (inferred ceiling: the snap kind of scrollOwner; the main loop's grant row at the merge base is the number the gate reads)
adds: []
updated: 2026-10-26
---

# Row 464: Measured port 1: 3.CA carousel (C1-C4, C6, L20) as Rust arms next to the frozen legacy arm

## Item

Each carousel script becomes a Rust arm beside the frozen legacy arm: C1 dissolves, C2 a `transaction` (`scrollStep` is on the plan's exits list), C3 R, C4 `pan` (recogniser, capture, click suppression; the `scrollBy` output a Rust `transaction`, a page-side variant only on a `lag` or `order` verdict), C6 and L20 (an owner call to the focus owner, which carries `preventScroll`; there is no `focus_prevent_scroll` row and no caller TSV). This row builds the **snap kind** of `scrollOwner` (a realized touch, wheel or key is a transition, a re-snap is neutral), and the carousel's autoplay tick runs against it (see [row 470](./470-bug-carousel-autoplay-tick-fights-a-user-swipe.md)). C4's bounce stays on inline `transform` until 3.WB. The owner measures the carousel as one switch.

Part of the [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.4 (the owner-measured ports) and section 4.

## Status / trigger

BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan.

## Size

L

## Detail

- **Files owned:** `carousel/*` except `wheel.rs`, `wheel_band.rs` and `js_macros.rs`, `carousel/root.rs`, `carousel/context.rs`, plus `ts/behaviours/pan.ts` (the new variant). Must not touch `carousel/wheel.rs`, `carousel/wheel_band.rs`, `js_macros.rs`, `ts/legacy/`.
- **Depends on:** [row 463](./463-port-3-fz-freeze-legacy-scripts-behind-legacy-impl.md) (freeze), [row 457](./457-port-3-vs-virtual-settle-and-the-scrollowner-behaviour.md) (the one-shot `scrollOwner` this row extends), [row 455](./455-port-3-gs-gestures-slider-resizable-drawer-toast-pan.md) (`pan` is built by 3.GS), [row 443](./443-owner-measurement-kit-perf-recorder-and-impl-switch.md) (the measurement kit); the owner's measurement gates the next port.
- **Proof:** The owner's `verdict.json` for the carousel; carousel specs on both arms; a `pan` witness per effect.
- **Hours (inferred):** 24-32 (inferred; it includes the snap kind of `scrollOwner`); removes 412 at the legacy removal, relocates 659 (15 per hour).
