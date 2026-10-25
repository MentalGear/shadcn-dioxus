//! The engines behind windowed lists.
//!
//! - [`window`]: the pure "which indices are in the window, with overscan" math, shared by
//!   `VirtualList` (`wrap: false`) and the looping carousel (`wrap: true`).
//! - [`cv_chunks`]: how rows are grouped for `content-visibility` skipping, shared by
//!   `VirtualList`'s content-visibility mode and `MessageScroller`.
//! - [`size_index`]: item positions from measured or estimated sizes, as a Fenwick tree
//!   (`O(log N)` measure and lookup).
//! - [`virtualizer`]: `VirtualList`'s windowed engine, plain Rust with no Dioxus types: the
//!   mounted range (quantised so a scroll re-renders only when it must), the frozen scroll
//!   canvas, and the scroll corrections for rows that resize above the viewport.
//! - [`grid`]: the second axis. `VirtualGrid`'s engine is one [`Engine`] per axis, plus the
//!   pinned edges and the grid pattern's key map.
//! - [`layout`]: pure photo-wall geometry (justified rows, two-axis windowing), and
//!   [`justified_engine`]: `VirtualJustifiedGrid`'s engine and spatial keyboard model on top of it.
//! - [`pin`]: keep the focused item rendered when it leaves the window, shared by the grids'
//!   active cell and `VirtualList`'s focused row.
//! - [`identity`]: item identity (`item_key`): the one `render_key`, the engine's key hash and where a
//!   remembered item is now (backlog row 361).
//! - [`mode`]: [`VirtualizeMode`] (`Auto | ContentVisibility | Windowed`) and its `auto_threshold`,
//!   the one "how are off-screen rows kept cheap" choice of `VirtualList`, `Table`'s data body
//!   and `VirtualGrid` (backlog row 343).
//! - [`find`]: the find-in-page layer of `hidden="until-found"` placeholders a windowed
//!   `VirtualList`, `VirtualGrid` and `VirtualWrapGrid` lay over their canvas (rows 153, 309).
//! - [`scroll_to`]: the public scroll-to-index handle ([`VirtualScroller`]) and the alignment math
//!   (backlog row 398).
//! - [`bridge`]: the DOM bridge every windowed component consumes (one passive scroll
//!   listener, one ResizeObserver, scope-owned cleanup, a render only when the window moves).
//!
//! Shared vs per component: backlog row 309.

mod bridge;
mod cv_chunks;
mod find;
mod grid;
mod identity;
mod justified_engine;
pub(crate) mod layout;
mod mode;
mod pin;
mod scroll_to;
mod size_index;
pub(crate) mod types;
mod utils;
mod virtualizer;
mod window;

pub(crate) use bridge::{use_virtual_bridge, Bridge};
pub(crate) use cv_chunks::{cv_chunks, ChunkSkip, CHUNK_ROWS};
pub(crate) use find::{parse_cell, use_find_in_page, FindLayer, FindLayout};
pub(crate) use grid::{GridEngine, GridKey, GridShape, Pins};
pub(crate) use identity::{engine_keys, engine_row_keys, render_key, resolve_index};
pub(crate) use justified_engine::{navigate as justified_navigate, JustifiedEngine, RowPlan};
pub(crate) use layout::justified::JustifiedOptions;
pub use mode::{VirtualizeMode, DEFAULT_AUTO_THRESHOLD};
pub(crate) use pin::{pin_in_flow, with_pinned};
pub(crate) use scroll_to::{scroll_dom_to, use_scroller_attach, ScrollRequest};
pub use scroll_to::{use_virtual_scroller, ScrollAlign, VirtualScroller};
pub(crate) use size_index::{SizeEstimates, DEFAULT_SIZE};
pub(crate) use types::ScrollReport;
pub(crate) use virtualizer::Engine;
// Not consumed inside `virtual`: `default_range_extractor` reaches the module
// directly (`super::window::window`). Re-exported here per the shared-window-math API
// contract so the looping `CarouselVirtualContent` can
// call `crate::r#virtual::window(..., wrap: true)` without reaching into a submodule.
pub(crate) use window::{window, WindowItem};
