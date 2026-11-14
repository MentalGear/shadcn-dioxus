---
id: 447
title: "Phase 0.7: the order runtime (inbound pump, slot fence, slot table, focus owner, boot, scroll classifier spike)"
status: "The runtime is on lane/order-runtime-447 and lands as it is: - The pump. Every frame the page sends…"
size: "M"
area: primitives
js-grant: js_file_bytes=5490 (300 lines at 18.3 bundle bytes per source line, inferred ceiling: pump, epochs and the focus owner in bridge.ts about 150, the one-shot classifier about 100, the fallback port about 50; the main loop's grant row at the merge base is the number the gate reads)
adds: []
updated: 2026-11-14
---

# Row 447: Phase 0.7: the order runtime

## Item

The order contract's implementation in the bridge: one inbound pump for every page-to-Rust fact, the slot fence, the slot table, the focus owner, the bridge boot and the ready attribute, with the order specs and the scroll classifier spike. Items that need a consumer in a component file, and the contract details without one, are [row 515](./515-order-contract-consumers-the-find-in-page-port-and-the-gesture-epoch.md) and rows [448](./448-phase-0-3-proof-port-a-focus-trap-and-otpfilter.md) and [449](./449-phase-0-4-proof-port-b-the-executor-and-popover-spreads.md).

Part of the [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.4 (phase 0).

## Status / trigger

The runtime is on `lane/order-runtime-447` and lands as it is:

- **The pump.** Every frame the page sends is `{seq, sub, m | t}` with a page-minted sequence number. One inbound eval per webview (`Program::Inbound`) carries every scope's frames, held in order until it runs; one root task dispatches through a registry entry that `Subscription::drop` removes, so a frame of an ended scope is still taken and reaches no handler. `bridge::page_seen()` is the contiguous prefix of taken numbers; the numbers held ahead of a gap are bounded (1024; past it the missing number is declared lost).
- **The epoch rule.** Install epochs count per webview (`behaviour:<kind>` per target, newest replaces). A slot `write` and a `call` carry `basedOn` and are dropped when a slot they can touch had a page-caused transition after it (focus is one document-wide slot). A transition inside a Rust write is reported with its cause (`c`) and fences nothing.
- **Transition listeners.** Page-lifetime listeners, started with the inbound eval, mint `focusin`, a `focusout` with a null `relatedTarget`, `beforetoggle` and `cancel`.
- **The slot table** (`scripts/check-bridge-behaviours.slots.tsv`), held equal to `METHODS` (each row names its slots), `WRITES` and `APPLY` by `check-bridge-behaviours`; `FORBIDDEN_VERBS` is derived from it.
- **The focus owner**, the first slot of the kernel's generic `slots` effect (`Decision.slots`): it focuses the host with `preventScroll`, and the focus it causes is Rust's own.
- **The scroll classifier spike** (`playwright/witness/spikes/scroll-classifier.spec.ts`, Chromium): input recency, the position a write asked for and a user-scroll memory tell a user scroll, an own write, an anchoring adjustment and a snap apart. Limit: a write within 600 ms of a user input reads as user.
- **Boot.** `bridge::boot()` opens the inbound eval and waits for `html[data-dx-bridge="ready"]`. The preview's root effect awaits it before `data-hydrated`, and `gotoHydrated` waits for both attributes, once the owner adds `bridge::boot` to the non-markup allowlist of `scripts/check-preview-composition.sh`.
- **The order specs** (`playwright/witness/order.spec.ts`, six cases on fixture pages over the shipped bridge and kernel) and `primitives/ts-test/order.test.ts`.

Grants this row needs from the owner: the source-byte columns (`ts/bridge.ts`, `types/bridge.d.ts`), `js_file_bytes` for `bridge.js` and the kernel's `gzip_bytes`; the numbers are in the lane report.

## Size

M

## Detail

- **Files owned:** `primitives/src/bridge/mod.rs`, `bridge/session.rs`, `bridge/transport.rs` (the inbound program), `ts/bridge.ts`, `ts/types/bridge.d.ts`, `primitives/tsconfig.json`, the slot table and its check in `scripts/check-bridge-behaviours.sh`, `decision-kernel/` (the `slots` effect only), `playwright/witness/order.spec.ts`, `playwright/witness/spikes/`, `primitives/ts-test/`. Must not touch the behaviour rows' files or `top_layer/`.
- **Depends on:** [row 446](./446-phase-0-6-witness-harness-and-the-first-witnesses.md), [row 509](./509-rust-decision-contract.md). Precedes rows 448, 449 and 515.
- **Proof:** the order specs and `order.test.ts` (each instance red before the piece); `decision-kernel.spec.ts`, `bridge-dom.spec.ts` and the harness specs on the new wire; the slot-table gate's self-test scenarios; the hydration and first-paint specs on an SSG build for the real site.
