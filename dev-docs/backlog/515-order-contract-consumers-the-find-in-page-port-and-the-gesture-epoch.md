---
id: 515
title: "Order contract consumers: the find-in-page (beforematch) port, the gesture epoch and the contract details row 447 leaves unbuilt"
status: "Depends on row 447 (landed 2026-11-14). Not started."
size: "M (inferred)"
area: primitives
updated: 2026-11-14
---

# Row 515: order contract consumers

## Item

[Row 447](./447-phase-0-7-the-order-runtime.md) builds the order runtime (the inbound pump, the slot fence, the slot table, the focus owner, the boot). The remaining items of its original text each need a consumer: a component or behaviour whose user-visible case the item fixes. Code with no consumer is not built, and a vdom handler lives in the owner's own file, which row 447 may not touch. This row records, per item, the first consumer, the files, the proof, and where another row already owns those files (so that no two rows own one file). The grant for any of it is "needs a grant; the number is set at build".

## Status / trigger

Depends on [row 447](./447-phase-0-7-the-order-runtime.md) (landed 2026-11-14). Not started.

## Size

M (inferred)

## Detail

Items that live in this row:

- **The `beforematch` fallback port and the owner for `use_beforematch`.** Consumer: [`collapsible.rs`](../../primitives/src/collapsible.rs) (`use_beforematch`, `until_found`), `tabs.rs` (its panels call it) and `accordion.rs`. Case: find-in-page opens a collapsed `hidden="until-found"` panel; with N panels in one accordion the N facts must reach one decision in order (the case the pump exists for). Dioxus 0.7.9 has no `onbeforematch`, so a hidden element the owner renders receives a synthetic event from a window-capture listener. Files: `collapsible.rs`, `tabs.rs`, `accordion.rs`, the hook in `bridge/`. Proof: an order spec (find-in-page style `hidden=until-found` reveal through `scrollIntoView`/fragment, Chromium) where two panels reveal in one task and the accordion ends with one panel open. Inferred 4 hours.
- **The gesture epoch.** Consumer: the toast swipe (`toast.rs`, a `pointercancel`/release that never arrives) and `drawer.rs` (`pointercancel` ends a gesture). Case: a swipe whose release never arrives leaves the toast stuck mid-drag; the epoch minted at the stream's start rides each move frame and the stream ends by timeout or `visibilitychange`. Files: `toast.rs`, `drawer.rs`, the `listen` option in `bridge/session.rs` and `ts/bridge.ts`. Proof: an order spec that drops the release (a context-switch to another tab through `visibilitychange`) and asserts the toast returns. Inferred 4 hours; the shape (a `gesture` option of `listen` with start, end events and a timeout) is built when the first consumer is.
- **An owner row for `use_form_reset_listener`.** Consumers: `slider.rs`, `switch.rs`, `select/components/select.rs`. Case: the form's `reset` (row 468's cause) must reach the control through the value owner's `input` re-dispatch rather than a bespoke listener. This item waits for the value owner, which row 448 builds (below); it then replaces the three call sites. Files: those three and `form_reset.rs`. Proof: `form-reset-controlled.spec.ts` and one order spec (reset then a keystroke in one gesture). Inferred 2 hours.
- **An owner row for `pointer.rs`.** Consumer search: `rg "crate::pointer"` finds `drawer.rs` (a doc reference), the chart modules (their own `pointer` module, a different file) and no other caller of `primitives/src/pointer.rs`'s root-owned pointer bridge. No consumer found beyond the file itself; it stays as it is until a pointer-held-down decision needs the order contract.

Items that move to another row (the row that already owns the files):

- **The vdom carrier for popover open (`onbeforetoggle`)**: [row 449](./done/449-phase-0-4-proof-port-b-the-executor-and-popover-spreads.md) already owns the handler at its 13 call sites and the "dismiss then `onclick`" spec.
- **The vdom carrier for dialog open (`oncancel`)**: row 449 (the executor's modal kind). The premise note finds `use_dialog_open_driver` in `lib.rs`, `dialog.rs` and `alert_dialog.rs` with no owner; row 449's file list takes them. Case: Escape on a modal then Enter on the refocused trigger.
- **The value owner's `input` re-dispatch (reset, restoration, install-time level), the carrier `oninput`, the composition hold and the value-write effect**: [row 448](./done/448-phase-0-3-proof-port-a-focus-trap-and-otpfilter.md). Its `otpFilter` is the first value-write consumer (`input_otp.rs`, the paste defect of row 474). The kernel's `slots` effect (row 447) carries the `value` write as its second entry.
- **The `(scope, nonce)` identity and SSR-stable ids**: row 449 (the executor is keyed by them). Case: a dismissal made before the bridge installs survives hydration on an SSG page.
- **The roving-focus carrier (`onfocusin`/`onfocusout`)**: the handlers exist (`tabs.rs`, `virtual_list.rs`, `date_picker/segment.rs`); the bridge now reports the same transitions. The writer side is [row 462](./462-port-roving-focus-78-set-focus-sites.md), conditional on its witness.

Items with no consumer found (searched: `rg "re-creat|keyed"` over `primitives/src`, the date-picker, select and data-table sources, and the `checked`/`selected` writers):

- **A kept level for `value`/`checked`/`selected` of a re-created element.** No consumer found. Not built.
- **`onscrollend` as a carrier.** `onscrollend` appears only inside the carousel's own scripts; no rsx decision reads a page-written scroll position. The classifier spike (row 447) shows the signals a `scrollOwner` would use; build it with the first Rust-written scroller that needs it (candidate: the message scroller's stick-to-end detach). No consumer found for an rsx handler.

**Open question.**

Does a held `value` write ever apply? A write held during composition carries the `basedOn` of before the composition, and the input that ends the composition is a page transition after it, so the fence drops the held write every time. The one answer to build: during composition the write is not held but dropped at once with a reply `dropped`, and the value owner re-sends from the committed text at `compositionend` (the owner decides again from the new level, which is what a fence is for). The row 447 text "never drops the write" is then read as "the hold never loses the decision, the owner re-decides". This question is raised in row 447 and answered here with row 448's first value write.
