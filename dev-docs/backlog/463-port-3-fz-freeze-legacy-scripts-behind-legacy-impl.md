---
id: 463
title: "Measured group support: 3.FZ freeze (legacy message scroller and carousel scripts into ts/legacy behind legacy-impl)"
status: "APPROVED 2026-10-26 (scope 2): after phase 0, taken as an ordinary row in order of JavaScript…"
size: "M"
area: primitives
js-grant: js_file=0 (the legacy scripts move into ts/legacy and keep counting; the main loop's grant row at the merge base is the number the gate reads)
adds: []
updated: 2026-10-26
---

# Row 463: Measured group support: 3.FZ freeze (legacy message scroller and carousel scripts into ts/legacy behind legacy-impl)

## Item

Freeze the legacy message scroller, carousel C2 to C5 and the macros into `ts/legacy/*.ts` (wrapped, behaviour-neutral, excluded from `tsconfig`), behind a non-default `legacy-impl` feature that only the preview enables, with the `?dx-impl` switch (reading rows from the closed tables), through the `Program::Frozen{module}` entry of item 0.1. Why: the owner measures each port as old versus new, and removal is keyed to the owner's `verdict.json` saying `pass`.

Part of the [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.4 (the owner-measured ports) and section 4.

## Status / trigger

APPROVED 2026-10-26 (scope 2): after phase 0, taken as an ordinary row in order of JavaScript removed per hour.

## Size

M

## Detail

- **Files owned:** `message_scroller.{rs,js}`, `carousel/{scroll,drag,wheel,js_macros,mod}.rs` (the string replaced by the legacy call), `primitives/src/ts/legacy/**`, `primitives/Cargo.toml`. Must not touch the rest of `carousel/*`, `primitives/src/bridge/*`.
- **Depends on:** [row 444](./444-gate-half-of-lane-3-0b-grants-bans-and-witness-presence.md) (`Program::Frozen`), [row 445](./445-phase-0-2-the-legacy-freeze.md); goes before [row 464](./464-port-3-ca-carousel-rust-arms-next-to-the-frozen-legacy.md), [row 465](./465-port-3-ms-message-scroller-sticktoend-and-native-anchoring.md), [row 466](./466-port-3-wb-wheel-band-pure-rust-classifier.md).
- **Proof:** The shipped feature-off arm's own clippy and test arm; legacy behaviour byte-identical on the carousel and message-scroller specs.
- **Hours (inferred):** 10-12 (inferred); removes 0, moves about 2,700 lines.
