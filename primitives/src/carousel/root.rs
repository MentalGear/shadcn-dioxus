use super::*;

/// The props for the [`Carousel`] component.
#[derive(Props, Clone, PartialEq)]
pub struct CarouselProps {
    /// The axis the carousel pages along. Defaults to
    /// [`CarouselOrientation::Horizontal`].
    #[props(default)]
    pub orientation: ReadSignal<CarouselOrientation>,

    /// Where each slide rests against the scrollport. Defaults to
    /// [`CarouselAlign::Start`] -- matches shadcn's own default and every
    /// one of its demos (`opts={{ align: "start" }}`, backlog row 91's shadcn-parity research).
    /// See [`CarouselAlign`]'s own doc for what changes with `Center`/`End`.
    #[props(default)]
    pub align: ReadSignal<CarouselAlign>,

    /// The controlled selected slide index (0-based). `None` for
    /// uncontrolled use (see [`Self::default_value`]).
    pub value: ReadSignal<Option<usize>>,

    /// Whether Previous/Next (and the root's own `ArrowLeft`/`ArrowRight`)
    /// wrap around at the ends at all. Whether that wrap is the seamless
    /// [`CarouselVirtualContent`] illusion or a visible instant rewind --
    /// or, for content [`LoopMode`] has no seamless path for, whether it
    /// happens at all -- is [`Self::loop_mode`]'s decision; see that
    /// prop's own doc. No cloned edge slides are ever added under either
    /// mode -- they would violate the overscroll port's own invariant 5
    /// (they'd enter the snap engine's candidate list, the "N of M" slide
    /// count, and `:nth-child` styling). Dragging or wheeling past a
    /// physical edge still rubber-bands (drag: mode B3, wheel: mode D) regardless of this flag
    /// -- there is no wrap on a drag/wheel gesture, only on Previous/Next/
    /// the root keyboard. Defaults to `false` (matches shadcn's own
    /// `opts={{ loop: false }}` default). See
    /// backlog row 91 / `dev-docs/archive/research/carousel-2026-09-19.md` §8.2.
    #[props(default)]
    pub r#loop: ReadSignal<bool>,

    /// Free scrolling (Embla `dragFree`): the track drops `scroll-snap-type`
    /// and a released mouse/pen drag coasts on its release velocity instead
    /// of settling on a slide (touch and trackpad coast natively once the
    /// snap is gone). Previous/Next and the keyboard still page by slide and
    /// land on a slide edge, `selected` is still the slide nearest the
    /// anchor, and variable-width items (`basis-auto`) work. One switch: the
    /// prop renders `data-drag-free` and `data-snap` (the track's
    /// `scroll-snap-type` value, also what the inline style carries) on
    /// [`CarouselContent`], and the script's snap restores read `data-snap`
    /// -- no second code path decides snap vs. no snap. Reduced motion: the
    /// release does not coast (the track stays where it was let go). Not
    /// supported by [`CarouselVirtualContent`] (ignored there). Defaults to
    /// `false`.
    #[props(default)]
    pub drag_free: ReadSignal<bool>,

    /// How `r#loop` wraps at the ends -- see [`LoopMode`]'s own doc.
    /// Defaults to [`LoopMode::Seamless`].
    #[props(default)]
    pub loop_mode: ReadSignal<LoopMode>,

    /// The initial selected slide index when uncontrolled.
    #[props(default)]
    pub default_value: usize,

    /// Called whenever the selected slide settles on a new index, from
    /// any source: [`CarouselPrevious`]/[`CarouselNext`], the root
    /// keyboard handler, [`use_carousel`]'s `scroll_to`, or a user's own
    /// drag/wheel/trackpad scroll.
    #[props(default)]
    pub on_value_change: Callback<usize>,

    /// The text direction of the carousel: it lays the slides out right to
    /// left under RTL and mirrors the drag direction, but the root-level
    /// `ArrowLeft`/`ArrowRight` paging keys never swap. Defaults to the
    /// nearest [`crate::direction::DirectionProvider`], or LTR if there is
    /// none. See [`crate::direction::use_direction`].
    pub dir: Option<Direction>,

