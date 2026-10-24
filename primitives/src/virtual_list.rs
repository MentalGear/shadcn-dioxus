//! Defines the [`VirtualList`] component for rendering large lists cheaply.
//!
//! # Modes
//!
//! [`VirtualizeMode`] (the `virtualize` prop) chooses how rows that are off-screen are kept
//! cheap. It uses the same prop name, the same `ContentVisibility` variant and the same
//! `data-virtualize` spelling as [`crate::message_scroller::Virtualization`].
//!
//! - [`VirtualizeMode::ContentVisibility`]: **every row is in the DOM** (and in the server's
//!   HTML) and the browser skips laying out and painting the ones that are off-screen
//!   (`content-visibility: auto` with `contain-intrinsic-block-size: auto <estimate>`). There is
//!   no scroll listener, no measurement and no Rust work while scrolling. The accessibility tree,
//!   cross-row selection and focus reach every row, and so does the browser's find-in-page
//!   wherever the engine searches skipped content (Chromium, Firefox 125+, Safari 26+; **not
//!   Safari 18-25**, WebKit bug 283846). The chunk holding focus or an end of the selection is
//!   never skipped: that is the browser's own "relevant to the user" rule for
//!   `content-visibility: auto`, so no controller is needed to keep them rendered. The ceiling
//!   is that DOM and memory grow linearly with the row count (a 9-node row is about 4.5 KB: ~72
//!   MB at 10k rows, ~445 MB at 100k, against ~12 MB for a window) and that the first mount
//!   renders every row (about 90 ms of main-thread time for 1,000 rows and 205 ms for 5,000 in
//!   the demo, release build). Rows are skipped in groups of [`CONTENT_VISIBILITY_CHUNK`] (20):
//!   the browser does per-frame work for every element it may skip, so one skippable element
//!   per row made a scroll cost O(rows) of main-thread time (7.7 ms per frame at 1,000 rows and
//!   31 ms at 5,000 in the demo), and one per 20 rows brings it to a windowed list's cost (1.1
//!   and 1.5 ms, against 1.4 ms windowed).
//! - [`VirtualizeMode::Windowed`]: only the rows in view plus `buffer` rows each side are
//!   mounted, and a spacer gives the scrollbar the full height. DOM and memory stay flat from
//!   10k to millions of rows, rows may have any height (measured, with `estimate_size` as the
//!   first guess), and the cost is that rows outside the window do not exist: native find-in-page
//!   cannot reach them, the accessibility tree holds only the window (each row announces
//!   `aria-posinset` of `aria-setsize`) and the server renders no rows. The row that last
//!   received focus is never unmounted: when the window leaves it, it stays drawn at its own
//!   position (`data-virtual-pinned`), as the grids keep their active cell. Browsers also cap an element's
//!   height (about 17M px in Firefox, 33M px in Chromium), which bounds a single scroll canvas
//!   to roughly 300k rows of 100px.
//! - [`VirtualizeMode::Auto`] (the default): `ContentVisibility` while
//!   `count <= auto_threshold` (1000 unless overridden), `Windowed` above it. The mode is
//!   decided from the live `count`, so a list that grows past the threshold re-mounts its rows
//!   in the other mode. Pass an explicit mode to opt out (a heavy row at 2000 rows may want
//!   `Windowed`; a 20k-row list of one-line rows may be fine as `ContentVisibility`).
//!
//! Both modes render the same container (`role="list"`, `tabindex="0"`, `data-virtualize`) and
//! the same rows (`role="listitem"`, `aria-setsize`, `aria-posinset`, `data-virtual-index`).
//!
//! # Windowed mode: what a scroll costs
//!
//! The container reports scroll offsets to Rust, but a re-render is requested only when the
//! rendered output would differ: the engine (`primitives/src/virtual/`) keeps the mounted range with
//! hysteresis (it is re-centred once the viewport comes within `buffer / 2` rows of its edge),
//! so a steady scroll re-renders about once per `buffer / 2` rows, not once per scroll event.
//! Row positions are a Fenwick tree, so measuring a row and finding the row at a pixel are
//! `O(log N)`; nothing is copied per measurement.

