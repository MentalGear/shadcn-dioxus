---
id: 459
title: "Port 3.SL: scroll lock (L10, L11, L13)"
status: "APPROVED 2026-10-26 (scope 2): after phase 0, taken as an ordinary row in order of JavaScript…"
size: "M"
area: overlays
js-grant: js_file=94 (relocated lines, root doc section 5.4; the main loop's grant row at the merge base is the number the gate reads)
adds: [scrollLock]
updated: 2026-10-26
---

# Row 459: Port 3.SL: scroll lock (L10, L11, L13)

## Item

Port L11 and L10 (`scrollLock`, a named behaviour) and L13 (the restore of `window.scrollY` goes with the executor's scroll save and restore around `showModal()`, item 0.4: the document scroller is not a slot and has no `scrollOwner`); characterisation specs first. Low yield (about 0.2 lines per hour, relocating about 94 lines) but it closes a blocking-listener class.

Part of the [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.4 (candidate rows) and the ledger of section 3.3.

## Status / trigger

APPROVED 2026-10-26 (scope 2): after phase 0, taken as an ordinary row in order of JavaScript removed per hour.

## Size

M

## Detail

- **Files owned:** `scroll_lock.rs` (cited by symbol: the audit's `:422,432,450` are `:413,423,441` in batch 28), `context_menu.rs` (the L10 block only), `ts/behaviours/scroll_lock.ts`, `bridge/behaviours/scroll_lock.rs`, `primitives/conformance/behaviours/scroll_lock.json`, `playwright/` specs. Must not touch `top_layer/`.
- **Depends on:** [row 449](./449-phase-0-4-proof-port-b-the-executor-and-popover-spreads.md) (the scroll save and restore that replaces `scroll_lock.rs:267-298`), [row 448](./448-phase-0-3-proof-port-a-focus-trap-and-otpfilter.md) (the registry macro and adapter), [row 452](./452-port-3-pl-placement-anchor-fallback-and-the-paint-safe-gate.md) (`context_menu.rs` order: 0.3, 0.4, 3.PL, then this row), [row 444](./444-gate-half-of-lane-3-0b-grants-bans-and-witness-presence.md); an `adds: [scrollLock]` row.
- **Proof:** The witness per `scrollLock` effect; characterisation specs written first; context-menu and dialog scroll-lock specs.
- **Hours (inferred):** 7 (8 in round 6; round 7 -1: no document-scroller owner); removes about 2 lines, relocates 94.
