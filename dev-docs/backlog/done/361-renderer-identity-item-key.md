---
id: 361
title: "Renderer identity: item_key on VirtualList, TableRows and VirtualGrid, engine sizes keyed by item"
status: "LANDED 2026-10-22 (batch 8, PR #135). BACKLOG (owner asked 2026-10-22 to deconstruct the collection…"
size: "M."
area: primitives
updated: 2026-10-23
---

# Row 361: Renderer identity: item_key on VirtualList, TableRows and VirtualGrid, engine sizes keyed by item

## Item

Migration step 3: every windowed renderer tracks items by position today (DOM keys, pinned focus row, grid active cell, measured-height cache), so after a sort or move focus and heights stick to the slot. Optional `item_key` (default the index) on VirtualList, TableRows, VirtualGrid/VirtualWrapGrid; `check-render-keys.sh`; across-sort specs. Parallel with row 362.

## Status / trigger

**LANDED 2026-10-22 (batch 8, PR #135).** BACKLOG (owner asked 2026-10-22 to deconstruct the collection model to its roots; design note [`research/collection-model-2026-10-22.md`](../../research/collection-model-2026-10-22.md)). Order: 360, then 361 and 362 in parallel, then 363, then 341, then 342.

## Size

M.

## Detail

See the design note's "The model", "Migration path" and "Backlog impact" sections.

## Landed (2026-10-22, lane identity)

- `primitives/src/virtual/identity.rs`: `render_key(item_key, idx)` (the one place a rendered key is made), `key_hash`, `engine_keys`, `engine_row_keys`, `resolve_index` (where a remembered item is now).
- Engine: `SizeEstimates::with_keys`, `SizeIndex` stores measured sizes by item key (index without keys, so default callers are unchanged); unit tests for reorder, leave-and-return and a reversed engine.
- `item_key` on `VirtualList` (DOM keys, pinned focus row follows the item), `TableRows` (primitive and themed), `VirtualWrapGrid` (cells keyed by item, active item follows, rows keyed by the items they hold); `row_key` on `VirtualGrid` (row DOM key, active cell's row follows). `VirtualJustifiedGrid` has no `item_key` (its items are bare aspects); its keys carry a `key-ok` reason.
- `scripts/check-render-keys.sh` (registered in `run-gates.sh`, named in CLAUDE.md): red against main's three renderer files (14 positional keys), green here.
- Docs: shared `identity` block in the list and grid docs (parity gate green), `item_key` bullet in the table docs. Demos: `virtual_list/variants/reorder`, `table/variants/reorder` (each with an `item_key` on/off toggle). Spec: `playwright/item-key.spec.ts`.
- Known limit: a browser drops focus from a node the diff has to move, so focus survives a reorder only while the focused node itself stays in place (one row moving, or the pinned row).
- Browser spec run 2026-10-22 (it had never run): 3 reds. (1) `TableRowsWindowed` put the keyed `tr` inside a `for` body written as `{ let idx = ..; rsx! { tr { key } } }`; the block wraps the `tr` in a keyless node, so Dioxus diffed rows by position and remounted every row on a change of order (focus lost, `check-render-keys` cannot see it). Fixed: the `tr` is the loop body's own root. (2) The table "full reverse" spec checked item 5, which a reverse sends to row 35, outside the 40-row window (an unmounted uncontrolled checkbox resets by design): it now checks item 19 (row 20, stays in the window). (3) The canvas spec's warm-up scan read `scrollHeight` once, before rows were measured, so it covered only two thirds of the list and the unscanned rows were measured during the reverse (29163 -> 30127): it re-reads the end on every step. `item-key.spec.ts` 6/6 green, 3 runs in a row.

- Follow-up 2026-10-23 (lane demofix2, owner feedback): the Virtual List `reorder` demo always sets `item_key`
  (no switch) and fills the demo frame's width (`.dx-virtual-list-demo[data-vl-reorder-demo] { width: 100% }`:
  the preview frame centres its child, so a wrapper without a width shrank to its longest label). What
  `item_key` changes is explained in the shared `identity` block of the Virtual List and Virtual Grid docs. The
  key-off proof of `playwright/item-key.spec.ts` moved to the engine: `without_an_item_key_a_row_is_its_slot`
  (`primitives/src/virtual/identity.rs`) and `without_keys_a_measured_size_stays_on_its_slot`
  (`virtualizer.rs`), each with a key-on twin; the Table demo keeps its toggle and its key-off spec.
