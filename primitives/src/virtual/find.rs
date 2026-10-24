//! Native find-in-page for windowed views (backlog rows 153 and 309): the one find layer that
//! `VirtualList` (windowed mode), `VirtualGrid` and `VirtualWrapGrid` lay over their canvas.
//!
//! An item that is not mounted has no text in the DOM, so the browser's find-in-page (and a
//! `#:~:text=` link) cannot reach it. The layer keeps every item's text in the DOM as a
//! `hidden="until-found"` placeholder: it costs no layout until revealed, and the browser
//! searches it (Chromium 102+, Firefox 139+, Safari 26.2; elsewhere it is plain `hidden`, so an
//! unmounted item stays unfindable, as without the layer). On a match the browser fires
//! `beforematch` on that placeholder, which names the item; [`use_find_in_page`] hands the name to
//! the component, which scrolls the real item into view and marks it `data-find-match="true"`.
//!
//! What every consumer gets the same way:
//!
//! - **a stable layer**: it depends only on the count, the layout and the text, so a scroll never
//!   re-renders it and the DOM the browser's find walks stays put while it walks it (a layer that
//!   dropped the placeholders of mounted items would change on every window move, and find-next
//!   would skip items, row 153). The price is that a mounted item's text is in the DOM twice, so its
//!   match counts twice;
//! - **chunks**: rows in groups of [`FIND_CHUNK`], each a wrapper element (Dioxus 0.7.9 appends new
//!   children with a 16-bit count, so one parent receiving 65,536 or more in one edit keeps only
//!   the last `n mod 65536`) and a `content-visibility: auto` box of its rows' summed estimate
//!   (without it every window move re-lays out every placeholder: 100,000 rows, 20 scroll frames
//!   take 2.6 s against 0.3 s, debug build); find-in-page still searches skipped content;
//! - **no stray paint**: a revealed placeholder is hidden again in the next frame, and the find
//!   bar's highlight on it is transparent (`::search-text`, Chromium 144+): a placeholder sits at
//!   its item's estimated position, not on the item's own text;
//! - **no pointer, no accessibility tree**: the layer is `aria-hidden` and ignores the pointer.
//!
//! What stays per component is the layout ([`FindLayout`]) and what a match does.

use dioxus::prelude::*;

use crate::js_listener::{use_js_listeners, JsListeners};

/// How many rows of placeholders one chunk holds, so a count change re-renders the layer as
/// `rows / FIND_CHUNK` components rather than one component of `rows` children.
pub(crate) const FIND_CHUNK: usize = 100;

/// How a row of placeholders is laid out; it decides the `data-find-index` a match reports.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum FindLayout {
    /// One placeholder per row, as wide as the layer (a list). Reports `"{row}"`.
    Rows,
    /// A data grid: the cells of a row at fixed inline offsets, one `(start, width)` in px per
    /// column. Reports `"{row}:{column}"`.
    Columns(Vec<(u32, u32)>),
    /// A wrapping grid: `count` items in rows of `columns` equal tracks. Reports the item's
    /// index, `"{row * columns + column}"`.
    Tracks {
        /// The number of items.
        count: usize,
        /// Items per row, at least 1.
        columns: usize,
    },
}

impl FindLayout {
    /// The cells of row `row`: `(slot, data-find-index)` for each placeholder it holds.
    fn cells(&self, row: usize) -> Vec<(usize, String)> {
        match self {
            Self::Rows => vec![(0, row.to_string())],
            Self::Columns(columns) => (0..columns.len())
                .map(|column| (column, format!("{row}:{column}")))
                .collect(),
            Self::Tracks { count, columns } => {
                let columns = (*columns).max(1);
                let first = row * columns;
                (0..count.saturating_sub(first).min(columns))
                    .map(|slot| (slot, (first + slot).to_string()))
                    .collect()
            }
        }
    }
}

