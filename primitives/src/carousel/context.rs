use super::*;

/// Shared state and actions, provided by [`Carousel`] and consumed by
/// every sub-component plus [`use_carousel`].
#[derive(Clone, Copy)]
pub(super) struct CarouselContext {
    pub(super) orientation: ReadSignal<CarouselOrientation>,
    /// Where each slide rests against the scrollport -- see
    /// [`CarouselAlign`]'s own doc. Read by [`CarouselItem`]/
    /// [`CarouselVirtualContent`] for `scroll-snap-align`, and by every
    /// JS bridge that computes a paging delta or a "nearest slide" search
    /// ([`CAROUSEL_SCROLL_TO_JS`], [`CAROUSEL_SCROLL_TRACKING_JS`],
    /// [`CAROUSEL_DRAG_JS`]).
    pub(super) align: ReadSignal<CarouselAlign>,
    /// Always in `[0, count)` (or `0` if `count == 0`) -- see
    /// [`clamp_selected`]. The single source of truth every sub-component
    /// reads instead of re-deriving its own notion of "which slide is
    /// current."
    pub(super) selected: Memo<usize>,
    pub(super) set_selected: Callback<usize>,
    /// Number of currently-registered [`CarouselItem`]s. Empty
    /// (`count == 0`) until the first render's effects have run --
    /// deliberately: see [`CarouselItem`]'s doc, "SSR stability."
    pub(super) count: Memo<usize>,
    /// `index -> DOM id`, grown as each [`CarouselItem`] registers (the
    /// same grow-only-`Vec`-by-index idiom `tabs.rs`'s own
    /// `tab_content_ids` uses for the identical reason: SSR and a
    /// pre-effect client render must agree, and both start from an empty
    /// `Vec`).
    pub(super) item_ids: Signal<Vec<String>>,
    /// [`CarouselContent`]'s own rendered id, published by itself so
    /// [`CarouselPrevious`]/[`CarouselNext`] can point `aria-controls` at
    /// it (mirrors the `trigger_id`-publishing construction
    /// `dev-docs/conformance-harness.md`'s tier-2 Rule 14 describes for
    /// `SelectTrigger`/`DropdownMenuTrigger`/etc.).
    pub(super) content_id: Signal<String>,
    /// Whether `loop`ing is enabled -- see `CarouselProps::r#loop`. This
    /// is the raw prop value; [`Self::loop_wraps`] is what every consumer
    /// that decides "does the boundary actually wrap" should read instead.
    pub(super) loop_enabled: ReadSignal<bool>,
    /// See [`CarouselProps::loop_mode`].
    pub(super) loop_mode: ReadSignal<LoopMode>,
    /// See [`CarouselProps::drag_free`].
    pub(super) drag_free: ReadSignal<bool>,
    /// Drag-free only: the index the track's own scroll settle just selected.
    /// The scroll-into-view effect skips exactly that change (the user
    /// already put the track there, and "scrolling to" the slide would snap a
    /// free position onto its edge); every other source clears it first.
    pub(super) scroll_origin: Signal<Option<usize>>,
    /// Published (`true`) by [`CarouselVirtualContent`] exactly when its
    /// own seamless-loop path (windowed AND `r#loop` on) is genuinely
    /// active -- see that component's own "Seamless loop" doc. Always
    /// `false` for the plain children API, which has no such path.
    /// [`Self::loop_wraps`] is what actually reads this.
    pub(super) virtualized_loop_active: Signal<bool>,
    /// The resolved text direction -- the exact same value [`Carousel`]'s
    /// own root already computed via `use_direction(props.dir)`, republished
    /// here so [`CarouselIndicators`]/[`CarouselIndicator`] reuse it verbatim rather
    /// than calling `use_direction(None)` a second time, which would
    /// silently disagree with the root's own resolution whenever a caller
    /// passes an explicit `dir` prop on [`Carousel`] itself rather than
    /// relying on an ambient [`crate::direction::DirectionProvider`].
    pub(super) direction: ReadSignal<Direction>,
    /// Autoplay/rotation-control state -- always present, every field
    /// initialized to an inert "no autoplay" value, so [`CarouselContent`]
    /// can read `autoplay.present`/`autoplay.rotating` unconditionally for
    /// its own `aria-live` whether or not a [`CarouselAutoplay`] child
    /// actually exists. The same "publish into a shared, always-initialized
    /// slot" idiom `content_id` above already uses.
    pub(super) autoplay: AutoplayContext,
    /// Published (`true`) by [`CarouselIndicators`] on mount so
    /// [`CarouselItem`] can switch its own role from `group` to
    /// `tabpanel` -- see that component's own "Tablist variant" doc.
    pub(super) tablist_present: Signal<bool>,
    /// `[start, end]` (inclusive) item indices actually inside
    /// [`CarouselContent`]'s own viewport *at rest* -- the a11y contract's
    /// own "visible" set (`dev-docs/archive/research/carousel-loop-2026-09-25/loop-a11y-guidance.md`
    /// §7, `dev-docs/archive/research/carousel-loop-2026-09-25/loop-libraries.md`'s cross-library convergence table).
    /// Defaults to `(0, 0)` -- deterministic across SSR and a client's own
    /// pre-hydration first render, matching [`Self::selected`]'s own
    /// forced-`0`-until-registration value (see [`CarouselItem`]'s own
    /// "Visible slides / `inert`" doc for why this makes the two agree at
    /// first render without either needing a special case). Updated only
    /// by [`use_carousel_scroll_tracking`]'s own settle (on `scrollend`/the
    /// debounced-`scroll` fallback, and on a `ResizeObserver` firing for
    /// the scroller's own box -- see that constant's own doc for why a
    /// resize matters here too) -- never mid-drag, mid-momentum, or
    /// mid-smooth-page (the same "freeze" rule already applies to
    /// [`Self::selected`]; a drag's own release settle reaches this
    /// signal only indirectly, through the ordinary native
    /// scroll/scrollend event its release `scrollBy` produces once
    /// `data-dragging` is gone).
    pub(super) visible_range: Signal<(usize, usize)>,
    /// The index of the [`CarouselItem`] that currently contains DOM
    /// focus, if any -- maintained purely by each item's own
    /// `onfocusin`/`onfocusout` (native, synchronous; no `document::eval`
    /// involved). Read once, synchronously, by the focus-redirect effect
    /// below whenever [`Self::selected`] changes, to decide whether the
    /// slide that just lost "current" status (and is about to -- or just
    /// did -- become `inert`) held focus at the moment of the transition.
    pub(super) focused_item: Signal<Option<usize>>,
}

