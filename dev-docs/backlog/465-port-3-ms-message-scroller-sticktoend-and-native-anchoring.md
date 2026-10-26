---
id: 465
title: "Measured port 2: 3.MS message scroller (the Rust invariant, native anchoring as accelerator, stickToEnd on every engine)"
status: "APPROVED 2026-10-26 (scope 2): after phase 0, taken as an ordinary row in order of JavaScript…"
size: "L"
area: components
js-grant: js_file=40 (stickToEnd about 40 lines; the main loop's grant row at the merge base is the number the gate reads)
adds: [stickToEnd]
updated: 2026-10-26
---

# Row 465: Measured port 2: 3.MS message scroller (the Rust invariant, native anchoring as accelerator, stickToEnd on every engine)

## Item

W8: a Rust invariant ("end sentinel visible, else scroll to end") with native `overflow-anchor` as the accelerator where verified (Safari 27 is its floor) and the `stickToEnd` named behaviour (about 40 lines, a ResizeObserver callback writing `scrollTop` while the Rust-published mode is `following-bottom`) for the same-frame write on every engine, idempotent with A; `stickToEnd` owns its detach, decided by the level (the `scrollOwner`'s **level kind, built by this row**, so a wheel-up, find-in-page or screen-reader scroll-up detach it while anchoring does not); G opening position; `settle` jumps. The legacy arm already exists (3.FZ). Time-boxed with the named-behaviour exit.

Part of the [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.4 (the owner-measured ports) and section 4.

## Status / trigger

APPROVED 2026-10-26 (scope 2): after phase 0, taken as an ordinary row in order of JavaScript removed per hour.

## Size

L

## Detail

- **Files owned:** `message_scroller.rs`, its `style.css`, the preview component, `ts/behaviours/stick_to_end.ts`, `bridge/behaviours/stick_to_end.rs`, `primitives/conformance/behaviours/stick_to_end.json`.
- **Depends on:** [row 463](./463-port-3-fz-freeze-legacy-scripts-behind-legacy-impl.md) (freeze), [row 457](./457-port-3-vs-virtual-settle-and-the-scrollowner-behaviour.md) (the one-shot `scrollOwner`, which this row extends with the level kind), [row 443](./443-owner-measurement-kit-perf-recorder-and-impl-switch.md); an `adds: [stickToEnd]` row; the witness runs under fallback emulation (`overflow-anchor: none !important`).
- **Proof:** The owner's `verdict.json` for the message scroller; chat-stream specs; the `stickToEnd` witness pair.
- **Hours (inferred):** 31-46 (inferred; 30-45 in round 6, +1 for the level kind of `scrollOwner`), time-boxed; removes 1,025 at the legacy removal (+40 added) (27 per hour).
