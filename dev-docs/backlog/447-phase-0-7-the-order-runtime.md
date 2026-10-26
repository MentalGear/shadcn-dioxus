---
id: 447
title: "Phase 0.7: the order runtime (inbound pump, per-slot epochs, transition port, slot table, focus owner, scroll classifier spike)"
status: "BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan."
size: "M"
area: primitives
updated: 2026-10-26
---

# Row 447: Phase 0.7: the order runtime (inbound pump, per-slot epochs, transition port, slot table, focus owner, scroll classifier spike)

## Item

The order contract's implementation: one inbound pump for every page-to-Rust bridge fact (a page-minted sequence number and one root task that dispatches through a registry entry `Drop` removes, so it never calls a dropped scope's `Callback`), the one epoch rule (`based_on >= last_page_transition`, epochs keyed by `(scope id, mount nonce)`) replacing the as-built `Epoch::next` (one process-global Rust counter, which can never lose to the page), the rule that Rust-caused transitions are reported but never advance the fence, the transition port (an element each owner renders on the vdom's ordered path for discrete, human-rate page transitions that rsx decisions read: fired from synchronous hooks, `stopPropagation` on both sides, never inside a `<form>`, a liveness handshake, installed before `data-hydrated`), the slot table (with the contenteditable and document-scroller rows) and its gate check, the verb partition with `FORBIDDEN_VERBS` derived from the slot table, the focus owner (a bridge internal on the document slot), the scroll classifier spike (realized default action and intent kinds, against an anchoring adjustment and a snap), and the bridge boot from the preview's root effect with `html[data-dx-bridge="ready"]` set after every owner's install report and `data-hydrated` after it. Owner rows for the legacy fact sources `use_beforematch`, `use_form_reset_listener` and `pointer.rs`. Why: board round 5 found neither the epoch rule nor the pump had an owner, and Dioxus events carry no epoch, so rsx decisions on a page-written slot diverge between web and desktop; round 6 found the port needed a host, a synchronous trigger and the Rust-caused rule.

Part of the [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.4 (phase 0).

## Status / trigger

BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan.

## Size

M

## Detail

- **Files owned:** `primitives/src/bridge/mod.rs` (`Epoch`, and its doc line 51, which routes agents to `MountedData::set_focus`), `primitives/src/bridge/session.rs` (the pump; item 0.3 edits `use_behaviour` after it), `primitives/src/ts/bridge.ts`, `primitives/src/ts/scroll_classifier.ts` (new), `primitives/tsconfig.json`, the focus owner's TypeScript, the slot table (a TSV beside the behaviour budgets), `preview/src/main.rs` (the bridge boot and the `data-hydrated` line only), `playwright/hydration.ts` (`gotoHydrated` waits for the ready attribute), `playwright/witness/spikes/scroll-classifier.spec.ts` (new). Must not touch the behaviour rows' files or `top_layer/`.
- **Depends on:** [row 444](./444-gate-half-of-lane-3-0b-grants-bans-and-witness-presence.md), [row 446](./446-phase-0-6-witness-harness-and-the-first-witnesses.md) (the order counterfactual in the channel wrapper). It precedes [row 448](./448-phase-0-3-proof-port-a-focus-trap-and-otpfilter.md) and [row 449](./449-phase-0-4-proof-port-b-the-executor-and-popover-spreads.md).
- **Proof:** One order spec per instance of the order contract (round 5 and 6 specs lists, root doc section 5.6); the scroll classifier spike's verdict; the slot-table gate; the desktop-cadence build of [row 442](./442-desktop-cadence-harness-and-desktop-pass.md) shows the order cases.
- **Hours (inferred):** about 18 (11 after round 5, new then: pump about 4, epoch rule about 2, transition port about 2, focus owner about 2, slot table and gate about 1; round 6 +7: the Rust-caused rule and its order witnesses about 1, the owner-rendered port with its hooks and handshake about 1, the scroll classifier spike about 3, the verb partition, the two new slot rows, the slot nonce, the pump registry and the ready ordering about 2).