impl CarouselContext {
    /// Whether a boundary genuinely wraps -- what every consumer that
    /// decides "is Previous/Next `disabled`" or "does `step_prev`/
    /// `step_next` wrap" should read, instead of the raw [`Self::loop_enabled`].
    ///
    /// `false` unless `r#loop` is on AND (this is [`CarouselVirtualContent`]'s
    /// own already-seamless windowed path, OR the caller explicitly opted
    /// into [`LoopMode::Rewind`]) -- see [`LoopMode`]'s own doc for why
    /// [`LoopMode::Seamless`] (the default) is a no-op everywhere else.
    pub(super) fn loop_wraps(&self) -> bool {
        (self.loop_enabled)()
            && ((self.loop_mode)() == LoopMode::Rewind || (self.virtualized_loop_active)())
    }

    /// Whether [`CarouselPrevious`] is genuinely `disabled` right now. The
    /// one definition: the button's `disabled` attribute, its focus
    /// hand-off ([`use_nav_focus_hand_off`]) and its blur guard all read it,
    /// so they cannot disagree.
    pub(super) fn prev_disabled(&self) -> bool {
        !self.loop_wraps() && !can_scroll_prev((self.selected)())
    }

    /// The [`CarouselNext`] mirror of [`Self::prev_disabled`].
    ///
    /// While no slide has registered (`count == 0`: SSR, the hydrating first render and the first
    /// client frame) the answer is "not disabled": the count is unknown, not zero, and a disabled
    /// guess would paint Next at the 0.5 disabled opacity and then fade it up to 1 after the items
    /// register (backlog row 353). The optimistic pose is
    /// the right one for every carousel that has slides; a one-slide carousel settles to disabled
    /// once its item registers.
    pub(super) fn next_disabled(&self) -> bool {
        let count = (self.count)();
        !self.loop_wraps() && count > 0 && !can_scroll_next((self.selected)(), count)
    }
}

