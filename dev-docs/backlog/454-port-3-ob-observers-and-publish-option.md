---
id: 454
title: "Port 3.OB: observers and the publish option (L27, S16, S14)"
status: "APPROVED 2026-10-26 (scope 2): after phase 0, taken as an ordinary row in order of JavaScript…"
size: "S"
area: primitives
js-grant: none
adds: []
updated: 2026-10-26
---

# Row 454: Port 3.OB: observers and the publish option (L27, S16, S14)

## Item

Port L27 (`keyboard_inset.rs`), S16 (`chart/components/chart.rs`) and S14 (`scroll_area.rs`, as an `observe(..., publish: var)` option of the bridge; the depth stays CSS where scroll-driven animations exist) to bridge observers.

Part of the [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.4 (candidate rows) and the ledger of section 3.3.

## Status / trigger

APPROVED 2026-10-26 (scope 2): after phase 0, taken as an ordinary row in order of JavaScript removed per hour.

## Size

S

## Detail

- **Files owned:** `keyboard_inset.rs`, `chart/components/chart.rs`, `scroll_area.rs`. Must not touch `activity.rs`; `motion` gate semantics.
- **Depends on:** [row 444](./444-gate-half-of-lane-3-0b-grants-bans-and-witness-presence.md) (grant rule) and [row 445](./445-phase-0-2-the-legacy-freeze.md) (legacy freeze); a candidate row also needs its own grant row, and an `adds:` row where it adds a behaviour.
- **Proof:** Chart, scroll-area and keyboard-inset specs on a release build; `main-thread` stays at 0 long tasks.
- **Hours (inferred):** 7 (inferred); removes about 144 lines (21 per hour).
