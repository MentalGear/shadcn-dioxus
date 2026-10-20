//! The one `sleep` the primitives use: `tokio::time::sleep` on native, `gloo_timers` on wasm.
//!
//! It replaces `dioxus_sdk_time::sleep` (the only item used from that crate), which is exactly this
//! pair of calls. Awaited inside a `spawn`ed task, so the timer is dropped with the owning scope and
//! there is no JS `setTimeout` for anything to forget to clear.

use std::time::Duration;

/// Waits for `duration`.
pub async fn sleep(duration: Duration) {
    #[cfg(not(target_family = "wasm"))]
    tokio::time::sleep(duration).await;

    #[cfg(target_family = "wasm")]
    gloo_timers::future::sleep(duration).await;
}
