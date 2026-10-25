use super::*;

/// The props for the [`CarouselItem`] component.
#[derive(Props, Clone, PartialEq)]
pub struct CarouselItemProps {
    /// The index of this slide (0-based). Determines both its default
    /// `"{n} of {m}"` accessible name and where it counts toward
    /// [`CarouselApi::count`] -- siblings must use contiguous indices
    /// starting at `0`, the same convention `tabs.rs`'s `TabTrigger`/
    /// `TabContent` already require of their own `index` prop.
    pub index: ReadSignal<usize>,

    /// The ID of the carousel item element.
    pub id: ReadSignal<Option<String>>,

    /// Additional attributes to apply to the carousel item element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the carousel item -- the slide's own content.
    pub children: Element,
}

/// # CarouselItem
///
/// A single slide: `role="group"` + `aria-roledescription="slide"`, with
/// a default `aria-label` of `"{n} of {m}"` unless the caller supplies
/// its own `aria-label`/`aria-labelledby`.
///
/// This must be used inside a [`CarouselContent`] component.
///
/// ## SSR stability
///
/// The `"{n} of {m}"` count is only known once every sibling
/// [`CarouselItem`] has registered itself, which -- like `tabs.rs`'s own
/// `tab_content_ids` -- happens via each item's own effect, and effects
/// do not run during SSR. So an SSR (or a client's own pre-hydration
/// first) render always sees `count == 0` and renders **no** `aria-label`
/// at all, filling it in once mounted. Server and the client's own first
/// render therefore always agree (both start from the same empty
/// registry) -- nothing here changes what SSR produced out from under
/// hydration; the label simply arrives, correctly, a moment later, the
/// same tradeoff `tabs.rs`'s `aria-controls` makes.
///
/// ## Visible slides / `inert`
///
/// Every [`CarouselItem`] the a11y contract's own "visible" set does not
/// include (`dev-docs/archive/research/carousel-loop-2026-09-25/loop-a11y-guidance.md`
/// §7, cross-checked against every a11y-layer-having carousel library's
/// own converged construction in `dev-docs/archive/research/carousel-loop-2026-09-25/loop-libraries.md`) renders `inert` --
/// removed from both the Tab order and the accessibility tree in one
/// attribute, the Chrome "make accessible carousels" recipe's own
/// mechanism. "Visible" is the current slide (always, regardless of the
/// bridges below -- a safety net, not the primary signal: never leaves
/// the slide the rest of this component's own state calls "selected"
/// stranded `inert` by a stale/lagging geometry read) unioned with
/// `CarouselContext::visible_range`, the `[start, end]` item indices
/// `use_carousel_scroll_tracking`/`use_carousel_drag` last measured
/// actually inside [`CarouselContent`]'s own viewport at rest -- which for
/// a single-slide-per-view layout is always just the current slide again,
/// and for a multi-per-view one (e.g. `flex-basis: 33%`) additionally
/// covers however many full neighbours are genuinely on screen.
///
/// **First render / SSR is deterministic.** `CarouselContext::selected`
/// is forced to `0` until every sibling has registered (this component's
/// own "SSR stability" doc above), and `CarouselContext::visible_range`
/// starts at its own matching default, `(0, 0)` -- so server and a
/// client's own pre-hydration first render both render only index `0` as
/// non-`inert`, agreeing without either needing a special case for the
/// other. A multi-per-view layout's true visible count is not knowable
/// from Rust-only state at all (it depends on `flex-basis`/gap CSS this
/// component never reads) -- widening past that single slide is real, but
/// always happens through a later effect's own signal write
/// (`use_carousel_scroll_tracking`'s own `ResizeObserver`, whose first
/// callback fires for the scroller's *current* box the moment it attaches
/// -- reached even for an already-aligned `default_value == 0` mount that
/// never scrolls on its own at all -- see that constant's own doc), never
/// inside this component's own first render.
///
/// **Frozen while moving.** `use_carousel_scroll_tracking`'s own settle
/// only ever writes `visible_range` at a genuine rest point -- on
/// `scrollend` (or the debounced-`scroll` fallback) or a `ResizeObserver`
/// firing, explicitly skipped for as long as `data-dragging` is set -- never
/// mid-drag, mid-momentum, or mid-smooth-page (the same rule
/// `dev-docs/archive/research/carousel-overscroll-2026-09-23.md` §3/§6 already
/// binds every other scroll-position write in this module to). A drag's own
/// release settle reaches this only indirectly: `data-dragging` is removed
/// before `endDrag`'s own settle `scrollBy` runs, so the resulting native
/// scroll/scrollend event is what this bridge actually observes.
///
/// ## Styling
///
/// The [`CarouselItem`] component defines the following data attribute
/// you can use to control styling:
/// - `data-selected`: `true` when this is the currently selected slide,
///   `false` otherwise.
#[component]
pub fn CarouselItem(props: CarouselItemProps) -> Element {
    let mut ctx: CarouselContext = use_context();
    let uuid = use_unique_id();
    let id = use_id_or(uuid, props.id);
    let index = props.index;

    // Grow-only registration by index, mirroring `tabs.rs`'s
    // `tab_content_ids` exactly (see this component's "SSR stability"
    // doc above for why that shape is what keeps this SSR-stable).
    use_effect(move || {
        let idx = index();
        let mut ids = ctx.item_ids.write();
        while ids.len() <= idx {
            ids.push(String::new());
        }
        ids[idx] = id();
    });

    let selected = (ctx.selected)();
    let count = (ctx.count)();
    let is_selected = selected == index();
    let has_name = has_own_accessible_name(&props.attributes);
    let default_label = (!has_name && count > 0).then(|| slide_label(index(), count));

    // See this component's own "Visible slides / `inert`" doc above.
    //
    // `Option<&'static str>` (`None` when visible, never a bare `false`
    // `bool`) is load-bearing, not a style preference: `dioxus-interpreter-js`
    // 0.7.9's `set_attribute.ts` special-cases which HTML attributes are
    // "boolean" (present/absent) via its own hardcoded `isBoolAttr`
    // allowlist -- `disabled`/`hidden`/`checked`/etc. are on it, `inert` is
    // not (a raw `inert: bool` here renders
    // `inert="false"` as a PRESENT attribute on the web target, which HTML
    // boolean-attribute semantics read as inert regardless of the string
    // value -- every slide comes back inert, including the selected one).
    // A `bool`-typed dynamic attribute always carries a value (true or
    // false) as far as the vdom is concerned, so Dioxus can only ever emit
    // a `set_attribute` mutation for it -- through exactly that buggy
    // allowlist. An `Option`-typed one is genuinely absent when
    // `None`, so Dioxus instead emits a `remove_attribute` mutation
    // (`unified_bindings.rs`'s own `remove_attribute`, a plain
    // `node.removeAttribute(field)` for any field with no special case),
    // which never touches `isBoolAttr` at all -- and `Some("true")` still
    // reaches `setAttributeDefault`, but `truthy("true")` is `true`, so the
    // (buggy, allowlist-gated) removal branch is simply never reached
    // either way. This sidesteps the whole allowlist rather than trying to
    // special-case around it, and is the same `Option<&str>`-for-omission
    // construction this component's own `default_label`/`CarouselPrevious`'s
    // `aria_controls` already use for the identical "must be entirely
    // absent, not merely falsy" reason -- see `dev-docs/backlog.md` row 91.
    let (visible_start, visible_end) = (ctx.visible_range)();
    let in_visible_range = index() >= visible_start && index() <= visible_end;
    let is_visible = is_selected || in_visible_range;
    let inert: Option<&'static str> = (!is_visible).then_some("true");

    // Plain, synchronous native focus tracking -- no `document::eval`
    // involved (unlike almost everything else in this module): this is
    // read back, synchronously, by `Carousel`'s own focus-redirect effect
    // (see that effect's own doc for why a JS-side `document.activeElement`
    // read cannot answer the same question after the fact).
    let mut focused_item = ctx.focused_item;
    let onfocusin = move |_| focused_item.set(Some(index()));
    let onfocusout = move |_| {
        if (focused_item)() == Some(index()) {
            focused_item.set(None);
        }
    };

    // `role="tabpanel"` (in lieu of `group`) once a `CarouselIndicators` has
    // registered -- the APG tabbed style, `carousel-2-tablist.html`'s own
    // markup. `aria-roledescription="slide"` stays regardless: the
    // vendored tabbed example's own markup keeps it alongside `tabpanel`
    // (`<div ... role="tabpanel" aria-roledescription="slide" ...>`),
    // contradicting the pattern page's own (unfollowed) prose that a
    // tabpanel "does not have the aria-roledescription property" --
    // `dev-docs/archive/research/carousel-2026-09-19.md` §1.3 point 1 already
    // recommends following the tested example over the abstract prose.
    let role = if (ctx.tablist_present)() {
        "tabpanel"
    } else {
        "group"
    };

    // `scroll-snap-stop:always` -- see `CarouselContent`'s own doc, "##
    // scroll-snap-stop" section, for what this is, its known limitations,
    // and why this is a plain inline declaration rather than a stylesheet
    // gated by `@supports`.
    let gap_padding = item_gap_padding((ctx.orientation)());
    let basis = item_basis_style();
    let align_str = (ctx.align)().as_str();
    let (caller_style, rest_attrs) = fold_style_attributes(props.attributes);
    let style = format!(
        "{basis}scroll-snap-align:{align_str};scroll-snap-stop:always;min-width:0;min-height:0;{gap_padding}{}",
        caller_style.map(|s| format!(" {s}")).unwrap_or_default()
    );

    // Merged (owned-wins for structural/ARIA state; `aria_label` is a
    // presentational *default* -- caller-overridable, per `default_label`'s
    // own `has_own_accessible_name` gating above, which already never
    // computes a `Some` once the caller supplied their own name -- so it
    // is placed first rather than last, same as `CarouselPrevious`/
    // `CarouselNext`'s own default label below). Never a literal beside a
    // bare `..rest_attrs` spread (`scripts/check-attr-spread-collision.sh`).
    let attributes = merge_attributes(vec![
        attributes!(div {
            aria_label: default_label
        }),
        rest_attrs,
        attributes!(div {
            role: role,
            aria_roledescription: "slide",
            style: style.clone(),
            "data-selected": is_selected,
            // The stable data index and this child's logical position --
            // see `use_carousel_scroll_tracking`'s own doc for why every
            // JS bridge reads these rather than a child's array position.
            // Equal here (the plain children API has no window), so this
            // is the identical wire shape `CarouselVirtualContent` renders
            // for a real virtualised window -- one code path serves both.
            "data-index": index(),
            "data-position": index(),
            inert,
        }),
    ]);

    rsx! {
        div {
            id,
            onfocusin,
            onfocusout,
            ..attributes,

            {props.children}
        }
    }
}