/// Installs the `beforematch` listener on the container `container_id` when `enabled`, and calls
/// `on_match` with the `data-find-index` of every placeholder the browser is about to reveal.
/// Returns whether the layer may mount: after hydration, so the server's HTML carries no layer
/// and the client's first frame matches it.
pub(crate) fn use_find_in_page(
    container_id: Signal<String>,
    enabled: bool,
    on_match: impl FnMut(String) + Clone + 'static,
) -> ReadSignal<bool> {
    let mut hydrated = use_signal(|| false);
    use_effect(move || hydrated.set(true));
    use_js_listeners(move || {
        if !enabled {
            return None;
        }
        Some(
            JsListeners::install(BEFOREMATCH_BRIDGE_JS)
                .arg(container_id.cloned())
                .on_message(on_match.clone()),
        )
    });
    hydrated.into()
}

/// The find layer: `rows` rows of placeholders, row `r` `row_height(r)` px tall, holding
/// `text((r, slot))` for each cell [`FindLayout`] gives it (an empty text has no placeholder).
/// Absolutely positioned at the top of its parent, which is the component's canvas.
#[component]
pub(crate) fn FindLayer(
    /// The number of rows.
    rows: usize,
    /// How a row's placeholders are laid out.
    layout: FindLayout,
    /// The estimated height of a row, by row index, in px.
    row_height: Callback<usize, u32>,
    /// The searchable text of a cell, by `(row, slot)`: the slot is the column in a grid and 0
    /// in a list.
    text: Callback<(usize, usize), String>,
) -> Element {
    let chunks = rows.div_ceil(FIND_CHUNK);
    rsx! {
        div {
            "data-find-layer": "",
            "aria-hidden": "true",
            style: "position: absolute; inset: 0 0 auto 0; pointer-events: none; color: transparent;",
            // The find bar's highlight is not painted on a placeholder (Chromium 144+; elsewhere
            // the re-hide in `BEFOREMATCH_BRIDGE_JS` keeps it to one frame).
            style { "[data-find-index]::search-text {{ color: transparent; background-color: transparent; }}" }
            {(0..chunks).map(|chunk| {
                let start = chunk * FIND_CHUNK;
                let end = (start + FIND_CHUNK).min(rows) - 1;
                rsx! {
                    // key-ok: a chunk of find placeholders is a slice of positions, not an item
                    FindChunk { key: "{chunk}", start, end, layout: layout.clone(), row_height, text }
                }
            })}
        }
    }
}

/// Rows `start..=end` of the find layer.
#[component]
fn FindChunk(
    /// The first row of the chunk.
    start: usize,
    /// The last row of the chunk, inclusive.
    end: usize,
    /// How a row's placeholders are laid out.
    layout: FindLayout,
    /// The estimated height of a row, by row index, in px.
    row_height: Callback<usize, u32>,
    /// The searchable text of a cell, by `(row, slot)`.
    text: Callback<(usize, usize), String>,
) -> Element {
    let heights: Vec<u32> = (start..=end).map(|row| row_height.call(row)).collect();
    let total = heights.iter().copied().fold(0u32, u32::saturating_add);
    rsx! {
        div { style: "content-visibility: auto; contain-intrinsic-block-size: {total}px;",
            {(start..=end).zip(heights).map(|(row, height)| match &layout {
                // A list's row is its own placeholder: one element per item.
                FindLayout::Rows => rsx! {
                    div {
                        // key-ok: a placeholder is a position's text, rebuilt from `text`
                        key: "{row}",
                        // Upper case on purpose: Dioxus' web interpreter treats an attribute NAMED
                        // `hidden` as boolean and drops "until-found"; `setAttribute` lower-cases the
                        // name (the same slot as `collapsible::hidden_attribute`).
                        "HIDDEN": "until-found",
                        "data-find-index": "{row}",
                        style: "height: {height}px; overflow: hidden;",
                        {text((row, 0))}
                    }
                },
                FindLayout::Columns(columns) => rsx! {
                    div {
                        // key-ok: a row of find placeholders is a position, not an item
                        key: "{row}",
                        style: "position: relative; height: {height}px;",
                        {layout.cells(row).into_iter().filter_map(|(column, index)| {
                            let value = text((row, column));
                            let (start, width) = columns[column];
                            (!value.is_empty()).then(|| rsx! {
                                div {
                                    // key-ok: a cell placeholder is a position of the grid
                                    key: "{column}",
                                    "HIDDEN": "until-found",
                                    "data-find-index": "{index}",
                                    style: "position: absolute; inset-block: 0; inset-inline-start: {start}px; width: {width}px; overflow: hidden;",
                                    {value}
                                }
                            })
                        })}
                    }
                },
                FindLayout::Tracks { columns, .. } => rsx! {
                    div {
                        // key-ok: a row of find placeholders is a position, not an item
                        key: "{row}",
                        style: "display: grid; grid-template-columns: repeat({columns}, minmax(0, 1fr)); height: {height}px;",
                        {layout.cells(row).into_iter().filter_map(|(slot, index)| {
                            let value = text((row, slot));
                            (!value.is_empty()).then(|| rsx! {
                                div {
                                    // key-ok: a cell placeholder is a position of the grid
                                    key: "{slot}",
                                    "HIDDEN": "until-found",
                                    "data-find-index": "{index}",
                                    style: "grid-column: {slot + 1}; min-width: 0; overflow: hidden;",
                                    {value}
                                }
                            })
                        })}
                    }
                },
            })}
        }
    }
}

