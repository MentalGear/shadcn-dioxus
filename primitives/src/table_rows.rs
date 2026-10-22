//! Defines [`TableRows`], the data-driven body of a `<table>` (backlog row 343): a `<tbody>`
//! whose rows come from a row count and a renderer, so a table of tens of thousands of rows can
//! mount only the ones in view while staying a real `<table>`.
//!
//! The mode is the shared [`VirtualizeMode`] of the virtual engine (`virtualize` and
//! `auto_threshold`, the same props as `VirtualList`):
//!
//! - **`ContentVisibility`**: every row is in the DOM and in the server's HTML. Unlike
//!   `VirtualList` there is no skipping: browsers do not apply `content-visibility` to table
//!   rows or row groups (measured in Chromium: a `tbody` or `tr` with `content-visibility: auto`
//!   keeps its full height, as the specification excludes internal table elements from size
//!   containment), so this mode is a plain table. Find-in-page, selection, focus and the
//!   accessibility tree reach every row.
//! - **`Windowed`**: only the rows in view plus `buffer` rows each side are mounted, between a top
//!   and a bottom spacer `<tr>` (`aria-hidden`, one cell spanning `columns`) that give the scroll
//!   container the full height. Rows may have any height (they are measured). The scroll
//!   container is the element that [`TableScroller`] names (the styled `Table`'s container, which
//!   the caller gives a `max-height`); a `<thead>` made `position: sticky` stays in view above
//!   the window.
//! - **`Auto`** (the default): `ContentVisibility` while `count <= auto_threshold` (1000),
//!   `Windowed` above it.
//!
//! Every row is a real `<tr>` with `aria-rowindex` (1-based over the whole table, header rows
//! included: `row_offset` says how many rows precede the first data row, default 1), and a
//! windowed body also publishes `aria-rowcount` (data rows plus `row_offset`) to the `<table>`
//! through [`TableScroller`], so a screen reader announces `row 4,001 of 50,001` although only
//! the window is in the DOM.
//!
//! What a windowed table does not do: it does not pin the focused row (a row that holds focus is
//! unmounted when the window leaves it, unlike `VirtualList`: a spacer layout has nowhere to
//! draw a detached row), and its rows must not wrap onto different heights per window move
//! (they can differ from each other, but a row that resizes is re-measured).

use dioxus::prelude::*;
use dioxus_attributes::attributes;

use crate::r#virtual::{measured_px, use_virtual_bridge, Bridge, Engine, SizeEstimates};
pub use crate::r#virtual::{VirtualizeMode, DEFAULT_AUTO_THRESHOLD};
use crate::{merge_attributes, use_unique_id};

/// The height guess, in px, of a row that has not been measured: one 14px text line (20px) with the
/// styled table's `0.5rem` cell padding and a 1px border, measured in the demo.
pub const DEFAULT_ROW_ESTIMATE: u32 = 37;

/// What a `<table>` shares with the [`TableRows`] inside it: the id of the element that scrolls
/// (the table's container) and the total row count the table announces as `aria-rowcount`.
/// Provided by the table with [`use_table_scroller`], read by [`TableRows`].
#[derive(Clone, Copy, PartialEq)]
pub struct TableScroller {
    /// The id the table renders on the element that scrolls.
    pub id: Signal<String>,
    /// The total number of rows (header rows included) a windowed body publishes, for the
    /// table's `aria-rowcount`. `None` while no windowed body is mounted.
    pub row_count: Signal<Option<usize>>,
}

/// Creates the [`TableScroller`] of a table and provides it to its descendants. Render `id` on
/// the scrolling container and `row_count` as the `<table>`'s `aria-rowcount`.
pub fn use_table_scroller() -> TableScroller {
    let id = use_unique_id();
    let row_count = use_signal(|| None::<usize>);
    use_context_provider(|| TableScroller { id, row_count })
}

