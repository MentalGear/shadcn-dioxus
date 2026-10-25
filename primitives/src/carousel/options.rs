#[cfg(doc)]
use super::*;

/// The axis a [`Carousel`] pages along.
///
/// Matches [`crate::resizable::ResizableDirection`]'s own
/// horizontal/vertical shape and `data-orientation` convention. Vertical
/// is the same component with an axis-swapped CSS declaration and
/// scroll-delta axis, and `ArrowUp`/`ArrowDown` in place of the RTL-aware
/// `ArrowLeft`/`ArrowRight` pair, so it is supported alongside the
/// horizontal shape -- see the module doc's "Scope" section.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CarouselOrientation {
    /// Slides are arranged left-to-right (or right-to-left under RTL) and
    /// paged with `ArrowLeft`/`ArrowRight`.
    #[default]
    Horizontal,
    /// Slides are arranged top-to-bottom and paged with
    /// `ArrowUp`/`ArrowDown` -- never RTL-aware (see the private
    /// `carousel_key_intent`, this module's own key-resolution function).
    Vertical,
}

impl CarouselOrientation {
    /// Returns `"horizontal"` or `"vertical"` -- the exact string this
    /// module's `data-orientation` styling hook uses.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }
}

/// Where each [`CarouselItem`] snaps to rest against
/// [`CarouselContent`]'s own scrollport -- matches Embla's (and shadcn's
/// own `opts={{ align }}`) three-way `start`/`center`/`end`, and maps
/// directly onto the CSS Scroll Snap spec's own `scroll-snap-align`
/// keywords of the same names (this crate's own [`CarouselItem`] sets
/// `scroll-snap-align` to whichever one is in force -- see
/// [`Self::as_str`]).
///
/// Every place this module computes "where a slide rests" -- the explicit
/// `scrollBy`-by-delta paging call ([`CAROUSEL_SCROLL_TO_JS`]), the
/// "which slide is nearest" search a native scroll/drag release settles
/// to ([`CAROUSEL_SCROLL_TRACKING_JS`]/[`CAROUSEL_DRAG_JS`]) -- reads this
/// value and anchors on the matching point (an item/the scrollport's own
/// leading edge, midpoint, or trailing edge) rather than always the
/// leading edge, so a caller who sets `align: CarouselAlign::Center` (for
/// example) gets a *consistent* rest position regardless of whether the
/// user paged there with a button, the keyboard, a drag, or native
/// wheel/trackpad/touch scrolling (the last of which the browser itself
/// already snaps correctly from `scroll-snap-align` alone -- this value
/// only needs to reach this module's own JS so its explicit paging/nearest-
/// slide logic agrees with what the browser will have already done
/// natively).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CarouselAlign {
    /// The item's leading edge rests against the scrollport's leading
    /// edge -- shadcn's own default (`opts={{ align: "start" }}` in every
    /// one of its demos that sets `align` explicitly, and Embla's
    /// documented default even where a demo omits the option entirely).
    #[default]
    Start,
    /// The item's midpoint rests against the scrollport's midpoint.
    Center,
    /// The item's trailing edge rests against the scrollport's trailing
    /// edge.
    End,
}

impl CarouselAlign {
    /// Returns `"start"`, `"center"`, or `"end"` -- both this module's own
    /// JS anchor-point argument and the exact keyword [`CarouselItem`]'s
    /// `scroll-snap-align` uses, so a single value serves both without a
    /// second mapping to keep in sync.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Center => "center",
            Self::End => "end",
        }
    }
}

/// How [`CarouselProps::r#loop`] wraps at the ends, for content this
/// module cannot seamlessly loop on its own.
///
/// [`CarouselVirtualContent`] with virtualisation genuinely active is
/// unaffected by this enum entirely -- `loop` there is *already* the
/// seamless illusion (see that component's own "Seamless loop" doc): one
/// already-mounted window slide slides physically forward/backward, never
/// a visible rewind across every intervening slide. That path has no
/// equivalent construction available to it at all -- there is no cloned-
/// node illusion here (the overscroll port's own invariant 5) and no
/// windowed-anchor bookkeeping for a fixed `CarouselItem` sequence to
/// piggyback on.
///
/// For everything else -- the plain children API
/// ([`CarouselContent`]/[`CarouselItem`]), and [`CarouselVirtualContent`]
/// with `virtualize: Some(false)` (or a data set too small to auto-window)
/// -- this is what decides whether `r#loop: true` does anything at all:
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LoopMode {
    /// `r#loop: true` alone has NO effect for the content types above --
    /// [`CarouselPrevious`]/[`CarouselNext`] stay genuinely `disabled` at
    /// the real ends, exactly as if `r#loop` were `false`. Default,
    /// because it is the least surprising choice for the one content type
    /// this crate CAN loop seamlessly ([`CarouselVirtualContent`],
    /// windowed): a single `loop_mode` prop that defaults to "the good
    /// version, where one exists" reads correctly regardless of which
    /// content a caller ends up using, without secretly picking a
    /// visibly-worse fallback (a long rewind scroll) they never asked for.
    #[default]
    Seamless,
    /// The explicit opt-in for a visible **rewind**: from the last slide,
    /// Next goes to the first (and vice versa for Previous) via an
    /// INSTANT jump (not the smooth, scroll-across-every-intervening-slide
    /// transition every other single-step paging call uses) -- matching
    /// the APG reference implementation's own basic-style example, and
    /// avoiding what would otherwise be a multi-second scroll through an
    /// entire large data set just to wrap once. No cloned edge slides are
    /// ever added (same reasoning as [`Self::Seamless`]'s own doc).
    Rewind,
}
