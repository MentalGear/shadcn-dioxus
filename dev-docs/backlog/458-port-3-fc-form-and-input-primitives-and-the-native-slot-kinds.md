---
id: 458
title: "Port 3.FC: form and input primitives, label click, and the native slot kinds on the executor"
status: "BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan."
size: "L"
area: inputs
updated: 2026-10-26
---

# Row 458: Port 3.FC: form and input primitives, label click, and the native slot kinds on the executor

## Item

Port S11 (`otpFilter`), S12, S13, L25 (`dropAnywhere`, with its `dragSource` half for the three `DataTransfer` sites), L23 (`labelClick`, a named behaviour that keeps the primitive's contract: `label-double-click.spec.ts` lifts `user-select: none` to test the unstyled Label and asserts a drag still selects), S8 and S9 (executor consumers), S10, S17, S18, L24 and S4 (after `scrollOwner`). It also moves the native slot kinds (`value`, `checked`, `selected`, `hidden=until-found`, `<details open>`) onto the executor, with the rsx writes removed and the install-time level and `reset` transitions reported. Why: the vdom is a writer of these slots too, which is what loses typed text and shows a panel open while `aria-expanded` is false.

Part of the [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.4 (candidate rows) and the ledger of section 3.3.

## Status / trigger

BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan.

## Size

L

## Detail

- **Files owned:** `input_otp.rs`, `drag_and_drop_list.rs`, `label.rs`, `checkbox.rs`, `default_checked.rs`, `radio_group.rs`, `avatar.rs`, `questionnaire.rs`, `collapsible.rs`, `accordion.rs`, `tabs.rs` (the hidden slot only), `combobox/components/{option,input}.rs`, `preview/src/components/data_table/reorder.rs` and `preview/src/components/virtual_list/variants/reorder/mod.rs` (the `dragSource` lines), `ts/behaviours/{otp_filter,drop_anywhere,label_click}.ts`, `bridge/behaviours/{otp_filter,drop_anywhere,label_click}.rs`, `primitives/conformance/behaviours/{otp_filter,drop_anywhere,label_click}.json`. Must not touch `collection.rs` (3.SC), `preview/src/components/questionnaire/` (3.PV).
- **Depends on:** [row 449](./449-phase-0-4-proof-port-b-the-executor-and-popover-spreads.md) (executor), [row 457](./457-port-3-vs-virtual-settle-and-the-scrollowner-behaviour.md) (S4), [row 447](./447-phase-0-7-the-order-runtime.md) (value owner), [row 444](./444-gate-half-of-lane-3-0b-grants-bans-and-witness-presence.md); `adds:` rows for the three kinds.
- **Proof:** The witness per effect for each behaviour (the `dragSource` half is expected green on Chromium and judged on Firefox or under fallback emulation); form-reset and pre-hydration typing specs (see rows 468 and 469); otp-input, label, checkbox, accordion, combobox specs.
- **Hours (inferred):** 18 (10 before; +1 L23, +1 the two preview files, +6 the native slot kinds); removes about 91 lines, relocates 46 (5 per hour).