/// Keeps keyboard focus from being stranded when a focused
/// [`CarouselPrevious`]/[`CarouselNext`] becomes `disabled` -- returns the
/// `(onfocusin, onfocusout)` handlers the button must wire.
///
/// **The failure it prevents.** Pressing `Enter`/`Space` on Next to reach the last
/// slide disables Next *while it holds focus*. Chromium then runs its focus
/// fixup and drops focus to `<body>`, so the next `Tab` starts from the top of
/// the page; a browser that skips the fixup leaves focus on the now-dead
/// button, where no key reaches it. Either way a keyboard user
/// loses their place (WCAG 2.4.3). It is the same class as the slide that
/// becomes `inert` while focused (`Carousel`'s own focus-safety effect), and
/// the same remedy applies: hand focus to the carousel's own scroller
/// ([`CarouselContent`], always focusable and never disabled or hidden),
/// where `ArrowLeft`/`ArrowRight` keep paging, instead of the other arrow
/// (which can itself be disabled, or absent, on a one-slide carousel). This
/// is also what keeps the arrow safe to hide (`nav_disabled_opacity: 0.0`
/// renders it `visibility: hidden`, which would otherwise strand focus the
/// same way).
///
/// **Why a flag and not `document.activeElement`.** By the time the effect
/// runs the browser may already have moved focus, so the button's own
/// `focusin`/`focusout` keep a Rust-side flag, and `focusout` ignores a blur
/// that arrives while the button is already disabled (that blur is the
/// browser's fixup, not the user leaving). A mouse click that never focused
/// the button (Safari, Firefox on macOS) leaves the flag `false`, so nothing
/// moves.
pub(super) fn use_nav_focus_hand_off(
    ctx: CarouselContext,
    is_disabled: fn(&CarouselContext) -> bool,
) -> (Callback<Event<FocusData>>, Callback<Event<FocusData>>) {
    let mut focused = use_signal(|| false);
    use_effect(move || {
        let disabled = is_disabled(&ctx);
        if !disabled || !*focused.peek() {
            return;
        }
        focused.set(false);
        let target = ctx.content_id.peek().clone();
        if target.is_empty() {
            return;
        }
        document::eval(&format!(
            "var root = document.getElementById('{target}'); \
             if (root) {{ root.focus({{ preventScroll: true }}); }}"
        ));
    });
    let onfocusin = use_callback(move |_: Event<FocusData>| focused.set(true));
    let onfocusout = use_callback(move |_: Event<FocusData>| {
        if !is_disabled(&ctx) {
            focused.set(false);
        }
    });
    (onfocusin, onfocusout)
}

