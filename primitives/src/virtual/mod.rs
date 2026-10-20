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
//!   sticky edges and the grid pattern's key map.
//! - [`pin`]: keep the focused item rendered when it leaves the window, shared by the grids'
//!   active cell and `VirtualList`'s focused row.
//! - [`bridge`]: the DOM bridge every windowed component consumes (one passive scroll
//!   listener, one ResizeObserver, scope-owned cleanup, a render only when the window moves).
//!
//! Shared vs per component: backlog row 309.

mod bridge;
mod cv_chunks;
mod grid;
mod pin;
mod size_index;
pub(crate) mod types;
mod utils;
mod virtualizer;
mod window;

pub(crate) use bridge::{measured_px, use_virtual_bridge, Bridge};
pub(crate) use cv_chunks::{cv_chunks, ChunkSkip, CHUNK_ROWS};
pub(crate) use grid::{GridEngine, GridKey, GridShape};
pub(crate) use pin::{pin_in_flow, with_pinned};
pub(crate) use size_index::{SizeEstimates, DEFAULT_SIZE};
pub(crate) use types::ScrollReport;
pub(crate) use virtualizer::Engine;
// Not yet consumed within this lane: `default_range_extractor` reaches the module
// directly (`super::window::window`). Re-exported here per the shared-window-math API
// contract so the planned looping `CarouselVirtual` (a separate, concurrent lane) can
// call `crate::r#virtual::window(..., wrap: true)` without reaching into a submodule.
#[allow(unused_imports)]
pub(crate) use window::{window, WindowItem};