    /// The opacity of a disabled [`CarouselPrevious`]/[`CarouselNext`] -- the
    /// arrow at the first/last slide of a carousel that does not loop. One
    /// setting for both arrows. `None` (the default) renders nothing and the
    /// stylesheet decides; the shipped theme dims them to its disabled
    /// opacity (0.5, shadcn's `disabled:opacity-50`). A value is clamped
    /// into `0.0..=1.0` (a non-finite one counts as unset) and written as
    /// `--dx-carousel-nav-disabled-opacity:<n>;` on the root, so it reaches
    /// both arrows by inheritance and a stylesheet can set the same variable
    /// without the prop. `0.0` also writes
    /// `--dx-carousel-nav-disabled-visibility:hidden;`: the theme fades the
    /// arrow out and then removes it from the accessibility tree and the hit
    /// test, without moving anything (see this component's "Styling" doc).
    ///
    /// The primitive does no styling itself -- an unthemed carousel ignores
    /// both variables.
    #[props(default)]
    pub nav_disabled_opacity: Option<f32>,

    /// Additional attributes to apply to the carousel's root element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the carousel -- typically [`CarouselPrevious`] and
    /// [`CarouselNext`] followed by [`CarouselContent`] (see
    /// [`Carousel`]'s own doc for why that order matters).
    pub children: Element,
}

