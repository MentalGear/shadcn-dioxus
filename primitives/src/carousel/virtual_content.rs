use super::*;

/// The props for the [`CarouselVirtualContent`] component.
#[derive(Props, Clone, PartialEq)]
pub struct CarouselVirtualContentProps<T: Clone + PartialEq + 'static> {
    /// The ID of the carousel content element.
    pub id: ReadSignal<Option<String>>,

    /// The full data set, in true order. Rendered in full (no window, no
    /// wrap) on the server and on the client's own first render -- see
    /// this component's own "SSR / first render" doc -- narrowed to a
    /// `radius`-wide window around the current slide in a post-mount
    /// effect once virtualisation is active.
    pub items: ReadSignal<Vec<T>>,

    /// Renders one slide's content, given its stable data index and
    /// value.
    pub render_item: Callback<(usize, T), Element>,

    /// Slides kept mounted on each side of the current one once
    /// virtualisation is active -- the window is `2 * radius + 1` slides
    /// wide. Defaults to `10`.
    ///
    /// **Why 10.** The window only re-centres once a gesture is over (the
    /// browser-idle gate: nothing writes the scroll position while the
    /// browser is scrolling), so how far ONE trackpad flick can travel
    /// before reaching the rendered slice's end is exactly `radius` slides.
    /// Measured from recorded real sessions (Firefox, MacBook trackpad;
    /// excursion of each uninterrupted input stream, in slides of a 336px
    /// track): 252 ordinary bursts had a median of 3.0, p75 8.6, p90 15.2;
    /// a deliberate fast-flick session had a median of 10.8 and a max of 29;
    /// the recorded fixture gestures reach 15.6. `10` absorbs roughly 85% of
    /// everyday flicks without any mid-gesture write while keeping the DOM at
    /// 21 slides; the long tail is caught by the pinned-at-a-false-end
    /// re-centre (`CAROUSEL_SCROLL_TRACKING_JS`). A radius of `2` (5 slides)
    /// would be exhausted by a single flick after 0-2 slides.
    ///
    /// **Looping lists** never render the same item twice: the effective
    /// radius is capped at `(items.len() - 2) / 2` (5 for 12 items, so 11 of
    /// them), keeping one `data-index` per rendered slide -- see
    /// `effective_radius`.
    #[props(default = 10usize)]
    pub radius: usize,

    /// Whether to virtualise at all. `None` (the default) auto-decides:
    /// virtualise only once `items.len() > 2 * radius + 1` (with the
    /// effective radius -- see `effective_radius`; there is
    /// something to save by not rendering every slide). `Some(false)`
    /// always renders the full list (find-in-page/browse-mode reach every
    /// slide, at the cost of the DOM holding all of them). `Some(true)`
    /// always virtualises, even for a small data set.
    #[props(default)]
    pub virtualize: Option<bool>,

    /// Whether a mouse/pen pointer drag on the track pages the carousel --
    /// see [`CarouselContent`]'s own "Pointer drag" doc. Defaults to
    /// `true`; touch is never affected either way.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub draggable: ReadSignal<bool>,

    /// The space between slides, as any CSS length or variable (e.g.
    /// `"1rem"`, `"var(--dx-space-4)"`). When set, it is written as
    /// `--dx-carousel-gap:<value>;` on this element's own inline style, and
    /// the existing orientation-aware gap model applies it on the scroll
    /// axis: each slide's leading-edge padding plus this element's matching
    /// negative margin (`padding-inline-start`/`margin-inline-start`
    /// horizontally, `padding-block-start`/`margin-block-start` vertically).
    ///
    /// shadcn has no `gap` prop -- its carousel spaces slides with utility
    /// classes (`-ml-N`/`pl-N`, `-mt-N`/`pt-N` when vertical). This prop is a
    /// typed convenience over that same padding + negative-margin technique,
    /// nothing more: the `--dx-carousel-gap` custom property remains the
    /// override hook (a caller's own inline `style` setting it still wins, as
    /// it comes later in the same attribute), and `None` (the default)
    /// renders nothing, leaving any stylesheet value in force.
    #[props(default, into)]
    pub gap: Option<String>,

    /// Additional attributes to apply to the carousel content element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// # CarouselVirtualContent
