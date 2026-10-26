---
id: 447
title: "Phase 0.7: the order runtime (inbound pump, per-slot epochs, vdom carrier, slot table, focus owner, one-shot scroll classifier)"
status: "BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan."
size: "M"
area: primitives
js-grant: js_file=300 (inferred ceiling: pump, epochs and the focus owner in bridge.ts about 150, the one-shot classifier about 100, the fallback port about 50; the main loop's grant row at the merge base is the number the gate reads)
adds: []
updated: 2026-10-26
---

# Row 447: Phase 0.7: the order runtime (inbound pump, per-slot epochs, vdom carrier, slot table, focus owner, one-shot scroll classifier)

## Item

The order contract's implementation: one inbound pump for every page-to-Rust bridge fact (a page-minted sequence number and one root task that dispatches through a registry entry `Drop` removes, so it never calls a dropped scope's `Callback`), the one epoch rule (`based_on >= last_page_transition`; a write carries `based_on` for **every slot it can touch**, captured when the decision is made; epochs keyed by `(scope, nonce)` and stable across server render and hydration through `use_server_cached`, `lib.rs:205-218`) replacing the as-built `Epoch::next` (one process-global Rust counter, which can never lose to the page), the rule that Rust-caused transitions are reported but never advance the fence, **a gesture epoch** minted at a stream's start that rides every per-move frame (a swipe whose release never arrives ends by timeout or `visibilitychange`), a `focusout` with a null `relatedTarget` that mints, a `value` hold during composition that times out and never drops the write, a kept level for `value`/`checked`/`selected`, **the vdom carrier** (each owner's own rsx handler on `onbeforetoggle`, `oncancel`, `onfocusin`/`onfocusout`, `oninput`, `onreset` and `onscrollend`, with the value owner re-dispatching a real `input` for reset, restoration and the install-time level; a hidden fallback port only for `beforematch`, fired from a window-capture listener; no root `stopPropagation`, no no-form rule, no hydration-parity line, no liveness handshake), the slot table (with the contenteditable row; the document scroller is not a slot) and its gate check, the partition of `FORBIDDEN_VERBS` by the slot table (item 0.1 owns the list), the focus owner (a bridge internal on the document slot that carries `preventScroll`, replacing the `focus_prevent_scroll` row), the one-shot scroll classifier spike (realized default action and the one-shot intent kind, against an anchoring adjustment and a snap), and the bridge boot from the preview's root effect with `html[data-dx-bridge="ready"]` set after every owner's install report and `data-hydrated` after it. Owner rows for the legacy fact sources `use_beforematch`, `use_form_reset_listener` and `pointer.rs`. Why: board round 5 found neither the epoch rule nor the pump had an owner, and Dioxus events carry no epoch, so rsx decisions on a page-written slot diverge between web and desktop; round 6 found the port needed a host, a synchronous trigger and the Rust-caused rule, and round 7 found Dioxus already delivers the events, so the owners' own rsx handlers are the carrier and the port shrinks to a fallback.

Part of the [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.4 (phase 0).

## Status / trigger

BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan.

## Size

M

## Detail

- **Files owned:** `primitives/src/bridge/mod.rs` (`Epoch`, and its doc line 51, which routes agents to `MountedData::set_focus`), `primitives/src/bridge/session.rs` (the pump; item 0.3 edits `use_behaviour` after it), `primitives/src/ts/bridge.ts`, `primitives/src/ts/scroll_classifier.ts` (new), `primitives/tsconfig.json`, the focus owner's TypeScript, the slot table (a TSV beside the behaviour budgets), `preview/src/main.rs` (the bridge boot and the `data-hydrated` line only), `playwright/hydration.ts` (`gotoHydrated` waits for the ready attribute), `playwright/witness/spikes/scroll-classifier.spec.ts` (new). Must not touch the behaviour rows' files or `top_layer/`.
- **Depends on:** [row 444](./444-gate-half-of-lane-3-0b-grants-bans-and-witness-presence.md), [row 446](./446-phase-0-6-witness-harness-and-the-first-witnesses.md) (the order counterfactual in the channel wrapper). It precedes [row 448](./448-phase-0-3-proof-port-a-focus-trap-and-otpfilter.md) and [row 449](./449-phase-0-4-proof-port-b-the-executor-and-popover-spreads.md).
- **Proof:** One order spec per instance of the order contract (round 5 and 6 specs lists, root doc section 5.6); the scroll classifier spike's verdict; the slot-table gate; the desktop-cadence build of [row 442](./442-desktop-cadence-harness-and-desktop-pass.md) shows the order cases (not runnable in the lane container). A dismissal made before the bridge installs survives hydration on an SSG page (the SSR-stable ids).
- **Hours (inferred):** about 20: 18 in round 6, less about 2 for the port element, its hooks and the handshake, plus about 4 for the contract details of round 7 (every-slot `based_on`, the gesture epoch, the null-focus mint, SSR-stable ids, the composition timeout and kept levels, the value owner's `input` re-dispatch). (11 after round 5: pump about 4, epoch rule about 2, focus owner about 2, slot table and gate about 1; round 6 +7: the Rust-caused rule about 1, the scroll classifier spike about 3, the verb partition, the slot rows, the nonce, the pump registry and the ready ordering about 2.)
