---
id: 539
title: "Hot-path allocation sites in filtering and drag paths (low priority, prove before changing)"
status: "BACKLOG, filed 2026-11-19, low priority. Start only with a measurement."
size: "M"
area: primitives
updated: 2026-11-19
---

# Row 539: Hot-path allocation sites

## Item

Sites that allocate per keystroke or per pointer move (read, not run; nothing was profiled):

- Combobox and command filter, `combobox/context.rs:174-176,219` and `command.rs:129-151,743`: with a non-empty query about
  five Strings per option per evaluation; `is_visible` is read 2-3 times per option render.
- `data_view/derive.rs:75-83` (`derive_by`): a `Vec<String>` per row on every recompute, empty filter included.
- `pointer.rs:70-92`: the legacy pointer listener sends a JSON message per raw pointermove, not coalesced per frame.
- `resizable.rs:700-725` (four certain allocations per effect run) and `slider.rs:638-668,1030` (`granular_thumbs.cloned()`
  per effect run, a `format!` per thumb render); `drawer.rs:581` pushes a sample per pointermove.

**Severity: low.** The paths run only while a user drags or types, and each such event also pays for a render.

## Status / trigger

BACKLOG, filed 2026-11-19, low priority. Start only with a measurement.

## Size

M

## Detail

- **Proof idea:** a counting allocator in a native test over the pure functions: `derive_by` and `default_combobox_filter`
  are callable natively; `resize_pair` is `pub(crate)`; `pointer_move` is not pure. No counting allocator, `dhat` or
  `memory.grow` check exists in the repository today (ledger B-23).
- Source: [`research/repo-evaluation-2026-11-19.md`](../research/repo-evaluation-2026-11-19.md), section 2.
