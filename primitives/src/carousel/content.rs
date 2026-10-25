use super::*;

/// The props for the [`CarouselContent`] component.
#[derive(Props, Clone, PartialEq)]
pub struct CarouselContentProps {
    /// The ID of the carousel content element.
    pub id: ReadSignal<Option<String>>,

    /// Whether a mouse/pen pointer drag on the track pages the carousel --
    /// see this component's own "Pointer drag" doc. Defaults to `true`;
    /// touch is never affected either way (it already scrolls natively).
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

    /// The [`CarouselItem`] children of the carousel content.
    pub children: Element,
}

/// # CarouselContent
///
/// The track's `scroll-snap-type` value: the single place the snap / no-snap
/// decision is made. Rendered into the inline style AND `data-snap`, which
/// the scripts read when they restore a suspended snap
/// ([`CarouselProps::drag_free`]).
pub(super) fn snap_type_value(orientation: CarouselOrientation, drag_free: bool) -> &'static str {
    match (drag_free, orientation) {
        (true, _) => "none",
        (false, CarouselOrientation::Horizontal) => "x mandatory",
        (false, CarouselOrientation::Vertical) => "y mandatory",
    }
}

/// The scroll-snap track: a native `overflow-{x,y}: auto;
/// scroll-snap-type` element containing every [`CarouselItem`]. This
/// element itself already clips its own overflowing children (that is
/// what `overflow: auto` means), so -- unlike an embla-style engine,
/// which translates a strip via `transform` and therefore needs a
/// *separate* `overflow: hidden` clipping wrapper -- no extra wrapper
/// element is needed here at all.
///
/// This must be used inside a [`Carousel`] component.
///
/// Carries `tabindex="0"`, unconditionally: like [`crate::scroll_area::ScrollArea`]'s
/// own doc puts it, "if you don't have any focusable content within the
/// scroll area, you should make the scroll area focusable" -- a
/// [`CarouselItem`] is not itself focusable unless its own content
/// happens to be, and axe's `scrollable-region-focusable` rule (WCAG
/// 2.1.1/2.1.3) confirms this: without it, a keyboard user
/// who has no pointer at all has no way to reach this element's own
/// native scroll (arrow keys/Page Up/Page Down once focused) at all --
/// [`CarouselPrevious`]/[`CarouselNext`] page one slide at a time, but
/// axe is still right that the scrollable region itself should be
/// independently reachable, e.g. for finer scroll control than a full
/// slide-step in the `multiple`-per-view layout.
///
/// ## Pointer drag
///
/// A mouse or pen drag anywhere on the track pages the carousel, the same
/// way shadcn/embla-style carousels do -- layered on this element's own
/// scroll-snap track (a `scrollBy` per pointer move, never a second,
/// competing animation engine), and disableable per instance via
/// [`CarouselContentProps::draggable`] if a caller needs to opt out.
/// **Touch is never intercepted** -- it already scrolls this track
/// natively (`touch-action` is never set to `none` here), so a drag
/// gesture only ever starts for `pointerType === 'mouse' | 'pen'`.
///
/// Dragging is direct manipulation (the track tracks the pointer 1:1), so
/// under [`CarouselOrientation::Horizontal`] the physical direction that
/// reveals the next slide mirrors under `dir="rtl"` -- swipe-left-for-next
/// in LTR, swipe-right-for-next in RTL, the same familiar split a
/// right-to-left photo gallery or story viewer already has, and the same
/// direction [`Carousel`]'s own root-level `ArrowLeft`/`ArrowRight`
/// keyboard handling already swaps under RTL. No direction branch exists
/// in the drag code itself for this -- see `CAROUSEL_DRAG_JS`'s own doc
/// for why none is needed.
///
/// A drag has to move at least a few pixels from its origin before it
/// takes over -- below that threshold nothing happens at all, so a plain
/// click on a link/button inside a [`CarouselItem`] still works exactly
/// like a click. Once a real drag is recognized, the synthetic `click`
/// that would otherwise fire on release is suppressed (a capture-phase
/// listener on this element, ahead of both a native default action like
/// link navigation and any caller's own `onclick`), text selection is
/// suppressed for the gesture's duration, and pointer capture keeps the
/// drag tracking correctly even if the pointer leaves this element's
/// bounds mid-gesture.
///
/// On release, `endDrag` explicitly settles the track on whichever slide
/// is numerically nearest -- see this component's own "Release settle"
/// section below for the mechanism and why it is not simply
/// "restore `scroll-snap-type` and let the browser re-snap." That settle
/// (like every other native scroll) is what `use_carousel_scroll_tracking`
/// (already attached to this same element) picks up to update `selected`,
/// so a drag never invents a second source of truth for the index: the
/// Previous/Next buttons' `disabled` state, the "N of M" slide labels, and
/// a custom picker built on [`use_carousel`] all stay correct through the
/// exact same bridge a click or keypress already uses.
///
/// This gesture reads pointer state maintained entirely on the JS side
/// (see `CAROUSEL_DRAG_JS`'s own doc) and writes only a scroll position
/// per pointer move -- never a synchronous layout *read* in the same
/// handler (`dev-docs/reference/platform-rationale.md` §11 Rule 1); the one
/// read this construction needs (the container's own starting scroll
/// offset) is implicit in `scrollBy`'s own *relative* delta, so it is
/// never taken at all, on either side of the bridge. `endDrag` itself does
/// take a one-time `getBoundingClientRect` read once the pointer is
/// already released, to compute the settle target -- a discrete,
/// once-per-gesture event, not the continuous per-pointer-move path Rule 1
/// is about; see "Release settle" below.
///
/// ## Release settle
///
/// `endDrag` finds the slide nearest the track's raw (possibly unsnapped)
/// position at release -- the identical `getBoundingClientRect`-based
/// geometry `use_carousel_scroll_tracking`'s own `settle()` already uses
/// on this same element (a fresh `document::eval` string cannot import a
/// shared JS helper, so this is duplicated rather than called, but it is
/// the same technique, not a different one) -- and explicitly `scrollBy`s
/// this element (the scroller itself, never `nearest.scrollIntoView()`)
/// by the physical delta to that slide, with `behavior: 'smooth'` (or
/// `'auto'` under `prefers-reduced-motion: reduce`, the exact policy
/// `CAROUSEL_SCROLL_TO_JS` already applies to every other paging
/// path). This reuses that same scroller-only `scrollBy`-by-delta
/// mechanism this crate already pages with everywhere else -- not a
/// second, hand-rolled animation engine, and (see that constant's own
/// doc) not `scrollIntoView`, which would walk and scroll every
/// scrollable ancestor including the page.
///
/// Simply restoring `scroll-snap-type` and trusting the browser's own
/// re-snap to both choose the destination *and* animate the way there
/// does not work (`onPointerMove`'s own "why suspend scroll-snap-type"
/// comment describes the suspension): it never animates.
/// Sampling `.dx-carousel-content`'s own `scrollLeft` across such a
/// release yields exactly two distinct values -- the position mid-drag and
/// the final slide -- whatever the drag's length, both landing
/// instantly. Setting `scrollSnapType` back to `'x mandatory'` on a
/// position that is already stationary is not treated by this engine
/// (Chromium) as "a scrolling operation resuming through skipped snap
/// points"; it is a synchronous, static re-evaluation, and nothing
/// animates a style recalculation. Explicitly `scrollBy`-ing this element
/// to the destination slide's delta with `behavior: 'smooth'` sidesteps
/// that path entirely: it is a genuine, browser-animated scroll operation,
/// the same kind "## scroll-snap-stop" below documents as actually
/// constrained/animated by this engine.
///
/// Restoring `scroll-snap-type` is **deferred**, not synchronous, for the
/// same reason: doing it immediately, right after starting the
/// `scrollBy` call, would reintroduce the failure above the instant the
/// browser notices the position is (still, momentarily) not yet at a snap
/// point. So it waits for this specific scroll to actually finish
/// (`scrollend`, falling back to a fixed timeout where that event is
/// unsupported -- the same feature detection `use_carousel_scroll_tracking`
/// already uses, and the reason `CAROUSEL_SNAP_RESTORE_FALLBACK_MS`
/// exists) before touching the property again, so there is never a
/// stationary, unsnapped position for the browser to instantly correct
/// while this animation is still in flight. A fresh `pointerdown` cancels
/// any still-pending restore from a previous release rather than letting
/// it fire mid-*new*-drag and re-enable snapping while that drag's own
/// `scrollBy` calls are still running -- see `onPointerDown`'s own
/// comment.
///
/// This does not change *which* slide a long or fast drag lands on --
/// see "## scroll-snap-stop" below, "Known limitation," which applies
/// to the explicit-settle construction too.
///
/// ## Edge rubber-band (pointer: mode B3; wheel/trackpad: mode D)
///
/// Dragging (mouse/pen) or scrolling (wheel/trackpad) past either end
/// produces a damped elastic bounce, ported from a closed design (five
/// mechanisms benched side by side, `dev-docs/archive/research/carousel-overscroll-2026-09-23.md`,
/// `dev-docs/backlog.md` row 102) rather than reimplemented from scratch
/// here. The full mechanism, its measurements, and the eight binding port
/// invariants live in that doc's §3-§6; this section records only the
/// shape of the port and what stays true of it.
///
/// The two halves are separate bridges on the same element. The pointer
/// half (`CAROUSEL_DRAG_JS`) is mode B3. The wheel/trackpad half
/// (`CAROUSEL_WHEEL_BAND_JS`) is **mode D, the "platform-aware band"**,
/// chosen over B3's wheel path on real hardware (see the research doc):
/// it draws its band only for a gesture that *starts at rest* at an edge and
/// pushes into it, so a fling that arrives at the edge with momentum is left
/// to the platform's own overscroll instead of getting a second band stacked
/// on top. See that constant's own doc for the engine, its constants, its
/// deliberate differences from the bench, the debug telemetry flag, and the
/// open Safari double-bounce item.
///
/// **THE RULE this is built on:** at any moment during a wheel/trackpad
/// gesture on a scroll-snap container, the scroll position has three
/// would-be writers -- the compositor's own momentum animation (where
/// `preventDefault` is only advisory once momentum is live), scroll
/// snapping on its own schedule, and this crate's own code -- and two of
/// the three are neither observable nor sequenceable from JS. So where the
/// browser is already the scroller, this crate never writes the scroll
/// position and never disables snapping mid-gesture; the overdrag is
/// expressed purely as a `transform` on [`CarouselContent`]'s own element,
/// which the browser never writes and therefore never contests
/// (`CAROUSEL_WHEEL_BAND_JS`, above). A mouse/pen drag has no native
/// scrolling behind it, so that same rule permits `CAROUSEL_DRAG_JS`'s own
/// pointer path to keep owning the scroll position (`feedOverdrag`, which
/// asks the scroller for the whole of the requested motion and treats
/// whatever it refuses as the overdrag) -- the evidence behind this
/// asymmetry (the pointer path is clean untouched while the wheel path
/// glitches nearly every time) is measured, not assumed; see the research
/// doc §2.
///
/// **Why the transform lands on [`CarouselContent`]'s own *scroller*,
/// never the [`Carousel`] root.** The bench's own mode C (a whole-element
/// damped transform) felt closest to native but created a z-index
/// problem: a transform establishes a new containing block, and
/// translating the *root* -- an ancestor of
/// [`CarouselPrevious`]/[`CarouselNext`] -- would have dragged their own
/// absolutely-positioned placement along with it. The scroller is a
/// *sibling* of those buttons under [`Carousel`] (by way of the clipping
/// viewport wrapper described next), so transforming it alone leaves them
/// untouched; this is what dissolves that problem rather than working
/// around it (research doc §5).
///
/// **This element does *not* self-clip a translated overdrag**
/// ([research](../../dev-docs/archive/research/carousel-overscroll-2026-09-23.md)).
/// An element's own `overflow`
/// clips relative to *its own* box, and a `transform` moves that box (and
/// its clip region) as one unit -- it does not clip the box against where
/// it would have painted before the transform. So translating this
/// element by its own overdrag moves its clip boundary along with its
/// content, and nothing clips the result: a trackpad overdrag would push
/// the whole visible slide track outside the carousel, unclipped,
/// overlapping whatever sits beside it on the page. So the construction is
/// the bench's own: a *stationary* wrapper -- `data-slot="carousel-viewport"`, `overflow:
/// clip` -- around this element, so the element inside it is what
/// translates while the clip boundary around it never moves.
/// `overflow: clip`, not `overflow: hidden`: a `hidden` box is still a
/// scroll container, and this module's paging/drag-release-settle paths
/// do not risk scrolling it even so -- both call `scrollBy()` on
/// the scroller element directly rather than `scrollIntoView()` on a
/// slide (see `CAROUSEL_SCROLL_TO_JS`'s own doc for why, a hard
/// requirement rather than a nice-to-have)
/// -- but `clip` establishes no scrollport at all regardless, so nothing
/// here (or added later) can ever scroll this wrapper by accident, the
/// same defense-in-depth reasoning as never giving it an `id`/ARIA for a
/// stray selector to latch onto. See that wrapper's own doc comment,
/// right above this component's `rsx!` body, for the full construction.
///
/// **`display: flow-root`, alongside `overflow: clip`, on this same
/// wrapper.** `overflow: clip` alone does not stop this wrapper's
/// own vertical (block-axis) margin from collapsing through to its child
/// -- `CarouselContent`'s own scroller, under [`CarouselOrientation::Vertical`],
/// carries a NEGATIVE `margin-block-start` (the gap model's own
/// `content_gap_margin`, physically `margin-top` in this crate's only
/// writing mode), and margin collapsing is a block-axis-only phenomenon
/// (horizontal/inline margins never collapse at all, in any writing mode
/// -- which is exactly why the identical construction needs no such handling
/// for [`CarouselOrientation::Horizontal`]'s own width-absorption: no
/// collapsing is involved there). With only `overflow: clip`, this
/// wrapper's own rendered box shifts upward
/// by the gap's own size and its height silently absorbs the negative
/// margin (270px stays 270px instead of becoming 254px) -- i.e., this
/// wrapper's margin computes as if it had adopted its child's `-16px`
/// itself, encroaching 16px into `style.css`'s own `padding-block`
/// button-clearance reservation. `display: flow-root` is the explicit,
/// purpose-built CSS property for "this box establishes its own new block
/// formatting context" (unlike relying on `overflow` for the same effect,
/// which is really a side effect of a value meant for something else) --
/// alongside `overflow: clip` (kept for the scrollport-avoidance
/// reasoning above, which `flow-root` alone says nothing about), this
/// wrapper's own box neither shifts nor absorbs
/// its child's negative margin, and the child pokes out (clipped) by
/// exactly the margin's own size instead, matching the CSS2.1 auto-height
/// formula for a genuine new-BFC container.
///
/// **Hydration parity.** Neither bridge ever touches a Dioxus-rendered
/// attribute: both mutate `element.style.transform` as a plain DOM write,
/// the same escape hatch `CAROUSEL_DRAG_JS`'s own `scroll-snap-type`
/// suspend/restore already uses and for the same reason (`document::eval`
/// is outside the vdom entirely). So first-render/SSR markup is
/// unaffected by construction, not by a special case: there is no signal,
/// no conditional render, nothing for hydration to reconcile.
///
/// **`prefers-reduced-motion`** drops the whole effect on both paths
/// (checked fresh per gesture/event, not cached at mount) rather than
/// merely skipping the spring-back animation -- this is cosmetic feedback
/// layered on scrolling that already happens regardless, so "no bounce" is
/// the correct reading of that preference, not a faster one.
///
/// **Where the wheel band applies: only at a true end of the data.** Never on
/// a looping carousel (`r#loop` on, any [`LoopMode`], plain or virtual);
/// on a non-looping [`CarouselVirtualContent`] only when its real first or
/// last item is the rendered slide at that edge (not at its window's slice
/// edge mid-list); plain non-looping content at both ends. One
/// predicate, `isTrueEnd` in `CAROUSEL_WHEEL_BAND_JS`, reads it per gesture
/// from `data-loop`/`data-slide-count` on this element and each slide's
/// `data-index`. The pointer (mouse/pen) band has no such condition.
///
/// **What this does not close.** [`CarouselPrevious`]/[`CarouselNext`]
/// remain genuinely `disabled` at the ends regardless (module doc,
/// "Accessibility") -- the bounce is additional physical feedback for a
/// drag/scroll gesture, not a replacement for that signal. Headless
/// wheel events (`playwright/carousel-edge-wheel.spec.ts`'s own "wheel/trackpad edge
/// band" tests, dispatched in-page with real timing and sampled in-page)
/// exercise ownership, every release path and the spring-back with
/// recorded-shape delta sequences, but cannot reproduce a real trackpad's
/// compositor momentum or feel -- that remains a real-hardware check, as it
/// is for the bench itself (research doc's own header).
///
/// ## scroll-snap-stop
///
/// Every [`CarouselItem`] declares `scroll-snap-stop: always` as a plain
/// inline `style` property, right next to its own `scroll-snap-align`
/// (see that component's own source) -- matching every other
/// layout-critical property on this track/its slides (`scroll-snap-type`
/// here, `scroll-snap-align`/the flex layout there), this module's own
/// "works with zero theme CSS" convention (module doc, "Engine" section).
///
/// ### What this is for
///
/// `scroll-snap-stop: always` requires this scroll container to stop at
/// the *first* snap position a scroll operation would otherwise pass,
/// rather than skipping over several to settle on whichever is numerically
/// nearest. The design choice: express embla-carousel's default
/// (non-`dragFree`, non-`skipSnaps`) "never skip past more than one
/// slide" outcome natively, without reimplementing its velocity model, and
/// accept that a native-scroll engine will not *feel* identical to a
/// transform-based one. It caps a genuine
/// browser-animated scroll operation -- a caller's own
/// `el.scrollBy({ left, behavior: 'smooth' })`, and by extension a native
/// wheel/trackpad fling, since both are the same "smooth scrolling
/// operation" the CSS Scroll Snap spec's own `scroll-snap-stop` language
/// describes -- to exactly one slide of travel regardless of the
/// requested distance.
///
/// ### Why a plain inline declaration, not `@supports`
///
/// Gating the property behind a scoped `<style>` tag rendered as this
/// element's own preceding sibling -- `@supports (scroll-snap-stop:
/// always) { #id > * { ... } }` -- would express "only if the browser
/// supports it", which has no equivalent inside a plain `style="..."`
/// attribute, since `@supports` is a stylesheet at-rule. It does not work
/// under hydration. Dioxus emits hydration marker comments *inside* a
/// `<style>` element's own text content
/// (`<!--node-id413-->...<!--#-->`); `<!--`/`-->` are CDO/CDC tokens that
/// a stylesheet's top level normally ignores, but the ident immediately
/// following the opening one (`node-id413`) begins a qualified rule whose
/// prelude then swallows the real `@supports` rule and takes its own
/// `{...}` block as that rule's block, so the browser's CSS parser drops
/// the whole thing: on a hydrated release SSG build the `<style>` tag is
/// present with the correct `id`, matching the live element, and yet
/// `styleElement.sheet.cssRules.length` reads `0` and
/// `getComputedStyle(slide).scrollSnapStop` reads `"normal"` on every
/// slide -- a hydration-marker/CSS-parser hazard, not an id-scoping or
/// timing bug. (It would work in pure CSR, where Dioxus emits no
/// hydration markers at all.) A test that scrolls 2.5 slide widths and
/// asserts landing on slide 2 cannot catch it, because nearest-snap
/// settling alone gives that regardless of whether the property applies.
///
/// No conditional gating is actually needed to satisfy "only if the
/// browser supports it": an unrecognized property inside a `style="..."`
/// attribute is simply dropped by the browser's own CSS parser by
/// definition, so a browser without `scroll-snap-stop` support silently
/// keeps nearest-snap behavior -- the same fallback `@supports` would
/// produce, with no stylesheet, no id scoping, and no hydration-marker
/// hazard to get wrong.
///
/// ### Known limitation: this crate's own pointer-drag path does *not*
/// get this guarantee
///
/// `CAROUSEL_DRAG_JS`'s mouse/pen drag moves the track with many
/// discrete `scrollBy({ behavior: 'instant' })` calls while
/// `scroll-snap-type` is suspended for the gesture's own duration (see
/// that constant's own doc for why), so a long or fast drag's raw
/// position at release can land several slide-widths from the origin with
/// no scrolling operation having passed through the intervening snap
/// points at all -- each jump is independently instant and unsnapped.
///
/// Restoring `scroll-snap-type` on release and trusting the browser's own
/// re-snap to choose the destination does not constrain it either: that
/// restore is not treated as "a scrolling operation resuming past skipped
/// snap points," but as a fresh, static re-evaluation of an
/// already-stationary position, which this property does not appear to
/// constrain on this engine (Chromium). Raw `el.scrollBy({ behavior:
/// 'instant' })` in a loop with `scroll-snap-type` suspended, then
/// restored, on an element whose child already computes
/// `scroll-snap-stop: always` still settles on whichever slide is
/// numerically nearest the raw position, identical to what happens with
/// the property absent altogether. A same-tick zero-distance and 1px
/// `behavior: 'smooth'` nudge issued immediately after restoring
/// `scroll-snap-type`, as a cheap way to route the correction through the
/// constrained "smooth scroll" path instead, does not change this.
///
/// "## Release settle" above therefore does not rely on restoring
/// `scroll-snap-type` to pick a destination at all: it explicitly targets
/// whichever slide its own geometry search finds nearest, which is why
/// release visibly *animates* there. But the *destination* that search
/// finds is "whichever slide is numerically nearest the raw drag
/// position," with no notion of "cap to one slide from the origin" --
/// `scroll-snap-stop`'s own guarantee is about a scroll *operation* the
/// browser resolves against intervening snap points, and a plain
/// nearest-neighbor geometry search never asks the browser to resolve
/// anything, so it cannot inherit that guarantee either. So a long or
/// fast drag can still land several slides from the origin. This remains
/// an open gap for this crate's own pointer-drag feature specifically --
/// not for wheel/trackpad/native scrolling on the same track, and not for
/// a caller's own `scrollBy`, both of which this property does correctly
/// constrain.
///
/// Also not covered, and unreachable by construction rather than merely
/// unimplemented: a short, fast flick that settles back to its origin
/// instead of advancing is a velocity-shaped symptom, and no CSS property
/// can observe release speed. Closing it would mean a JS
/// distance-or-speed threshold -- i.e. reintroducing the velocity model
/// under another name -- which is declined by design; see
/// `dev-docs/backlog.md` row 96 for the fuller story.
///
/// ## Styling
///
/// The [`CarouselContent`] component defines the following data
/// attributes you can use to control styling:
/// - `data-orientation`: `horizontal` or `vertical`, matching the parent
///   [`Carousel`]'s own.
/// - `data-draggable`: `true` or `false`, matching
///   [`CarouselContentProps::draggable`].
/// - `data-dragging`: present (`"true"`) only while an active mouse/pen
///   drag has crossed the movement threshold above; this is what the
///   themed package's own `cursor: grabbing` styling keys off.
///
/// The scroller is wrapped in a presentational `div
/// [data-slot="carousel-viewport"]` (see this component's "Edge
/// rubber-band" doc, "does *not* self-clip" paragraph, above) that a caller's own
/// stylesheet can target for purely visual purposes; it carries none of
/// this component's own attributes/ARIA and is not part of the public
/// props surface.
#[component]
pub fn CarouselContent(props: CarouselContentProps) -> Element {
    let mut ctx: CarouselContext = use_context();
    let uuid = use_unique_id();
    let id = use_id_or(uuid, props.id);

    // Publish this element's own rendered id into the shared context so
    // `CarouselPrevious`/`CarouselNext` can point `aria-controls` at it --
    // the same construction `SelectTrigger`/`DropdownMenuTrigger` already
    // use for `trigger_id` (`dev-docs/conformance-harness.md` tier-2 Rule
    // 14), one layer in reverse (content publishing to its controls,
    // rather than a trigger publishing to its content).
    use_effect(move || {
        ctx.content_id.set(id());
    });

    let mut scroll_origin = ctx.scroll_origin;
    let tracked_set_selected = use_callback(move |index: usize| {
        ctx.set_selected.call(index);
        // After `set_selected` (which clears it): mark this one change as
        // the track's own, for drag-free's scroll-into-view skip.
        if (ctx.drag_free)() {
            scroll_origin.set(Some(index));
        }
    });
    use_carousel_scroll_tracking(
        id,
        ctx.orientation,
        ctx.align,
        tracked_set_selected,
        ctx.count,
        ctx.visible_range,
        None,
    );
    use_carousel_drag(id, ctx.orientation, ctx.align, props.draggable);
    use_carousel_wheel_band(id, ctx.orientation);

    let orientation = (ctx.orientation)();
    let draggable = (props.draggable)();

    // `aria-live` -- only when a `CarouselAutoplay` is actually present
    // (APG's own "optional" framing, `carousel-pattern.html`'s Roles
    // table -- a manually-paged carousel has no rotation to announce
    // around, so this stays absent, matching every
    // non-autoplay variant/test). `"off"` while rotating (so a screen reader is not
    // interrupted every few seconds), `"polite"` once stopped (so
    // activating a tab/pressing Previous/Next while stopped is announced)
    // -- `dev-docs/archive/research/carousel-2026-09-19.md` §1.2/§1.3 point 2:
    // deliberately no `aria-atomic`, matching the tested reference rather
    // than its own (self-contradicting) prose.
    let aria_live = (ctx.autoplay.present)().then(|| {
        if (ctx.autoplay.rotating)() {
            "off"
        } else {
            "polite"
        }
    });

    let (caller_style, rest_attrs) = fold_style_attributes(props.attributes);
    let gap_margin = content_gap_margin(orientation);
    let gap_var = content_gap_var(props.gap.as_deref());
    // Plain content renders every slide: both ends are true unless it loops.
    let overscroll = content_overscroll_style(orientation, !(ctx.loop_enabled)());
    let drag_free = (ctx.drag_free)();
    let snap_type = snap_type_value(orientation, drag_free);
    let axis_style = match orientation {
        CarouselOrientation::Horizontal => {
            "display:flex;flex-direction:row;overflow-x:auto;overflow-y:hidden;"
        }
        CarouselOrientation::Vertical => {
            "display:flex;flex-direction:column;overflow-y:auto;overflow-x:hidden;"
        }
    };
    let style = format!(
        "{axis_style}scroll-snap-type:{snap_type};{gap_margin}{gap_var}{overscroll}{}",
        caller_style.map(|s| format!(" {s}")).unwrap_or_default()
    );

    // Merged (owned-wins), spreading only the merged result -- never a
    // literal `style`/`tabindex`/`data-*` beside a raw `..rest_attrs`
    // spread (`scripts/check-attr-spread-collision.sh`). `rest_attrs`
    // already excludes any caller `style` (`fold_style_attributes` above
    // folded it into `style`, which is why it is safe to place `style`
    // itself in the owned group rather than concatenate it again here);
    // `tabindex`/the two `data-*` attributes are structural state this
    // component must keep correct (the scroll region's own focusability,
    // and the axis/drag-opt-out this crate's own CSS keys off), so they go
    // last, same as `Carousel`'s own root.
    let attributes = merge_attributes(vec![
        rest_attrs,
        attributes!(div {
            style: style.clone(),
            tabindex: "0",
            aria_live: aria_live,
            "data-orientation": orientation.as_str(),
            "data-draggable": draggable,
            "data-drag-free": drag_free,
            "data-snap": snap_type,
            // Read per gesture by the wheel band's `isTrueEnd` (the band only
            // engages at a true end of the data -- see "Edge rubber-band").
            // Both come from props/registration state, never an effect-only
            // value: SSR and the hydrating first render agree (count is 0 in
            // both until the items register, like their "N of M" labels).
            "data-loop": (ctx.loop_enabled)(),
            "data-slide-count": (ctx.count)(),
        }),
    ]);

    rsx! {
        // Clipping viewport (module doc's "Engine" section, and this
        // component's own "Edge rubber-band" doc above, have the full
        // reasoning): a stationary wrapper whose OWN box clips, so
        // translating the scroller inside it moves the content without
        // moving the clip boundary along with it. `overflow: clip`, never
        // `overflow: hidden` -- `hidden` is still a scroll container (the
        // CSSOM View spec's own "scrolling box" definition keys off any
        // `overflow` value other than `visible`/`clip`); the paging and
        // drag-release-settle paths never scroll THIS element either way
        // (both call `scrollBy()` on the scroller below directly, never
        // `scrollIntoView()` on a slide -- see `CAROUSEL_SCROLL_TO_JS`'s
        // own doc), but `clip` never creates a scrollport at all, so
        // nothing here or added later has anything to
        // scroll. Presentational only -- no `role`, no `id`, no ARIA, and
        // none of the caller's own `props.attributes` land here (every one
        // of those goes on the scroller below): this
        // element carries nothing hydration needs to reconcile and nothing
        // a caller can already reach through this component's own props,
        // so it needs no `attributes!`/`merge_attributes` construction
        // either -- there is no spread onto it for a literal to collide
        // with (`scripts/check-attr-spread-collision.sh`'s own class).
        // Present unconditionally on every render (SSR and CSR alike, and
        // never toggled by an effect), so hydration sees the identical
        // tree both times.
        div {
            "data-slot": "carousel-viewport",
            style: "overflow: clip; display: flow-root;",

            div {
                id,
                ..attributes,

                {props.children}
            }
        }
    }
}
