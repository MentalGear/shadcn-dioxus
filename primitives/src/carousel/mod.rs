//! Defines the [`Carousel`] component and its sub-components.
//!
//! # Scope
//!
//! Specified by `dev-docs/archive/research/carousel-2026-09-19.md`: the APG
//! "basic" (prev/next) style
//! (`carousel-1-prev-next` in the vendored reference), plus the opt-in
//! extensions the research filed (§8.2 infinite `loop`ing, §8.3 autoplay
//! with a rotation control and its `aria-live` region, §8.5 the tablist
//! picker, composed on the same focus engine as [`crate::tabs`]). The APG
//! basic style has no rotation requirement at all, so a manually-paged
//! carousel is fully conformant without autoplay (§1.2).
//!
//! # Engine
//!
//! CSS scroll-snap, not a ported physics engine and not a JS dependency
//! (research §8.1/§2.2): [`CarouselContent`] is a native
//! `overflow-{x,y}: auto; scroll-snap-type` track; [`CarouselItem`]s are
//! `scroll-snap-align` children. Paging (buttons, keyboard) computes the
//! physical pixel delta between the target item's own
//! `getBoundingClientRect()` and the scroller's, per axis, and calls
//! `scroller.scrollBy({ left | top: delta })` on the scroller itself --
//! never `Element.scrollIntoView()` on the target (backlog row 91 holds the
//! reasoning behind that choice: `scrollIntoView` walks and
//! scrolls *every* scrollable ancestor between the target and the
//! viewport, including the document, so autoplay/Next would drag the whole
//! page back to the carousel whenever it was scrolled elsewhere --
//! `scrollBy`, called on the scroller directly, only ever writes that one
//! element's own scroll position). No manual RTL sign-flip step is needed
//! either: `getBoundingClientRect`/`scrollBy`'s delta are both physical
//! (per research §2.2's RTL paragraph and
//! `dev-docs/archive/research/carousel-overscroll-2026-09-23.md` §6 invariant 1),
//! so "the next slide" resolving to the correct physical position under
//! `dir="rtl"` falls out of the same physical-delta construction the drag
//! gesture below already uses, rather than needing its own. A second,
//! independent JS bridge (`use_carousel_scroll_tracking`, a private
//! helper) keeps `CarouselContext`'s own `selected` correct after a
//! *native* drag/wheel/trackpad scroll the caller's own buttons/keyboard
//! never drove. A third (`use_carousel_drag`, also private -- see
//! [`CarouselContent`]'s own "Pointer drag" doc) adds a mouse/pen
//! drag-to-scroll gesture on top of the *same* track, moving it with
//! plain `scrollBy` calls rather than a parallel transform-based engine;
//! every `scrollBy` it issues is native scrolling as far as the browser
//! (and the second bridge above) is concerned, so dragging is never a
//! second source of truth for `selected`. Release settles the track with
//! the same scroller-only `scrollBy`-by-delta paging mechanism
//! buttons/keyboard already use, not a hand-rolled offset -- see
//! [`CarouselContent`]'s own "Release settle" doc. All three are
//! legacy `document::eval` call sites (ported by row 434), and
//! all three no-op harmlessly off a real document (native/Blitz, or a
//! plain `cargo test`), as the other un-gated eval helpers in `crate::lib`
//! do: this module's rendered markup never branches on `feature = "web"`
//! at all, so unlike the native-`<dialog>` overlays there is nothing here
//! for a `feature = "web"` gate to protect.
//!
//! # Accessibility
//!
//! Follows `dev-docs/archive/research/carousel-2026-09-19.md` §1.2/§5.2's account
//! of the APG Carousel pattern's basic style: [`Carousel`] is
//! `role="region"` + `aria-roledescription="carousel"` with a **required**
//! accessible name (a `tracing::warn!` when neither `aria-label` nor
//! `aria-labelledby` is supplied -- shadcn's own `carousel.tsx` leaves this
//! entirely to the caller, undocumented; see the research's §1.4 gap
//! table), each [`CarouselItem`] is `role="group"` +
//! `aria-roledescription="slide"` with a default `"{n} of {m}"` label
//! (APG's own sanctioned exception to "don't put position/size in an
//! accessible name," since `group` supports neither `aria-setsize` nor
//! `aria-posinset`), and [`CarouselPrevious`]/[`CarouselNext`] are native
//! `<button>`s, genuinely `disabled` at the ends (unless `loop` is on, the
//! ends are real, reachable boundaries, not merely visual ones). A root-level
//! `ArrowLeft`/`ArrowRight` (or `ArrowUp`/`ArrowDown` for
//! [`CarouselOrientation::Vertical`]) key is a Radix/shadcn-style
//! addition, not an APG requirement (research §1.2/§1.5) -- RTL-swapped
//! via [`crate::direction`], Radix's convention rather than shadcn's own
//! (shadcn's `handleKeyDown` never swaps; it relies on embla's engine
//! silently compensating underneath it, which this crate does not have).

use crate::sleep::sleep;
use crate::{
    collection::{collection_item, use_collection_provider, use_item, CollectionState},
    direction::{use_direction, use_direction_signal, Direction, HorizontalNav},
    fold_style_attributes, has_own_accessible_name, merge_attributes, use_controlled,
    use_effect_cleanup, use_id_or, use_previous, use_unique_id,
};
use dioxus::prelude::*;
use dioxus_attributes::attributes;
use dioxus_core::Task;
use std::time::Duration;

#[macro_use]
mod js_macros;

mod autoplay;
mod content;
mod context;
mod drag;
mod helpers;
mod indicators;
mod item;
mod nav;
mod options;
mod root;
mod scroll;
mod virtual_content;
mod wheel;

#[cfg(test)]
mod ssr_tests;
#[cfg(test)]
mod tests;

pub use self::autoplay::*;
pub use self::content::*;
pub use self::indicators::*;
pub use self::item::*;
pub use self::nav::*;
pub use self::options::*;
pub use self::root::*;
pub use self::virtual_content::*;

use self::context::*;
use self::drag::*;
use self::helpers::*;
use self::scroll::*;
use self::wheel::*;
