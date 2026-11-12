//! The decision kernel: the pure `decide` of each named behaviour, with no dependency and no `dioxus` or
//! bridge type in any signature.
//!
//! Three routes reach the same functions. A renderer with a page loads the crate built for
//! `wasm32-unknown-unknown` (the `ffi` module's `extern "C"` exports) into the page, where the bridge's listener
//! calls it synchronously inside the event's dispatch. A renderer with no page calls [`fixture::decide`]
//! directly. `cargo test` runs both against the shared vectors.
//!
//! A decision is `decide(config, memory, snapshot, event) -> Decision`: [`value::Value`] carries the four
//! inputs across the boundary in one request ([`wire`]), and the [`fixture::Decision`] comes back the same way.

pub mod fixture;
pub mod value;
pub mod wire;

#[cfg(target_arch = "wasm32")]
mod ffi;
