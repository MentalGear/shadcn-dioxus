use super::*;

/// Shared roving-focus state for one [`CarouselIndicators`], consumed by its
/// [`CarouselIndicator`] children -- the tablist's *own* [`CollectionState`],
/// entirely separate from [`CarouselContext`]'s own `selected`/`count`
/// (which stay the single source of truth for which slide is current;
/// this collection only tracks *focus* among the tab buttons themselves,
/// the same separation `tabs.rs`'s own `TabsContext`/`CollectionState`
/// pair keeps).
#[derive(Clone, Copy)]
pub(super) struct CarouselIndicatorsContext {
    focus: CollectionState,
    /// Reused verbatim from [`CarouselContext::direction`] -- see that
    /// field's own doc for why a fresh `use_direction(None)` call here
    /// would risk disagreeing with the [`Carousel`] root's own resolution.
    direction: ReadSignal<Direction>,
}

/// The props for the [`CarouselIndicators`] component.
#[derive(Props, Clone, PartialEq)]
pub struct CarouselIndicatorsProps {
    /// Additional attributes to apply to the tablist element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The [`CarouselIndicator`] children.
    pub children: Element,
}

/// # CarouselIndicators
///
/// The slide-picker row for the APG **tabbed** carousel style:
/// `role="tablist"` containing one [`CarouselIndicator`] per slide. Composing
/// this on this crate's own `crate::collection` roving-focus machinery
/// (the same module [`crate::tabs::Tabs`] itself is built on) rather than
/// [`crate::tabs::Tabs`]/`TabTrigger`/`TabContent` directly is a
/// deliberate choice, not an oversight -- two concrete reasons that rule
/// out the literal composition:
///
/// 1. **`TabContent` hides its inactive panels** (`hidden: !selected()`).
///    A carousel's slides must all stay simultaneously present in
///    [`CarouselContent`]'s own scroll-snap track -- "showing" a slide
///    here means scrolling to it, never toggling `hidden` -- so
///    `TabContent` is structurally the wrong shape regardless of any
///    other consideration. [`CarouselItem`] already exists and is reused
///    as is (just switching its own `role` to `tabpanel`, see that
///    component's own doc).
/// 2. **`TabTrigger`'s own arrow-key handling is manual activation**
///    (`tabs.rs`: arrow keys only move focus; only `Enter`/`Space`/click
///    actually switches the active tab, via `TabTrigger`'s `onclick`).
///    APG's tabbed carousel requires **automatic** activation instead --
///    "Right Arrow: Moves focus to the next tab ... Shows the slide
///    associated with the newly focused tab" (no `Enter` needed,
///    `examples/carousel-2-tablist.html`'s own keyboard table). Building
///    [`CarouselIndicator`] directly on `crate::collection` (below) makes this
///    one line (`onfocus` calling [`CarouselContext::set_selected`]
///    directly) instead of a manual-activation component fighting its own
///    contract.
///
/// What *is* reused (compose, don't reimplement): the exact same `collection_item`/`use_item`/
/// `CollectionState` roving-tabindex machinery `TabTrigger` itself is
/// built on (`crate::collection`, `pub(crate)` and already shared across
/// this crate) -- so this is still composition on `tabs.rs`'s own
/// underlying focus engine, one layer below the `Tabs`/`TabTrigger`
/// components themselves, not a hand-rolled reimplementation of roving
/// focus.
///
/// Arrow-key wrap (`ArrowLeft`/`ArrowRight` -- RTL-aware, reusing
/// [`Carousel`]'s own resolved [`Direction`] -- wraps at both ends
/// unconditionally, per APG's own tablist keyboard table ("If focus is on
/// the last tab, moves focus to the first tab"). This is the *tablist's*
/// own always-circular navigation, independent of [`CarouselProps::r#loop`]
/// (which governs [`CarouselPrevious`]/[`CarouselNext`]/the root keyboard
/// instead, and defaults to `false`).
///
/// This must be used inside a [`Carousel`] component.
#[component]
pub fn CarouselIndicators(props: CarouselIndicatorsProps) -> Element {
    let ctx: CarouselContext = use_context();
    let mut tablist_present = ctx.tablist_present;

    use_effect(move || {
        tablist_present.set(true);
    });

    // Always wraps (`ReadSignal::new(Signal::new(true))`) -- see this
    // component's own doc, "Arrow-key wrap."
    let focus = use_collection_provider(ReadSignal::new(Signal::new(true)));
    use_context_provider(|| CarouselIndicatorsContext {
        focus,
        direction: ctx.direction,
    });

    let has_name = has_own_accessible_name(&props.attributes);
    // "Choose slide to display" -- this crate's own design sketch's
    // suggested name (`dev-docs/archive/research/carousel-2026-09-19.md` §5.4);
    // the vendored reference itself uses the shorter `aria-label="Slides"`
    // (`examples/carousel-2-tablist.html`) -- APG requires *a* meaningful
    // name, not this exact string, so either is conformant; overridable
    // either way.
    let default_label = (!has_name).then_some("Choose slide to display");

    let attributes = merge_attributes(vec![
        attributes!(div {
            aria_label: default_label
        }),
        props.attributes,
        attributes!(div { role: "tablist" }),
    ]);

    rsx! {
        div {
            ..attributes,
            {props.children}
        }
    }
}

