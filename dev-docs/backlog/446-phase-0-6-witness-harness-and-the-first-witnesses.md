---
id: 446
title: "Phase 0.6: the witness harness and the first witnesses"
status: "APPROVED 2026-10-26 (owner: \"okay for the plan\"; scope 2, the recommended default, since no scope…"
size: "L"
area: tests
js-grant: js_file=360 (inferred ceiling: witness.ts about 300 that never ships, the apply table about 60; the main loop's grant row at the merge base is the number the gate reads)
adds: []
updated: 2026-10-26
---

# Row 446: Phase 0.6: the witness harness and the first witnesses

## Item

The counterfactual-witness harness that admission rests on: the four library probes (`wrongPaint` by per-element trajectory inclusion over a wide projection that sees `checked`, `indeterminate` and `value`, with unrelated animation paused through `[data-dx-motion="paused"]`; `defaultAction` and `outcomeOrder` over library-defined observables, the first with a closed observable table per event type; `focusAtNextInput`), the `witness(kind, effect)` wrapper over `test.fail()` with the typed `WitnessViolation` and a real-against-real control arm, witness mode as a Playwright `addInitScript` plus the adapter's frozen apply table and a `{site, effect}` marker (no second build, no cargo feature), the page-side order counterfactual in the channel wrapper, a focus-write delay hook (a patched `HTMLElement.prototype.focus`) with a red scenario whose result **decides whether the `MountedData` write ban and the roving-focus row are built at all** (green: neither, and about 16 h conditional leave the plan; the focus owner keeps its counterfactual), completed-paint delays, frame-counted inputs, the scenario-binding schema (one file per `site!`), the `verdict.json` schema and reader, and the `select.mjs` rule. Its first witnesses are `otpFilter`'s (`prevent` and `setValue`, as `test.fixme` keyed to the site list until item 0.3 builds the site) and the executor's internals (exit wait, watchdog), pre-authored for item 0.4 to flip. A witness on a debug build pauses the host subtree's own animations, and `ts/witness.ts` never ships as a bundle entry. Round 6 removed the prototype reconciler and the per-kind witnesses: there is one page executor, admitted as a whole. Why: item 0.4 could not write the witness of its own executor, and nobody owned the first listener's; a lane cannot admit its own behaviour.

Part of the [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.4 (phase 0).

## Status / trigger

APPROVED 2026-10-26 (owner: "okay for the plan"; scope 2, the recommended default, since no scope was named). Phase 0 of the row 434 step 3 plan; the next session starts it after the approved bug rows.

## Size

L

## Detail

- **Files owned:** `primitives/src/ts/witness.ts` (new), `primitives/src/ts/bridge.ts` (the apply table and marker, the channel wrapper, the resize-observer dispatcher), `primitives/tsconfig.json`, `playwright/witness/**` (new: `probes/**`, `bindings/**`, the init script), `playwright/tools/select.mjs` (the witness rule), `playwright/durations.json` entries. Must not touch the behaviour modules.
- **Depends on:** [row 444](./444-gate-half-of-lane-3-0b-grants-bans-and-witness-presence.md) (the `WitnessViolation` ban; the witness-test presence check is item 0.3's). Runs beside [row 445](./445-phase-0-2-the-legacy-freeze.md) (disjoint files).
- **Proof:** The harness's own control-arm and probe self-tests; the pre-authored `otpFilter` and executor-internal witnesses exist as `test.fixme` keyed to the site list (they are item 0.3's and 0.4's to flip).
- **Hours (inferred):** about 14 (unchanged by round 7: -1 for the `dispatchPort` witness and the coverage cross-check, +1 for the debug-build animation pause; 15 after round 5, 7 before it). Witness authoring inside the consuming rows is kept at 12 h and is the largest known understatement (18-28 h at the same unit cost) until the first witnesses give a measured cost per pair.
- **Not provable in the lane container:** timing specs run on release builds only; Firefox and WebKit witness runs need the deploy wiring (Chromium is the only engine here).
