---
id: 508
title: "Proof: a behaviour's Rust decide runs in the page as a wasm kernel, inside an event's dispatch"
status: "APPROVED 2026-11-11 (owner). A proof on a branch (lane/decision-kernel-proof), not queued for…"
size: "S"
area: primitives
js-grant: none
adds: []
updated: 2026-11-12
---

# Row 508: Proof: a behaviour's Rust decide runs in the page as a wasm kernel, inside an event's dispatch

## Item

The owner's decision of 2026-11-11 (decisions.md, "Behaviour is decided in Rust") rests on one unproven mechanism: a
pure Rust `decide`, compiled to a small wasm module that the bridge loads into the page, called synchronously by the
bridge's listener while the event is still being dispatched. This row proves or refutes it on the one behaviour that
exists, the test fixture (`ts/behaviours/fixture.ts`), before any rule or port depends on it. Why first: the kernel's
size, its build reproducibility and its load path are assumptions, and the contract of row 509 is written from this
row's measurements.

Part of [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md).

## Status / trigger

APPROVED 2026-11-11 (owner). A proof on a branch (`lane/decision-kernel-proof`), not queued for merge: only this row's
verdict and the research note land. Starts now; it depends on nothing unlanded.

## Size

S

## Detail

- **What is built, on the branch only:** the fixture's `decide` as a Rust function with no `dioxus` or bridge types in
  its signature; a kernel crate that exports it for `wasm32-unknown-unknown` through plain `extern "C"` exports (no
  `wasm-bindgen`, no new tool, no new dependency); the loader and the byte-passing glue in the bridge; the fixture's
  listener calling the kernel instead of its TypeScript `decide`.
- **Pass, all of:**
  1. **Decides in the dispatch.** On a release SSG build in Chromium, a capture-phase, non-passive listener calls the
     kernel synchronously and cancels the default action from its answer; the fixture's existing tests pass unchanged
     against the kernel arm.
  2. **Fails open.** With the kernel absent or its load blocked, the behaviour stands down and no input is trapped.
  3. **Reproducible.** Two clean builds on one toolchain give identical bytes. The note says which toolchain pin a
     freshness gate would need (the repo has no `rust-toolchain` file).
  4. **Small.** The kernel with the fixture alone is under 50,000 bytes uncompressed (`bridge.js` is 19,956). The note
     reports the raw and gzip size and what one more behaviour is expected to add.
  5. **Cheap per event.** Over 10,000 synthetic events the kernel call's p99 is under 0.5 ms.
- **Reported, not pass or fail:** the time from the bridge's install to the kernel being ready; how the bytes reach
  the page and what that costs on web; the Content-Security-Policy source the kernel needs (`'wasm-unsafe-eval'`).
- **Not provable here, recorded as open:** loading the kernel inside the desktop webview. No desktop build exists
  (row [23](./023-preview-s-desktop-feature-lacks-dioxus.md)); the note names the one check a desktop build must make.
- **Verdict:** `KERNEL` (every pass line holds) or `MIRROR` (any fails: the Rust `decide` stays the source of truth and
  a TypeScript copy is held equal by the shared vectors). The verdict and the five measurements go into this row's
  Status and into `dev-docs/research/decision-kernel-proof.md`; on `MIRROR` the decisions.md line is rewritten and the
  owner is asked before row 509 opens.
- **Files owned (branch only):** a new kernel crate, `primitives/src/ts/**`, `primitives/src/bridge/behaviours.rs`,
  the fixture's tests. **Must not touch:** gate scripts and their baselines, any component.
- **Depends on:** [row 444](./444-gate-half-of-lane-3-0b-grants-bans-and-witness-presence.md) (landed).
  **Blocks:** row 509; the `otpFilter` half of [row 448](./448-phase-0-3-proof-port-a-focus-trap-and-otpfilter.md);
  every row that adds a behaviour.
- **Hours (inferred):** about 4 to 6, one Sonnet agent.
