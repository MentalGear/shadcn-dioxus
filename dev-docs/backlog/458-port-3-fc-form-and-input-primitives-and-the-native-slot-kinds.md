---
id: 458
title: "Port 3.FC: form and input primitives, label click, and the native slot owners"
status: "BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan."
size: "L"
area: inputs
js-grant: js_file=46 (relocated lines, root doc section 5.4; the main loop's grant row at the merge base is the number the gate reads)
adds: [dropAnywhere, labelClick]
updated: 2026-10-26
---

# Row 458: Port 3.FC: form and input primitives, label click, and the native slot owners

## Item

Port S11 (`otpFilter` is built by item 0.3; this row moves `input_otp.rs`'s value slot to the value owner and its click-to-slot caret write to the owner's selection), S12, S13, L25 (`dropAnywhere`, with its `dragSource` half for the three `DataTransfer` sites), L23 (`labelClick`, a named behaviour that keeps the primitive's contract: `label-double-click.spec.ts` lifts `user-select: none` to test the unstyled Label and asserts a drag still selects), S8 and S9 (executor consumers), S10, S17, S18, L24 and S4 (after `scrollOwner`). It also moves the native slot kinds onto their owners: `hidden=until-found` on the executor, and `value`, `checked` and `selected` as owner writes over the bridge (never an attribute; a mount-frozen helper emits the native attribute at first render), with the rsx writes removed, the native-slot sink scan's baseline drained, one owner per element for `checked` and `indeterminate`, the install-time level, restoration and the predicted post-`reset` level each re-dispatched by the value owner as a real `input` on the control (`reset` fires before the controls reset; this also fixes [row 468](./468-bug-form-reset-leaves-controlled-input-out-of-sync.md)'s user-visible defect), a kept level so a re-created element gets the last Rust write, and an IME hold for composing input that times out and never drops the write (Android). `<details open>` has no consumer and is not a kind. Why: the vdom is a writer of these slots too, which is what loses typed text and shows a panel open while `aria-expanded` is false.

Part of the [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.4 (candidate rows) and the ledger of section 3.3.

## Status / trigger

BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan.

## Size

L

## Detail

- **Files owned:** `input_otp.rs`, `drag_and_drop_list.rs`, `label.rs`, `checkbox.rs`, `default_checked.rs`, `radio_group.rs`, `avatar.rs`, `questionnaire.rs`, `collapsible.rs`, `accordion.rs`, `tabs.rs` (the hidden slot only), `combobox/components/{option,input}.rs`, `preview/src/components/data_table/reorder.rs` and `preview/src/components/virtual_list/variants/reorder/mod.rs` (the `dragSource` lines), `ts/behaviours/{drop_anywhere,label_click}.ts`, `bridge/behaviours/{drop_anywhere,label_click}.rs`, `primitives/conformance/behaviours/{drop_anywhere,label_click}.json`. Must not touch `collection.rs` (3.SC), `preview/src/components/questionnaire/` (3.PV).
- **Depends on:** [row 449](./449-phase-0-4-proof-port-b-the-executor-and-popover-spreads.md) (executor), [row 457](./457-port-3-vs-virtual-settle-and-the-scrollowner-behaviour.md) (S4), [row 447](./447-phase-0-7-the-order-runtime.md) (value owner), [row 444](./444-gate-half-of-lane-3-0b-grants-bans-and-witness-presence.md); `adds:` rows for the two kinds (`dropAnywhere`, `labelClick`).
- **Proof:** The witness per effect for each behaviour (the `dragSource` half is expected green on Chromium and judged on Firefox or under fallback emulation); form-reset and pre-hydration typing specs (see rows 468 and 469); the OTP paste spec of [row 474](./474-bug-input-otp-paste-loses-digits-to-maxlength.md); the date-picker segment specs of [row 473](./473-bug-date-picker-segments-let-the-browser-edit-the-vdom-text-node.md); otp-input, label, checkbox, accordion, combobox specs.
- **Hours (inferred):** 19 (18 after round 5, 10 before it; round 6 +2 for the owner-write carrier, the IME hold and draining the sink scan's baseline, -1 for `otpFilter`, built in item 0.3); removes about 91 lines, relocates 46 (5 per hour).
