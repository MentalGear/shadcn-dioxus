---
id: 459
title: "Port 3.SL: scroll lock (L10, L11, L13)"
status: "BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan."
size: "M"
area: overlays
updated: 2026-10-26
---

# Row 459: Port 3.SL: scroll lock (L10, L11, L13)

## Item

Port L11 and L10 (`scrollLock`, a named behaviour) and L13 (through `scrollOwner`); characterisation specs first. Low yield (about 0.2 lines per hour, relocating about 94 lines) but it closes a blocking-listener class.

Part of the [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.4 (candidate rows) and the ledger of section 3.3.

## Status / trigger

BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan.

## Size

M

## Detail

- **Files owned:** `scroll_lock.rs`, `context_menu.rs` (the L10 block only), `ts/behaviours/scroll_lock.ts`, `bridge/behaviours/scroll_lock.rs`, `primitives/conformance/behaviours/scroll_lock.json`, `playwright/` specs. Must not touch `top_layer/`.
- **Depends on:** [row 457](./457-port-3-vs-virtual-settle-and-the-scrollowner-behaviour.md) (L13 through `scrollOwner`), [row 444](./444-gate-half-of-lane-3-0b-grants-bans-and-witness-presence.md); an `adds: [scroll_lock]` row.
- **Proof:** The witness per `scrollLock` effect; characterisation specs written first; context-menu and dialog scroll-lock specs.
- **Hours (inferred):** 8 (inferred); removes about 2 lines, relocates 94.
