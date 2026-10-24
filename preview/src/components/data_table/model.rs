//! The pure state machine and derivation behind `use_data_table` (backlog row
//! 340), which is `dioxus_primitives::data_view` (row 360): [`TableState`] is
//! `ViewState`, and `derive` is `derive_rows`, the index-returning pipeline
//! with the page's rows cloned into a [`TableView`]. Re-exported here under
//! this component's own names. Same `#[path]` sibling
//! wiring as `state.rs`.

pub use dioxus_primitives::data_view::{derive_rows as derive, RowCompare, TableState, TableView};
