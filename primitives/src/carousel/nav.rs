use super::*;

/// The props shared by [`CarouselPrevious`] and [`CarouselNext`].
#[derive(Props, Clone, PartialEq)]
pub struct CarouselPreviousProps {
    /// Additional attributes to apply to the button element.
    #[props(extends = GlobalAttributes)]
    #[props(extends = button)]
    pub attributes: Vec<Attribute>,

    /// The children of the button -- its visible content. The accessible
    /// name comes from `aria-label` (defaulted to `"Previous slide"`)
    /// regardless of what is rendered here, so this can be purely
    /// decorative (an icon, an arrow glyph).
    pub children: Element,
}

/// # CarouselPrevious
///
/// A native `<button type="button">` that moves to the previous slide,
/// genuinely `disabled` (not merely `aria-disabled`) when already at the
/// first slide -- unless [`CarouselProps::r#loop`] is enabled, in which
/// case this is never `disabled` at all (it wraps to the last slide
/// instead). Default accessible name `"Previous slide"`, overridable with
/// your own `aria-label`/`aria-labelledby`. `aria-controls` points at the
/// sibling [`CarouselContent`]'s own id.
///
/// This must be used inside a [`Carousel`] component.
#[component]
pub fn CarouselPrevious(props: CarouselPreviousProps) -> Element {
    let ctx: CarouselContext = use_context();
    let disabled = ctx.prev_disabled();
    let (onfocusin, onfocusout) = use_nav_focus_hand_off(ctx, CarouselContext::prev_disabled);
    let default_label = (!has_own_accessible_name(&props.attributes)).then_some("Previous slide");
    let content_id = (ctx.content_id)();
    let aria_controls = (!content_id.is_empty()).then_some(content_id);

    // Merged (caller-wins for the presentational defaults, then
    // component-owned state wins last), not a literal beside a bare
    // `..props.attributes` spread (`scripts/check-attr-spread-collision.sh`).
    // `type`/`aria_label` are overridable defaults (a caller wanting
    // `type="submit"` or their own label still can); `aria_controls`/
    // `disabled` are ARIA-relationship and real-boundary state this
    // component must keep correct, so they go last -- owned-wins preserves
    // the same markup in SSR and hydrate, since a caller
    // could otherwise (attempt to) re-enable a button already correctly
    // disabled at a real boundary.
    let attributes = merge_attributes(vec![
        attributes!(button {
            r#type: "button",
            aria_label: default_label
        }),
        props.attributes,
        attributes!(button {
            aria_controls,
            disabled
        }),
    ]);

    rsx! {
        button {
            onclick: move |_| {
                let wraps_now = ctx.loop_wraps();
                ctx.set_selected.call(step_prev((ctx.selected)(), (ctx.count)(), wraps_now));
                ctx.autoplay.note_interaction();
            },
            onfocusin,
            onfocusout,
            ..attributes,

            {props.children}
        }
    }
}

/// # CarouselNext
///
/// The mirror of [`CarouselPrevious`]: moves to the next slide, `disabled`
/// at the last slide unless [`CarouselProps::r#loop`] is enabled (in which
/// case it wraps to the first slide instead and is never `disabled`),
/// default accessible name `"Next slide"`.
///
/// This must be used inside a [`Carousel`] component.
#[component]
pub fn CarouselNext(props: CarouselPreviousProps) -> Element {
    let ctx: CarouselContext = use_context();
    let disabled = ctx.next_disabled();
    let (onfocusin, onfocusout) = use_nav_focus_hand_off(ctx, CarouselContext::next_disabled);
    let default_label = (!has_own_accessible_name(&props.attributes)).then_some("Next slide");
    let content_id = (ctx.content_id)();
    let aria_controls = (!content_id.is_empty()).then_some(content_id);

    // See `CarouselPrevious`'s identical construction, just above, for the
    // full precedence rationale.
    let attributes = merge_attributes(vec![
        attributes!(button {
            r#type: "button",
            aria_label: default_label
        }),
        props.attributes,
        attributes!(button {
            aria_controls,
            disabled
        }),
    ]);

    rsx! {
        button {
            onclick: move |_| {
                let wraps_now = ctx.loop_wraps();
                ctx.set_selected.call(step_next((ctx.selected)(), (ctx.count)(), wraps_now));
                ctx.autoplay.note_interaction();
            },
            onfocusin,
            onfocusout,
            ..attributes,

            {props.children}
        }
    }
}

/// A read-only snapshot of a [`Carousel`]'s state, plus the one action
/// ([`Self::scroll_to`]) that does not already have a dedicated button
/// component -- returned by [`use_carousel`]. Mirrors shadcn's own
/// `CarouselApi` shape (`selected`/`count`/`can_scroll_prev`/
/// `can_scroll_next`) closely enough that a caller who has used shadcn's
/// Carousel (or `dignifiedquire/dx-components`' own `CarouselApi`,
/// `use_carousel`) will recognize it immediately.
#[derive(Clone, Copy, PartialEq)]
pub struct CarouselApi {
    /// The currently selected slide's index (0-based).
    pub selected: usize,
    /// The number of currently-registered [`CarouselItem`]s.
    pub count: usize,
    /// Whether [`Self::scroll_to`] to `selected - 1` would move anywhere.
    pub can_scroll_prev: bool,
    /// Whether [`Self::scroll_to`] to `selected + 1` would move anywhere.
    pub can_scroll_next: bool,
    scroll_to: Callback<usize>,
    autoplay: AutoplayContext,
}

impl CarouselApi {
    /// Move directly to the given slide index (clamped into range the
    /// same way every other path in this module is). For building a
    /// custom picker (e.g. a row of dot indicators) alongside
    /// [`CarouselPrevious`]/[`CarouselNext`], which only ever step by one.
    /// Counts as "interaction" for [`CarouselAutoplayProps::stop_on_interaction`]
    /// the same way clicking Previous/Next does.
    pub fn scroll_to(&self, index: usize) {
        self.autoplay.note_interaction();
        self.scroll_to.call(index);
    }
}

/// Read the nearest ancestor [`Carousel`]'s current state and reach its
/// `scroll_to` action, for building custom controls (e.g. dot indicators)
/// alongside or instead of [`CarouselPrevious`]/[`CarouselNext`].
///
/// Panics (via [`use_context`]) if called outside a [`Carousel`].
///
/// ## Example
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::carousel::{Carousel, CarouselContent, CarouselItem, use_carousel};
/// #[component]
/// fn Indicators() -> Element {
///     let api = use_carousel();
///     rsx! {
///         for i in 0..api.count {
///             button {
///                 "aria-label": "Go to slide {i + 1}",
///                 "data-active": i == api.selected,
///                 onclick: move |_| api.scroll_to(i),
///             }
///         }
///     }
/// }
/// ```
pub fn use_carousel() -> CarouselApi {
    let ctx: CarouselContext = use_context();
    let selected = (ctx.selected)();
    let count = (ctx.count)();
    let wraps = ctx.loop_wraps();
    CarouselApi {
        selected,
        count,
        can_scroll_prev: (wraps && count > 0) || can_scroll_prev(selected),
        can_scroll_next: (wraps && count > 0) || can_scroll_next(selected, count),
        scroll_to: ctx.set_selected,
        autoplay: ctx.autoplay,
    }
}
