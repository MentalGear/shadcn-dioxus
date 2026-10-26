---
id: 457
title: "Port 3.VS: virtual settle (W1-W7) and the scrollOwner one-shot kind"
status: "BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan."
size: "M"
area: virtual
js-grant: js_file=122 (relocated lines, root doc section 5.4; the main loop's grant row at the merge base is the number the gate reads)
adds: [settle, scrollOwner]
updated: 2026-10-26
---

# Row 457: Port 3.VS: virtual settle (W1-W7) and the scrollOwner one-shot kind

## Item

Port the virtual-engine blocks W3, W5, W7 (`settle`, if its witness is red), W4, W6, W1 and W2, and build `scrollOwner`, **the one-shot kind only**: the page-side half of every Rust-written scroller (it classifies by the realized default action (one passive window bubble listener reading `defaultPrevented`) with an intent kind per owner, applies `based_on` writes and reports settle). The document scroller is not a slot (the executor saves and restores scroll around `showModal()`). The level kind (`stickToEnd`'s detach) and the snap kind (the carousel) are built by [row 465](./465-port-3-ms-message-scroller-sticktoend-and-native-anchoring.md) and [row 464](./464-port-3-ca-carousel-rust-arms-next-to-the-frozen-legacy.md), the owner-scheduled ports that need them. The classifier is built first as item 0.7's spike, so this row builds the owner around it. The first consumer is W2's `scrollTop`/`scrollLeft` setters, which a verb row cannot hold. Why: scroll position has page writers no `scroll` event can tell apart, so each Rust-written scroller needs one owner; S4, V6, the message scroller and the carousel consume it (L13 no longer does: the document scroller is not a slot).

Part of the [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.4 (candidate rows) and the ledger of section 3.3.

## Status / trigger

BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan.

## Size

M

## Detail

- **Files owned:** `virtual/{scroll_to,find,bridge}.rs`, `virtual_grid.rs` (`FOCUS_CELL_JS` and the cell-controls block only), `ts/behaviours/{settle,scroll_owner}.ts`, `bridge/behaviours/{settle,scroll_owner}.rs`, `primitives/conformance/behaviours/{settle,scroll_owner}.json`. Must not touch `virtual_list.rs` and the rest of `virtual_grid.rs` (3.HV).
- **Depends on:** [row 447](./447-phase-0-7-the-order-runtime.md) (epoch rule and slot table), [row 448](./448-phase-0-3-proof-port-a-focus-trap-and-otpfilter.md) (the registry macro, `site!` and the adapter that every behaviour needs), [row 444](./444-gate-half-of-lane-3-0b-grants-bans-and-witness-presence.md); an `adds: [settle, scroll_owner]` row. It precedes [row 453](./453-port-3-pv-preview-glue-and-platform-rule-rows.md) (V6), [row 458](./458-port-3-fc-form-and-input-primitives-and-the-native-slot-kinds.md) (S4), [row 464](./464-port-3-ca-carousel-rust-arms-next-to-the-frozen-legacy.md), [row 465](./465-port-3-ms-message-scroller-sticktoend-and-native-anchoring.md).
- **Proof:** The witness per `scrollOwner` effect (a wheel in between wins; an anchoring adjustment and a re-snap stay neutral); virtual-list, virtual-grid and find-in-page specs on a release build.
- **Hours (inferred):** 10 (11 in round 6; round 7 -1: the document scroller is not a slot and only the one-shot kind is built here; 14 after round 5, 10 before it); removes about 87 lines, relocates 122 (6 per hour).
