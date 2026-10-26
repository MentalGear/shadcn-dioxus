---
id: 467
title: "Measured group: legacy removal, keyed to each port's verdict.json"
status: "BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan."
size: "S"
area: primitives
js-grant: none
adds: []
updated: 2026-10-26
---

# Row 467: Measured group: legacy removal, keyed to each port's verdict.json

## Item

Delete `carousel/js_macros.rs` and rewrite `ssr_tests.rs:412` as a Rust unit test of the true-end function; delete `ts/legacy/` and its allowlist entries on a `pass`; retire `js_listener.rs`'s PRELUDE with the last `JsListeners::install`; retire the JS halves of the older gates. Removal is keyed to the owner's `verdict.json` saying `pass`, never to a date, and a `fail` deletes nothing.

Part of the [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.4 (the owner-measured ports) and section 4.

## Status / trigger

BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan.

## Size

S

## Detail

- **Files owned:** `carousel/js_macros.rs`, `ssr_tests.rs`, `ts/legacy/`, `js_listener.rs`, the allowlists and the older gates' JS halves.
- **Depends on:** [row 464](./464-port-3-ca-carousel-rust-arms-next-to-the-frozen-legacy.md), [row 465](./465-port-3-ms-message-scroller-sticktoend-and-native-anchoring.md), [row 466](./466-port-3-wb-wheel-band-pure-rust-classifier.md) (each port's verdict).
- **Proof:** The three `verdict.json` files; the full gates after deletion.
- **Hours (inferred):** 2 (inferred); books the removals of 3.MS and 3.CA.
