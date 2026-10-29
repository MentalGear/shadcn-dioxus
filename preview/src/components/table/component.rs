//! Themed `Table` -- shadcn/ui catalog parity
//! (dev-docs/archive/rounds/component-backlog.md row 68).
//!
//! Unlike almost every other component in this crate, there is no
//! `dioxus_primitives::table` module to wrap: shadcn/ui's own `table.tsx`
//! is plain, unstyled semantic HTML (`<table>`/`<thead>`/`<tbody>`/
//! `<tfoot>`/`<tr>`/`<th>`/`<td>`/`<caption>`) with no Radix primitive and
//! no ARIA widget role underneath, so there is no ARIA engineering to do
//! (dev-docs/archive/rounds/component-backlog.md row 68). So this file
//! *is* both the "primitive" and the theme layer: every element below is
//! the plain HTML tag plus this crate's `dx-table-*` class, nothing more.
//!
//! Every component here merges its own `dx-table-*` base class with any
//! caller-supplied `class` via `merge_attributes`, the same construction
//! `Input` uses (backlog row 12): a literal
//! `class: "dx-foo"` field followed by a plain `..attributes` spread lets
//! Dioxus's "later duplicate attribute wins" rule silently *replace* the
//! base class instead of combining with it. `merge_attributes` special-
//! cases `class` to concatenate (space-joined) instead, so a caller's own
//! class (e.g. `DataTableColumnHeader`'s `"dx-data-table-column-header"`,
//! composed on top of `TableHead` here) always ends up alongside
//! `"dx-table-head"`, never in place of it.
//!
//! `TableHead`'s `scope: "col"` default (APG's Table pattern:
//! "columnheader ... if the cell contains a title or header information
//! for the column") is written as a literal attribute placed *before* the
//! `..merged` spread in each `th { ... }` body, not folded into the merged
//! `attributes!` base -- the same "later wins" rule then lets a caller's
//! own `scope` (e.g. a row-header `<th>` wanting `scope="row"`) override
//! the literal default, while callers who don't set one keep `"col"`.

use dioxus::prelude::*;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;
use dioxus_primitives::state_attrs;
pub use dioxus_primitives::table_rows::{VirtualizeMode, DEFAULT_AUTO_THRESHOLD};
use dioxus_primitives::table_rows::{use_table_scroller, TableRows as TableRowsPrimitive};

/// Props for [`Table`].
#[derive(Props, Clone, PartialEq)]
pub struct TableProps {
    /// Attributes for the scrolling container `<div>` (`dx-table-container`): give it a
    /// `max-height` (and the container scrolls vertically) so a windowed [`TableRows`] has a
    /// viewport and a sticky [`TableHeader`] has something to stick to.
    #[props(default)]
    pub container_attributes: Vec<Attribute>,

    #[props(extends = GlobalAttributes)]
    #[props(extends = table)]
    pub attributes: Vec<Attribute>,

    /// The table's sections: caption, header, body, footer.
    pub children: Element,
}

/// Wraps a `<table>` in a horizontally-scrollable container, matching
/// shadcn/ui's own `Table` (`div.overflow-x-auto` + `table`). The
/// container needs no `dx-table` class of its own beyond `dx-table-
/// container` -- only the `<table>` element carries `dx-table`. The
/// container is also the element a windowed [`TableRows`] scrolls in
/// (it owns an `id` for that) and the table carries the `aria-rowcount`
/// such a body announces.
#[component]
pub fn Table(props: TableProps) -> Element {
    let scroller = use_table_scroller();
    let base = attributes!(table { class: "dx-table", "data-dx-root": "", "data-slot": "table" });
    let row_count: Option<usize> = *scroller.row_count.read();
    let owned = attributes!(table { "aria-rowcount": row_count.map(|n| n.to_string()) });
    let merged = merge_attributes(vec![base, props.attributes, owned]);
    let container = attributes!(div {
        class: "dx-table-container",
        "data-slot": "table-container",
    });
    let container_owned = attributes!(div { id: scroller.id });
    let container = merge_attributes(vec![container, props.container_attributes, container_owned]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/table/style.css") }
        div { ..container,
            table { ..merged, {props.children} }
        }
    }
}

