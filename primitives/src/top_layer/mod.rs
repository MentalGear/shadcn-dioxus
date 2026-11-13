//! Top-layer wiring for non-modal overlays — the `popover` attribute.
//!
//! Concept ported from `dignifiedquire/dx-components` (MIT OR Apache-2.0),
//! `primitives/src/top_layer.rs` @ 5af3cc292559a0e8d73c7b9a827c4ca08ef34d99 (ref-ok: path in the upstream fork repo, not this one)
//! (`use_top_layer`, `TopLayerKind`). Adapted: the DOM calls are legacy
//! `document::eval` scripts (ported by row 434, `dev-docs/js-interop.md`)
//! instead of `wasm_bindgen`/`web_sys` — this crate has neither as
//! a dependency, and `dev-docs/archive/fork-research/lifting-from-forks.md` §4 says not to take dependency
//! edits when a lift can avoid them. Narrowed to the Popover API only; the
//! fork's `TopLayerKind::DialogModal` arm is dropped, since native
//! `<dialog>` + `showModal()` is the modal engine's job (`dialog.rs`), not
//! this module's.
//!
//! Two WHATWG HTML spec sections define everything below:
//! - The `popover` attribute and its `auto`/`manual` states:
//!   <https://html.spec.whatwg.org/multipage/popover.html#the-popover-attribute>
//! - Light dismiss (Escape, outside pointerdown, closing sibling `auto`
//!   popovers) for `popover="auto"`:
//!   <https://html.spec.whatwg.org/multipage/popover.html#popover-light-dismiss>
//! - `showPopover()`/`hidePopover()` and the `toggle` event fired on every
//!   state change (browser- or script-driven alike):
//!   <https://html.spec.whatwg.org/multipage/popover.html#dom-showpopover>,
//!   <https://html.spec.whatwg.org/multipage/popover.html#dom-hidepopover>

#[cfg(not(feature = "native"))]
use dioxus::document;
// Unconditional, unlike the rest of this block: `anchored_trigger_attributes`
// (below) is called by every anchored trigger on BOTH arms -- mirroring
// `anchor_name_style`, see its own doc for why -- and needs `Attribute` in
// scope, plus whatever `dioxus_attributes::attributes!`'s expansion resolves
// against (`dioxus_core`/`dioxus_elements`, brought in by this same glob),
// regardless of the arm. A plain glob import is never itself an error when
// some of what it would bring in is `#[cfg]`'d away, so this costs the
// browser arm nothing.
use dioxus::prelude::*;

mod anchor_fallback;
mod attributes;
mod popover_sync;
mod scroll_clamp;
mod styles;

#[cfg_attr(
    feature = "native",
    expect(unused_imports, reason = "only the browser build uses it")
)]
pub(crate) use anchor_fallback::*;
pub(crate) use attributes::*;
#[cfg_attr(
    feature = "native",
    expect(unused_imports, reason = "only the browser build uses it")
)]
pub(crate) use popover_sync::*;
#[cfg_attr(
    all(feature = "native", not(test)),
    expect(unused_imports, reason = "only the browser build and the tests use it")
)]
pub(crate) use scroll_clamp::*;
pub(crate) use styles::*;

#[cfg(all(test, not(feature = "native")))]
mod tests;

/// `anchored_trigger_attributes` is NOT `not(feature = "native")`-gated (see its own
/// doc for why: every trigger calls it unconditionally, on both arms), so
/// unlike every other test in this module it is meaningful -- and needed --
/// on the native arm too (`cargo test --features native` runs it; the default
/// browser arm does not). This covers the `anchor.is_empty()` arms of its
/// `match` that `not(feature = "native")` can never reach (`anchor_name_style`
/// always returns a real, non-empty string there) -- the browser-arm folding
/// behavior itself is covered by `tests::trigger_*` above.
///
/// `feature = "native"`, not just `test`: on the browser arm,
/// `anchor_name_style` stops being empty and these exact assertions would
/// fail -- this module is only meaningful on the arm its own name says.
#[cfg(all(test, feature = "native"))]
mod trigger_attributes_native_tests;

#[cfg(test)]
mod viewport_tests;
