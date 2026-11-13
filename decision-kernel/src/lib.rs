//! The decision kernel: the pure `decide` of each named behaviour, with no dependency and no `dioxus` or
//! bridge type in any signature (backlog row 509).
//!
//! A decision is `decide(config, memory, snapshot, event) -> Decision`. Three routes reach the same function:
//!
//! * a renderer with a page loads this crate built for `wasm32-unknown-unknown` (the `ffi` module's five fixed
//!   `extern "C"` exports) into the page, where the bridge's listener calls it synchronously inside the event's
//!   dispatch;
//! * a renderer with no page (Blitz) calls the kind's `decide` directly;
//! * `cargo test` runs both against the shared vectors.
//!
//! The `Config`, `Snapshot` and `Event` input types of a kind are the one definition of its wire keys
//! (`wire_input!`): the keys the Rust side reads, the event fields and attribute reads the page's adapter
//! collects and the kind table of the generated `behaviours.gen.ts` are all the same list. `kinds!` is the one
//! registration of a kind: it builds the kind table ([`KINDS`]), its hash ([`table_hash`], which the module exports as
//! `dx_kinds`) and the dispatch by kind index ([`decide`]).

pub mod decision;
pub mod fixture;
pub mod kinds;
pub mod value;
pub mod wire;

pub use decision::Decision;
pub use kinds::{decide, decide_named, table_hash, table_text, KindSpec, KINDS};

#[cfg(target_arch = "wasm32")]
mod ffi;
