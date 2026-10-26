---
id: 462
title: "Port: roving focus (78 set_focus sites in 32 primitives files through the focus owner)"
status: "BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan."
size: "L"
area: primitives
updated: 2026-10-26
---

# Row 462: Port: roving focus (78 set_focus sites in 32 primitives files through the focus owner)

## Item

The rule's largest application: every `set_focus` site (78 in 32 primitives files; the workspace has 82 in 36) gets `aria-activedescendant` on the container or the `roving` behaviour (only on a red witness), through the focus owner; the typeahead of `typeahead.rs` and `listbox.rs:74` fall under it. Why: a late roving write after a click elsewhere steals focus back, and `MountedData::set_focus` is an unordered second channel on desktop. Last of the candidates: its 32 files overlap 0.3, 0.4, 3.PL, 3.OV, 3.GS and 3.FC. No board sized it before round 5.

Part of the [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.4 (candidate rows) and the ledger of section 3.3.

## Status / trigger

BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan.

## Size

L

## Detail

- **Files owned:** The 32 files that carry `set_focus`. Must not touch `lib.rs` focus plumbing.
- **Depends on:** [row 447](./447-phase-0-7-the-order-runtime.md) (focus owner), [row 448](./448-phase-0-3-proof-port-a-focus-trap-and-hotkey.md), [row 449](./449-phase-0-4-proof-port-b-the-executor-and-popover-spreads.md), [row 452](./452-port-3-pl-placement-anchor-fallback-and-the-paint-safe-gate.md), [row 455](./455-port-3-gs-gestures-slider-resizable-drawer-toast-pan.md), [row 456](./456-port-3-ov-overlay-core-escape-dismiss-open-focus.md), [row 458](./458-port-3-fc-form-and-input-primitives-and-the-native-slot-kinds.md) (file overlap).
- **Proof:** A key-repeat spec per widget family; the `roving` behaviour's witness (2-4 h, only if red).
- **Hours (inferred):** about 28 (24-32), inferred: 32 files at about 0.6-0.8 h plus a key-repeat spec per widget family, and 2-4 h for the `roving` behaviour only if its witness is red.