///
/// A data-driven, virtualised drop-in for [`CarouselContent`] +
/// [`CarouselItem`]s: used INSIDE the same [`Carousel`] root, in place of
/// them, whenever the slides come from a plain `Vec<T>` rather than a
/// fixed set of children (`dev-docs/backlog.md` row 91 has the
/// construction this implements).
///
/// Publishes its own count into `CarouselContext` the same way
/// [`CarouselItem`]'s own registration effect does, so
/// [`CarouselPrevious`]/[`CarouselNext`]/[`CarouselIndicators`]/[`CarouselIndicator`]/
/// [`CarouselAutoplay`]/[`CarouselRotationControl`]/the root's own arrow
/// keys/[`use_carousel`] all work the same with it -- none of
/// them know this component exists; every one of them only ever reads or
/// writes `CarouselContext`.
///
/// This must be used inside a [`Carousel`] component, in place of
/// [`CarouselContent`] (never alongside it).
///
/// ## SSR / first render
///
/// Renders every item, in true data order (`position == data index`, no
/// window, no wrap), on the server and on the client's own first render --
/// identical markup both times, so hydration has nothing to reconcile.
/// Virtualising (below) only ever begins in a post-mount effect,
/// client-side -- the same "first client render == SSR" property
/// [`CarouselItem`]'s own "SSR stability" doc already relies on for
/// `selected`/`visible_range`.
///
/// ## Virtualisation
///
/// Once mounted, if `virtualize` resolves to `true` (explicit, or the
/// auto default once `items.len() > 2 * radius + 1`), only a window of
/// `2 * radius + 1` logical POSITIONS around an internal `anchor` renders,
/// via `crate::r#virtual::window` (`wrap` = `CarouselProps::r#loop` AND
/// at least two slides -- one slide never loops, per this component's own
/// doc, "N=1 disables loop"). Each rendered slide carries both its stable
/// `data-index` (`position.rem_euclid(N)`, may repeat across positions
/// once `N < 2 * radius + 1`) and its own unique `data-position` (the
/// value every slide is keyed and DOM-id'd by, so a position that survives
/// an anchor move keeps its own DOM node -- and [`CarouselVirtualContentProps::render_item`]'s
/// own component state along with it).
///
/// With virtualisation inactive (a small data set, or
/// `virtualize: Some(false)`), `CarouselProps::r#loop` has no seamless
/// path available to it at all -- exactly like the plain children API, it
/// is gated by [`CarouselProps::loop_mode`] (see that enum's own doc):
/// [`LoopMode::Rewind`] is the APG **rewind** style --
/// `position == data index` throughout, and this component publishes a
/// real per-data-index DOM id into `CarouselContext::item_ids` exactly
/// like [`CarouselItem`] does, so [`Carousel`]'s own built-in mount/
/// selected `scrollBy`-by-id paging effect (and drag, wheel bounce, scroll
/// tracking) drive it without any bespoke code in this component at all,
/// INCLUDING that effect's own instant-jump-on-wrap handling. The default,
/// [`LoopMode::Seamless`], is a no-op here (`CarouselContext::loop_wraps`
/// returns `false`): [`CarouselPrevious`]/[`CarouselNext`] stay genuinely
/// `disabled` at the real ends, matching what a caller who never
/// specifically opted into `Rewind` should expect from a non-windowed
/// data set with no seamless illusion available to it.
///
/// ## Seamless loop
///
/// Only reached once virtualisation is active AND `CarouselProps::r#loop`
/// is on -- **unaffected by [`CarouselProps::loop_mode`] entirely**: this
/// path publishes `true` into `CarouselContext::virtualized_loop_active`
/// (a small effect just above, keyed off the identical condition this
/// window-building code already computes), which is the OTHER thing (besides
/// `LoopMode::Rewind`) `CarouselContext::loop_wraps` treats as "wraps" --
/// so `loop_mode`'s default staying at [`LoopMode::Seamless`] never
/// disables this path; it only ever matters for the two content shapes
/// that have no seamless illusion to offer, above. `position` is allowed
/// to run arbitrarily far past `[0, N)`
/// (mapped down to a data index only via `.rem_euclid(N)` for rendering),
/// so paging forward past the last data index does not rewind visibly
/// across every intervening slide the way the non-virtualised loop above
/// does -- it keeps sliding physically forward, one already-mounted window
/// slide at a time. `CarouselContext::item_ids` stays empty in this mode
/// (a data index can label more than one position at once, so there is no
/// single DOM id to publish per data index) -- the built-in paging effect
/// therefore harmlessly no-ops, and this component owns the reaction to
/// `CarouselContext::selected` changing itself:
///
/// - A change of exactly one data index -- [`CarouselPrevious`]/
///   [`CarouselNext`]/the root's arrow keys/autoplay all step by exactly
///   one -- resolves to the nearest matching POSITION (`anchor \u{b1} 1`)
///   and is animated there with the very same scroller-only
///   `scrollBy`-by-id helper (`CAROUSEL_SCROLL_TO_JS`) every other
///   paging path in this module already uses. The target position is
///   already a rendered window slide (`radius >= 1`), so this alone never
///   touches the window.
/// - A change to any other data index (a distant [`CarouselIndicator`] click, a
///   controlled `value` jump) resolves to whichever POSITION nearest the
///   current anchor carries that data index, sets `anchor` to it directly
///   -- an instant re-render around the new centre, so it is simply the
///   current slide the moment this runs, never a visible scroll across
///   the intervening ones -- and instantly re-aligns the scroller to it.
/// - Once a physical scroll genuinely settles (a real drag release, a
///   wheel/trackpad gesture, or the animated paging call above finishing
///   -- see `use_carousel_scroll_tracking`'s own doc for what "settles"
///   means), the settle's own reported POSITION (not just its data index)
///   becomes the new `anchor` directly, covering a native gesture that
///   moved the track by more than one slide with no extra arithmetic.
///
/// Either way, `anchor` changing re-renders the window keyed by
/// `position` (so every surviving slide keeps its DOM node across the
/// move), and a dedicated effect immediately re-aligns the scroller to
/// the new centre with one more instant `CAROUSEL_SCROLL_TO_JS` call --
/// needed because the window's own far edge changing (one position
/// dropped, one added) shifts every later flex sibling's physical layout
/// offset by one slide's width, which a plain DOM patch does not
/// otherwise compensate for.
///
/// **Deviation from the bench's own microtask-timed correction:** the bench prototype
/// (`playwright/fixtures/carousel-overscroll-bench-rev40.html`, `seamlessLoop()`)
/// runs this same re-alignment from a `MutationObserver` callback -- a
/// microtask, guaranteed to run before the browser paints the DOM patch
/// that triggered it. This component instead runs it from a plain
/// `use_effect`, scheduled after the triggering render has already
/// committed; whether that is early enough to still land before paint on
/// every engine is an empirical question the Playwright
/// per-`requestAnimationFrame` sampling test answers, not
/// an assumption; upgrade to a `MutationObserver` if it ever fails.
///
/// ## Non-loop, virtualised
///
/// With virtualisation active but `CarouselProps::r#loop` off, the
/// window still clamps at the ends (`wrap = false` -- position never
/// leaves `[0, N)`, so it equals data index throughout), and the existing
/// edge rubber-band bridges apply exactly as they do for the plain
/// children API.
///
/// ## Dynamic `N`, and the degenerate counts
///
/// `N == 0` renders no slides at all (the scroller shell still mounts, an
/// empty scroll region, matching [`CarouselContent`]'s own shape with zero
/// children). `N == 1` disables looping regardless of
/// `CarouselProps::r#loop` -- there is nothing to wrap to. A dynamic
/// change in `N` reclamps `CarouselContext::selected` the same way it
/// already does for the plain children API (`clamp_selected`); this
/// component's own `anchor` degrades gracefully too, since
/// `anchor.rem_euclid(N)` is well-defined for any `N > 0` regardless of
/// how large `anchor` has drifted.
#[component]
pub fn CarouselVirtualContent<T: Clone + PartialEq + 'static>(
    props: CarouselVirtualContentProps<T>,
) -> Element {
    let mut ctx: CarouselContext = use_context();
    let uuid = use_unique_id();
    let id = use_id_or(uuid, props.id);
    let items = props.items;
    let render_item = props.render_item;
    let requested_radius = props.radius.max(1);
    let explicit_virtualize = props.virtualize;

    use_effect(move || {
        ctx.content_id.set(id());
    });

    let count = use_memo(move || items().len());

    // SSR / first render always renders the full list -- see this
    // component's own "SSR / first render" doc. `mounted` flips exactly
    // once, client-side, in a post-mount effect.
    let mut mounted = use_signal(|| false);
    use_effect(move || {
        mounted.set(true);
    });

    // Recomputed (not cached) inside every effect below that needs it, so
    // each one tracks `count()`/`mounted()` directly rather than trusting
    // that a fresh closure alone reruns it (a closure that captures no
    // signal of its own is not re-run by a changed `count()`).
    let virtualize_active_now = move || {
        let n = count();
        let radius = effective_radius(requested_radius, n, (ctx.loop_enabled)());
        n > 0 && mounted() && explicit_virtualize.unwrap_or_else(|| n > 2 * radius + 1)
    };

    let mut anchor = use_signal(|| (ctx.selected)() as isize);
    // The position the next physical step should be computed relative to
    // -- see the paging effect's own doc, "Load-bearing guard" and the
    // dedicated doc just above it, for why this must be tracked
    // separately from `anchor` (which only ever moves once a physical
    // scroll has genuinely settled).
    let mut intended = use_signal(|| (ctx.selected)() as isize);

    // Publish `count` into the shared context, and either a real
    // per-data-index id (small/non-virtualised -- the existing built-in
    // paging effect just works, see "Virtualisation" doc) or an empty
    // registry (seamless loop -- see "Seamless loop" doc for why this
    // component owns paging itself in that mode instead).
    use_effect(move || {
        let scroller_id = id();
        let n = count();
        let active = virtualize_active_now();
        let mut ids = ctx.item_ids.write();
        ids.clear();
        for i in 0..n {
            ids.push(if active {
                String::new()
            } else {
                format!("{scroller_id}-p{i}")
            });
        }
    });

    // A settle (native drag/wheel/trackpad, or one of this component's own
    // animated paging calls below finishing) reports the slide it actually
    // rested on -- `anchor` (drives the render window) and `intended`
    // (the paging effect's own "what to compute the next step from")
    // both snap to it together, so the two can never drift apart once a
    // physical scroll has genuinely finished.
    let on_settle_position = use_callback(move |position: isize| {
        anchor.set(position);
        intended.set(position);
    });
    use_carousel_scroll_tracking(
        id,
        ctx.orientation,
        ctx.align,
        ctx.set_selected,
        ctx.count,
        ctx.visible_range,
        Some(on_settle_position),
    );
    use_carousel_drag(id, ctx.orientation, ctx.align, props.draggable);
    // Where the band may engage (only a true end of the data, never a loop,
    // never this window's slice edge mid-list) is `isTrueEnd` in the script,
    // read from the `data-loop`/`data-slide-count` rendered below.
    use_carousel_wheel_band(id, ctx.orientation);

    // Translate a `selected` (data index) change into a physical move --
    // see "Seamless loop" doc. A no-op whenever virtualisation is
    // inactive: the built-in `Carousel`-level effect (real per-data-index
    // ids, published above) already handles that case.
    //
    // **Two load-bearing rules:**
    //
    // 1. This must compute the next target from `intended` (the last
    //    position THIS effect itself has already asked for), never from
    //    `anchor` (the last position a physical scroll actually SETTLED
    //    on). `anchor` only updates once `on_settle_position` above fires
    //    -- which for an animated (not instant) step happens only after
    //    the browser's own `scrollend` -- so a second `selected` change
    //    arriving before the first step has finished animating (for example
    //    consecutive `CarouselAutoplay` ticks close enough together)
    //    would otherwise still read the PRE-first-step anchor, compute the
    //    same target the first step already asked for, and silently
    //    swallow one entire step (a skipped index, landing on 5 then 7).
    //    Reading
    //    `intended` instead -- updated immediately, in the same tick this
    //    effect runs, in both branches below -- means a rapid-fire second
    //    change always computes relative to where the first one is
    //    already headed, not where the scroller has physically gotten to
    //    yet.
    // 2. Skip entirely once `intended` already denotes `new_sel`: a settle
    //    calls `ctx.set_selected` and `on_settle_position` together for
    //    the SAME reported slide, so by the time this effect observes
    //    that `selected` change, `intended` (like `anchor`) may already
    //    agree with it. Recomputing a jump from `old_sel` regardless would
    //    treat the settle's own report as a
    //    fresh navigation request layered on the *previous* selected
    //    value, landing on a second, wrong position and triggering
    //    another settle -- an unbounded feedback loop in which the
    //    anchor drifts continuously in one direction for as long as the
    //    page is left open, entirely without user interaction. Only a
    //    change nothing has already reconciled (a button/key/autoplay/
    //    tab/`scroll_to` call, none of which ever touch `intended`
    //    themselves) reaches the jump logic below.
    let prev_selected = use_previous(ctx.selected.into());
    let orientation = ctx.orientation;
    let align = ctx.align;
    let content_id = ctx.content_id;
    use_effect(move || {
        let new_sel = (ctx.selected)();
        let old_sel = prev_selected();
        let n = count();
        if !virtualize_active_now() || n == 0 || old_sel == new_sel {
            return;
        }
        let cur_intended = *intended.peek();
        if n > 0 && cur_intended.rem_euclid(n as isize) as usize == new_sel {
            // `intended` already agrees with the new selected value -- see
            // this effect's own doc, rule 2, for why this must be a no-op.
            return;
        }
        let wrapped = shortest_signed_delta(old_sel, new_sel, n);
        if wrapped == 0 {
            return;
        }
        let target_position = cur_intended + wrapped;
        let scroller_id = content_id.peek().clone();
        if scroller_id.is_empty() {
            return;
        }
        let orientation_str = orientation().as_str().to_string();
        // **Third load-bearing rule.** Whether the target
        // is "already a rendered window slide" must be measured against
        // `anchor` (the last SETTLED position -- what the render window
        // is actually keyed from), never against `intended`'s own step
        // size. `intended` moves the instant this effect runs (rule 1,
        // above), but the window only re-centres once a settle actually
        // lands -- which, for an animated (non-instant) step, only
        // happens after the browser's own `scrollend`, real wall-clock
        // time later. A second (or third) `selected` change arriving
        // before that -- for example two `CarouselNext` clicks a few
        // hundred ms apart, comfortably human-paced, well inside a
        // `radius: 2` window's own margin -- can walk `intended` further
        // from `anchor` than `radius` even though each individual STEP is
        // still exactly one. Using `wrapped` (the step size) here instead
        // of this distance would let that second step's own animated call
        // target a position outside the still-`anchor`-centred window --
        // an id with no element behind it -- so the `scrollBy` would silently
        // no-op (`CAROUSEL_SCROLL_TO_JS`'s own null-target guard) and
        // the click would be lost outright, with the *original* (first) step's
        // own scroll finishing normally afterward and its genuinely
        // correct settle then overwriting `intended` back down to its own
        // target (the second click, computed correctly as "old_sel=2
        // new_sel=3", would be silently reverted moments later by the
        // first click's late-arriving, entirely valid "settle position=2").
        // Measuring against `anchor` instead means a target
        // that has drifted outside the true rendered window takes the
        // instant re-anchor branch below -- which needs no pre-rendered
        // element at all -- rather than silently failing to reach an
        // element that was never going to exist.
        let cur_anchor = *anchor.peek();
        let radius = effective_radius(requested_radius, n, (ctx.loop_enabled)());
        if (target_position - cur_anchor).unsigned_abs() as usize <= radius {
            // Already a rendered window slide -- animate the existing
            // scroller-only paging helper to it. `intended` moves right
            // away (rule 1); `anchor` (and the window it drives) only
            // moves once `on_settle_position` reports this scroll has
            // actually finished.
            intended.set(target_position);
            let target_id = format!("{scroller_id}-p{target_position}");
            let align_str = align().as_str().to_string();
            let eval = document::eval(CAROUSEL_SCROLL_TO_JS);
            let _ = eval.send((
                scroller_id,
                target_id,
                orientation_str,
                false,
                CAROUSEL_SNAP_RESTORE_FALLBACK_MS,
                align_str,
            ));
        } else {
            // Outside the current window (a distant jump, OR `intended`
            // has drifted further from the last settle than `radius`
            // allows) -- re-anchor both instantly; the alignment effect
            // below does the matching instant scroll once this render has
            // committed.
            intended.set(target_position);
            anchor.set(target_position);
        }
    });

    // Re-align the scroller to `anchor`'s own slide -- covers the
    // instant-jump branch above, every settle-driven move (paging, drag,
    // wheel/trackpad) reported back by `use_carousel_scroll_tracking`,
    // AND the one-time transition from full-list to windowed rendering
    // (`virtualize_active` flipping `false` -> `true`): the window's own
    // radius-worth of leading slides only exist once that flip happens,
    // so the scroller's physical position (wherever it happened to rest
    // showing the full list) needs the identical realignment even though
    // `anchor`'s own numeric value does not change at that moment.
    // `align_key` bundles both triggers into one comparison rather than
    // two separate effects racing each other over the same scroller.
    // Always instant: by the time this runs, the physical "this should
    // feel like a slide" motion has already happened, either via the
    // browser's own native scroll or via the animated
    // `CAROUSEL_SCROLL_TO_JS` call above -- this call only ever
    // compensates the window's own re-centring (or its own initial
    // appearance), which must not itself be seen to animate.
    let align_key = use_memo(move || (anchor(), virtualize_active_now()));
    let prev_align_key = use_previous(align_key.into());
    use_effect(move || {
        let (a, active) = align_key();
        let (pa, pactive) = prev_align_key();
        if !active || (a == pa && active == pactive) {
            return;
        }
        let scroller_id = content_id.peek().clone();
        if scroller_id.is_empty() {
            return;
        }
        let target_id = format!("{scroller_id}-p{a}");
        let orientation_str = orientation().as_str().to_string();
        let align_str = align().as_str().to_string();
        let eval = document::eval(CAROUSEL_SCROLL_TO_JS);
        let _ = eval.send((
            scroller_id,
            target_id,
            orientation_str,
            true,
            CAROUSEL_SNAP_RESTORE_FALLBACK_MS,
            align_str,
        ));
    });

    let n = count();
    let virtualize_active = virtualize_active_now();
    let loop_now = (ctx.loop_enabled)() && n >= 2;

    // Publish "the seamless windowed loop is genuinely active right now"
    // into the shared context -- see `CarouselContext::loop_wraps`'s own
    // doc for why every other loop-aware consumer (Previous/Next's
    // `disabled`, the root keyboard handler, `CarouselAutoplay`'s ticker)
    // needs this, not just `loop_enabled` alone: without it, those would
    // treat a non-windowed (small data set, or `virtualize: Some(false)`)
    // `CarouselVirtualContent` under `LoopMode::Seamless` (the default) as
    // wrapping, when this component's own "Virtualisation" doc says that
    // case is `LoopMode::Rewind`-or-nothing like the plain children API.
    let mut virtualized_loop_active = ctx.virtualized_loop_active;
    use_effect(move || {
        let active = virtualize_active_now() && (ctx.loop_enabled)() && count() >= 2;
        virtualized_loop_active.set(active);
    });

    let win: Vec<crate::r#virtual::WindowItem> = if n == 0 {
        Vec::new()
    } else if virtualize_active {
        let radius = effective_radius(requested_radius, n, (ctx.loop_enabled)());
        crate::r#virtual::window(n, anchor(), anchor(), radius, loop_now)
    } else {
        crate::r#virtual::window(n, 0, n as isize - 1, 0, false)
    };
    // Does the rendered window hold the real first / last item? (Feeds
    // `content_overscroll_style`: a slice edge mid-list is a false end.)
    let win_first_last = (
        win.iter().any(|item| item.data_index == 0),
        n > 0 && win.iter().any(|item| item.data_index == n - 1),
    );

    let selected = (ctx.selected)();
    let (visible_start, visible_end) = (ctx.visible_range)();
    let tablist_present = (ctx.tablist_present)();
    let mut focused_item = ctx.focused_item;
    let items_now = items();
    let scroller_id_now = id();

    let orientation_now = (ctx.orientation)();
    let align_now = (ctx.align)();
    let draggable = (props.draggable)();
    let aria_live = (ctx.autoplay.present)().then(|| {
        if (ctx.autoplay.rotating)() {
            "off"
        } else {
            "polite"
        }
    });

    let (caller_style, rest_attrs) = fold_style_attributes(props.attributes);
    let gap_margin = content_gap_margin(orientation_now);
    let gap_var = content_gap_var(props.gap.as_deref());
    // Both ends true only when not looping AND the rendered window holds the
    // real first and last item (always so before windowing: SSR and the
    // hydrating first render render the whole list).
    let both_ends_true = !(ctx.loop_enabled)() && win_first_last.0 && win_first_last.1;
    let overscroll = content_overscroll_style(orientation_now, both_ends_true);
    let axis_style = match orientation_now {
        CarouselOrientation::Horizontal => {
            "display:flex;flex-direction:row;overflow-x:auto;overflow-y:hidden;\
             scroll-snap-type:x mandatory;"
        }
        CarouselOrientation::Vertical => {
            "display:flex;flex-direction:column;overflow-y:auto;overflow-x:hidden;\
             scroll-snap-type:y mandatory;"
        }
    };
    let style = format!(
        "{axis_style}{gap_margin}{gap_var}{overscroll}{}",
        caller_style.map(|s| format!(" {s}")).unwrap_or_default()
    );

    let attributes = merge_attributes(vec![
        rest_attrs,
        attributes!(div {
            style: style.clone(),
            tabindex: "0",
            aria_live: aria_live,
            "data-orientation": orientation_now.as_str(),
            "data-draggable": draggable,
            "data-snap": snap_type_value(orientation_now, false),
            // Read per gesture by the wheel band's `isTrueEnd` (the band only
            // engages at a true end of the data). `n` is `items().len()`, known
            // from props on the very first render, so SSR and hydration agree.
            "data-loop": (ctx.loop_enabled)(),
            "data-slide-count": n,
        }),
    ]);

    let slides = win.into_iter().map(move |item| {
        let data_index = item.data_index;
        let position = item.position;
        let is_selected = data_index == selected;
        let in_visible_range = data_index >= visible_start && data_index <= visible_end;
        let is_visible = is_selected || in_visible_range;
        let inert: Option<&'static str> = (!is_visible).then_some("true");
        let role = if tablist_present { "tabpanel" } else { "group" };
        let label = slide_label(data_index, n);
        let slide_id = format!("{scroller_id_now}-p{position}");
        let value = items_now[data_index].clone();
        let content = render_item.call((data_index, value));
        let item_style = format!(
            "{}scroll-snap-align:{};scroll-snap-stop:always;min-width:0;min-height:0;{}",
            item_basis_style(),
            align_now.as_str(),
            item_gap_padding(orientation_now)
        );
        rsx! {
            div {
                key: "{position}",
                id: slide_id,
                role,
                aria_roledescription: "slide",
                aria_label: label,
                style: item_style,
                "data-selected": is_selected,
                "data-index": data_index,
                "data-position": position,
                inert,
                onfocusin: move |_| focused_item.set(Some(data_index)),
                onfocusout: move |_| {
                    if (focused_item)() == Some(data_index) {
                        focused_item.set(None);
                    }
                },
                {content}
            }
        }
    });

    rsx! {
        // Clipping viewport -- see `CarouselContent`'s own "Edge
        // rubber-band" doc for the full construction; this wrapper is
        // identical to that one's.
        div {
            "data-slot": "carousel-viewport",
            style: "overflow: clip; display: flow-root;",

            div {
                id,
                ..attributes,

                {slides}
            }
        }
    }
}
