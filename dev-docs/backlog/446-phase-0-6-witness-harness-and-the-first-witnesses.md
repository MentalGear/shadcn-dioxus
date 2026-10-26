---
id: 446
title: "Phase 0.6: the witness harness and the first witnesses"
status: "BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan."
size: "L"
area: tests
updated: 2026-10-26
---

# Row 446: Phase 0.6: the witness harness and the first witnesses

## Item

The counterfactual-witness harness that admission rests on: the four library probes (`wrongPaint` by per-element trajectory inclusion over a wide projection that sees `checked`, `indeterminate` and `value`, with unrelated animation paused through `[data-dx-motion="paused"]`; `defaultAction` and `outcomeOrder` over library-defined observables, the first with a closed observable table per event type; `focusAtNextInput`), the `witness(kind, effect)` wrapper over `test.fail()` with the typed `WitnessViolation` and a real-against-real control arm, witness mode as a Playwright `addInitScript` plus the adapter's frozen apply table and a `{site, effect}` marker (no second build, no cargo feature), the page-side order counterfactual in the channel wrapper, a focus-write delay hook (a patched `HTMLElement.prototype.focus`) with a red scenario so the focus owner, the `MountedData` write ban and the roving row each have a counterfactual, completed-paint delays, frame-counted inputs, the coverage cross-check, the scenario-binding schema (one file per `site!`), the `verdict.json` schema and reader, and the `select.mjs` rule. Its first witnesses are `otpFilter`'s (`prevent` and `setValue`, as `test.fixme` keyed to the site list until item 0.3 builds the site) and the executor's internals (exit wait, watchdog, port), pre-authored for item 0.4 to flip. Round 6 removed the prototype reconciler and the per-kind witnesses: there is one page executor, admitted as a whole. Why: item 0.4 could not write the witness of its own executor, and nobody owned the first listener's; a lane cannot admit its own behaviour.

Part of the [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.4 (phase 0).

## Status / trigger

BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan.

## Size

L

## Detail

- **Files owned:** `primitives/src/ts/witness.ts` (new), `primitives/src/ts/bridge.ts` (the apply table and marker, the channel wrapper, the coverage cross-check, the resize-observer dispatcher), `primitives/tsconfig.json`, `playwright/witness/**` (new: `probes/**`, `bindings/**`, the init script), `playwright/tools/select.mjs` (the witness rule), `playwright/durations.json` entries. Must not touch the behaviour modules.
- **Depends on:** [row 444](./444-gate-half-of-lane-3-0b-grants-bans-and-witness-presence.md) (the witness-test presence check and the `WitnessViolation` ban). Runs beside [row 445](./445-phase-0-2-the-legacy-freeze.md) (disjoint files).
- **Proof:** The harness's own control-arm and probe self-tests; the pre-authored `otpFilter` and executor-internal witnesses exist as `test.fixme` keyed to the site list (they are item 0.3's and 0.4's to flip).
- **Hours (inferred):** about 14 (15 after round 5, 7 before it; round 6 -3 for the prototype reconciler and the per-kind witnesses, +2 net for the per-element projection, the wider state, the real-against-real control, the focus-write hook and the init-script mode that replaces the feature bundle).
