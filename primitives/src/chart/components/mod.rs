//! Component definitions for the chart primitive -- the Dioxus-facing
//! layer built on [`super::engine`] (see the parent module's doc for the
//! engine/components seam and why it exists).
//!
//! `layout` (shared Cartesian plot geometry) is a private module: nothing
//! in it is part of this crate's public API (its own doc explains why),
//! so it is reached only via `super::layout` from `chart`/`series::*`,
//! never re-exported. `series` (one file per mark family) IS `pub`, since
//! every family's `*Options` struct is real, caller-facing API -- see its
//! own module doc for the extension-point ownership map.
//!
//! Glob re-exported: a new `pub` item in a file under this tree (e.g.
//! `tooltip.rs`) needs no edit here to expose it -- see `series`'s own doc
//! for the structure of the per-family files.

mod center_text;
pub mod chart;
pub mod container;
mod fragment;
mod layout;
pub mod legend;
mod pointer;
pub mod series;
pub mod tooltip;

pub use chart::*;
pub use container::*;
pub use legend::*;
pub use series::*;
pub use tooltip::*;
