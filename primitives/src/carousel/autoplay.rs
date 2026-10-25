use super::*;

/// One-shot: reads `window.matchMedia('(prefers-reduced-motion: reduce)').matches`
/// and reports it back. The one required JS read even under this module's
/// "almost no JS" posture (`dev-docs/archive/research/carousel-2026-09-19.md` §5.2)
/// that has to be observed from the *Rust* side rather than only ever
/// branched on inside a script -- [`CarouselAutoplay`] needs the answer to
/// decide whether to start its own timer at all, a decision that lives in
/// Rust. Mirrors the reduced-motion checks already inline inside
/// `CAROUSEL_DRAG_JS`/`CAROUSEL_WHEEL_BAND_JS`, just surfaced instead of
/// only ever consulted in place.
pub(super) const CAROUSEL_REDUCED_MOTION_JS: &str =
    "dioxus.send(window.matchMedia('(prefers-reduced-motion: reduce)').matches);";

/// The props for the [`CarouselAutoplay`] component.
#[derive(Props, Clone, PartialEq)]
pub struct CarouselAutoplayProps {
    /// Milliseconds between automatic slide advances. Mirrors
    /// `embla-carousel-autoplay`'s own `delay` option, including its
    /// default (`4000`).
    #[props(default = ReadSignal::new(Signal::new(4000)))]
    pub delay_ms: ReadSignal<u64>,

    /// Whether rotation stops for good (until [`CarouselRotationControl`]
    /// is explicitly clicked again) after a manual paging action --
    /// Previous/Next, the root keyboard handler, a [`CarouselIndicator`], or a
    /// custom picker's own [`CarouselApi::scroll_to`]. Mirrors
    /// `embla-carousel-autoplay`'s own `stopOnInteraction` option and its
    /// default (`true`).
    ///
    /// **Scope note:** a native pointer-drag or wheel/trackpad scroll
    /// does **not** count as "interaction" here -- only the discrete
    /// paging actions above do. Wiring the drag/wheel bridges too is an
    /// deliberate scope cut, not a silent gap: see [`AutoplayContext::note_interaction`]'s
    /// own doc for exactly which call sites are wired.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub stop_on_interaction: ReadSignal<bool>,

    /// Whether hovering anywhere in the carousel pauses rotation --
    /// resuming once the pointer leaves, unless focus is *also* currently
    /// holding it paused (see [`AutoplayContext`]'s own "pause/resume
    /// state machine" doc for the hover-vs-focus asymmetry, taken directly
    /// from the vendored tabbed reference's own accessibility prose).
    /// Mirrors `embla-carousel-autoplay`'s own `stopOnMouseEnter` option
    /// and its default (`true`).
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub stop_on_mouse_enter: ReadSignal<bool>,

    /// Whether this carousel should start rotating on mount at all --
    /// before the one-shot `prefers-reduced-motion` check below, which
    /// always forces it off regardless of this value (APG's own reference
    /// does the same: `dev-docs/archive/research/carousel-2026-09-19.md` §1.3
    /// point 4). Defaults to `true`, matching `embla-carousel-autoplay`'s
    /// own default of starting immediately.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub default_playing: ReadSignal<bool>,
}