/// Tells Rust which placeholder the browser is about to reveal for a find-in-page or
/// text-fragment match (its `data-find-index`). `beforematch` bubbles; one capturing listener on
/// the container sees all. The browser removes `hidden` after the event; the placeholder gets it
/// back in the next frame, which the real find bar (headed Chromium 153) walks past in order,
/// forwards and back.
const BEFOREMATCH_BRIDGE_JS: &str = r#"
    const container = document.getElementById(await dioxus.recv());
    if (!container) return;
    listen(container, "beforematch", (event) => {
        const holder = event.target.closest && event.target.closest("[data-find-index]");
        if (!holder) return;
        dioxus.send(holder.dataset.findIndex);
        requestAnimationFrame(() => holder.setAttribute("hidden", "until-found"));
    }, true);
"#;

/// A `data-find-index` of [`FindLayout::Columns`], `"{row}:{column}"`, as a cell.
pub(crate) fn parse_cell(index: &str) -> Option<(usize, usize)> {
    let (row, column) = index.split_once(':')?;
    Some((row.parse().ok()?, column.parse().ok()?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_list_row_is_one_placeholder_named_by_its_index() {
        assert_eq!(FindLayout::Rows.cells(7), vec![(0, "7".to_string())]);
    }

    #[test]
    fn a_data_grid_row_names_each_cell_by_row_and_column() {
        let layout = FindLayout::Columns(vec![(0, 80), (80, 112), (192, 112)]);
        let cells = layout.cells(4);
        assert_eq!(
            cells,
            vec![
                (0, "4:0".to_string()),
                (1, "4:1".to_string()),
                (2, "4:2".to_string())
            ]
        );
        for (_, index) in cells {
            assert_eq!(parse_cell(&index).map(|(row, _)| row), Some(4));
        }
    }

    #[test]
    fn a_wrapping_grid_names_items_and_shortens_the_last_row() {
        let layout = FindLayout::Tracks {
            count: 10,
            columns: 4,
        };
        assert_eq!(
            layout.cells(1),
            vec![
                (0, "4".to_string()),
                (1, "5".to_string()),
                (2, "6".to_string()),
                (3, "7".to_string())
            ]
        );
        assert_eq!(
            layout.cells(2),
            vec![(0, "8".to_string()), (1, "9".to_string())]
        );
        assert!(layout.cells(3).is_empty());
    }

    #[test]
    fn a_cell_index_parses_back_and_rejects_anything_else() {
        assert_eq!(parse_cell("12:3"), Some((12, 3)));
        assert_eq!(parse_cell("12"), None);
        assert_eq!(parse_cell("a:3"), None);
    }
}
