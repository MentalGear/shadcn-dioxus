---
id: 453
title: "Port 3.PV: preview glue (V3-V23) with the V14 allowlist row and the Ctrl/Cmd+B platform rule row"
status: "BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan."
size: "M"
area: preview
js-grant: none
adds: []
updated: 2026-10-26
---

# Row 453: Port 3.PV: preview glue (V3-V23) with the V14 allowlist row and the Ctrl/Cmd+B platform rule row

## Item

Port the preview-only blocks: V3, V4 (class (c), stays), V6 (ported last, after `scrollOwner` exists), V7, V10, V12, V13, V14, V18, V19 (G, a bridge `observe` and a data attribute), V20, V21, V22 (a `hotkey` consumer), V23; V5, V8 and V9 take reasoned allowlist rows (item 0.1). V14 is a reasoned preview-only allowlist row, not a platform rule row: the `code_block` component takes a callback and carries no sink, and the preview's own clipboard writer is the allowlisted one (a library `copyText` effect would be category (b), which is on the plan's exits list while no library code uses the clipboard); V22's `prevent` for Ctrl/Cmd+B is a platform rule row on a registry `site!` (category (a), a UA-reserved shortcut: Firefox opens the Bookmarks sidebar).

Part of the [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.4 (candidate rows) and the ledger of section 3.3.

## Status / trigger

BACKLOG. Awaiting the owner's approval of the row 434 step 3 plan.

## Size

M

## Detail

- **Files owned:** `preview/src/{main,charts_gallery,docs_nav}.rs`, `preview/src/site/{mod,component_page,chrome,code}.rs`, `preview/src/components/{code_block,form,top_layer,questionnaire}/...`, `preview/src/components/virtual_list/variants/*` except `reorder/mod.rs` (3.FC, then 3.HV), `preview/src/components/sidebar/component.rs`, `preview/src/dashboard/views/email_client/scroll_memory.rs`. Must not touch `primitives/`, `preview/src/theme.rs`, `preview/index.html` (3.PW).
- **Depends on:** [row 452](./452-port-3-pl-placement-anchor-fallback-and-the-paint-safe-gate.md) (V19 needs the pending CSS), [row 455](./455-port-3-gs-gestures-slider-resizable-drawer-toast-pan.md) or [row 456](./456-port-3-ov-overlay-core-escape-dismiss-open-focus.md), whichever builds `hotkey` first (V22), [row 457](./457-port-3-vs-virtual-settle-and-the-scrollowner-behaviour.md) (V6 only, ported last), [row 444](./444-gate-half-of-lane-3-0b-grants-bans-and-witness-presence.md) (allowlist and platform-rule rows; the owner approves each platform rule row).
- **Proof:** The preview specs of the touched pages; V22's check line in `desktop-pass.md` (a platform rule row is checked, not witnessed).
- **Hours (inferred):** 11 (10 before round 5; +1 for V22's platform rule row and V14's allowlist row); removes about 245 lines (22 per hour).