use dioxus::prelude::*;
use dioxus_attributes::attributes;

use crate::r#virtual::{
    cv_chunks, engine_keys, pin_in_flow, render_key, resolve_index, use_find_in_page,
    use_virtual_bridge, Bridge, ChunkSkip, Engine, FindLayer, FindLayout, SizeEstimates,
    CHUNK_ROWS, DEFAULT_SIZE,
};
pub use crate::r#virtual::{VirtualizeMode, DEFAULT_AUTO_THRESHOLD};
use crate::scroll_area::use_scroll_fade;
pub use crate::scroll_area::ScrollFade;
use crate::{merge_attributes, use_unique_id};

/// How many consecutive rows [`VirtualizeMode::ContentVisibility`] skips (and un-skips)
/// together. Chromium does per-frame work for every element that `content-visibility: auto` may
/// skip, so skipping each row separately costs the main thread `O(rows)` on every scroll frame;
/// groups of 20 keep that to `O(rows / 20)`. The grouping lives in `primitives/src/virtual/`
/// and is shared with `MessageScroller`.
pub const CONTENT_VISIBILITY_CHUNK: usize = CHUNK_ROWS;

/// A deprecated alias of [`VirtualizeMode`], which lives in the shared virtual engine (backlog row
/// 343) and is the one `virtualize` type of every virtual component. Kept for one release.
#[deprecated(since = "0.1.0", note = "renamed `VirtualizeMode`")]
pub type VirtualListMode = VirtualizeMode;