/// # CarouselAutoplay
///
/// Installs the APG "auto-rotating" carousel's timer behavior: ticks every
/// [`CarouselAutoplayProps::delay_ms`], paging through the same [`step_next`]
/// path [`CarouselNext`] uses (so [`CarouselProps::r#loop`] applies
/// identically -- without `loop`, rotation simply stops once it reaches
/// the last slide, rather than ticking forever against a no-op the way
/// `embla-carousel-autoplay`'s own documented behavior does; with `loop`,
/// it rewinds and keeps going). Renders no DOM of its own -- pair it with
/// a [`CarouselRotationControl`] (placed **first**, before any other
/// focusable content, per APG's own requirement) to give the user a way to
/// toggle it, and see [`AutoplayContext`]'s own doc for the full
/// pause/resume state machine (hover, focus, and the sticky
/// "does not auto-resume after focus" rule).
///
/// `aria-live` on [`CarouselContent`] (`"off"` while rotating, `"polite"`
/// once stopped) and the rotation control's own toggling label are both
/// driven by this component's presence -- neither exists at all in a
/// [`Carousel`] with no [`CarouselAutoplay`] mounted, so a non-autoplay
/// carousel's markup carries neither.
///
/// `prefers-reduced-motion: reduce` always forces rotation off at mount,
/// regardless of [`CarouselAutoplayProps::default_playing`] -- checked
/// once, client-side only, via a post-mount effect that starts from the
/// same deterministic value SSR and the client's own pre-hydration first
/// render already agree on, so there is no hydration mismatch (mirrors
/// [`Carousel`]'s own `is_first`/mount-settle construction).
///
/// This must be used inside a [`Carousel`] component.
#[component]
pub fn CarouselAutoplay(props: CarouselAutoplayProps) -> Element {
    let ctx: CarouselContext = use_context();
    let autoplay = ctx.autoplay;

    // Publish presence + the two opt-outs every render -- plain signal
    // writes, cheap even when unchanged (Dioxus signals already no-op a
    // same-value write), the same "publish into a shared context slot"
    // idiom `CarouselContent`'s own `content_id` publish above uses.
    use_effect(move || {
        let mut present = autoplay.present;
        let mut stop_on_interaction = autoplay.stop_on_interaction;
        let mut stop_on_mouse_enter = autoplay.stop_on_mouse_enter;
        present.set(true);
        stop_on_interaction.set((props.stop_on_interaction)());
        stop_on_mouse_enter.set((props.stop_on_mouse_enter)());
    });

    // Mount-time default + the one-shot `prefers-reduced-motion` read,
    // consumed exactly once -- mirrors `Carousel`'s own `is_first`
    // construction (see that effect's own doc): SSR and the client's
    // pre-hydration first render both start `playing` from the same
    // deterministic value (`default_playing`), so there is no hydration
    // mismatch; this effect then corrects it once, client-side only, if
    // the platform prefers reduced motion.
    let mut checked_reduced_motion = use_signal(|| false);
    use_effect(move || {
        if *checked_reduced_motion.peek() {
            return;
        }
        checked_reduced_motion.set(true);
        let mut playing = autoplay.playing;
        playing.set((props.default_playing)());
        spawn(async move {
            let mut eval = document::eval(CAROUSEL_REDUCED_MOTION_JS);
            if let Ok(true) = eval.recv::<bool>().await {
                playing.set(false);
            }
        });
    });

    // The ticking loop. Restarts fresh (no partial-elapsed carry-over
    // across a pause/resume -- a deliberate simplification) every time
    // `rotating` flips, cancelling whatever was previously running first
    // -- mirrors `menu_sub.rs`'s own `DelayedAction::schedule` cancel-
    // before-reschedule idiom, this crate's established precedent for a
    // cancellable spawned timer. Always torn down: both on every flip to
    // `false` (this effect's own re-run, via the `take()` below) and on
    // unmount (`use_effect_cleanup`), so no timer ever outlives the
    // component or a single "rotating" span -- `Task::cancel()` is
    // immediate, not cooperative.
    let mut running: Signal<Option<Task>> = use_signal(|| None);
    let count = ctx.count;
    let selected = ctx.selected;
    let loop_enabled = ctx.loop_enabled;
    let loop_mode = ctx.loop_mode;
    let virtualized_loop_active = ctx.virtualized_loop_active;
    let set_selected = ctx.set_selected;
    // Autoplay is motion nobody can see while the carousel is off-screen or the tab is hidden:
    // the timer is dropped then (no wakeups, no slide changing under a reader who scrolled
    // away) and restarts with a fresh full delay on return, never an instant jump. The
    // stopped-for-good and hover/focus rules above are independent of it; this only adds "visible".
    let in_view = crate::activity::use_motion_active_for(ctx.content_id);
    use_effect(move || {
        let active = (autoplay.rotating)() && in_view();
        if let Some(task) = running.write().take() {
            task.cancel();
        }
        if !active {
            return;
        }
        let delay = Duration::from_millis((props.delay_ms)().max(1));
        let task = spawn(async move {
            loop {
                sleep(delay).await;
                let count_now = *count.peek();
                if count_now == 0 {
                    continue;
                }
                // `.peek()` on each field individually, matching
                // `count`/`selected` just above -- this is inside a spawned
                // async loop, not a reactive scope, so nothing here should
                // subscribe to anything (`CarouselContext::loop_wraps`
                // itself calls each field via tracked syntax, the wrong
                // read for this position).
                let loop_now = *loop_enabled.peek()
                    && (*loop_mode.peek() == LoopMode::Rewind || *virtualized_loop_active.peek());
                let current = *selected.peek();
                // Stops rather than ticking forever against a no-op once a
                // non-looping carousel reaches its last slide -- see this
                // component's own doc, "pick shadcn's behavior and
                // document it": `embla-carousel-autoplay`'s own documented
                // behavior keeps its interval alive indefinitely once
                // `scrollNext()` becomes a permanent no-op at the end;
                // this construction instead cancels the timer outright,
                // which is observably identical (nothing further ever
                // advances) but does not spin a no-op interval forever.
                if !loop_now && current + 1 >= count_now {
                    break;
                }
                set_selected.call(step_next(current, count_now, loop_now));
            }
        });
        running.set(Some(task));
    });
    use_effect_cleanup(move || {
        if let Some(task) = running.write().take() {
            task.cancel();
        }
    });

    rsx! {}
}