/// The props for the [`TableRows`] component.
#[derive(Props, Clone, PartialEq)]
pub struct TableRowsProps {
    /// The number of data rows.
    pub count: ReadSignal<usize>,
    /// The number of columns, which the windowed spacer rows span (one cell with `colspan`).
    pub columns: usize,
    /// Renders the cells of row `idx` (an absolute index); the body owns the `<tr>`.
    pub render_row: Callback<usize, Element>,
    /// Extra attributes for the `<tr>` of row `idx`: a class, `data-state`, `aria-selected`.
    /// The body's own `data-virtual-index` and `aria-rowindex` win over a same-named one.
    pub row_attributes: Option<Callback<usize, Vec<Attribute>>>,
    /// How many rows precede the first data row in the table (the header). `aria-rowindex` of
    /// row `idx` is `idx + 1 + row_offset`. Defaults to 1.
    #[props(default = 1)]
    pub row_offset: usize,
    /// How many rows to mount beyond each edge of the viewport when windowed (the window is
    /// re-centred once fewer than half remain). Defaults to 8.
    #[props(default = ReadSignal::new(Signal::new(8)))]
    pub buffer: ReadSignal<usize>,
    /// The height of row `idx` in px until it is measured (windowed mode). Defaults to
    /// [`DEFAULT_ROW_ESTIMATE`] for every row.
    pub estimate_size: Option<Callback<usize, u32>>,
    /// How rows are kept cheap. Defaults to [`VirtualizeMode::Auto`].
    #[props(default = ReadSignal::new(Signal::new(VirtualizeMode::Auto)))]
    pub virtualize: ReadSignal<VirtualizeMode>,
    /// With [`VirtualizeMode::Auto`]: the largest `count` that keeps every row in the DOM.
    /// Defaults to [`DEFAULT_AUTO_THRESHOLD`] (1000).
    #[props(default = ReadSignal::new(Signal::new(DEFAULT_AUTO_THRESHOLD)))]
    pub auto_threshold: ReadSignal<usize>,
    /// Additional attributes to apply to the `<tbody>`.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// The props the two mode bodies share.
#[derive(Props, Clone, PartialEq)]
struct BodyProps {
    count: ReadSignal<usize>,
    columns: usize,
    render_row: Callback<usize, Element>,
    row_attributes: Option<Callback<usize, Vec<Attribute>>>,
    row_offset: usize,
    buffer: ReadSignal<usize>,
    estimate_size: Option<Callback<usize, u32>>,
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
}

/// # TableRows
///
/// The data-driven `<tbody>` of a `<table>`: `count` rows rendered by `render_row`, in every
/// row in the DOM mode or windowed over spacer rows (see the module documentation). It must be
/// inside a table that called [`use_table_scroller`] to window; without one it renders every row.
#[component]
pub fn TableRows(props: TableRowsProps) -> Element {
    let TableRowsProps {
        count,
        columns,
        render_row,
        row_attributes,
        row_offset,
        buffer,
        estimate_size,
        virtualize,
        auto_threshold,
        attributes,
    } = props;

    // A separate component per mode, so crossing the threshold drops the windowed body's
    // listeners and engine with its scope; a memo, so only a change of mode re-renders here.
    let has_scroller = try_use_context::<TableScroller>().is_some();
    let windowed = use_memo(move || {
        has_scroller && virtualize().resolve(count(), auto_threshold()) == VirtualizeMode::Windowed
    });

    if windowed() {
        rsx! {
            TableRowsWindowed {
                count,
                columns,
                render_row,
                row_attributes,
                row_offset,
                buffer,
                estimate_size,
                attributes,
            }
        }
    } else {
        rsx! {
            TableRowsAll {
                count,
                columns,
                render_row,
                row_attributes,
                row_offset,
                buffer,
                estimate_size,
                attributes,
            }
        }
    }
}

/// The attributes of one data row: the caller's, then the body's own.
fn row_attrs(
    row_attributes: Option<Callback<usize, Vec<Attribute>>>,
    idx: usize,
    row_offset: usize,
) -> Vec<Attribute> {
    let theirs = row_attributes.map(|f| f(idx)).unwrap_or_default();
    let index = (idx + 1 + row_offset).to_string();
    let owned = attributes!(tr {
        "data-virtual-index": idx.to_string(),
        "aria-rowindex": index,
    });
    merge_attributes(vec![theirs, owned])
}

/// Every row in the DOM.
#[component]
fn TableRowsAll(props: BodyProps) -> Element {
    let BodyProps {
        count,
        render_row,
        row_attributes,
        row_offset,
        attributes,
        ..
    } = props;
    let owned = attributes!(tbody { "data-virtualize": "content-visibility" });
    let merged = merge_attributes(vec![attributes, owned]);
    rsx! {
        tbody { ..merged,
            for idx in 0..count() {
                tr { key: "{idx}", ..row_attrs(row_attributes, idx, row_offset), {render_row(idx)} }
            }
        }
    }
}

/// Only the rows in view, between two spacer rows.
#[component]
fn TableRowsWindowed(props: BodyProps) -> Element {
    let BodyProps {
        count,
        columns,
        render_row,
        row_attributes,
        row_offset,
        buffer,
        estimate_size,
        attributes,
    } = props;

    let mut scroller = use_context::<TableScroller>();
    let bridge = use_virtual_bridge(scroller.id, Engine::new, Bridge::scroll_axis);
    let engine = bridge.engine();

    let estimates = use_memo(move || {
        let estimate = move |i: usize| match estimate_size {
            Some(f) => f(i),
            None => DEFAULT_ROW_ESTIMATE,
        };
        SizeEstimates::evaluate(count(), Some(&estimate as &dyn Fn(usize) -> u32))
    });

    // The table announces the whole table's size, since the DOM holds a window.
    use_effect(move || scroller.row_count.set(Some(count() + row_offset)));
    use_drop(move || scroller.row_count.set(None));

    let onresize = move |idx| {
        move |event: Event<ResizeData>| {
            let rect = event.data().get_border_box_size().unwrap_or_default();
            bridge.measure(idx, measured_px(rect.height));
        }
    };

    bridge.track();
    let (items, top, bottom) = {
        let estimates = estimates.read();
        let mut engine = engine;
        engine.with_mut(|engine| {
            engine.sync(&estimates);
            let plan = engine.plan(buffer());
            let end = plan
                .items
                .last()
                .map_or(plan.top_offset, |l| l.start() + l.size());
            let bottom = plan.canvas_height.saturating_sub(end);
            (plan.items, plan.top_offset, bottom)
        })
    };

    let owned = attributes!(tbody { "data-virtualize": "windowed" });
    let merged = merge_attributes(vec![attributes, owned]);
    let columns = columns.max(1);

    rsx! {
        tbody { ..merged,
            tr {
                "data-virtual-spacer": "top",
                "aria-hidden": "true",
                style: "height: {top}px;",
                td { colspan: "{columns}", style: "height: inherit; padding: 0; border: 0;" }
            }
            for item in items {
                {
                    let idx = item.index();
                    rsx! {
                        tr {
                            key: "{item.key()}",
                            onresize: onresize(idx),
                            ..row_attrs(row_attributes, idx, row_offset),
                            {render_row(idx)}
                        }
                    }
                }
            }
            tr {
                "data-virtual-spacer": "bottom",
                "aria-hidden": "true",
                style: "height: {bottom}px;",
                td { colspan: "{columns}", style: "height: inherit; padding: 0; border: 0;" }
            }
        }
    }
}
