# One collection model for every view: deconstruction and recommendation

Evaluated on `queue/batch-6` at `31986d1b` (read only). Context: backlog rows 340 (done), 341, 342, 343 (done).

## Summary

- Adopt one pure, render-free **view model** in `dioxus-primitives` (working name `data_view`): row identity (a key function), one flat `ViewState` (sort, filter, page, selection, and a new manual order) with pure transitions, and one `derive` that returns **indices and keys**, never cloned rows. Row 340's `TableState`/`derive` move there; `TableState`, `TableView`, `use_data_table` and `Column<T>` stay as compatible names.
- Tom's idea is right **at the model layer** (a list is the same model with one field, there is no `ListState`) and wrong **at the renderer layer** (a list rendered as a one-column `<table>` loses list/listbox semantics, the sortable-list keyboard model, free-form item layout and, measured in row 343, `content-visibility`, which does not apply to table rows).
- The better framing of the same intuition: **a table is a list of rows crossed with a list of columns.** The shared unit is the *ordered keyed set* (`KeyOrder`: a permutation of current keys plus move transitions). Rows use it for manual order, columns use it for column order, and both are reordered through one extracted, keyboard-accessible reorder gesture (today locked inside `drag_and_drop_list.rs`).
- The second root cause, independent of the model: every windowed renderer identifies items by **position** (DOM keys, pinned focus, active cell, measurement cache). Renderers gain an optional `item_key` so focus, measurement and DOM nodes follow the item through a sort, filter or move.
- Children-registered collections (Select, Combobox, Command, Tag Group, menus, Tabs, Toggle Group) are **not** moved onto the data model; they keep `collection.rs` (roving focus, already one construction) and `selectable.rs`. They share only the one text matcher.
- Scope stays at shadcn parity: sort, one filter, paging, row selection, column visibility (shadcn's DataTable) plus row drag handles (shadcn's `dashboard-01` block, to my knowledge dnd-kit drag handles and a "Customize Columns" menu). No multi-sort, grouping, pinning, resizing, server adapters.

## Inventory

Two families of collections exist, and the distinction matters for everything below.

**A. Data-driven** (`count` + `render(index)`, or `rows: Vec<T>`):

| Place | State it holds or derives | Identity |
|---|---|---|
| `preview/src/components/data_table/model.rs` (row 340) | `TableState` sort, filter, page, page_size, selected `HashSet<String>`; `derive` filter, sort, paginate, tri-state | key fn `row_key`, good |
| `data_table/state.rs` | `sort_rows(_by)`, `filter_rows`, `includes_string`, `paginate`, `tri_state` | n/a |
| `data_table/columns.rs` | `Column<T>` (id, header, cell, typed comparator, filter text, width, align), `use_data_table` | column id string |
| `table/variants/rich/mod.rs:46,72` | selection `[bool; 4]` indexed by row position | position |
| `primitives/src/table_rows.rs`, themed `table/component.rs:246` | windowing; `row_selected: Callback<usize, bool>` | position (`key: "{idx}"`, `:201`) |
| `primitives/src/virtual_list.rs` | windowing, CV chunks, pinned focused row `:317` | position (`key: "{idx}"`, `:271`, `:485`) |
| `primitives/src/virtual_grid.rs` | active cell `(row, col)` `:198`, wrap grid active index `:463`; row 341 adds sort/selection hooks | position |
| `primitives/src/virtual/virtualizer.rs:154`, `types.rs:4` | measurement cache keyed by `Key = usize` (the index) | position |
| `primitives/src/drag_and_drop_list.rs` | items `Vec<Element>` copied into a signal once `:439`; `DragState` gesture; keyboard lift/move/drop; announcements; remove; focus by key via `collection.rs` | vnode key, else index (`:24`); duplicated in themed `component.rs:71-78` |
| `preview/src/dashboard/views/email_client/state.rs:89,116` | app-level: `message_order` keys + filters derive visible keys; selection by key with fallback | key, good |

**B. Children-registered** (items declare themselves via context):

