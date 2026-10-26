---
id: 462
title: "Port: roving focus (about 28 real DOM focus writes in 18 files through the focus owner), conditional on the focus-write witness"
status: "BACKLOG. Conditional (round 7): exists only if the phase-0 focus-write witness is red; the row 434…"
size: "L"
area: primitives
js-grant: js_file=40 (inferred ceiling, only if built: the `roving` behaviour about 40 lines; the main loop's grant row at the merge base is the number the gate reads)
adds: [roving] (only if the witness is red)
updated: 2026-10-26
---

# Row 462: Port: roving focus (about 28 real DOM focus writes in 18 files through the focus owner), conditional on the focus-write witness

## Item

**Conditional on the phase-0 focus-write witness (item 0.6):** the race is desktop-only and unreproduced, so this row exists only if that witness is red. The rule's largest application: every real DOM focus write (`MountedData::set_focus`; the census separates it from `FocusState::set_focus`, which is a Rust-side index: about 28 writes in 18 files, not the 84 raw matches; the hub is `collection.rs:520`, 3.SC's file) gets `aria-activedescendant` on the container or the `roving` behaviour (built only if the witness is red), through the focus owner; the typeahead of `typeahead.rs` and `listbox.rs:74` fall under it, and so do the date picker's segments (`date_picker/segment.rs`: Rust's active index routes the edit keys, DOM focus follows through the focus owner, every editing default action is cancelled). Why: a late roving write after a click elsewhere steals focus back, and `MountedData::set_focus` is an unordered second channel on desktop. Last of the candidates: its 18 files overlap 0.3, 0.4, 3.PL, 3.OV, 3.GS and 3.FC.

Part of the [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.4 (candidate rows) and the ledger of section 3.3.

## Status / trigger

BACKLOG. Conditional (round 7): exists only if the phase-0 focus-write witness is red; the row 434 step 3 plan is approved (2026-10-26, scope 2).

## Size

L

## Detail

- **Files owned:** The 18 files that carry a real DOM focus write (re-counted by the census when the row opens), `collection.rs` after [row 451](./451-port-3-sc-shared-collection-and-activity.md). Must not touch `lib.rs` focus plumbing.
- **Depends on:** [row 446](./446-phase-0-6-witness-harness-and-the-first-witnesses.md) (the focus-write witness that decides whether this row exists), [row 447](./447-phase-0-7-the-order-runtime.md) (focus owner), [row 448](./448-phase-0-3-proof-port-a-focus-trap-and-otpfilter.md), [row 449](./449-phase-0-4-proof-port-b-the-executor-and-popover-spreads.md), [row 452](./452-port-3-pl-placement-anchor-fallback-and-the-paint-safe-gate.md), [row 455](./455-port-3-gs-gestures-slider-resizable-drawer-toast-pan.md), [row 456](./456-port-3-ov-overlay-core-escape-dismiss-open-focus.md), [row 458](./458-port-3-fc-form-and-input-primitives-and-the-native-slot-kinds.md) (file overlap).
- **Proof:** A key-repeat spec per widget family; the `roving` behaviour's witness (2-4 h, only if red).
- **Hours (inferred):** about 16 (12-20), **conditional**, not in the firm total: 30 (26-34) before round 7 (32 files at about 0.6-0.8 h), cut by the census to 18 files at about 0.6-0.8 h each, a key-repeat spec per widget family, 2-4 h for the `roving` behaviour and 2 h for the date segments and the focus owner's ensure-visible hand-off.
