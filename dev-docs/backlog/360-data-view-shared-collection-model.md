---
id: 360
title: "Shared collection model: data_view in primitives (identity, ViewState with manual order, fields, index-returning derive) and the selection count fix"
status: "BACKLOG (owner asked 2026-10-22 to deconstruct the collection model to its roots; design note…"
size: "M."
area: primitives
updated: 2026-10-22
---

# Row 360: Shared collection model: data_view in primitives (identity, ViewState with manual order, fields, index-returning derive) and the selection count fix

## Item

Migration steps 1 and 2 of the note: lift row 340's `TableState`/`derive` into `primitives/src/data_view/` with no behaviour change (data_table re-exports, current names keep compiling), one `contains_query` for the two substring filters, memoised `derive` returning indices; then fix the footer count (`selected_in_view of filtered_count`, shadcn parity; today it can read "3 of 1"). Law tests and `check-collection-derive.sh`.

## Status / trigger

BACKLOG (owner asked 2026-10-22 to deconstruct the collection model to its roots; design note [`research/collection-model-2026-10-22.md`](../research/collection-model-2026-10-22.md)). Order: 360, then 361 and 362 in parallel, then 363, then 341, then 342.

## Size

M.

## Detail

See the design note's "The model", "Migration path" and "Backlog impact" sections.

## Landed (migration steps 1 and 2)

- `primitives/src/data_view/` (std only, no cargo feature: `check-primitives-features` and `check-cargo-dep-versions` need nothing new): `ViewState` (= row 340's `TableState` plus `order: Option<KeyOrder>`, unused by any UI), `KeyOrder::reconcile`, `Field<T>`, `derive` (indices + keys + counts, never rows), `derive_by` (closure form), `derive_rows` and `TableView<T>` (the compatibility wrapper, clones only the page), the stage helpers moved from preview's `state.rs`, and the one `contains_query`.
- Data Table: `model.rs`/`state.rs` re-export every old name (`derive` = `derive_rows`); `Column<T>` holds a `Field<T>` (the `id` field stays); `use_data_table` memoises `derive` and no longer clones every row three times. `includes_string` and `default_combobox_filter` call `contains_query`.
- Count fix: `View`/`TableView` carry `selected_in_view` and `selected_total`; the footer reads `selected_in_view of filtered_count`; `selected_count` is a deprecated alias of `selected_total`. Spec: `data_table.spec.ts` "the selected count only counts rows in the filtered view".
- Law tests (`data_view/laws.rs`, fixed-seed LCG, 40 seeds x 200 steps): keys are a duplicate-free permutation of the filtered keys, the page is the clamped window of them, `selected_in_view <= filtered_count`, `reconcile` is idempotent and a permutation.
- Gate `check-collection-derive.sh` (in `run-gates.sh` and the CLAUDE.md gate sentence): no `.to_lowercase().contains(`, `HashSet<usize>`/`Vec<bool>`/`[bool; N]` selection or ad-hoc `sort_by` in `preview/src/components/**`; allowlist for ToggleGroup's upstream API (and its three consumers) and the catalogue sort.