/// The props for the [`CarouselIndicator`] component.
#[derive(Props, Clone, PartialEq)]
pub struct CarouselIndicatorProps {
    /// The index of the slide this tab controls (0-based) -- the same
    /// convention [`CarouselItem::index`] and `tabs.rs`'s own
    /// `TabTrigger`/`TabContent` `index` props use.
    pub index: ReadSignal<usize>,

    /// Additional attributes to apply to the tab button element.
    #[props(extends = GlobalAttributes)]
    #[props(extends = button)]
    pub attributes: Vec<Attribute>,

    /// The children of the tab -- its visible content (e.g. a dot/thumbnail
    /// icon). The accessible name comes from `aria-label` (defaulted to
    /// `"Slide {n}"`, matching the vendored reference's own
    /// `aria-label="Slide X"`) regardless of what is rendered here.
    pub children: Element,
}

/// # CarouselIndicator
///
/// One slide-picker button: `role="tab"`, roving `tabindex`, `aria-selected`,
/// `aria-controls` pointing at the matching [`CarouselItem`]'s own id.
/// Focusing a tab -- by `Tab`/`Shift+Tab` landing on it (already the
/// selected one, roving tabindex) or by `ArrowLeft`/`ArrowRight`/`Home`/`End`
/// moving focus onto it -- **automatically** selects and scrolls to its
/// slide (no `Enter`/click needed), matching APG's own tabbed-style
/// automatic-activation contract. See [`CarouselIndicators`]'s own doc for
/// why this is a purpose-built component on `crate::collection` rather
/// than [`crate::tabs::TabTrigger`] directly (manual activation there is
/// the wrong contract for this pattern).
///
/// This must be used inside a [`CarouselIndicators`] component, with indices
/// matching the sibling [`CarouselItem`]s one-to-one.
#[component]
pub fn CarouselIndicator(props: CarouselIndicatorProps) -> Element {
    let carousel_ctx: CarouselContext = use_context();
    let tablist_ctx: CarouselIndicatorsContext = use_context();
    let index = props.index;

    let is_selected = use_memo(move || (carousel_ctx.selected)() == index());

    let item =
        use_item(collection_item(tablist_ctx.focus, index).selected(move || is_selected.cloned()));
    let onmounted = item.onmounted();
    let tabindex = item.tabindex;

    let default_label =
        (!has_own_accessible_name(&props.attributes)).then(|| format!("Slide {}", index() + 1));
    let aria_controls = (carousel_ctx.item_ids)()
        .get(index())
        .cloned()
        .filter(|s| !s.is_empty());

    // Merged (caller-wins for the overridable label default, then
    // component-owned ARIA/roving state last), not a literal beside a
    // bare `..props.attributes` spread (`scripts/check-attr-spread-collision.sh`).
    let attributes = merge_attributes(vec![
        attributes!(button {
            r#type: "button",
            aria_label: default_label
        }),
        props.attributes,
        attributes!(button {
            role: "tab",
            tabindex: tabindex,
            aria_selected: is_selected(),
            aria_controls: aria_controls,
        }),
    ]);

    rsx! {
        button {
            onmounted,
            onfocus: move |_| {
                let mut focus = tablist_ctx.focus;
                focus.set_focus(Some(index()));
                carousel_ctx.set_selected.call(index());
                carousel_ctx.autoplay.note_interaction();
            },
            onkeydown: move |event: Event<KeyboardData>| {
                let key = event.key();
                let mut focus = tablist_ctx.focus;
                let mut prevent_default = true;
                match key {
                    Key::ArrowLeft | Key::ArrowRight => {
                        match (tablist_ctx.direction)().resolve_horizontal(&key) {
                            Some(HorizontalNav::Prev) => focus.focus_prev(),
                            Some(HorizontalNav::Next) => focus.focus_next(),
                            None => {}
                        }
                    }
                    Key::Home => focus.focus_first(),
                    Key::End => focus.focus_last(),
                    _ => prevent_default = false,
                }
                if prevent_default {
                    event.prevent_default();
                }
            },
            ..attributes,

            {props.children}
        }
    }
}