/// The props for the [`VirtualList`] component.
#[derive(Props, Clone, PartialEq)]
pub struct VirtualListProps {
    /// The total number of items.
    pub count: ReadSignal<usize>,
    /// How many rows to mount beyond each edge of the viewport, so a scroll never shows a blank edge
    /// (the window is re-centred once fewer than half of them remain, so between `buffer / 2` and
    /// `buffer` rows are always mounted beyond each edge). Only a windowed layout reads it:
    /// `VirtualList` in content-visibility mode ignores it. Defaults to 8 rows for `VirtualList` and 4
    /// for `VirtualWrapGrid`.
    #[props(default = ReadSignal::new(Signal::new(8)))]
    pub buffer: ReadSignal<usize>,
    /// Estimates the height of an item by index, in px. Windowed mode uses it until a row is
    /// measured; content-visibility mode uses it as `contain-intrinsic-block-size` (the height
    /// the browser assumes for a row it has not rendered yet, then remembers the real one).
    /// For the steadiest scrollbar, return values close to the real heights. If not provided,
    /// windowed mode estimates from the average of the measured rows and content-visibility
    /// mode assumes 100px.
    pub estimate_size: Option<Callback<usize, u32>>,
    /// Renders the item at an absolute index.
    pub render_item: Callback<usize, Element>,
    /// Who is at index `idx`: a string unique among the items that stays with the item when the
    /// items are sorted, filtered or reordered. Defaults to the index (an item is its slot). With
    /// it the item's DOM node (and so focus and any state inside it), the item that stays mounted
    /// while it holds focus and the measured sizes follow the item instead of the slot. Read
    /// while rendering, so signals it reads re-render the component.
    pub item_key: Option<Callback<usize, String>>,
    /// Windowed layouts, opt-in (experimental, backlog rows 153 and 309): the searchable text of
    /// an item, plain text without markup (a `VirtualList` row or a `VirtualWrapGrid` item by
    /// index, a `VirtualGrid` cell by `(row, column)`). When given, every item's text is also kept
    /// in the DOM inside a `hidden="until-found"` placeholder, so the browser's find-in-page (and a
    /// `#:~:text=` link) can match items that are not mounted; when the browser reveals a match it
    /// fires `beforematch` on that placeholder, and the component scrolls the real item into view
    /// and marks it `data-find-match="true"` (the find bar highlights the hidden placeholder, not
    /// the item, so style the attribute to show which item matched). Costs about two DOM nodes
    /// per item (an empty text gets no placeholder), mounted after hydration (not
    /// server-rendered), and a mounted item's match counts twice. Engines without `until-found`
    /// (Firefox before 139, Safari before 26.2) treat it as `hidden`: items that are not mounted
    /// stay unfindable, as without the prop. Ignored by content-visibility mode, whose items are
    /// all in the DOM already.
    pub text_value: Option<Callback<usize, String>>,
    /// How off-screen items are kept cheap. Defaults to [`VirtualizeMode::Auto`].
    #[props(default = ReadSignal::new(Signal::new(VirtualizeMode::Auto)))]
    pub virtualize: ReadSignal<VirtualizeMode>,
    /// With [`VirtualizeMode::Auto`]: the largest number of items that keeps every item in the
    /// DOM (a list counts its rows, a grid its cells: `count` for `VirtualWrapGrid`,
    /// `row_count x column_count` for `VirtualGrid`). Defaults to [`DEFAULT_AUTO_THRESHOLD`] (1000).
    #[props(default = ReadSignal::new(Signal::new(DEFAULT_AUTO_THRESHOLD)))]
    pub auto_threshold: ReadSignal<usize>,
    /// A scroll-edge fade on the container, which is the element that scrolls (the `dx-scroll-fade-*`
    /// classes of `dx-effects.css`; see [`ScrollFade`]). Defaults to [`ScrollFade::None`]. It follows
    /// the container's own scroll offset, whatever rows or cells are mounted, and a container whose
    /// content fits has no fade. The fade is a mask, so it also fades a border or background on the
    /// container (put those on a wrapper) and a `VirtualGrid`'s sticky header (prefer
    /// [`ScrollFade::Bottom`] / [`ScrollFade::End`] with `render_header`); [`ScrollFade::Both`] masks
    /// all four edges of a two-axis scroller.
    #[props(default)]
    pub fade: ScrollFade,
    /// Additional attributes to apply to the container element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// The props the two mode bodies share.
#[derive(Props, Clone, PartialEq)]
struct ListBodyProps {
    count: ReadSignal<usize>,
    buffer: ReadSignal<usize>,
    estimate_size: Option<Callback<usize, u32>>,
    render_item: Callback<usize, Element>,
    item_key: Option<Callback<usize, String>>,
    text_value: Option<Callback<usize, String>>,
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
}

/// # VirtualList
///
/// The `VirtualList` component renders a long list cheaply, in one of two modes (see the
/// module documentation for the full trade-offs): every row in the DOM with the browser skipping
/// the off-screen ones ([`VirtualizeMode::ContentVisibility`]), or only a window of rows mounted
/// over a full-height canvas ([`VirtualizeMode::Windowed`]). By default
/// ([`VirtualizeMode::Auto`]) it picks the first up to [`DEFAULT_AUTO_THRESHOLD`] rows and the
/// second above.
///
/// Each row is a `role="listitem"` with `aria-setsize` and `aria-posinset`, in both modes, so a
/// screen reader announces the list's real size even when only a window of it is in the DOM.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::virtual_list::{VirtualList, VirtualizeMode};
///
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         VirtualList {
///             count: 100usize,
///             // Optional: the height guess for rows that have not been measured/rendered.
///             estimate_size: |_idx| 48,
///             // Optional: `Auto` (the default) picks by `count`.
///             virtualize: VirtualizeMode::ContentVisibility,
///             render_item: move |idx: usize| rsx! {
///                 article { "Row {idx}" }
///             },
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The container is a `div` the caller makes a scroll container (give it a height or
/// `max-height` and `overflow-y: auto`); the styled layer adds the class
/// `dx-virtual-list-container`. All user-provided `attributes` are spread onto it, except `id`,
/// `role` and `tabindex`, which the list owns. The container carries `data-virtualize`
/// (`"content-visibility"` or `"windowed"`) for styling and tests.
#[component]
pub fn VirtualList(props: VirtualListProps) -> Element {
    let VirtualListProps {
        count,
        buffer,
        estimate_size,
        render_item,
        item_key,
        text_value,
        virtualize,
        auto_threshold,
        fade,
        attributes,
    } = props;
    let attributes = merge_attributes(vec![attributes, use_scroll_fade(fade)]);

    // The mode body is a separate component, so switching modes (a count crossing the
    // threshold, or a new `virtualize`) drops the old body's scope and everything it owns (the
    // windowed engine, its scroll listeners) with it. A memo, so only a *change of mode*
    // re-renders this component, not every count change.
    let windowed = use_memo(move || {
        virtualize().resolve(count(), auto_threshold()) == VirtualizeMode::Windowed
    });

    if windowed() {
        rsx! {
            VirtualListWindowed {
                count,
                buffer,
                estimate_size,
                render_item,
                item_key,
                text_value,
                attributes,
            }
        }
    } else {
        rsx! {
            VirtualListContentVisibility {
                count,
                buffer,
                estimate_size,
                render_item,
                item_key,
                text_value: None,
                attributes,
            }
        }
    }
}

/// Every row in the DOM, off-screen ones skipped by the browser. No hooks beyond an id, no
/// effects, no listeners: a scroll costs the page nothing but the browser's own work.
#[component]
fn VirtualListContentVisibility(props: ListBodyProps) -> Element {
    let ListBodyProps {
        count,
        estimate_size,
        render_item,
        item_key,
        attributes,
        ..
    } = props;

    let container_id = use_unique_id();
    let owned = attributes!(div {
        id: container_id,
        role: "list",
        tabindex: "0",
        "data-virtualize": "content-visibility",
    });
    let merged = merge_attributes(vec![attributes, owned]);

    let rows = count();
    let set_size = rows.to_string();

    // The browser skips layout and paint for a chunk of rows that is off-screen and remembers
    // its real height afterwards (`auto`); until it has rendered the chunk it assumes the sum of
    // the rows' estimates. A chunk that holds focus or an end of the selection is never skipped
    // (the browser's "relevant to the user" rule), and `window.find` un-skips the chunk holding
    // a match.
    let skip = ChunkSkip::Inline {
        estimate: estimate_size,
        fallback: DEFAULT_SIZE,
    };

    rsx! {
        div { ..merged,
            {cv_chunks(rows, 0, skip, &[], |idx| rsx! {
                div {
                    key: "{render_key(item_key, idx)}",
                    role: "listitem",
                    "data-virtual-index": "{idx}",
                    "aria-setsize": "{set_size}",
                    "aria-posinset": "{idx + 1}",
                    {render_item(idx)}
                }
            })}
        }
    }
}

/// Only the rows in view (plus a buffer) mounted over a full-height canvas.
#[component]
fn VirtualListWindowed(props: ListBodyProps) -> Element {
    let ListBodyProps {
        count,
        buffer,
        estimate_size,
        render_item,
        item_key,
        text_value,
        attributes,
    } = props;

    let container_id = use_unique_id();

    // The shared DOM bridge (`primitives/src/virtual/bridge.rs`): one passive scroll listener,
    // one ResizeObserver, removed with this scope; the engine is non-reactive state and a render
    // runs only when it says the window moved.
    let bridge = use_virtual_bridge(container_id, Engine::new, Bridge::scroll_axis);
    let engine = bridge.engine();

    // The estimates are evaluated in a memo so that signals the `estimate_size` callback reads
    // are tracked, as they always were. Measurements are *not* an input: they update the engine
    // in `O(log N)` instead of rebuilding anything.
    let estimates = use_memo(move || {
        let estimate = estimate_size.as_ref().map(|c| move |i: usize| c(i));
        SizeEstimates::evaluate(
            count(),
            estimate.as_ref().map(|f| f as &dyn Fn(usize) -> u32),
        )
        .with_keys(engine_keys(item_key, count()))
    });

    // The row that last received focus stays mounted when the window leaves it (the grids keep
    // their active cell the same way, `virtual/pin.rs`): unmounting it would drop focus to
    // `<body>` mid-interaction.
    // Remembered as the index it had and, with an `item_key`, who it is: when the list is
    // reordered the pin follows the item (`resolve_index`). `pin_tick` only asks for a render.
    let mut pin = use_hook(|| CopyValue::new(None::<(usize, Option<String>)>));
    let mut pin_tick = use_signal(|| 0u32);

    // Find-in-page placeholders (opt-in, row 153; the shared layer is `virtual/find.rs`): the
    // `beforematch` listener names the item whose placeholder the browser is about to reveal; the list then
    // scrolls that row's real position into the middle of the viewport, which mounts it, and
    // marks it as the match (`data-find-match`), because the find bar's own highlight is on the
    // placeholder, not on the row.
    let mut find_match = use_signal(|| None::<usize>);
    let hydrated = use_find_in_page(container_id, text_value.is_some(), move |index: String| {
        if let Ok(index) = index.parse() {
            find_match.set(Some(index));
            bridge.reveal(index);
        }
    });

    let onresize = move |idx| {
        move |event: Event<ResizeData>| {
            let rect = event.data().get_content_box_size().unwrap_or_default();
            bridge.measure_exact(idx, rect.height);
        }
    };

    // Subscribe: this render runs again whenever the bridge publishes a new window.
    bridge.track();
    pin_tick();
    let pinned_index = {
        let current = pin.peek().clone();
        current.and_then(|(index, key)| {
            let now = resolve_index(item_key, key.as_deref(), index, count());
            if now != Some(index) {
                pin.set(now.map(|index| (index, key)));
            }
            now
        })
    };
    let (plan, items, detached) = {
        let estimates = estimates.read();
        let mut engine = engine;
        engine.with_mut(|engine| {
            engine.sync(&estimates);
            let mut plan = engine.plan(buffer());
            let pinned = pinned_index.and_then(|index| engine.item(index));
            let (items, detached) =
                pin_in_flow(std::mem::take(&mut plan.items), pinned, plan.top_offset);
            (plan, items, detached)
        })
    };
    let set_size = count.to_string();
    let rows_total = count();
    let find_match = find_match();

    // `id` is owned: the bridge looks this element up by id (a JS eval), so a caller override
    // would silently break scroll sync (backlog row 93 names this site explicitly).
    // `role`/`tabindex` are this container's own widget semantics, also owned.
    let owned = attributes!(div {
        id: container_id,
        role: "list",
        tabindex: "0",
        "data-virtualize": "windowed",
    });
    let merged = merge_attributes(vec![attributes, owned]);

    rsx! {
        div {
            ..merged,

            div {
                style: "position: relative; height:{plan.canvas_height}px; width: 100%;",
                if let Some(text_value) = text_value.filter(|_| hydrated()) {
                    FindLayer {
                        rows: rows_total,
                        layout: FindLayout::Rows,
                        row_height: move |idx| estimate_size.map_or(DEFAULT_SIZE, |f| f(idx)),
                        text: move |(idx, _)| text_value(idx),
                    }
                }
                div {
                    style: "position: absolute; inset: 0 auto auto 0; width: 100%; transform: translateY({plan.top_offset}px); will-change: transform;",
                    {items.into_iter().map(move |item| {
                        let idx = item.index();
                        // The focused row, when the window has left it: drawn at its own
                        // position without pushing the rows in flow (same parent, same key, so
                        // it rejoins the flow without remounting).
                        let detached_style = detached
                            .filter(|(index, _)| *index == idx)
                            .map(|(_, top)| format!("position: absolute; inset-inline: 0; top: {top}px;"));

                        rsx! {
                            div {
                                key: "{render_key(item_key, idx)}",
                                role: "listitem",
                                "data-virtual-index": "{idx}",
                                "data-virtual-pinned": detached_style.is_some().then_some("true"),
                                "data-find-match": (find_match == Some(idx)).then_some("true"),
                                "aria-setsize": "{set_size}",
                                "aria-posinset": "{idx + 1}",
                                style: detached_style,
                                onresize: onresize(idx),
                                onfocusin: move |_| {
                                    let now = (idx, item_key.map(|_| render_key(item_key, idx)));
                                    if pin.peek().as_ref() != Some(&now) {
                                        pin.set(Some(now));
                                        pin_tick += 1;
                                    }
                                },
                                {render_item(idx)}
                            }
                        }
                    })}
                }
            }
        }
    }
}
