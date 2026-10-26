---
id: 447
title: "Phase 0.7: the order runtime (inbound pump, per-slot epochs, transition port, slot table, focus owner)"
status: "BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan."
size: "M"
area: primitives
updated: 2026-10-26
---

# Row 447: Phase 0.7: the order runtime (inbound pump, per-slot epochs, transition port, slot table, focus owner)

## Item

The order contract's implementation: one inbound pump for every page-to-Rust bridge fact (a page-minted sequence number and one root task), the one epoch rule (`based_on >= last_page_transition`, epochs keyed by slot id) replacing the as-built `Epoch::next` (one process-global Rust counter, which can never lose to the page), the transition port (a hidden element on the vdom's ordered path for low-rate page transitions that rsx decisions read), the slot table with its gate check, and the focus owner (a bridge internal on the document slot). Owner rows for the legacy fact sources `use_beforematch`, `use_form_reset_listener` and `pointer.rs`. Why: board round 5 found neither the epoch rule nor the pump had an owner, and Dioxus events carry no epoch, so rsx decisions on a page-written slot diverge between web and desktop.

Part of the [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.4 (phase 0).

## Status / trigger

BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan.

## Size

M

## Detail

- **Files owned:** `primitives/src/bridge/mod.rs` (`Epoch`), `primitives/src/bridge/session.rs` (the pump; item 0.3 edits `use_behaviour` after it), `primitives/src/ts/bridge.ts`, `primitives/tsconfig.json`, the focus owner's TypeScript, the slot table (a TSV beside the behaviour budgets). Must not touch the behaviour rows' files or `top_layer/`.
- **Depends on:** [row 444](./444-gate-half-of-lane-3-0b-grants-bans-and-witness-presence.md), [row 446](./446-phase-0-6-witness-harness-and-prototype-reconciler.md) (the order counterfactual in the channel wrapper). It precedes [row 448](./448-phase-0-3-proof-port-a-focus-trap-and-hotkey.md) and [row 449](./449-phase-0-4-proof-port-b-the-executor-and-popover-spreads.md).
- **Proof:** One order spec per instance of the order contract (round 5 specs list, root doc section 5.6); the slot-table gate; the desktop-cadence build of [row 442](./442-desktop-cadence-harness-and-desktop-pass.md) shows the order cases.
- **Hours (inferred):** about 11 (new in round 5, inferred: pump about 4, epoch rule about 2, transition port about 2, focus owner about 2, slot table and gate about 1).
