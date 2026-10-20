//! Defines the [`VirtualList`] component for rendering large lists cheaply.
//!
//! # Modes
//!
//! [`VirtualListMode`] (the `virtualize` prop) chooses how rows that are off-screen are kept
//! cheap. It uses the same prop name, the same `ContentVisibility` variant and the same
//! `data-virtualize` spelling as [`crate::message_scroller::Virtualization`].
//!
//! - [`VirtualListMode::ContentVisibility`]: **every row is in the DOM** (and in the server's
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
//! - [`VirtualListMode::Windowed`]: only the rows in view plus `buffer` rows each side are
//!   mounted, and a spacer gives the scrollbar the full height. DOM and memory stay flat from
//!   10k to millions of rows, rows may have any height (measured, with `estimate_size` as the
//!   first guess), and the cost is that rows outside the window do not exist: native find-in-page
//!   cannot reach them, the accessibility tree holds only the window (each row announces
//!   `aria-posinset` of `aria-setsize`) and the server renders no rows. The row that last
//!   received focus is never unmounted: when the window leaves it, it stays drawn at its own
//!   position (`data-virtual-pinned`), as the grids keep their active cell. Browsers also cap an element's
//!   height (about 17M px in Firefox, 33M px in Chromium), which bounds a single scroll canvas
//!   to roughly 300k rows of 100px.
//! - [`VirtualListMode::Auto`] (the default): `ContentVisibility` while
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

use crate::js_listener::{use_js_listeners, JsListeners};
use crate::r#virtual::{
    cv_chunks, measured_px, pin_in_flow, use_virtual_bridge, Bridge, ChunkSkip, Engine,
    SizeEstimates, CHUNK_ROWS, DEFAULT_SIZE,
};
use crate::scroll_area::use_scroll_fade;
pub use crate::scroll_area::ScrollFade;
use crate::{merge_attributes, use_unique_id};

/// The row count up to which [`VirtualListMode::Auto`] keeps every row in the DOM.
///
/// 1,000 because the cost of keeping every row grows with the count: mounting them is one
/// main-thread task proportional to the row count (about 90 ms for 1,000 rows and 205 ms, one
/// 147 ms task, for 5,000 in the demo, release build), while a window mounts about 40 rows
/// whatever the count. Raise it for lists of cheap rows, lower it for heavy ones.
pub const DEFAULT_AUTO_THRESHOLD: usize = 1_000;

/// How many consecutive rows [`VirtualListMode::ContentVisibility`] skips (and un-skips)
/// together. Chromium does per-frame work for every element that `content-visibility: auto` may
/// skip, so skipping each row separately costs the main thread `O(rows)` on every scroll frame;
/// groups of 20 keep that to `O(rows / 20)`. The grouping lives in `primitives/src/virtual/`
/// and is shared with `MessageScroller`.
pub const CONTENT_VISIBILITY_CHUNK: usize = CHUNK_ROWS;

/// How a [`VirtualList`] keeps off-screen rows cheap. See the module doc's "Modes".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VirtualListMode {
    /// [`ContentVisibility`](Self::ContentVisibility) while `count <= auto_threshold`,
    /// [`Windowed`](Self::Windowed) above it. The default.
    #[default]
    Auto,
    /// Every row stays in the DOM and the browser skips the ones that are off-screen
    /// (`content-visibility: auto`). Native find-in-page, selection, focus and the accessibility
    /// tree reach every row (find-in-page: not Safari 18-25). DOM and memory grow with the row
    /// count.
    ContentVisibility,
    /// Only the rows in view plus `buffer` rows each side are mounted. Flat cost for any row
    /// count; rows outside the window are unreachable by find-in-page and the server renders
    /// none.
    Windowed,
}

impl VirtualListMode {
    /// The `virtualize` prop's wire spelling: `"auto"`, `"content-visibility"` or `"windowed"`.
    /// A rendered list's `data-virtualize` is always one of the last two.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::ContentVisibility => "content-visibility",
            Self::Windowed => "windowed",
        }
    }

    /// The concrete mode for a list of `count` rows: `self` unless it is `Auto`, which becomes
    /// `ContentVisibility` up to `auto_threshold` rows and `Windowed` above. Never returns
    /// `Auto`.
    pub fn resolve(self, count: usize, auto_threshold: usize) -> Self {
        match self {
            Self::Auto if count <= auto_threshold => Self::ContentVisibility,
            Self::Auto => Self::Windowed,
            concrete => concrete,
        }
    }
}

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
    /// Windowed mode, opt-in (experimental, backlog row 153): the searchable text of item `idx`,
    /// plain text without markup. When given, every row's text is also kept in the DOM inside a
    /// `hidden="until-found"` placeholder, so the browser's find-in-page (and a `#:~:text=` link)
    /// can match rows that are not mounted; when the browser reveals a match it fires
    /// `beforematch` on that placeholder and the list scrolls the real row into view. Costs about
    /// two DOM nodes per row, mounted after hydration (not server-rendered), and a mounted row's
    /// match counts twice. Engines without `until-found` (Firefox before 139, Safari before
    /// 26.2) treat it as `hidden`: rows that are not mounted stay unfindable, as without the
    /// prop. Ignored by content-visibility mode, whose rows are all in the DOM already.
    pub text_value: Option<Callback<usize, String>>,
    /// How off-screen rows are kept cheap. Defaults to [`VirtualListMode::Auto`].
    #[props(default = ReadSignal::new(Signal::new(VirtualListMode::Auto)))]
    pub virtualize: ReadSignal<VirtualListMode>,
    /// With [`VirtualListMode::Auto`]: the largest `count` that keeps every row in the DOM.
    /// Defaults to [`DEFAULT_AUTO_THRESHOLD`] (1000).
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
    text_value: Option<Callback<usize, String>>,
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
}