| Place | State | Identity |
|---|---|---|
| `primitives/src/collection.rs` | roving focus, disabled/hidden, text for typeahead; used by 20 components | optional key, else index |
| `selectable.rs`, `selection.rs`, `listbox.rs` | Select/Combobox/Listbox values `Vec<RcPartialEqValue>`, `SelectionMode` | value |
| `command.rs:119-136`, `combobox/context.rs:7,122` | query + filter predicate over registered option text | value |
| `tag_group.rs:17-28,223-320` | value selection, removal that prunes selection | value |
| `toggle_group.rs:20` | pressed `HashSet<usize>` (public API) | position |
| `typeahead.rs`, `select/text_search.rs` | two typeahead matchers, deliberately separate (documented `typeahead.rs:19-30`) | n/a |
| `carousel.rs` | slide index; not a collection view | n/a |

No component loads collection data asynchronously; nothing to unify there.

**Duplications (classes):** substring filter twice (`includes_string` `state.rs:125`, `default_combobox_filter` `combobox/context.rs:7`, same trim+lowercase+contains); selection five ways (`TableState.selected`, `[bool; N]`, `HashSet<usize>`, `Vec<RcPartialEqValue>`, Tag Group's own); position identity in four renderers plus the engine cache; DnD key derivation twice; "derive visible keys from order + filters" twice (DataTable, email client).

## Root causes per concern

**Identity.** Invariant: everything that must survive a change of the view (selection, focus, measured size, DOM node, drag source) is keyed by item identity, never by position. DataTable and the email client honour it; the renderers do not. `VirtualList` keys rows `"{idx}"` and pins focus as `Option<usize>`, `VirtualGrid` keeps the active cell as indices, the engine caches measured heights by index, `TableRows` keys `<tr>` by index. Today that is harmless because nobody sorts under them; the moment row 342 puts a sortable model on VirtualGrid or a list is reordered, focus stays on a slot while the item moves, and a measured 130 px message height is applied to a 34 px day header (the email client's filter already changes which kind sits at an index). Root: the renderer contract is `count + render(index)` with no way to say who is at an index.

**Sort.** Invariant: one stable comparator, typed (`K: Ord`), direction applied once, `aria-sort` emitted by the component. Row 340 already holds this (`columns.rs` `.sortable`, `state.rs` stable `sort_by`). Not broken, only located in the copied component directory where VirtualList and the DnD list cannot reach it.

**Filter.** Invariant: one matching rule for "the text contains the query". Broken by duplication across the two families. Root: the matcher lives in two component-specific modules.

**Paging.** Invariant: sort, filter and page-size changes return to page 0; the page is clamped. Held once in `TableState`. Paging is a model concern (user-visible state with reset rules); virtual windowing is a renderer concern (scroll position). They must stay separate concepts even though both "show a slice".

**Selection.** Invariants: keyed by identity; counts and the header tri-state are computed over the current view, never stored; what "all" means is explicit. Broken in three ways: (1) `model.rs:144` reports `selected_count: state.selected.len()`, which includes rows hidden by the filter and rows deleted from the data, while `component.rs:410-411` prints it as "N of `filtered_count`", so the footer can read "3 of 1 row(s) selected" (shadcn counts selected rows *within the filtered model*); (2) the Table `rich` demo and `ToggleGroup` key selection by position; (3) five representations. Root: selection is a stored set with no derived "selected in view" stage, and no shared selection type.

**Order / reorder.** Invariant: the manual order is a permutation of the current keys; a move is expressed against keys, not slots; with an active sort a manual move is not meaningful. Broken: `DragAndDropList` stores the *rendered elements* as its order (`:439`), reads `items` only on the first render, and exposes no reorder callback (no `on_reorder`/`EventHandler` anywhere in the file), so an app cannot learn or persist the order; its identity falls back to the index when a child has no key. Root: order is held as view output (elements) instead of model state (keys), so it cannot be shared, controlled or persisted. The gesture (pointer drag, Space lift, arrows, Space/Enter drop, Escape cancel, live announcements) is good and reusable but is fused to that storage. It also still emits `aria-grabbed` (`:774`), deprecated since ARIA 1.1.

**Focus / roving.** Invariant: one tab stop, focus by key. Already one construction for family B (`collection.rs`). Family A renderers each have their own (pinned row, active cell); correct as renderer-owned, wrong only in being positional (see Identity).

**Windowing.** Invariant: one mode type and threshold (row 343 landed `VirtualizeMode`). Fine; renderer-owned.

**Derived views stored.** Invariant: derived data is never stored and is cheap to recompute. Mostly held, but `derive` clones `T` three times per render (`filter_rows` `model.rs:129`, `sort_rows_by` `:134`, `paginate` `:137`) and is not memoised. At 50,000 rows (row 342) that is three full clones on every render. Root: the view is "rows" rather than "indices into rows".

## The model

Seven orthogonal pieces, each owning exactly one concern.

1. **Identity** (primitives, `data_view`): the model is constructed with `key: Fn(&T) -> String`. Every later piece speaks keys. `String` keeps row 340's API; a generic key can come later without changing the shape.
2. **`KeyOrder`** (primitives): a `Vec<String>` with `reconcile(current_keys)` (drop missing, append new in source order, idempotent) and `move_key(key, to)`, `move_by(key, delta)`, `move_before(key, anchor)`. Reconciliation runs inside `derive`, so the permutation invariant holds by construction, not by caller discipline. Used twice: row manual order and table column order.
3. **`ViewState`** (primitives): flat, `Clone + PartialEq`, fields `sort`, `filter`, `page`, `page_size`, `selected: HashSet<String>`, `order: Option<KeyOrder>`. `TableState` becomes a type alias, so row 340's pub fields and transitions keep compiling (adding the `order` field only breaks struct-literal construction, which no caller uses; `TableState::new` is the documented path). Transitions per concern stay pure methods: sort/filter/page as today; `move_*` refused (returns `false`) while a sort is active, plus `freeze_sort()` that writes the sorted order into `order` and clears the sort (the "drag a sorted list" escape hatch); a move while filtered or paged lands before the next *visible* neighbour's key, so hidden rows keep their relative place.
4. **`Field<T>`** (primitives): `id`, optional typed comparator, optional filter text. `Column<T>` (component layer) = `Field<T>` + header + cell + width + align; its builder (`.sortable`, `.filterable`) forwards to the field, so row 340 callers compile unchanged. A list passes fields directly (often one, or none: a list that only reorders needs no field).
5. **`derive`** (primitives, one pipeline): `filter -> sort or manual order -> page`, producing `View { indices: Vec<usize>, keys, filtered_count, page_count, page, selected_in_view, selected_total, select_all }`. Counts are computed, never stored; `selected_in_view` intersects the selection with filtered keys (the shadcn count). `TableView<T>` remains as a thin compatibility wrapper that clones only the page rows. The hook wraps `derive` in `use_memo`.
6. **View contract for renderers** (primitives): renderers do not take the model. They take `count`, `render(index)` and a new optional `item_key: Callback<usize, String>` (default: the index, so every current call site compiles). With a key, DOM keys, pinned focus, the active row, the measurement cache (engine `Key` becomes a key hash, sizes kept in a key map) and drag source all follow the item. `row_selected` stays `Callback<usize, bool>` over *view* positions; the model maps index to key, so selection is still keyed where it is stored.
7. **Reorder gesture** (primitives, extracted from `drag_and_drop_list.rs`): a renderer-agnostic state machine (idle, lifted from key, target slot) with pointer and keyboard input and the existing announcements, emitting `on_move(key, to)`. Consumers: DnD list items, a DataTable row drag-handle cell, and the column list inside the "Columns" menu. `aria-grabbed` is dropped; instructions via `aria-describedby` and the assertive live region stay.

**Renderer-owned, deliberately outside the model:** ARIA roles and keyboard model (list, listbox, sortable list, table, grid), roving focus and the active cell, windowing and `VirtualizeMode`, drag visuals, `aria-sort`/`aria-selected`/`data-state` emission, grouping rows such as the email client's day headers.

**Table-only state:** `ColumnLayout { order: KeyOrder, hidden: HashSet<FieldId> }`. Visibility is shadcn parity (DataTable "Columns" dropdown with checkbox items); order is the same `KeyOrder`, reordered keyboard-first in that menu through the gesture (Space lift, arrows, Space drop), with header dragging as a pointer-only extra. No APG pattern exists for column reorder, so the accessible path is the list, not the header.

**Primitives vs component layer.** Pieces 1 to 7 are pure or behaviour-level and go into `dioxus-primitives` without a cargo feature (std only), next to `collection.rs` and `virtual/`, so `dx components add` of DataTable, VirtualList or the DnD list needs no new wiring (the primitives crate is already in their closure; `check-installed-paths` and `check-cargo-dep-versions` keep that honest). `Column<T>`, `use_data_table`, `DataTable<T>`, the toolbar, pagination and "Columns" menu stay themed in `data_table/`.

**Subsumed:** `TableState`/`derive` (moved), `state.rs` helpers (kept as the stage functions), the Table `rich` demo selection (renders from the model), DnD order storage and both key derivations, the stale selection count, the three row clones, positional identity in VirtualList, TableRows, VirtualGrid and the engine cache, and the substring filter duplicate (one `contains_query` in primitives; `includes_string` and `default_combobox_filter` become re-exports). **Not subsumed:** family B collections (correctly compositional, matching Radix/cmdk); `ToggleGroup`'s `HashSet<usize>` (public upstream API, a separate parity row if wanted); the two typeahead matchers (settled decision); Carousel; the email client (an app; it may adopt the model but need not).

## Evaluation of the alternatives

**Tom's "a list is a 1-column (data)table, re-orderable like a datatable column".**
- *Model:* correct, and adopted: there is one `ViewState`; a list is the model with zero or one field.
- *Semantics and ARIA:* a `<table>` announces "table, 1 column, N rows", requires a column header to make sense, and gives `row`/`cell` roles to what are list items or listbox options. A listbox needs `aria-selected` options with selection-follows-focus and typeahead; a sortable list needs `aria-roledescription` and the lift/drop model. None of these exist on table rows without fighting the role.
- *Keyboard:* table (no widget, Tab through controls) and grid (2-D arrows, one tab stop) differ from list (Tab per item) and listbox (1-D arrows, Home/End, typeahead). One renderer cannot carry all four without a mode switch that changes behaviour, which row 340's amendment already ruled out for size-triggered changes.
- *Layout and cost:* list items are free-form (the DnD demo's task cards with avatars); table rows constrain layout, animate poorly under `transform` while dragging, and **cannot use `content-visibility`** (row 343, measured), so a list rendered as a table would lose VirtualList's ContentVisibility mode and its cheap find-in-page.
- *Ergonomics:* a list caller would have to write `Column::new("item", "", ...)` to render one thing. Worse than `render_item`.
- *What it gets right that the proposal missed:* "re-orderable like a column". Column order and row order are the same structure, so the reorder machinery should be one piece, which is piece 2 plus piece 7.

**Status quo plus copy-paste** (each renderer grows its own sort/selection, as row 341 is filed): repeats the five-way selection class and the positional identity bug on VirtualGrid. Rejected.

**A TanStack-style table framework** (row models, plugins, multi-sort, grouping, pinning): outside the owner scope rule and larger than shadcn's own example needs. Rejected; `ViewState` is plain data, so an app with server data reads it and fetches.

**Move family B onto the data model too** (Select from `items: Vec<T>`): breaks shadcn/Radix composition parity and the 20-component `collection.rs` construction that already works. Rejected; share only the matcher.

## Migration path

Each step compiles every current caller and passes the current specs before the next starts.

1. **Lift without behaviour change.** Create `primitives/src/data_view/` with `ViewState` (= today's `TableState` plus `order: None`), `Field<T>`, `derive` returning indices plus the `TableView<T>` wrapper, the stage helpers from `state.rs` and one `contains_query`. `data_table/model.rs` and `state.rs` become re-exports (`pub use dioxus_primitives::data_view::{TableState, ...}`), `includes_string`/`default_combobox_filter` call `contains_query`. `use_data_table` memoises `derive`. Existing 34 unit tests move with the code and stay green.
2. **Selection count fix.** `selected_in_view` plus `selected_total`; the footer reads `selected_in_view of filtered_count` (shadcn parity). `selected_count` stays as a deprecated alias of `selected_total`. One `data_table.spec.ts` case: select, filter it out, count reads "0 of 1".
3. **Renderer identity.** `item_key` on `VirtualList`, `TableRows` (themed and primitive), `VirtualGrid`/`VirtualWrapGrid` (`row_key`), engine sizes keyed by key. Default stays the index.
4. **`KeyOrder` and the reorder gesture.** Extract the gesture; `DragAndDropList` gains a controlled mode (`order: ReadSignal<Vec<String>>`, `on_reorder: Callback<Vec<String>>`, or `on_move(key, to)`), while `items: Vec<Element>` stays as the uncontrolled mode implemented on `KeyOrder`. Drop `aria-grabbed`; one key derivation.
5. **DataTable reorder and columns.** `reorderable` adds a drag-handle column (`dashboard-01` parity) driving `ViewState.order`; `ColumnLayout` with the "Columns" menu (visibility checkboxes, keyboard move). Table `rich` demo renders from the model.
6. **Rows 341 and 342** on top: VirtualGrid hooks take `row_key`; `layout: Grid` renders the same `View`.

**Gates and tests that make the class unable to recur:**
- `data_view` law tests (deterministic pseudo-random sequences, no new dependency): after any mix of data edits and transitions, `view.keys` is a duplicate-free permutation of the filtered keys, `reconcile` is idempotent, `move` then inverse move is identity, `selected_in_view <= filtered_count`, page always clamped.
- New `check-render-keys.sh`: in `primitives/src` renderers, a `key: "{idx}"`-style positional key is red unless it goes through the one `render_key(item_key, idx)` helper; reasoned allowlist, stale entries fail.
- New `check-collection-derive.sh`: no `.to_lowercase().contains(`, no `HashSet<usize>`/`[bool; N]` selection and no `sort_by` derivation in `preview/src/components/**` outside `data_view` re-exports; reasoned allowlist (ToggleGroup) and stale entries fail.
- Specs: focus and selection stay on the same item across a sort in `VirtualList` and `TableRows`; the DnD `on_reorder` order equals DOM order after keyboard and pointer moves; a column moved in the "Columns" menu by keyboard moves header and cells together; random-heights list reversed keeps measured sizes (no jump).

## Backlog impact

- **342:** shrinks back to its original S scope (VirtualGrid layout, large-data demo, shared table look). The collection-model amendment moves to new row A.
- **341:** hooks keyed by row identity: add `row_key` to VirtualGrid; `row_selected` stays over view positions; the cell-control work is unchanged. Depends on row B.
- **340 / 343 (done):** no reopen; compatibility aliases land in row A, `TableRows.item_key` in row B.
- **New row A (M):** `data_view` in primitives: identity, `ViewState` (+ `KeyOrder` order), `Field<T>`, index-returning memoised `derive`, `contains_query`, the selection count fix, law tests, `check-collection-derive.sh`.
- **New row B (M):** renderer identity: `item_key` on VirtualList, TableRows, VirtualGrid, engine sizes by key, `check-render-keys.sh`, the across-sort specs.
- **New row C (M):** reorder gesture extracted; DragAndDropList controlled mode with `on_reorder`; drop `aria-grabbed`; one key derivation.
- **New row D (M):** DataTable `reorderable` rows (drag handle) and `ColumnLayout` (visibility, shadcn parity, plus keyboard column order) with the "Columns" menu.
- **Optional row E (S):** `ToggleGroup` value-keyed pressed state (shadcn parity), deprecating the index set.
- Order: A, then B and C in parallel (disjoint files), then D, then 341, then 342.
