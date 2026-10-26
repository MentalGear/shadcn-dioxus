---
id: 466
title: "Measured port 3: 3.WB wheel band (C5 to a pure Rust classifier)"
status: "APPROVED 2026-10-26 (scope 2): after phase 0, taken as an ordinary row in order of JavaScript…"
size: "M"
area: components
js-grant: none (a `wheelBand` entry would need the owner's lag or order verdict and its own row)
adds: []
updated: 2026-10-26
---

# Row 466: Measured port 3: 3.WB wheel band (C5 to a pure Rust classifier)

## Item

C5 to a pure Rust classifier with a batched stream flushed per event task, CSS-variable output, parity and property tests, the measurement protocol and C4's bounce lines. If Rust loses with the reason `lag` or `order`, that verdict admits a `wheelBand` named behaviour; `parity` or `integrity` keeps the Rust arm to be fixed.

Part of the [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.4 (the owner-measured ports) and section 4.

## Status / trigger

APPROVED 2026-10-26 (scope 2): after phase 0, taken as an ordinary row in order of JavaScript removed per hour.

## Size

M

## Detail

- **Files owned:** `carousel/wheel.rs`, `carousel/wheel_band.rs` (new), `carousel/drag.rs` (the bounce lines only), `playwright/fixtures/carousel-wheel-expected.json` (new). Must not touch the other `carousel/*` files.
- **Depends on:** [row 463](./463-port-3-fz-freeze-legacy-scripts-behind-legacy-impl.md), [row 464](./464-port-3-ca-carousel-rust-arms-next-to-the-frozen-legacy.md) (C4's bounce lines and C1's true-end attribute), [row 443](./443-owner-measurement-kit-perf-recorder-and-impl-switch.md); the owner's measurement of 3.CA comes first.
- **Proof:** The owner's `verdict.json` for the wheel band; parity against the recorded fixture.
- **Hours (inferred):** 12-16 (inferred); removes 477 (0 if Rust loses) (34 per hour).
