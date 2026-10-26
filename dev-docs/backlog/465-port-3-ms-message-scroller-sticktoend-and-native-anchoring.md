---
id: 465
title: "Measured port 2: 3.MS message scroller (the Rust invariant, native anchoring as accelerator, stickToEnd on every engine)"
status: "BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan."
size: "L"
area: components
updated: 2026-10-26
---

# Row 465: Measured port 2: 3.MS message scroller (the Rust invariant, native anchoring as accelerator, stickToEnd on every engine)

## Item

W8: a Rust invariant ("end sentinel visible, else scroll to end") with native `overflow-anchor` as the accelerator where verified (Safari 27 is its floor) and the `stickToEnd` named behaviour (about 40 lines, a ResizeObserver callback writing `scrollTop` while the Rust-published mode is `following-bottom`) for the same-frame write on every engine, idempotent with A; `stickToEnd` owns its detach, decided by the level (the `scrollOwner`'s level intent kind, so a wheel-up, find-in-page or screen-reader scroll-up detach it while anchoring does not); G opening position; `settle` jumps. The legacy arm already exists (3.FZ). Time-boxed with the named-behaviour exit.

Part of the [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.4 (the owner-measured ports) and section 4.

## Status / trigger

BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan.

## Size

L

## Detail

- **Files owned:** `message_scroller.rs`, its `style.css`, the preview component, `ts/behaviours/stick_to_end.ts`, `bridge/behaviours/stick_to_end.rs`, `primitives/conformance/behaviours/stick_to_end.json`.
- **Depends on:** [row 463](./463-port-3-fz-freeze-legacy-scripts-behind-legacy-impl.md) (freeze), [row 457](./457-port-3-vs-virtual-settle-and-the-scrollowner-behaviour.md) (`scrollOwner`), [row 443](./443-owner-measurement-kit-perf-recorder-and-impl-switch.md); an `adds: [stick_to_end]` row; the witness runs under fallback emulation (`overflow-anchor: none !important`).
- **Proof:** The owner's `verdict.json` for the message scroller; chat-stream specs; the `stickToEnd` witness pair.
- **Hours (inferred):** 30-45 (inferred), time-boxed; removes 1,025 at the legacy removal (+40 added) (27 per hour).
