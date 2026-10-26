---
id: 451
title: "Port 3.SC: shared collection and activity scripts (S7, S15)"
status: "BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan."
size: "S"
area: primitives
js-grant: none
adds: []
updated: 2026-10-26
---

# Row 451: Port 3.SC: shared collection and activity scripts (S7, S15)

## Item

Port S7 (`collection.rs`, with the per-row `onmounted` of `collection.rs:738`) and S15 (`activity.rs`) off bespoke eval. Highest yield of the candidates: about 30 JavaScript lines removed per agent-hour.

Part of the [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.4 (candidate rows) and the ledger of section 3.3.

## Status / trigger

BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan.

## Size

S

## Detail

- **Files owned:** `collection.rs`, `activity.rs`. Must not touch the rest. `collection.rs:520` is the roving hub that the conditional roving-focus row ([row 462](./462-port-roving-focus-78-set-focus-sites.md)) would route through the focus owner, so that row waits for this one.
- **Depends on:** [row 444](./444-gate-half-of-lane-3-0b-grants-bans-and-witness-presence.md) (grant rule) and [row 445](./445-phase-0-2-the-legacy-freeze.md) (legacy freeze); a candidate row also needs its own grant row, and an `adds:` row where it adds a behaviour.
- **Proof:** The components that use them (specs of `collection`/`activity` consumers on a release build); `check-js-volume` baseline rows lowered.
- **Hours (inferred):** 4 (inferred); removes about 118 lines (27-30 per hour).