/// Autoplay/rotation-control state, grouped out of [`CarouselContext`]
/// only for readability -- every field lives for the whole [`Carousel`]'s
/// lifetime regardless of whether a [`CarouselAutoplay`] is ever mounted.
///
/// ## The pause/resume state machine
///
/// `rotating` (the one field every other component actually reads) is
/// `true` only when ALL of the following hold: `playing` (the user's own
/// high-level intent, toggled only by [`CarouselRotationControl`]'s click
/// handler), NOT `resume_blocked`, NOT `focus_within`, and NOT (`hover`
/// AND `stop_on_mouse_enter`). Two different "pause" shapes are
/// deliberate, not an oversight, both read directly from the vendored
/// tabbed reference's own accessibility-features prose
/// (`examples/carousel-2-tablist.html`, "Controlling Automatic Slide
/// Rotation"), not the basic reference's own JS (which has a documented
/// latch bug on this exact point, `dev-docs/archive/research/carousel-2026-09-19.md`
/// §1.3 point 3 -- deliberately not ported):
///
/// - **Hover** clears freely: `hover` going back to `false` alone can make
///   `rotating` `true` again ("Automatic rotation resumes when the mouse
///   moves away... unless another condition... has been triggered").
/// - **Focus** is sticky: `focus_within` going back to `false` does *not*
///   by itself resume anything, because `resume_blocked` (set the instant
///   focus enters, alongside `focus_within`) stays `true` until
///   [`CarouselRotationControl`] is clicked again ("Automatic rotation
///   only resumes if the user explicitly activates the ... button").
///
/// [`CarouselRotationControl`]'s own click handler always clears
/// `resume_blocked` (whichever direction it toggles `playing`), matching
/// the same reference's own "if a user activates the rotation control
/// button ... it is assumed the user wants auto-rotation to start
/// immediately."
#[derive(Clone, Copy, PartialEq)]
pub(super) struct AutoplayContext {
    /// `true` once a [`CarouselAutoplay`] has mounted at least once.
    pub(super) present: Signal<bool>,
    /// The user's own high-level intent -- see this struct's own doc.
    pub(super) playing: Signal<bool>,
    pub(super) hover: Signal<bool>,
    pub(super) focus_within: Signal<bool>,
    pub(super) resume_blocked: Signal<bool>,
    /// Mirrors [`CarouselAutoplayProps::stop_on_interaction`]; published
    /// here (not read directly off the props) so [`AutoplayContext::note_interaction`]
    /// can be called from components (`CarouselPrevious`/`CarouselNext`/
    /// `CarouselIndicator`/[`use_carousel`]) that have no direct access to
    /// [`CarouselAutoplay`]'s own props.
    pub(super) stop_on_interaction: Signal<bool>,
    /// Mirrors [`CarouselAutoplayProps::stop_on_mouse_enter`]; see
    /// `stop_on_interaction`'s own doc for why this is republished here.
    pub(super) stop_on_mouse_enter: Signal<bool>,
    /// `true` exactly when the timer should be actively ticking -- see
    /// this struct's own "pause/resume state machine" doc.
    pub(super) rotating: Memo<bool>,
}

impl AutoplayContext {
    /// Stop autoplay for good (until [`CarouselRotationControl`] is
    /// explicitly clicked again) if a [`CarouselAutoplay`] is present with
    /// `stop_on_interaction` enabled (its own default `true`) -- called
    /// from a pointer press on the track (the `onpointerdown` of
    /// [`CarouselContent`]'s viewport: touch, pen or mouse, the start of a
    /// swipe or drag) and from every manual paging entry point:
    /// [`CarouselPrevious`]/[`CarouselNext`], [`Carousel`]'s own root
    /// keyboard handler, [`CarouselIndicator`], and [`CarouselApi::scroll_to`].
    /// Mirrors embla-carousel's own `embla-carousel-autoplay` plugin
    /// option of the same name, which stops on embla's pointer down.
    ///
    /// Not called from the scroll-tracking bridge (`use_carousel_scroll_tracking`)
    /// or the wheel band: a scroll that has no press behind it (wheel,
    /// trackpad, keyboard scrolling of the track) is not an interaction,
    /// as in embla. See [`CarouselAutoplayProps::stop_on_interaction`]'s
    /// own doc.
    pub(super) fn note_interaction(&self) {
        if (self.present)() && (self.stop_on_interaction)() {
            let mut playing = self.playing;
            playing.set(false);
        }
    }
}
