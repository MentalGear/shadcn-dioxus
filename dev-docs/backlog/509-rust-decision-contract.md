---
id: 509
title: "The Rust decision contract: one decide per behaviour, the decision kernel as a committed artefact, and the gates"
status: "APPROVED 2026-11-11 (owner). Opens when row 508's verdict is KERNEL. On MIRROR, items 2 to 4 are…"
size: "M"
area: primitives
js-grant: js_file_bytes=0 (the kernel loader's lines are paid for by the fixture's deleted TypeScript decide; row 508's note gives the measured number, and the main loop's grant row at the merge base is the number the gate reads)
adds: []
updated: 2026-11-12
---

# Row 509: The Rust decision contract: one decide per behaviour, the decision kernel as a committed artefact, and the gates

## Item

The rules every named behaviour follows from here on, built on the one behaviour that exists (the test fixture) so
that the first real behaviour (`otpFilter`, row 448) is written to them. Why now: no behaviour but the fixture exists,
so the contract costs one migration today and one per behaviour later.

1. **One Rust `decide` per behaviour.** `decide(config, memory, snapshot, event) -> Decision`, pure, in a crate with
   no dependencies and no `dioxus` or bridge types. That crate is used three ways: by `dioxus-primitives` as a direct
   call (the Blitz arm and every decision that may answer late), by the kernel build, and by `cargo test`.
2. **The decision kernel is a committed artefact.** `scripts/build-kernel.sh` builds the crate for
   `wasm32-unknown-unknown` and writes the module and a manifest (toolchain, sha256 of each source and of the module)
   beside the JS bundles; `dioxus-primitives` keeps having no build script. The bridge loads it at install; a
   behaviour is inactive until it is ready and stands down if it never is.
3. **TypeScript is the adapter.** `ts/behaviours/<kind>.ts` declares the event fields and attribute reads the decision
   needs and applies the effects it returns. It holds no `decide`.
4. **One set of vectors, two routes.** `primitives/conformance/behaviours.json` runs against the Rust function under
   `cargo test` and against the committed kernel through the bridge's loader under `bun test`, so the wasm route is
   proven without a browser.
5. **The Blitz arm is not forbidden.** `check-native-slots` and `check-hot-events` exempt items under
   `#[cfg(not(feature = "web"))]`: an rsx slot write and an rsx scroll or pointer handler are the only routes on a
   renderer with no page.

Admission is unchanged: an effect is still admitted by its counterfactual witness or its platform rule row (row 446).

Part of [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md).

## Status / trigger

APPROVED 2026-11-11 (owner). Opens when [row 508](./508-decision-kernel-proof.md)'s verdict is `KERNEL`. On `MIRROR`,
items 2 to 4 are rewritten around a TypeScript copy held equal by the vectors, and the owner approves that text first.

## Size

M

## Detail

- **Gates.**
  - New `check-decision-kernel`: the manifest matches the sources and the module; on the recorded toolchain it
    rebuilds and requires identical bytes, otherwise it reports the rebuild half as skipped (as `check-js-bundle`
    does for `bun`). Header, registration in `run-gates.sh`, `scripts/gates-doc.py --write`.
  - `check-bridge-behaviours`: a `Behaviour` variant with no Rust `decide` is red; a `decide` in
    `ts/behaviours/<kind>.ts` is red; every kind has vectors on both routes.
  - `check-native-slots`, `check-hot-events`: the `not(web)` exemption, each with a self-test case.
- **Files owned:** the new decision crate and its `Cargo.toml` entry; `scripts/build-kernel.sh`,
  `scripts/check-decision-kernel.sh`; `primitives/src/bridge/behaviours.rs`; `primitives/src/ts/behaviours/**`,
  `primitives/src/ts/lib/**` and the kernel loader in `ts/bridge.ts`; `primitives/ts-test/behaviours.test.ts`;
  `scripts/check-bridge-behaviours.*`, `scripts/check-native-slots.*`, `scripts/check-hot-events.*`;
  `dev-docs/js-interop.md`; `dev-docs/dependency-register.tsv` if the register lists workspace crates.
  **Must not touch:** any component, `scripts/check-js-volume.*` (row 445's).
- **Depends on:** row 508 (verdict `KERNEL`), [row 444](./444-gate-half-of-lane-3-0b-grants-bans-and-witness-presence.md)
  (landed). Shares `ts/bridge.ts` with rows 446, 447 and 449: it lands before them or after the `ts/lib` split, never
  beside them. **Blocks:** the `otpFilter` half of [row 448](./448-phase-0-3-proof-port-a-focus-trap-and-otpfilter.md)
  and every row whose `adds:` names a behaviour.
- **Proof:** the fixture's tests pass with its decision in the kernel; a vector that the Rust function fails is red
  under `cargo test`, and the same vector is red under `bun test` against a stale kernel; a `decide` added to
  `fixture.ts` is red; an edited source without a rebuild is red; each exempted gate passes a `not(web)` item and
  still fails the same item under `web`; a unit test calls the fixture's `decide` natively with no bridge;
  `scripts/registry-install-smoke.sh` is green (the decision crate changes what an installed component depends on).
- **Hours (inferred):** not estimated. The lane sets it from row 508's note; the gate half alone is likely several hours.
