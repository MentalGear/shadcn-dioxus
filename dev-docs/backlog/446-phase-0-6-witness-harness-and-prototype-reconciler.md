---
id: 446
title: "Phase 0.6: the witness harness, the prototype reconciler and the first witnesses"
status: "BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan."
size: "L"
area: tests
updated: 2026-10-26
---

# Row 446: Phase 0.6: the witness harness, the prototype reconciler and the first witnesses

## Item

The counterfactual-witness harness that admission rests on: the four library probes (`wrongPaint` by trajectory inclusion, `defaultAction`, `focusAtNextInput`, `outcomeOrder`), the `witness(kind, effect)` wrapper over `test.fail()` with the typed `WitnessViolation` and its control arm, the adapter's witness mode in a separate bundle behind a non-default cargo feature, the page-side order counterfactual in the channel wrapper, completed-paint delays, frame-counted inputs, the coverage hook, the `verdict.json` schema and reader, and the `select.mjs` rule. It also owns the prototype reconciler (a page reconciler against the delayed Rust `transaction`, never shipped) and the first two witnesses: `hotkey`'s `prevent` as a `test.fixme` keyed to the variant's existence until item 0.3 builds it, and `reconcile`'s per kind. Why: item 0.4 could not write the witness of its own executor, and nobody owned `hotkey`'s; a lane cannot admit its own behaviour.

Part of the [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.4 (phase 0).

## Status / trigger

BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan.

## Size

L

## Detail

- **Files owned:** `primitives/src/ts/witness.ts` (new), `primitives/src/ts/bridge.ts` (the witness seam, the channel wrapper, the coverage hook, the resize-observer dispatcher), `primitives/tsconfig.json`, `playwright/witness/**` (new: `probes/**`, `prototype/**`), `playwright/tools/select.mjs` (the witness rule), `playwright/durations.json` entries, `primitives/Cargo.toml` (the `witness` feature; `check-primitives-features.sh` lists it, a one-line edit the main loop makes at merge). Must not touch the behaviour modules.
- **Depends on:** [row 444](./444-gate-half-of-lane-3-0b-grants-bans-and-witness-presence.md) (the witness-test presence check and the `WitnessViolation` ban). Runs beside [row 445](./445-phase-0-2-the-legacy-freeze.md) (disjoint files).
- **Proof:** The harness's own control-arm and probe self-tests; `witness(reconcile, ...)` decides whether the executor is a page reconciler or a Rust `transaction` (it is the input of item 0.4).
- **Hours (inferred):** about 15 (7 before; +8 for the prototype, the first witnesses, the control arm, the trajectory projection and the page-side order counterfactual).