/// Props for [`TableHeader`].
#[derive(Props, Clone, PartialEq)]
pub struct TableHeaderProps {
    /// Keeps the header cells at the top of the table's scroll container while the body scrolls
    /// under them (`position: sticky`). Needs a container with a `max-height`
    /// ([`TableProps::container_attributes`]). Defaults to `false`.
    #[props(default)]
    pub sticky: bool,

    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The header rows.
    pub children: Element,
}

/// `<thead>`.
#[component]
pub fn TableHeader(props: TableHeaderProps) -> Element {
    let base = attributes!(thead { class: "dx-table-header", "data-slot": "table-header" });
    let owned = attributes!(thead { "data-sticky": props.sticky.then_some("true") });
    let merged = merge_attributes(vec![base, props.attributes, owned]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/table/style.css") }
        thead { ..merged, {props.children} }
    }
}

/// `<tbody>`.
#[component]
pub fn TableBody(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    /// The body rows.
    children: Element,
) -> Element {
    let base = attributes!(tbody { class: "dx-table-body", "data-slot": "table-body" });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/table/style.css") }
        tbody { ..merged, {children} }
    }
}

/// `<tfoot>`.
#[component]
pub fn TableFooter(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    /// The footer rows.
    children: Element,
) -> Element {
    let base = attributes!(tfoot { class: "dx-table-footer", "data-slot": "table-footer" });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/table/style.css") }
        tfoot { ..merged, {children} }
    }
}

/// Props for [`TableRow`].
#[derive(Props, Clone, PartialEq)]
pub struct TableRowProps {
    /// Whether this row is selected (e.g. by a leading checkbox column in
    /// the Data Table composition pattern). Renders `data-state="selected"`
    /// when `true`, matching shadcn/ui's own `table.tsx`
    /// (`data-state={row.getIsSelected() && "selected"}`); the attribute is
    /// omitted entirely rather than set to some "unselected" value when
    /// `false`, mirroring that same `&&` short-circuit.
    #[props(default)]
    pub selected: ReadSignal<bool>,

    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The row's cells.
    pub children: Element,
}

/// `<tr>`.
#[component]
pub fn TableRow(props: TableRowProps) -> Element {
    let base = attributes!(tr { class: "dx-table-row", "data-slot": "table-row" });
    // `data-state` is state the wrapper computes from `props.selected`, not a
    // caller-overridable default, so it is merged in LAST (owned wins) rather
    // than folded into `base` -- see the merge-precedence policy in
    // dev-docs/issues/duplicate-attribute-findings.md.
    let owned = attributes!(tr { ..state_attrs::selected((props.selected)()) });
    let merged = merge_attributes(vec![base, props.attributes, owned]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/table/style.css") }
        tr { ..merged, {props.children} }
    }
}

/// `<th>`. Defaults to `scope="col"` (this crate's `TableHead` is always
/// used as a column header in every demo/consumer -- a row-header
/// `<th scope="row">` remains reachable by passing `scope: "row"`
/// explicitly, which overrides the default; see this file's header doc for
/// the override mechanism).
#[component]
pub fn TableHead(
    #[props(extends = GlobalAttributes)]
    #[props(extends = th)]
    attributes: Vec<Attribute>,
    /// The header cell's content.
    children: Element,
) -> Element {
    let base = attributes!(th { class: "dx-table-head", "data-slot": "table-head", scope: "col" });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/table/style.css") }
        th { ..merged, {children} }
    }
}

/// `<td>`.
#[component]
pub fn TableCell(
    #[props(extends = GlobalAttributes)]
    #[props(extends = td)]
    attributes: Vec<Attribute>,
    /// The cell's content.
    children: Element,
) -> Element {
    let base = attributes!(td { class: "dx-table-cell", "data-slot": "table-cell" });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/table/style.css") }
        td { ..merged, {children} }
    }
}