/// # Carousel
///
/// A slideshow of [`CarouselItem`]s the user pages through with
/// [`CarouselPrevious`]/[`CarouselNext`] buttons, `ArrowLeft`/`ArrowRight`
/// at the root, or by dragging/scrolling the track natively.
/// Implements the APG Carousel pattern's "basic" (prev/next, no picker)
/// style -- see the module doc's "Scope" section.
///
/// `Carousel` itself renders `role="region"` +
/// `aria-roledescription="carousel"` and requires an accessible name (an
/// `aria-label`/`aria-labelledby` in `attributes`) -- a
/// `tracing::warn!` fires when neither is supplied, since the pattern
/// requires one and shadcn's own reference component leaves this gap
/// entirely undocumented (`dev-docs/archive/research/carousel-2026-09-19.md`
/// §1.4).
///
/// ## Tab order
///
/// The reference implementation keeps its prev/next buttons **before**
/// the slide content in the page's `Tab` sequence (its own keyboard
/// table: "Rotation control, previous slide, and next slide buttons
/// precede the slide content"). Because `Carousel` renders `children` in
/// whatever order the caller writes them, and [`CarouselPrevious`]/
/// [`CarouselNext`] are meant to be positioned visually with CSS (not
/// DOM order -- absolute positioning does not care which sibling comes
/// first), the recommended composition puts them **before**
/// [`CarouselContent`], matching the reference and giving any interactive
/// content inside a slide (e.g. a card's own link) its natural place
/// after the controls, not before:
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::carousel::{Carousel, CarouselContent, CarouselItem, CarouselNext, CarouselPrevious};
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         Carousel { aria_label: "Featured photos",
///             CarouselPrevious { "Previous" }
///             CarouselNext { "Next" }
///             CarouselContent {
///                 CarouselItem { index: 0usize, "Slide 1" }
///                 CarouselItem { index: 1usize, "Slide 2" }
///                 CarouselItem { index: 2usize, "Slide 3" }
///             }
///         }
///     }
/// }
/// ```
///
/// ## Keyboard
///
/// `ArrowLeft`/`ArrowRight` (or `ArrowUp`/`ArrowDown` under
/// [`CarouselOrientation::Vertical`]) page the carousel whenever focus is
/// anywhere inside it -- a Radix/shadcn-style addition, not an APG
/// requirement (the reference never binds arrow keys at the carousel-root
/// level at all). `ArrowLeft` is always the previous slide and `ArrowRight`
/// the next, also under RTL, as in shadcn's carousel. As with every native
/// `<button>`, activating [`CarouselPrevious`]/[`CarouselNext`] with
/// `Enter`/`Space` never moves focus.
///
/// ## Styling
///
/// The [`Carousel`] component defines the following data attributes you
/// can use to control styling:
/// - `data-orientation`: `horizontal` or `vertical`, matching
///   [`CarouselProps::orientation`].
/// - `data-direction`: the resolved text direction, `ltr` or `rtl`.
///
/// It also publishes two optional custom properties for a theme to read on
/// the disabled [`CarouselPrevious`]/[`CarouselNext`], both set by
/// [`CarouselProps::nav_disabled_opacity`] (or by a stylesheet directly):
/// - `--dx-carousel-nav-disabled-opacity`: the disabled arrow's opacity,
///   `0..=1`.
/// - `--dx-carousel-nav-disabled-visibility`: `hidden` when that opacity is
///   `0`, so an invisible arrow also leaves the accessibility tree.
///
/// ## Focus on the arrows
///
/// When the focused Previous/Next becomes `disabled` (it just paged to the
/// end), focus moves to the [`CarouselContent`] scroller rather than being
/// left on a dead button or dropped to `<body>`; `Enter`/`Space` on an arrow
/// never otherwise moves focus.
#[component]
pub fn Carousel(props: CarouselProps) -> Element {
    let (raw_selected, set_raw_selected) =
        use_controlled(props.value, props.default_value, props.on_value_change);
    let direction = use_direction(props.dir);
    let direction_signal = use_direction_signal(direction);
    let orientation = props.orientation;
    let align = props.align;

    let item_ids: Signal<Vec<String>> = use_signal(Vec::new);
    let count = use_memo(move || item_ids.len());
    let selected = use_memo(move || clamp_selected(raw_selected(), count()));
    let mut scroll_origin: Signal<Option<usize>> = use_signal(|| None);
    let set_selected = use_callback(move |index: usize| {
        // Any selection that is not the track's own settle must scroll.
        scroll_origin.set(None);
        set_raw_selected.call(clamp_selected(index, *count.peek()));
    });
    let content_id: Signal<String> = use_signal(String::new);
    let loop_enabled = props.r#loop;
    let drag_free = props.drag_free;

    // Autoplay state -- always created (see `AutoplayContext`'s own doc
    // for why: `CarouselContent` reads `autoplay.present`/`autoplay.rotating`
    // unconditionally for its own `aria-live`, regardless of whether a
    // `CarouselAutoplay` child exists at all). `stop_on_interaction`/
    // `stop_on_mouse_enter` default to `true` -- the same defaults
    // `CarouselAutoplayProps` itself uses, so a carousel with no
    // `CarouselAutoplay` at all behaves identically to one whose (never
    // mounted) autoplay would have used the ordinary defaults.
    let autoplay_present = use_signal(|| false);
    let autoplay_playing = use_signal(|| false);
    let mut autoplay_hover = use_signal(|| false);
    let mut autoplay_focus_within = use_signal(|| false);
    let mut autoplay_resume_blocked = use_signal(|| false);
    let autoplay_stop_on_interaction = use_signal(|| true);
    let autoplay_stop_on_mouse_enter = use_signal(|| true);
    let autoplay_rotating = use_memo(move || {
        if !autoplay_playing() || autoplay_resume_blocked() || autoplay_focus_within() {
            return false;
        }
        !(autoplay_hover() && autoplay_stop_on_mouse_enter())
    });
    let autoplay = AutoplayContext {
        present: autoplay_present,
        playing: autoplay_playing,
        hover: autoplay_hover,
        focus_within: autoplay_focus_within,
        resume_blocked: autoplay_resume_blocked,
        stop_on_interaction: autoplay_stop_on_interaction,
        stop_on_mouse_enter: autoplay_stop_on_mouse_enter,
        rotating: autoplay_rotating,
    };
    let tablist_present = use_signal(|| false);
    let visible_range: Signal<(usize, usize)> = use_signal(|| (0, 0));
    let focused_item: Signal<Option<usize>> = use_signal(|| None);
    let loop_mode = props.loop_mode;
    // Published (`true`) only by `CarouselVirtualContent`'s own seamless
    // path -- see `CarouselContext::loop_wraps`'s own doc. Stays `false`
    // for the plain children API for the whole lifetime of this
    // component, since nothing else ever writes it.
    let virtualized_loop_active: Signal<bool> = use_signal(|| false);

    use_context_provider(|| CarouselContext {
        orientation,
        align,
        selected,
        set_selected,
        count,
        item_ids,
        content_id,
        loop_enabled,
        loop_mode,
        drag_free,
        scroll_origin,
        virtualized_loop_active,
        direction: direction_signal,
        autoplay,
        tablist_present,
        visible_range,
        focused_item,
    });

    // Page the scroller to the selected slide whenever it changes,
    // regardless of source (buttons, keyboard, or
    // `use_carousel_scroll_tracking` reporting a native scroll). In the
    // last case the target slide is already at/near its snap point, so
    // the computed delta is ~0 and `scrollBy` is a harmless no-op.
    // `is_first` (peeked, then set -- never tracked-read, so this
    // effect never subscribes to its own write, matching
    // `scripts/check-self-subscribing-effects.sh`'s construction) forces
    // an instant jump on mount so a non-zero `default_value` never
    // visibly animates in on page load, and must be consumed *exactly*
    // once, by that mount-time settle -- never by a later navigation.
    //
    // The flag must not be consumed by "the first run that finds a valid
    // item id": that run is not always the mount settle. In the
    // overwhelmingly common `default_value == 0` case `clamp_selected(0,
    // count)` is `0` both before and after registration, so `selected()`'s
    // own *value* never changes when items register, and (since `item_ids`
    // below is read through `.peek()`, deliberately not a tracked
    // dependency) this effect would not be scheduled to re-run once they
    // did. Its first execution -- mount, before any `CarouselItem` has
    // registered -- early-returns, and nothing would run it again until the
    // user's first real navigation, which would then inherit a stale
    // `instant: true` (Next from slide 1 sending `{instant: true, behavior:
    // 'auto'}` while every later click sends `{instant: false, behavior:
    // 'smooth'}`). A document-level `scroll-behavior: smooth` (the preview's
    // `html[data-anchor-scroll]` rule in `preview/assets/main.css`) does not
    // reach this element: `scroll-behavior` is not inherited and this element
    // is its own scroll container.
    //
    // So it is consumed by construction, not by timing: `count()` below is
    // read with tracked syntax (unlike the `item_ids.peek()` a few lines
    // down), so registration completing re-runs this effect the instant it
    // does, even though `selected()`'s own clamped value stayed the same --
    // and `is_first` is consumed unconditionally the moment `count() > 0`
    // on *that* run, regardless of whether this particular index's id
    // happens to resolve this same tick. So the flag can never survive
    // past the first post-registration run of this effect -- which,
    // since registration is itself effect-driven and completes
    // synchronously within the same initial render/effect-flush wave
    // that mounts this component (long before a user could physically
    // interact with it), can never coincide with a genuine navigation.
    // This covers both the `default_value == 0` case above and the
    // nonzero-`default_value` case (still instant, consumed at the same
    // conceptual point, just via a value change rather than a `count()`
    // retry).
    let mut is_first = use_signal(|| true);
    // Also consumed below (the wrap-instant check) -- declared here,
    // ahead of the focus-safety effect's own identical-shaped
    // `use_previous(selected.into())` further down, so both effects can
    // read the same "what was `selected` a moment ago" snapshot without
    // provisioning two independent trackers of the same signal.
    let prev_selected_for_scroll = use_previous(selected.into());
    use_effect(move || {
        let index = selected();
        let has_items = count() > 0;
        if drag_free() && *scroll_origin.peek() == Some(index) {
            // The track's own scroll settle selected this slide: it is
            // already where the user left it.
            scroll_origin.set(None);
            return;
        }
        let first = is_mount_settle(*is_first.peek(), has_items);
        if has_items {
            is_first.set(false);
        }
        // A windowed `CarouselVirtualContent` registers an EMPTY id per
        // index (it owns its own paging -- see its "Seamless loop" doc), so
        // there is no target here: skip, rather than evaluating a scroll-to
        // that can only report "no target" (noise on every virtual
        // settle).
        let Some(id) = item_ids
            .peek()
            .get(index)
            .cloned()
            .filter(|id| !id.is_empty())
        else {
            return;
        };
        // Tracked (like `count()` above), for the identical reason: a
        // scroller id that only arrives *after* this effect's first
        // post-registration run must still re-run this effect once it
        // does, rather than being silently missed the way an untracked
        // `.peek()` would miss it. In practice this never actually races
        // `has_items` becoming true -- `CarouselContent` publishes
        // `content_id` from its own mount effect, which (unlike item
        // registration) depends on nothing any child does, so it
        // completes no later than the child `CarouselItem` registrations
        // that make `has_items` true, within the same initial
        // render/effect-flush wave this effect's own long comment above
        // already establishes for item registration.
        let scroller_id = content_id();
        if scroller_id.is_empty() {
            return;
        }
        let orientation_str = orientation().as_str().to_string();
        let align_str = align().as_str().to_string();
        // Instant on mount (`first`), OR this specific step is a
        // `LoopMode::Rewind` wraparound -- see `is_rewind_wrap`'s own doc
        // for the APG-reference-matching reasoning ("Make Rewind an
        // instant jump... instead of... a smooth scroll back across every
        // slide"). `virtualized_loop_active` never applies here: this
        // effect only ever fires at all when `item_ids` has a real entry
        // for `index`, which is either the plain children API or
        // `CarouselVirtualContent`'s own non-windowed fallback -- exactly
        // the two content shapes `loop_wraps` gates on `LoopMode::Rewind`
        // for, never the windowed/seamless one (see that method's own
        // doc).
        let wraps_now = loop_enabled() && loop_mode() == LoopMode::Rewind;
        let is_wrap = wraps_now && is_rewind_wrap(prev_selected_for_scroll(), index, count());
        let instant = first || is_wrap;
        let eval = document::eval(CAROUSEL_SCROLL_TO_JS);
        let _ = eval.send((
            scroller_id,
            id,
            orientation_str,
            instant,
            CAROUSEL_SNAP_RESTORE_FALLBACK_MS,
            align_str,
        ));
    });

    // Focus safety (module doc §3 / `dev-docs/archive/research/carousel-loop-2026-09-25/loop-a11y-guidance.md` §7's "Focus
    // handling on rotation"): whenever `selected` changes, the slide that
    // was current a moment ago is about to (or, since the DOM patch that
    // sets its `inert` attribute is already applied by the time this
    // effect runs, may already) drop out of the visible/interactive set.
    // Per the HTML spec, an element becoming `inert` while it contains the
    // focused element unfocuses it and falls back to the document itself
    // (`<body>`) -- exactly the fallback this must never be seen to do.
    //
    // `focused_item` (each `CarouselItem`'s own plain, synchronous
    // `onfocusin`/`onfocusout`, no `document::eval` involved) is read here
    // as a snapshot of "was a slide focused right before this transition,
    // and if so which one" -- captured independently of whichever DOM
    // mutation already ran, so it still answers correctly even though a
    // `.peek()` of `document.activeElement` at this point could not (the
    // browser's own auto-blur has already fired by the time any effect,
    // this one included, gets to run -- effects only ever see the DOM
    // *after* the patch that triggered them, the same ordering every other
    // post-render `document::eval` in this module already relies on for
    // reading fresh geometry).
    //
    // The redirect target is [`CarouselContent`]'s own scroller element
    // (`content_id`) -- already unconditionally focusable
    // (`tabindex="0"`), never `inert`, and explicitly sanctioned as a
    // "stable control" by the contract ("the carousel's own focusable
    // region/content element, *or* the Next/Previous button that caused
    // the move"). A single, source-agnostic target rather than
    // special-casing which button (if any) caused the move keeps this one
    // effect correct for every transition source (buttons, the root
    // keyboard handler, `CarouselIndicator`, `CarouselApi::scroll_to`, and a
    // drag/wheel/trackpad settle that changes `selected` with no button
    // involved at all) without threading "who caused this" through every
    // one of them.
    //
    // Deliberately not gated more finely than "the previously-focused
    // slide is not the new `selected`": knowing whether that slide
    // *remains* inside the new multi-per-view visible window without
    // actually performing the redirect would need the post-scroll geometry
    // this effect cannot see yet (the visible-range settle above is still
    // in flight at this exact point) -- so a focused slide other than the
    // new current one is always treated as "about to need a fixup" even in
    // the rare multi-per-view case where it happens to still be visible
    // afterward. Safety over precision: an unnecessary refocus onto the
    // scroller (still inside the carousel, still a sensible place for
    // focus to be) is a far smaller cost than the alternative -- focus
    // silently dropping to `<body>`.
    use_effect(move || {
        let new_selected = selected();
        let old_selected = prev_selected_for_scroll();
        if old_selected == new_selected {
            return;
        }
        let Some(focused_idx) = *focused_item.peek() else {
            return;
        };
        if focused_idx == new_selected {
            return;
        }
        let target = content_id.peek().clone();
        if target.is_empty() {
            return;
        }
        // Unconditional, not gated on re-reading `document.activeElement`
        // first: this effect's own
        // `document::eval` dispatch can run *before* Dioxus's own DOM patch
        // has applied the new `inert` attribute -- `document.activeElement`
        // at that moment still reports the slide's own descendant, still
        // genuinely focused, so a "only redirect if focus already looks
        // lost" check silently no-ops here and then never gets a second
        // chance once the patch (and the browser's own auto-blur-to-`body`
        // it triggers) actually lands a moment later. `focused_item` is
        // this component's own Rust-side judgment call, captured
        // synchronously and independently of DOM-patch timing (each
        // `CarouselItem`'s own `onfocusin`/`onfocusout`), so it is trusted
        // outright: whichever slide held focus right before this
        // transition is not the new current one, so a redirect is due
        // regardless of whether the browser has already carried out its
        // own fallback-to-`body` by the time this runs.
        document::eval(&format!(
            "var root = document.getElementById('{target}'); \
             if (root) {{ root.focus({{ preventScroll: true }}); }}"
        ));
    });

    let onkeydown = move |event: Event<KeyboardData>| {
        let key = event.key();
        let Some(intent) = carousel_key_intent(&key, orientation()) else {
            return;
        };
        // See `CarouselContext::loop_wraps`'s own doc -- the identical
        // formula, inlined here since the root itself is the context's
        // provider rather than one of its consumers.
        let wraps_now =
            loop_enabled() && (loop_mode() == LoopMode::Rewind || virtualized_loop_active());
        match intent {
            HorizontalNav::Prev => set_selected.call(step_prev(selected(), count(), wraps_now)),
            HorizontalNav::Next => set_selected.call(step_next(selected(), count(), wraps_now)),
        }
        autoplay.note_interaction();
        event.prevent_default();
    };

    // Autoplay pause-on-hover/pause-on-focus -- always wired (cheap plain
    // signal writes; a harmless no-op with no `CarouselAutoplay` mounted,
    // since `autoplay_rotating` above can never be `true` without one).
    // See `AutoplayContext`'s own doc for the hover-vs-focus asymmetry.
    let onmouseenter = move |_| autoplay_hover.set(true);
    let onmouseleave = move |_| autoplay_hover.set(false);
    let onfocusin = move |_| {
        autoplay_focus_within.set(true);
        autoplay_resume_blocked.set(true);
    };
    let onfocusout = move |_| autoplay_focus_within.set(false);

    if !has_own_accessible_name(&props.attributes) {
        tracing::warn!(
            "Carousel: no aria-label or aria-labelledby was supplied. The APG Carousel \
             pattern requires the carousel region to have an accessible name that does not \
             itself contain the word \"carousel\" \
             (dev-docs/research/carousel-2026-09-19.md \u{a7}1.2)."
        );
    }

    // Merged (caller-wins for anything presentational, then component-owned
    // state wins last), not a literal beside a bare `..props.attributes`
    // spread: `role`/`aria_roledescription`/the two `data-*` attributes have
    // no typed field in `CarouselProps` claiming their names, so SSR would
    // otherwise emit both this literal and a same-named caller override,
    // which the HTML parser and a hydrating client resolve to opposite
    // values (`scripts/check-attr-spread-collision.sh`,
    // `dev-docs/issues/duplicate-attribute-root-cause.md`). These are all
    // structural/ARIA state this component must control, so they go last
    // (owned-wins) -- the same value the browser's own parser resolves a
    // duplicate to (the first, library one) in SSR+hydrate.
    //
    // `style` is built the way `CarouselContent`'s is: the prop's variables
    // first, then whatever `style` the caller passed (so a caller-set
    // `--dx-carousel-nav-disabled-opacity` still wins, being later in the same
    // attribute), folded into ONE `style` attribute -- and none at all when
    // both are empty, so a carousel that uses neither renders no extra
    // markup.
    let (caller_style, rest_attrs) = fold_style_attributes(props.attributes);
    let style = format!(
        "{}{}",
        nav_disabled_vars(props.nav_disabled_opacity),
        caller_style.unwrap_or_default()
    );
    let style = (!style.is_empty()).then_some(style);
    let attributes = merge_attributes(vec![
        rest_attrs,
        attributes!(div {
            role: "region",
            aria_roledescription: "carousel",
            "data-orientation": orientation().as_str(),
            "data-direction": direction.as_str(),
            style: style,
        }),
    ]);

    rsx! {
        div {
            dir: direction.as_str(),
            onkeydown,
            onmouseenter,
            onmouseleave,
            onfocusin,
            onfocusout,
            ..attributes,

            {props.children}
        }
    }
}
