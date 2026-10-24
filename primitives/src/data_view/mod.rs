//! One pure, render-free view model for every collection that shows rows
//! (backlog row 360, `dev-docs/research/collection-model-2026-10-22.md`).
//!
//! Row identity is a key function; [`ViewState`] is the flat state a view
//! remembers (sort, filter, page, selection, manual order) with pure
//! transitions; [`derive`] is the one pipeline `filter -> sort or manual order
//! -> page` and returns indices and keys, never cloned rows. [`Field`] says how
//! a row sorts and which text the filter matches. Nothing here renders, owns a
//! signal or needs a cargo feature (std only), so the Data Table hook, a list
//! or a virtual grid can sit on the same model and the same rules.
//!
//! The text matcher every substring filter shares is [`contains_query`].

mod derive;
mod field;
mod stages;
mod state;
mod text;

#[cfg(test)]
mod laws;

pub use derive::{derive, derive_by, derive_rows, RowCompare, TableView, View};
pub use field::Field;
pub use stages::{filter_rows, paginate, sort_rows, sort_rows_by, tri_state, SortDirection};
pub use state::{KeyOrder, RowAnchor, TableState, ViewChanges, ViewState};
pub use text::{contains_query, contains_query_normalized, normalize_query};