/// # VirtualList
///
/// The `VirtualList` component renders a long list cheaply, in one of two modes (see the
/// module documentation for the full trade-offs): every row in the DOM with the browser skipping
/// the off-screen ones ([`VirtualListMode::ContentVisibility`]), or only a window of rows mounted
/// over a full-height canvas ([`VirtualListMode::Windowed`]). By default
/// ([`VirtualListMode::Auto`]) it picks the first up to [`DEFAULT_AUTO_THRESHOLD`] rows and the
/// second above.
///
/// Each row is a `role="listitem"` with `aria-setsize` and `aria-posinset`, in both modes, so a
/// screen reader announces the list's real size even when only a window of it is in the DOM.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::virtual_list::{VirtualList, VirtualListMode};
///
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         VirtualList {
///             count: 100usize,
///             // Optional: the height guess for rows that have not been measured/rendered.
///             estimate_size: |_idx| 48,
///             // Optional: `Auto` (the default) picks by `count`.
///             virtualize: VirtualListMode::ContentVisibility,
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
        virtualize().resolve(count(), auto_threshold()) == VirtualListMode::Windowed
    });

    if windowed() {
        rsx! {
            VirtualListWindowed {
                count,
                buffer,
                estimate_size,
                render_item,
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
                    key: "{idx}",
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
    });

    // The row that last received focus stays mounted when the window leaves it (the grids keep
    // their active cell the same way, `virtual/pin.rs`): unmounting it would drop focus to
    // `<body>` mid-interaction.
    let mut focused = use_signal(|| None::<usize>);

    // Find-in-page placeholders (opt-in, row 153): one capturing `beforematch` listener on the
    // container names the item whose placeholder the browser is about to reveal; the list then
    // scrolls that row's real position into the middle of the viewport, which mounts it.
    let find_enabled = text_value.is_some();
    let mut hydrated = use_signal(|| false);
    use_effect(move || hydrated.set(true));
    use_js_listeners(move || {
        if !find_enabled {
            return None;
        }
        Some(
            JsListeners::install(BEFOREMATCH_BRIDGE_JS)
                .arg(container_id.cloned())
                .on_message(move |index: usize| bridge.reveal(index)),
        )
    });

    let onresize = move |idx| {
        move |event: Event<ResizeData>| {
            let rect = event.data().get_content_box_size().unwrap_or_default();
            bridge.measure(idx, measured_px(rect.height));
        }
    };

    // Subscribe: this render runs again whenever the bridge publishes a new window.
    bridge.track();
    let pinned_index = focused();
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
                    FindPlaceholders { count: rows_total, estimate_size, text_value }
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
                                key: "{item.key()}",
                                role: "listitem",
                                "data-virtual-index": "{idx}",
                                "data-virtual-pinned": detached_style.is_some().then_some("true"),
                                "aria-setsize": "{set_size}",
                                "aria-posinset": "{idx + 1}",
                                style: detached_style,
                                onresize: onresize(idx),
                                onfocusin: move |_| {
                                    if *focused.peek() != Some(idx) {
                                        focused.set(Some(idx));
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

/// How many placeholders one [`FindChunk`] holds, so a count change re-renders the layer as
/// `count / FIND_CHUNK` components rather than one component of `count` children.
const FIND_CHUNK: usize = 100;

/// The find-in-page layer (row 153): a column of `hidden="until-found"` text placeholders, one per
/// row, laid over the canvas. It depends only on the count, the estimates and the text, so a
/// scroll never re-renders it: the DOM the browser's find walks stays stable while it walks it
/// (a layer that dropped the placeholders of mounted rows had to change on every window move,
/// and find-next then skipped rows; see the row file). The price is that a mounted row's text
/// is in the DOM twice, so its match counts twice. Each placeholder is as tall as its row's
/// estimate, so a revealed match sits where the real row is when the estimates are right; the
/// layer is transparent and ignores the pointer.
#[component]
fn FindPlaceholders(
    /// The number of rows.
    count: usize,
    /// The row height estimate, by index.
    estimate_size: Option<Callback<usize, u32>>,
    /// The searchable text of a row.
    text_value: Callback<usize, String>,
) -> Element {
    let chunks = count.div_ceil(FIND_CHUNK);
    rsx! {
        div {
            "data-find-layer": "",
            "aria-hidden": "true",
            style: "position: absolute; inset: 0 0 auto 0; pointer-events: none; color: transparent;",
            {(0..chunks).map(|chunk| {
                let start = chunk * FIND_CHUNK;
                let end = (start + FIND_CHUNK).min(count) - 1;
                rsx! {
                    FindChunk { key: "{chunk}", start, end, estimate_size, text_value }
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
    /// The row height estimate, by index.
    estimate_size: Option<Callback<usize, u32>>,
    /// The searchable text of a row.
    text_value: Callback<usize, String>,
) -> Element {
    // A wrapper element per chunk, not a fragment: Dioxus 0.7.9's web interpreter appends new
    // children with a 16-bit count, so a single parent that receives 65,536 or more new children in
    // one edit silently keeps only the last `n mod 65536` (switching this demo from 10,000 to
    // 100,000 rows lost rows 10,000..75,535 with fragment chunks).
    //
    // The chunk is also a `content-visibility: auto` box of its rows' summed estimate: without it
    // every window move re-laid out all the placeholders (100,000 rows: 20 scroll frames took
    // 2.6 s against 0.3 s without the layer, debug build); find-in-page still searches skipped
    // content.
    let heights: Vec<u32> = (start..=end)
        .map(|idx| estimate_size.map_or(DEFAULT_SIZE, |f| f(idx)))
        .collect();
    let total: u32 = heights.iter().sum();
    rsx! {
        div { style: "content-visibility: auto; contain-intrinsic-block-size: {total}px;",
            {(start..=end).zip(heights).map(|(idx, height)| {
            rsx! {
                div {
                    key: "{idx}",
                    // Upper case on purpose: Dioxus' web interpreter treats an attribute NAMED
                    // `hidden` as boolean and drops "until-found"; `setAttribute` lower-cases the
                    // name (the same slot as `collapsible::hidden_attribute`).
                    "HIDDEN": "until-found",
                    "data-find-index": "{idx}",
                    style: "height: {height}px; overflow: hidden;",
                    {text_value(idx)}
                }
            }
        })} }
    }
}

/// Tells Rust which row's placeholder the browser is about to reveal for a find-in-page or
/// text-fragment match. `beforematch` bubbles; one capturing listener on the container sees all.
const BEFOREMATCH_BRIDGE_JS: &str = r#"
    const container = document.getElementById(await dioxus.recv());
    if (!container) return;
    listen(container, "beforematch", (event) => {
        const holder = event.target.closest && event.target.closest("[data-find-index]");
        if (holder) dioxus.send(Number(holder.dataset.findIndex));
    }, true);
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_picks_by_count_and_threshold() {
        let auto = VirtualListMode::Auto;
        assert_eq!(auto.resolve(0, 1_000), VirtualListMode::ContentVisibility);
        assert_eq!(
            auto.resolve(1_000, 1_000),
            VirtualListMode::ContentVisibility
        );
        assert_eq!(auto.resolve(1_001, 1_000), VirtualListMode::Windowed);
        assert_eq!(auto.resolve(100, 10), VirtualListMode::Windowed);
    }

    #[test]
    fn an_explicit_mode_ignores_the_count() {
        for count in [0, 1, 1_000, 1_001, 1_000_000] {
            assert_eq!(
                VirtualListMode::ContentVisibility.resolve(count, 1_000),
                VirtualListMode::ContentVisibility
            );
            assert_eq!(
                VirtualListMode::Windowed.resolve(count, 1_000),
                VirtualListMode::Windowed
            );
        }
    }

    #[test]
    fn the_default_is_auto_and_resolve_never_returns_it() {
        assert_eq!(VirtualListMode::default(), VirtualListMode::Auto);
        for count in [0, 1, 999, 1_000, 1_001, usize::MAX] {
            for mode in [
                VirtualListMode::Auto,
                VirtualListMode::ContentVisibility,
                VirtualListMode::Windowed,
            ] {
                assert_ne!(
                    mode.resolve(count, DEFAULT_AUTO_THRESHOLD),
                    VirtualListMode::Auto
                );
            }
        }
    }

    #[test]
    fn wire_spellings_match_message_scroller() {
        assert_eq!(
            VirtualListMode::ContentVisibility.as_str(),
            "content-visibility"
        );
        assert_eq!(
            VirtualListMode::ContentVisibility.as_str(),
            crate::message_scroller::Virtualization::ContentVisibility.as_str()
        );
        assert_eq!(VirtualListMode::Windowed.as_str(), "windowed");
        assert_eq!(VirtualListMode::Auto.as_str(), "auto");
    }
}
