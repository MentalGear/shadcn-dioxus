---
id: 460
title: "Port 3.HV: frame-rate vdom handlers (the check-hot-events baseline)"
status: "BACKLOG. Deferred and conditional (round 7): the row 434 step 3 plan is approved (2026-10-26, scope…"
size: "M"
area: primitives
js-grant: none
adds: []
updated: 2026-10-26
---

# Row 460: Port 3.HV: frame-rate vdom handlers (the check-hot-events baseline)

## Item

Replace the existing hot vdom handlers with observe-by-key or sample streams: the per-row `onresize` of `virtual_list.rs:486`, `table_rows.rs:289`, `virtual_grid.rs:729,1360,1526`, Select `list.rs:503,591`, `chart.rs:701` and `context_menu.rs:531` `onpointermove`, slider, resizable, color-picker and justified `onresize`, the `ondragover` of `drag_and_drop_list` and of the preview reorder, and the preview `data_table` handlers (about 20 sites). Drains the `check-hot-events` baseline. **Deferred and conditional since round 7:** it removes no JavaScript (it removes blocking requests on desktop, an inferred and unmeasured cost), so it is built only if the owner wants it after the latency session. Why: one `onpointermove` makes the event a blocking synchronous request across the whole app on desktop.

Part of the [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) step 3 plan; source: [`sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.4 (candidate rows) and the ledger of section 3.3.

## Status / trigger

BACKLOG. Deferred and conditional (round 7): the row 434 step 3 plan is approved (2026-10-26, scope 2), and built only if the owner wants it.

## Size

M

## Detail

- **Files owned:** Those files only, at the lines named; `preview/src/components/**` for the preview ones. Must not touch `collection.rs` (3.SC) or the settle and gesture code of the same files.
- **Depends on:** [row 444](./444-gate-half-of-lane-3-0b-grants-bans-and-witness-presence.md) (`check-hot-events`), [row 451](./451-port-3-sc-shared-collection-and-activity.md) (`collection.rs`), [row 457](./457-port-3-vs-virtual-settle-and-the-scrollowner-behaviour.md) (`virtual_list.rs`), [row 458](./458-port-3-fc-form-and-input-primitives-and-the-native-slot-kinds.md) (`reorder/mod.rs` `dragSource` lines in the same file).
- **Proof:** `main-thread` specs on a release build; the hot-events baseline shrinks.
- **Hours (inferred):** 11, conditional and deferred (10 before; +1 for the reorder `ondragover`); not in the firm total of 340-370 h; removes 0 lines (it removes blocking requests).