/// `<caption>`.
#[component]
pub fn TableCaption(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    /// The caption text.
    children: Element,
) -> Element {
    let base = attributes!(caption { class: "dx-table-caption", "data-slot": "table-caption" });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/table/style.css") }
        caption { ..merged, {children} }
    }
}

/// Props for [`TableRows`].
#[derive(Props, Clone, PartialEq)]
pub struct TableRowsProps {
    /// The number of data rows.
    pub count: ReadSignal<usize>,
    /// The number of columns, which the spacer rows of a windowed body span.
    pub columns: usize,
    /// Renders the cells (`TableCell`s) of row `idx`, an absolute index; the body owns the
    /// `<tr>`, so it can set `aria-rowindex` and measure the row.
    pub render_row: Callback<usize, Element>,
    /// Who is at row `idx`: a string unique in the table that stays with the row through a sort,
    /// filter or reorder. Defaults to the index (a row is its slot). With it each `<tr>` is keyed
    /// by its item, so focus and state inside a row move with it and the measured row heights
    /// follow the item.
    pub item_key: Option<Callback<usize, String>>,
    /// Whether row `idx` is selected: sets `data-state="selected"` like [`TableRow`]'s
    /// `selected`. Read while rendering, so a selection a row reads re-renders the body.
    pub row_selected: Option<Callback<usize, bool>>,
    /// Extra attributes for the `<tr>` of row `idx`.
    pub row_attributes: Option<Callback<usize, Vec<Attribute>>>,
    /// How many rows precede the first data row (the header). Defaults to 1.
    #[props(default = 1)]
    pub row_offset: usize,
    /// How many rows to mount beyond each edge of the viewport when windowed. Defaults to 8.
    #[props(default = ReadSignal::new(Signal::new(8)))]
    pub buffer: ReadSignal<usize>,
    /// The height of row `idx` in px until it is measured. Defaults to 37 for every row.
    pub estimate_size: Option<Callback<usize, u32>>,
    /// How rows are kept cheap: every row in the DOM, or a window of them between spacer rows.
    /// `Auto` (the default) picks by `count`. A table cannot skip off-screen rows the way
    /// `VirtualList` does, so `ContentVisibility` here is a plain table.
    #[props(default = ReadSignal::new(Signal::new(VirtualizeMode::Auto)))]
    pub virtualize: ReadSignal<VirtualizeMode>,
    /// With `Auto`: the largest `count` that keeps every row in the DOM. Defaults to 1000.
    #[props(default = ReadSignal::new(Signal::new(DEFAULT_AUTO_THRESHOLD)))]
    pub auto_threshold: ReadSignal<usize>,

    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// The data-driven `<tbody>`: `count` rows from `render_row`, all in the DOM or windowed over
/// two spacer rows (see [`TableRowsProps::virtualize`]). Use [`TableBody`] for hand-written rows.
/// Inside a [`Table`] with a `max-height` and a sticky [`TableHeader`] it is a long table that
/// scrolls in place.
#[component]
pub fn TableRows(props: TableRowsProps) -> Element {
    let base = attributes!(tbody { class: "dx-table-body", "data-slot": "table-body" });
    let merged = merge_attributes(vec![base, props.attributes]);
    let (row_selected, row_attributes) = (props.row_selected, props.row_attributes);
    let row_attributes = Callback::new(move |idx: usize| {
        let base = attributes!(tr { class: "dx-table-row", "data-slot": "table-row" });
        let selected = row_selected.is_some_and(|f| f(idx));
        let state = attributes!(tr { ..state_attrs::selected(selected) });
        let theirs = row_attributes.map(|f| f(idx)).unwrap_or_default();
        merge_attributes(vec![base, theirs, state])
    });

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/table/style.css") }
        TableRowsPrimitive {
            count: props.count,
            columns: props.columns,
            render_row: props.render_row,
            item_key: props.item_key,
            row_attributes,
            row_offset: props.row_offset,
            buffer: props.buffer,
            estimate_size: props.estimate_size,
            virtualize: props.virtualize,
            auto_threshold: props.auto_threshold,
            attributes: merged,
        }
    }
}
