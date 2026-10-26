---
id: 455
title: "Port 3.GS: gestures and toast (L21, L28-L30, O1, O3, O5 with the pan behaviour, V17)"
status: "BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan."
size: "L"
area: primitives
updated: 2026-10-26
---

# Row 455: Port 3.GS: gestures and toast (L21, L28-L30, O1, O3, O5 with the pan behaviour, V17)

## Item

Port the gesture and toast blocks: L30 and L21 (Slider's and Resizable's drag starts in rsx and ends by a window `up` fact, so its start, end and cancel are transitions on the port and its per-move frames stay on one bridge session), L28, L29, O1 (a `hotkey` consumer, which this row builds if 3.OV has not; its F6 `prevent` is a category (a) platform rule row), O3 (G, a declared pending state), O5 (`pan`: builds the module, recogniser, capture and click suppression, output through `set`) and V17. Why: one channel for every gesture's start, move and end, because on desktop rsx and bridge events are unordered.

Part of the [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.4 (candidate rows) and the ledger of section 3.3.

## Status / trigger

BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan.

## Size

L

## Detail

- **Files owned:** `drawer.rs`, `pointer.rs`, `move_interaction.rs`, `toast.rs`, `ts/behaviours/pan.ts`, `bridge/behaviours/pan.rs`, `primitives/conformance/behaviours/pan.json`, `ts/behaviours/hotkey.ts`, `bridge/behaviours/hotkey.rs` and `primitives/conformance/behaviours/hotkey.json` (only if 3.OV has not built them), the photo-wall demo. Must not touch `carousel/`, `chart.rs`, `context_menu.rs`.
- **Depends on:** [row 452](./452-port-3-pl-placement-anchor-fallback-and-the-paint-safe-gate.md) (O3 needs the pending CSS), [row 448](./448-phase-0-3-proof-port-a-focus-trap-and-otpfilter.md) (the registry macro and `site!`; this row builds `hotkey` for O1 unless 3.OV has), [row 447](./447-phase-0-7-the-order-runtime.md) (one pump and the port), [row 444](./444-gate-half-of-lane-3-0b-grants-bans-and-witness-presence.md); an `adds: [pan]` row.
- **Proof:** The witness per `pan` effect (capture, click suppression); drawer, slider, resizable, toast specs on a release build; the touch specs.
- **Hours (inferred):** 17 (16 after round 5, 13 before it; round 6 +1: start, end and cancel on the port for Slider, Resizable and the drawer); removes about 246 lines, relocates 193 (15 per hour).
