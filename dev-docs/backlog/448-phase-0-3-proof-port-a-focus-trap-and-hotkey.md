---
id: 448
title: "Phase 0.3: proof port A (the 16 dead not(web) eval sites, focus-trap.ts, and the hotkey behaviour for L6)"
status: "BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan."
size: "L"
area: overlays
updated: 2026-10-26
---

# Row 448: Phase 0.3: proof port A (the 16 dead not(web) eval sites, focus-trap.ts, and the hotkey behaviour for L6)

## Item

The first listener port and the first named behaviour: lane 3.DJ (delete the 16 eval sites in `#[cfg(not(feature = "web"))]` arms that compile only for Blitz, where eval is a no-op, plus `ts/focus-trap.ts`, its `js/` output and manifest line, the allow-TSV entry and `focus_trap_script`), then `hotkey` for L6 (its witness needs the nesting scenario from the vectors). It also builds the registry restructure (one Rust file and one vectors file per behaviour, a registry macro that collects the variants), the per-site effect typestate with `#[track_caller]` site ids, the per-install effect set and the adapter for `hotkey`. L23 is not here (it keeps the primitive's behaviour, row 3.FC). Why: it proves the construct end to end on a real block and deletes dead JavaScript first.

Part of the [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.4 (phase 0).

## Status / trigger

BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan.

## Size

L

## Detail

- **Files owned:** `dialog.rs`, `alert_dialog.rs`, `popover.rs`, `dropdown_menu.rs`, `menubar.rs`, `context_menu.rs` (the `not(web)` arms only), `lib.rs` (the focus-trap bundle constants and the L6 Escape install), `ts/focus-trap.ts`, `primitives/src/js/focus-trap.js` and its manifest line, `ts/behaviours/hotkey.ts`, `bridge/behaviours/mod.rs` (the registry macro, replacing `bridge/behaviours.rs`), `bridge/behaviours/hotkey.rs`, `primitives/conformance/behaviours/hotkey.json`, `bridge/session.rs` (`use_behaviour` only), `ts/bridge.ts` (the adapter's per-install effect-set refusal), `primitives/tsconfig.json`. Must not touch `top_layer/`, the rest of `lib.rs` (L1 and L2 are 3.OV's), `label.rs`.
- **Depends on:** [row 444](./444-gate-half-of-lane-3-0b-grants-bans-and-witness-presence.md), [row 446](./446-phase-0-6-witness-harness-and-prototype-reconciler.md) (the `hotkey` witness `test.fixme` it turns green), [row 447](./447-phase-0-7-the-order-runtime.md) (`session.rs` order).
- **Proof:** `witness(hotkey, prevent)` goes from `test.fixme` to a red-then-passing expected failure; the dialog, alert-dialog and menu specs on a release build; `check-bridge-behaviours`.
- **Hours (inferred):** about 13 (11 before; L23 leaves, about -1; the restructure and the typestate, about +3).
