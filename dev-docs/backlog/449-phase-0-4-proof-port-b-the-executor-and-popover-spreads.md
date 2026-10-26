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

The one page executor (round 6 withdrew the per-kind choice between a page reconciler and a Rust `transaction`), built for the first four reconciled kinds (popover open, modal open, `indeterminate`, `defaultChecked`) on a table of kinds that the native slot kinds join later, and admitted as a whole by the reds of item 0.6's pre-authored witnesses: boot install, discovery by `childList`, the one epoch rule, the one show writer, the stale `placed` removal, the watchdog arming for declared pending states, the exit wait (an `AbortError` counts as done) and the transition hooks of the port (`beforetoggle`; the dialog's `open` observer and `cancel`). The popover hooks (`use_popover_sync`, `use_popover_shown_while_mounted`) return the attribute to spread and the port element instead of `()`, and this item owns the one-line spreads at their 13 call sites in 11 files. Consumed by T4 to T7 here and by 3.OV and 3.FC later. Why: one executor per element is what removes the lost-command class, the stale toggle echo (row 115) and the two-writer `<dialog open>` trap.

Part of the [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.4 (phase 0).

## Status / trigger

BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan.

## Size

L

## Detail

- **Files owned:** `top_layer/popover_sync.rs`, the executor in `bridge/`, its page half in `primitives/src/ts/executor.ts` (new; a bridge internal, not a registry module), `ts/bridge.ts`, `primitives/tsconfig.json`, and the spread line (nothing else) in `tooltip.rs:452`, `dropdown_menu.rs:765,1927`, `select/components/list.rs:378`, `hover_card.rs:600`, `navbar.rs:913`, `popover.rs:745`, `toast.rs:886`, `menubar.rs:853`, `context_menu.rs:728,1953`, `navigation_menu.rs:1013`, `combobox/components/list.rs:128` (after 0.3, which owns the `not(web)` arms of five of them). Must not touch the rest of `lib.rs` and of `top_layer/`.
- **Depends on:** [row 446](./446-phase-0-6-witness-harness-and-the-first-witnesses.md) (the pre-authored executor witnesses it flips), [row 447](./447-phase-0-7-the-order-runtime.md) (the epoch rule), [row 448](./448-phase-0-3-proof-port-a-focus-trap-and-otpfilter.md) (the shared files).
- **Proof:** The pre-authored executor witnesses (exit wait, watchdog, port) turned from `test.fixme` into reds that pass; the first-frame, open/close/open and stuck-channel specs; popover, select, combobox, tooltip, hover-card and menu specs on a release build.
- **Hours (inferred):** about 12 (13 after round 5, 9 before it; round 6 -2 for the per-kind decision and `reconcile`'s registry module, +1 for the port element and its two hooks).
