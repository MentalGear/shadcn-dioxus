---
id: 449
title: "Phase 0.4: proof port B (the reconciled-kinds executor, the popover hooks that return the attribute to spread)"
status: "BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan."
size: "L"
area: overlays
updated: 2026-10-26
---

# Row 449: Phase 0.4: proof port B (the reconciled-kinds executor, the popover hooks that return the attribute to spread)

## Item

The executor that item 0.6's `reconcile` witness selects, built for the first four reconciled kinds (popover open, modal open, `indeterminate`, `defaultChecked`) on a table of kinds that the native slot kinds join later: boot install, discovery by `childList` or the `transaction`, the one epoch rule, the one show writer, the stale `placed` removal, the watchdog arming for declared pending states, and the exit wait. The popover hooks (`use_popover_sync`, `use_popover_shown_while_mounted`) return the attribute to spread instead of `()`, and this item owns the one-line spreads at their 13 call sites in 11 files. Consumed by T4 to T7 here and by 3.OV and 3.FC later. Why: one executor per element is what removes the lost-command class, the stale toggle echo (row 115) and the two-writer `<dialog open>` trap.

Part of the [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.4 (phase 0).

## Status / trigger

BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan.

## Size

L

## Detail

- **Files owned:** `top_layer/popover_sync.rs`, the executor in `bridge/`, `bridge/behaviours/reconcile.rs` and `ts/behaviours/reconcile.ts` only if the page reconciler is kept, `primitives/conformance/behaviours/reconcile.json`, `ts/bridge.ts`, `primitives/tsconfig.json`, and the spread line (nothing else) in `tooltip.rs:452`, `dropdown_menu.rs:765,1927`, `select/components/list.rs:378`, `hover_card.rs:600`, `navbar.rs:913`, `popover.rs:745`, `toast.rs:886`, `menubar.rs:853`, `context_menu.rs:728,1953`, `navigation_menu.rs:1013`, `combobox/components/list.rs:128` (after 0.3, which owns the `not(web)` arms of five of them). Must not touch the rest of `lib.rs` and of `top_layer/`.
- **Depends on:** [row 446](./446-phase-0-6-witness-harness-and-prototype-reconciler.md) (the prototype and witnesses it consumes), [row 447](./447-phase-0-7-the-order-runtime.md) (the epoch rule), [row 448](./448-phase-0-3-proof-port-a-focus-trap-and-hotkey.md) (the shared files).
- **Proof:** The `reconcile` witness per kind; the first-frame, open/close/open and stuck-channel specs; popover, select, combobox, tooltip, hover-card and menu specs on a release build.
- **Hours (inferred):** about 13 (9 before; +4 for the 13 spreads and the per-kind executor decision).