/// The props for the [`CarouselRotationControl`] component.
#[derive(Props, Clone, PartialEq)]
pub struct CarouselRotationControlProps {
    /// Additional attributes to apply to the button element.
    #[props(extends = GlobalAttributes)]
    #[props(extends = button)]
    pub attributes: Vec<Attribute>,

    /// The children of the button -- its visible content (e.g. a
    /// play/pause icon). The accessible name comes from `aria-label`
    /// (defaulted to the two toggling strings below) regardless of what is
    /// rendered here.
    pub children: Element,
}

/// # CarouselRotationControl
///
/// A native `<button type="button">` that toggles [`CarouselAutoplay`]'s
/// rotation on and off. Accessible name toggles between `"Start automatic
/// slide show"`/`"Stop automatic slide show"` (overridable) -- deliberately
/// **no** `aria-pressed`: the changing label itself is the state (APG's
/// own explicit rotation-control contract, `dev-docs/archive/research/carousel-2026-09-19.md`
/// §1.2/§1.3). Activating it never moves focus (native `<button>`
/// semantics), so a user can toggle rotation repeatedly without losing
/// their place.
///
/// **Composition:** place this **first** among [`Carousel`]'s children --
/// before [`CarouselPrevious`]/[`CarouselIndicators`]/[`CarouselContent`] --
/// so it is the first focusable element in the carousel, matching APG's
/// own explicit requirement ("Rotation control ... precede\[s\] the slide
/// content in the Tab sequence"). This component cannot enforce that
/// order itself, only document it -- the same tradeoff [`Carousel`]'s own
/// doc already makes for [`CarouselPrevious`]/[`CarouselNext`] preceding
/// [`CarouselContent`].
///
/// This must be used inside a [`Carousel`] alongside a [`CarouselAutoplay`]
/// -- clicking it before one has mounted is a harmless no-op (there is
/// nothing to toggle yet).
#[component]
pub fn CarouselRotationControl(props: CarouselRotationControlProps) -> Element {
    let ctx: CarouselContext = use_context();
    let mut autoplay = ctx.autoplay;
    let playing = (autoplay.playing)();
    let default_label = (!has_own_accessible_name(&props.attributes)).then_some(if playing {
        "Stop automatic slide show"
    } else {
        "Start automatic slide show"
    });

    // Merged (caller-wins for the overridable label default), not a
    // literal beside a bare `..props.attributes` spread
    // (`scripts/check-attr-spread-collision.sh`).
    let attributes = merge_attributes(vec![
        attributes!(button {
            r#type: "button",
            aria_label: default_label
        }),
        props.attributes,
    ]);

    rsx! {
        button {
            onclick: move |_| {
                autoplay.playing.set(!playing);
                // An explicit click always re-arms resume, whichever
                // direction it toggles -- "if a user activates the
                // rotation control button ... it is assumed the user
                // wants auto-rotation to start immediately" (the vendored
                // tabbed reference's own accessibility-features prose) --
                // so a later focus-driven pause never inherits a stale
                // block left over from before this click.
                autoplay.resume_blocked.set(false);
            },
            // Stops its own `focusin`/`focusout` from reaching `Carousel`'s
            // root (the `focus_within`/`resume_blocked` pause) -- note
            // `focusin`/`focusout` specifically, the bubbling pair the root
            // actually listens for, not `focus`/`blur` (a non-bubbling
            // pair whose own propagation is a separate dispatch entirely;
            // stopping *that* one does nothing to the other, so stopping
            // `focus`/`blur` would leave the failure below in place). A
            // narrowly-scoped exemption, NOT the vendored reference's own
            // broader "ignore hover/focus once explicitly started" quirk
            // (this module's own doc already declines to port that class
            // of behavior, `dev-docs/archive/research/carousel-2026-09-19.md` §1.3
            // point 3): a mouse click on this very button moves DOM focus
            // onto it (standard Chromium `<button>` behavior), and without
            // this, `focus_within` would immediately re-pause the very
            // rotation this click just started, defeating a mouse click's
            // own visible effect (`playwright/carousel.spec.ts`'s own
            // "autoplay" describe block covers it). Keyboard/pointer focus on every
            // OTHER focusable piece of the carousel (Previous/Next, the
            // content track, a `CarouselIndicator`) still pauses normally.
            onfocusin: move |event: Event<FocusData>| event.stop_propagation(),
            onfocusout: move |event: Event<FocusData>| event.stop_propagation(),
            ..attributes,

            {props.children}
        }
    }
}
