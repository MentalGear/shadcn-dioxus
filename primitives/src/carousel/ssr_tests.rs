use super::*;

#[component]
fn ThreeSlideCarousel() -> Element {
    rsx! {
        Carousel { aria_label: "Featured photos",
            CarouselPrevious { "Previous" }
            CarouselNext { "Next" }
            CarouselContent {
                CarouselItem { index: 0usize, "One" }
                CarouselItem { index: 1usize, "Two" }
                CarouselItem { index: 2usize, "Three" }
            }
        }
    }
}

#[component]
fn CarouselWithoutAccessibleName() -> Element {
    rsx! {
        Carousel {
            CarouselContent {
                CarouselItem { index: 0usize, "One" }
            }
        }
    }
}

#[component]
fn CarouselItemWithOwnLabel() -> Element {
    rsx! {
        Carousel { aria_label: "Featured photos",
            CarouselContent {
                CarouselItem { index: 0usize, aria_label: "The Eiffel Tower at dusk", "One" }
            }
        }
    }
}

fn render(component: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(component);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn render_props<P: Clone + 'static>(component: fn(P) -> Element, props: P) -> String {
    let mut dom = VirtualDom::new_with_props(component, props);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

/// The opening tag of the `occurrence`-th (0-based) `<button>` in
/// `html`, in document order. Deliberately not "find the text
/// 'Previous'/'Next' and walk backward to the nearest `<button`" --
/// that text also appears inside each button's own `aria-label`
/// attribute value (`aria-label="Previous slide"`), which a naive
/// `str::find` matches *before* the button's visible children text,
/// truncating the "opening tag" slice before it ever reaches
/// `disabled`. Locating by structural position (`<button` occurrence
/// count, matching `Carousel`'s own documented DOM order --
/// `CarouselPrevious` then `CarouselNext`, both before
/// `CarouselContent`) has no such ambiguity.
fn button_tag(html: &str, occurrence: usize) -> &str {
    let tag_start = html
        .match_indices("<button")
        .nth(occurrence)
        .map(|(i, _)| i)
        .unwrap();
    let tag_end = html[tag_start..].find('>').unwrap() + tag_start;
    &html[tag_start..=tag_end]
}

#[test]
fn root_has_region_role_and_roledescription() {
    let html = render(ThreeSlideCarousel);
    assert!(html.contains(r#"role="region""#));
    assert!(html.contains(r#"aria-roledescription="carousel""#));
    assert!(html.contains(r#"aria-label="Featured photos""#));
}

#[test]
fn root_defaults_to_horizontal_ltr() {
    let html = render(ThreeSlideCarousel);
    assert!(html.contains(r#"data-orientation="horizontal""#));
    assert!(html.contains(r#"data-direction="ltr""#));
    assert!(html.contains(r#"dir="ltr""#));
}

#[test]
fn renders_without_an_accessible_name_supplied() {
    // Only a `tracing::warn!`, never a panic or missing markup --
    // this just proves the warning path doesn't also break rendering.
    let html = render(CarouselWithoutAccessibleName);
    assert!(html.contains(r#"role="region""#));
}

#[test]
fn item_has_group_role_and_slide_roledescription() {
    let html = render(ThreeSlideCarousel);
    assert!(html.contains(r#"role="group""#));
    assert!(html.contains(r#"aria-roledescription="slide""#));
}

#[test]
fn item_has_no_default_label_on_first_render_before_registration_effects_run() {
    // See `CarouselItem`'s own "SSR stability" doc: `count` is only
    // known once every sibling's registration effect has run, and
    // `rebuild_in_place` alone never runs effects at all -- so this
    // is also exactly what a real SSR render (and a client's own
    // pre-hydration first paint) produces. A `"1 of 0"`-shaped label
    // leaking through here is what this guards against.
    let html = render(ThreeSlideCarousel);
    assert!(!html.contains("of 0"));
    assert!(!html.contains("1 of 3"));
}

#[test]
fn item_zero_is_not_inert_on_first_render_before_registration_effects_run() {
    // Hydration parity (`CarouselItem`'s own "Visible slides / `inert`"
    // doc): `selected`/`visible_range` are both forced to their
    // deterministic default (`0`/`(0, 0)`) until every sibling's own
    // registration effect has run, and (like the label test above)
    // `rebuild_in_place` alone never runs effects at all -- so this is
    // also exactly what a real SSR render (and a client's own
    // pre-hydration first paint) produces regardless of how many
    // slides actually exist. Slide 0's own opening `<div ... role="group"`
    // tag (the first one in this fixture's DOM order) must never carry
    // `inert` -- it is always the deterministic first-render "visible"
    // slide.
    let html = render(ThreeSlideCarousel);
    let tag_start = html.find(r#"role="group""#).unwrap();
    let open_start = html[..tag_start].rfind("<div").unwrap();
    let open_end = html[open_start..].find('>').unwrap() + open_start;
    assert!(!html[open_start..=open_end].contains("inert"));
}

#[test]
fn items_after_zero_are_inert_on_first_render_before_registration_effects_run() {
    // The mirror of the test above: slides 1 and 2 are neither
    // `selected` (forced to `0`) nor inside `visible_range` (forced to
    // `(0, 0)`) on this same first render, so both must be `inert`
    // -- the a11y contract's "only visible slides are reachable"
    // (`dev-docs/archive/research/carousel-loop-2026-09-25/loop-a11y-guidance.md`
    // §7) already holding before a single effect has run, not
    // merely once the client catches up.
    let html = render(ThreeSlideCarousel);
    let group_tags: Vec<&str> = html
        .match_indices(r#"role="group""#)
        .map(|(tag_start, _)| {
            let open_start = html[..tag_start].rfind("<div").unwrap();
            let open_end = html[open_start..].find('>').unwrap() + open_start;
            &html[open_start..=open_end]
        })
        .collect();
    assert_eq!(group_tags.len(), 3);
    assert!(!group_tags[0].contains("inert"));
    assert!(group_tags[1].contains("inert"));
    assert!(group_tags[2].contains("inert"));
}

#[test]
fn item_with_own_aria_label_is_not_overridden() {
    let html = render(CarouselItemWithOwnLabel);
    assert!(html.contains(r#"aria-label="The Eiffel Tower at dusk""#));
    assert!(!html.contains("1 of"));
}

#[test]
fn previous_button_is_disabled_at_the_first_slide() {
    let html = render(ThreeSlideCarousel);
    // `CarouselPrevious` is the first `<button>` in this fixture's own
    // documented DOM order (`CarouselPrevious`, `CarouselNext`, then
    // `CarouselContent`'s slides).
    let opening_tag = button_tag(&html, 0);
    assert!(opening_tag.contains("disabled"));
    assert!(opening_tag.contains(r#"aria-label="Previous slide""#));
}

#[test]
fn next_button_is_enabled_before_registration_effects_run() {
    // The slide count is only known once the items register in an effect, so on the first
    // render (`count == 0`, what SSR and the first client frame paint) the count is unknown,
    // not zero: Next must already wear its final pose (enabled, opacity 1) instead of a
    // disabled guess that fades up after hydration (backlog row 353).
    // `next_button_becomes_enabled_once_items_have_registered` below is the same fixture
    // after effects run.
    let html = render(ThreeSlideCarousel);
    let opening_tag = button_tag(&html, 1);
    assert!(!opening_tag.contains("disabled"), "{opening_tag}");
    assert!(opening_tag.contains(r#"aria-label="Next slide""#));
}

#[test]
fn next_button_becomes_enabled_once_items_have_registered() {
    // `rebuild_in_place` alone never runs an effect (`use_animated_open`'s
    // own doc: "tasks will not be polled") -- `render_immediate`
    // additionally drains the effect queue and re-diffs every
    // now-dirty scope synchronously, which is what actually runs
    // each `CarouselItem`'s registration effect and, once `count`
    // is no longer `0`, re-renders `CarouselNext` with the correct
    // `disabled` state. No async executor is needed here (unlike
    // `tooltip.rs`'s own `render_immediate` test): every effect this
    // triggers (`CarouselItem`'s registration, `Carousel`'s own
    // scroll-into-view effect) is a plain synchronous `use_effect`
    // body, not one that must await a spawned task's own result to
    // finish applying its state.
    let mut dom = VirtualDom::new(ThreeSlideCarousel);
    dom.rebuild_in_place();
    // One `render_immediate` runs each `CarouselItem`'s registration
    // effect (which updates `item_ids`); a *second* is what actually
    // re-diffs `CarouselNext` (and every "N of M" label) against the
    // resulting, now-nonzero `count` -- a downstream `Memo` read
    // becoming dirty from an effect's write is its own scheduling
    // wave, not folded into the same pass that ran the effect
    // (a single call here still shows
    // `CarouselNext` disabled). Looping a few times rather than
    // hardcoding "exactly two" keeps this test from being brittle to
    // a future Dioxus scheduling change.
    for _ in 0..4 {
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
    }
    let html = dioxus_ssr::render(&dom);

    let opening_tag = button_tag(&html, 1);
    assert!(!opening_tag.contains("disabled"));
    assert!(html.contains("1 of 3"));
}

#[component]
fn RtlCarousel() -> Element {
    rsx! {
        Carousel {
            aria_label: "Featured photos",
            dir: Direction::Rtl,
            CarouselContent {
                CarouselItem { index: 0usize, "One" }
            }
        }
    }
}

#[test]
fn dir_prop_renders_rtl_on_the_root() {
    let html = render(RtlCarousel);
    assert!(html.contains(r#"dir="rtl""#));
    assert!(html.contains(r#"data-direction="rtl""#));
}

#[component]
fn CarouselWithDragDisabled() -> Element {
    rsx! {
        Carousel { aria_label: "Featured photos",
            CarouselContent { draggable: false,
                CarouselItem { index: 0usize, "One" }
            }
        }
    }
}

#[test]
fn content_is_draggable_by_default() {
    // The live pointer-drag *behavior* (threshold, scrollBy, click
    // suppression) needs a real browser -- `playwright/carousel.spec.ts`'s
    // own "pointer drag" describe block -- but the opt-out prop's own
    // wiring into this styling hook is plain, SSR-checkable markup.
    // Unquoted `true`/`false` (not `"true"`/`"false"`) is how a `bool`
    // data attribute actually renders -- matching `calendar.rs`'s own
    // `data-selected=true`/`data-disabled=false` assertions (the quoted
    // form fails against this same markup).
    let html = render(ThreeSlideCarousel);
    assert!(html.contains("data-draggable=true"));
}

#[test]
fn drag_free_is_the_only_snap_switch() {
    use CarouselOrientation::{Horizontal, Vertical};
    assert_eq!(snap_type_value(Horizontal, false), "x mandatory");
    assert_eq!(snap_type_value(Vertical, false), "y mandatory");
    assert_eq!(snap_type_value(Horizontal, true), "none");
    assert_eq!(snap_type_value(Vertical, true), "none");
}

#[test]
fn content_draggable_can_be_opted_out() {
    let html = render(CarouselWithDragDisabled);
    assert!(html.contains("data-draggable=false"));
}

#[test]
fn slide_declares_scroll_snap_stop_always_inline() {
    // `scroll-snap-stop: always` is a plain inline declaration on each
    // slide, alongside `scroll-snap-align`, not a scoped `<style>` tag
    // gated by `@supports`: Dioxus's hydration marker comments inside a
    // `<style>` tag's text content corrupt the CSS parse (every slide
    // would compute `scroll-snap-stop: normal`). See `CarouselContent`'s
    // own doc, "scroll-snap-stop" section ("Why a plain inline
    // declaration, not `@supports`"), for the full write-up. This is the
    // SSR-checkable half; the computed-style/behavioral
    // half needs a real browser -- `playwright/carousel.spec.ts`'s own
    // "scroll-snap-stop" describe block.
    let html = render(ThreeSlideCarousel);
    assert!(html.contains("scroll-snap-stop:always"));
    // No scoped `<style>` tag is rendered at all.
    assert!(!html.contains("<style>"));
}

// -- Gap model (backlog row 91's shadcn-parity research) -----------

#[component]
fn VerticalThreeSlideCarousel() -> Element {
    rsx! {
        Carousel { aria_label: "Featured photos", orientation: CarouselOrientation::Vertical,
            CarouselPrevious { "Previous" }
            CarouselNext { "Next" }
            CarouselContent {
                CarouselItem { index: 0usize, "One" }
                CarouselItem { index: 1usize, "Two" }
                CarouselItem { index: 2usize, "Three" }
            }
        }
    }
}

#[test]
fn horizontal_item_reserves_gap_as_leading_inline_padding() {
    // The gap lives INSIDE each item's own border-box (a leading-edge
    // padding), never a real flex `gap` between items -- see
    // `item_gap_padding`'s own doc for why a real `gap` would overflow
    // a fractional `flex-basis` track. `var(..., 0px)` is the
    // raw-primitive default (a themed stylesheet supplies the actual
    // token, e.g. `--dx-space-4`) -- so zero theme CSS still renders a
    // valid, harmless (zero-width) declaration rather than an
    // invalid/dropped one.
    let html = render(ThreeSlideCarousel);
    assert!(html.contains("padding-inline-start:var(--dx-carousel-gap, 0px)"));
    assert!(!html.contains("padding-block-start:var(--dx-carousel-gap"));
}

#[test]
fn vertical_item_reserves_gap_as_leading_block_padding() {
    let html = render(VerticalThreeSlideCarousel);
    assert!(html.contains("padding-block-start:var(--dx-carousel-gap, 0px)"));
    assert!(!html.contains("padding-inline-start:var(--dx-carousel-gap"));
}

#[test]
fn horizontal_content_compensates_with_a_negative_inline_margin() {
    // The exact mirror of the item's own leading padding -- see
    // `content_gap_margin`'s own doc for why this keeps the first
    // item's visible content flush with the viewport's own clip
    // boundary instead of shifting every slide by one gap's worth.
    let html = render(ThreeSlideCarousel);
    assert!(html.contains("margin-inline-start:calc(var(--dx-carousel-gap, 0px) * -1)"));
    assert!(!html.contains("margin-block-start:calc(var(--dx-carousel-gap"));
    // Never a real flex `gap` -- that would be additive to a
    // fractional `flex-basis` and overflow the track (the "~2.3
    // slides visible" failure this construction prevents).
    assert!(!html.contains("gap:var(--dx-carousel-gap"));
}

#[test]
fn vertical_content_compensates_with_a_negative_block_margin() {
    let html = render(VerticalThreeSlideCarousel);
    assert!(html.contains("margin-block-start:calc(var(--dx-carousel-gap, 0px) * -1)"));
    assert!(!html.contains("margin-inline-start:calc(var(--dx-carousel-gap"));
}

#[test]
fn carousel_viewport_wrapper_establishes_its_own_block_formatting_context() {
    // `display: flow-root`, alongside `overflow: clip` -- see the
    // viewport wrapper's own doc for the vertical-orientation
    // margin collapse this prevents (`overflow: clip` alone does
    // not stop this wrapper's own block-axis margin from
    // collapsing through to `CarouselContent`'s negative
    // `margin-block-start`, under `CarouselOrientation::Vertical`).
    let html = render(ThreeSlideCarousel);
    assert!(html.contains(r#"style="overflow: clip; display: flow-root;""#));
}

#[test]
fn virtual_content_viewport_wrapper_establishes_its_own_block_formatting_context() {
    let html = render(VirtualCarousel12Loop);
    assert!(html.contains(r#"style="overflow: clip; display: flow-root;""#));
}

// -- Sizes: whole slides by default, opt-in peek --------------------

#[test]
fn item_basis_defaults_to_the_per_view_peek_calc() {
    // `--dx-carousel-per-view`/`--dx-carousel-peek` default to `1`/`0%`
    // in the calc's own fallback, which is byte-identical to a
    // hardcoded `flex:0 0 100%` -- see `item_basis_style`'s own doc.
    let html = render(ThreeSlideCarousel);
    assert!(html.contains(
        "flex:0 0 calc((100% - var(--dx-carousel-peek, 0%)) / var(--dx-carousel-per-view, 1));"
    ));
    assert!(!html.contains("flex:0 0 100%;"));
}

#[test]
fn virtual_content_item_basis_uses_the_same_calc() {
    let html = render(VirtualCarousel12Loop);
    assert!(html.contains(
        "flex:0 0 calc((100% - var(--dx-carousel-peek, 0%)) / var(--dx-carousel-per-view, 1));"
    ));
}

// -- true-end rule: one shared predicate ------------------------------

#[test]
fn both_band_bridges_carry_the_one_shared_true_end_predicate() {
    // `carousel_true_end_js!` is the rule's only home: spliced into the
    // wheel band, the drag bridge and the tracking bridge's
    // pinned-at-a-false-end re-centre, defined nowhere else.
    let shared = carousel_true_end_js!();
    for script in [
        CAROUSEL_WHEEL_BAND_JS,
        CAROUSEL_DRAG_JS,
        CAROUSEL_SCROLL_TRACKING_JS,
    ] {
        assert!(script.starts_with(shared));
        assert_eq!(script.matches("function dxIsTrueEnd(").count(), 1);
        assert_eq!(script.matches("function isTrueEnd(").count(), 0);
    }
    assert!(CAROUSEL_DRAG_JS.contains("dxIsTrueEnd(el, orientation === 'horizontal', side)"));
    assert!(CAROUSEL_WHEEL_BAND_JS.contains("dxIsTrueEnd(el, horizontal, side)"));
    assert!(CAROUSEL_SCROLL_TRACKING_JS.contains("dxIsTrueEnd(container, horizontal, side)"));
}

#[test]
fn every_component_write_declares_an_intent_the_settle_verifies() {
    // The write declares its target before scrolling; the tracking
    // bridge's settle checks it before reporting any position, and every
    // kind of user input drops it.
    let declare = CAROUSEL_SCROLL_TO_JS
        .find("scroller.__dxIntent = { targetId, tries: 0 };")
        .expect("scroll-to declares its intent");
    let write = CAROUSEL_SCROLL_TO_JS
        .find("scroller.scrollBy({ left: delta, behavior })")
        .unwrap();
    assert!(declare < write);
    let check = CAROUSEL_SCROLL_TRACKING_JS
        .find("const intent = container.__dxIntent;")
        .unwrap();
    let report = CAROUSEL_SCROLL_TRACKING_JS
        .find("dioxus.send(['selected'")
        .unwrap();
    assert!(check < report);
    assert!(CAROUSEL_SCROLL_TRACKING_JS.contains("intent.tries < 2"));
    // Registered through `listen`, so the teardown removes them (row 167).
    for ev in ["pointerdown", "keydown"] {
        assert!(
            CAROUSEL_SCROLL_TRACKING_JS.contains(&format!("listen(container, '{ev}', clearIntent"))
        );
    }
}

#[test]
fn effective_radius_never_lets_a_looping_window_repeat_an_item() {
    // Non-looping: the caller's radius, untouched.
    assert_eq!(effective_radius(10, 200, false), 10);
    assert_eq!(effective_radius(10, 12, false), 10);
    // Looping: capped so `2r + 1 < count` -- the 12-item demo renders 11.
    assert_eq!(effective_radius(10, 12, true), 5);
    assert_eq!(effective_radius(2, 12, true), 2);
    assert_eq!(effective_radius(10, 200, true), 10);
    for count in 4..40 {
        let r = effective_radius(10, count, true);
        assert!(2 * r + 1 < count, "count {count}: radius {r}");
        assert!(r >= 1);
    }
    // Too small to window: the radius is the count (the auto rule then renders all).
    assert_eq!(effective_radius(10, 3, true), 10);
}

// -- wheel band: true-end markup (`isTrueEnd`) ------------------------

#[component]
fn VirtualCarouselNoLoop() -> Element {
    rsx! {
        Carousel { aria_label: "Featured photos",
            CarouselVirtualContent::<String> {
                items: string_items(100),
                render_item: move |(_idx, value): (usize, String)| rsx! { span { "{value}" } },
            }
        }
    }
}

#[component]
fn LoopingPlainCarousel() -> Element {
    rsx! {
        Carousel { aria_label: "Featured photos", r#loop: true, loop_mode: LoopMode::Rewind,
            CarouselContent {
                CarouselItem { index: 0usize, "One" }
                CarouselItem { index: 1usize, "Two" }
            }
        }
    }
}

#[test]
fn content_renders_the_loop_flag_and_data_count_the_wheel_band_reads() {
    // Virtual: the total data count, from props, on the very first render.
    let looped = render(VirtualCarousel12Loop);
    assert!(looped.contains("data-loop=true"));
    assert!(looped.contains("data-slide-count=12"));
    let not_looped = render(VirtualCarouselNoLoop);
    assert!(not_looped.contains("data-loop=false"));
    assert!(not_looped.contains("data-slide-count=100"));
    // Plain: any loop mode counts as looping; the count is the registered
    // slide count, 0 until the items' own effects register them (the same
    // SSR-stable rule their "N of M" labels follow), so SSR and the
    // hydrating first render agree.
    assert!(render(LoopingPlainCarousel).contains("data-loop=true"));
    let plain = render(ThreeSlideCarousel);
    assert!(plain.contains("data-loop=false"));
    assert!(plain.contains("data-slide-count=0"));
}

// -- overscroll-behavior at false ends ---------------------------------

#[component]
fn VerticalLoopingCarousel() -> Element {
    rsx! {
        Carousel { aria_label: "Featured photos", r#loop: true, orientation: CarouselOrientation::Vertical,
            CarouselContent {
                CarouselItem { index: 0usize, "One" }
                CarouselItem { index: 1usize, "Two" }
            }
        }
    }
}

#[test]
fn overscroll_is_left_to_the_platform_only_where_both_ends_are_true() {
    // Plain, non-looping: both ends are true -- no declaration.
    assert!(!render(ThreeSlideCarousel).contains("overscroll-behavior"));
    // Looping (any mode): no true end, so the platform must not bounce.
    assert!(render(LoopingPlainCarousel).contains("overscroll-behavior-inline:none;"));
    assert!(render(VerticalLoopingCarousel).contains("overscroll-behavior-block:none;"));
    assert!(render(VirtualCarousel12Loop).contains("overscroll-behavior-inline:none;"));
    // Virtual, non-looping: SSR (and the hydrating first render) renders
    // the whole list, so both ends are true there; only post-mount
    // windowing can make one false.
    assert!(!render(VirtualCarouselNoLoop).contains("overscroll-behavior"));
}

// -- `gap` prop ------------------------------------------------------

#[component]
fn GapCarousel(vertical: bool, gap: Option<String>) -> Element {
    let orientation = if vertical {
        CarouselOrientation::Vertical
    } else {
        CarouselOrientation::Horizontal
    };
    rsx! {
        Carousel { aria_label: "Featured photos", orientation,
            CarouselContent { gap,
                CarouselItem { index: 0usize, "One" }
                CarouselItem { index: 1usize, "Two" }
            }
        }
    }
}

#[component]
fn GapVirtualCarousel(gap: Option<String>) -> Element {
    rsx! {
        Carousel { aria_label: "Featured photos",
            CarouselVirtualContent::<String> {
                items: string_items(3),
                gap,
                render_item: move |(_idx, value): (usize, String)| rsx! { span { "{value}" } },
            }
        }
    }
}

#[test]
fn gap_prop_sets_the_variable_the_horizontal_gap_model_reads() {
    let html = render_props(
        GapCarousel,
        GapCarouselProps {
            vertical: false,
            gap: Some("var(--dx-space-2)".into()),
        },
    );
    assert!(html.contains(
        "margin-inline-start:calc(var(--dx-carousel-gap, 0px) * -1);--dx-carousel-gap:var(--dx-space-2);"
    ));
    assert!(html.contains("padding-inline-start:var(--dx-carousel-gap, 0px);"));
}

#[test]
fn gap_prop_sets_the_variable_the_vertical_gap_model_reads() {
    let html = render_props(
        GapCarousel,
        GapCarouselProps {
            vertical: true,
            gap: Some("12px".into()),
        },
    );
    assert!(html.contains(
        "margin-block-start:calc(var(--dx-carousel-gap, 0px) * -1);--dx-carousel-gap:12px;"
    ));
    assert!(html.contains("padding-block-start:var(--dx-carousel-gap, 0px);"));
}

#[test]
fn gap_prop_on_virtual_content_sets_the_same_variable() {
    let html = render_props(
        GapVirtualCarousel,
        GapVirtualCarouselProps {
            gap: Some("1rem".into()),
        },
    );
    assert!(html.contains("--dx-carousel-gap:1rem;"));
}

#[test]
fn gap_prop_none_renders_no_variable() {
    for html in [
        render_props(
            GapCarousel,
            GapCarouselProps {
                vertical: false,
                gap: None,
            },
        ),
        render_props(
            GapCarousel,
            GapCarouselProps {
                vertical: true,
                gap: None,
            },
        ),
        render_props(GapVirtualCarousel, GapVirtualCarouselProps { gap: None }),
    ] {
        assert!(!html.contains("--dx-carousel-gap:"));
    }
}

#[component]
fn GapAndCallerStyleCarousel() -> Element {
    rsx! {
        Carousel { aria_label: "Featured photos",
            CarouselContent { gap: "4px", style: "--dx-carousel-gap:9px;",
                CarouselItem { index: 0usize, "One" }
            }
        }
    }
}

#[test]
fn caller_style_overrides_the_gap_prop() {
    // The caller's own inline style comes later in the same attribute,
    // so a caller-set `--dx-carousel-gap` wins over the prop -- and it
    // is one merged `style` attribute, never two.
    let html = render(GapAndCallerStyleCarousel);
    let prop_at = html.find("--dx-carousel-gap:4px;").unwrap();
    let caller_at = html.find("--dx-carousel-gap:9px;").unwrap();
    assert!(prop_at < caller_at);
    let tag_start = html[..prop_at].rfind('<').unwrap();
    let tag = &html[tag_start..html[tag_start..].find('>').unwrap() + tag_start];
    assert_eq!(tag.matches(" style=").count(), 1, "{tag}");
}

// -- `nav_disabled_opacity` ------------------------------------------

#[test]
fn nav_disabled_vars_pass_a_value_through_and_clamp_it() {
    assert_eq!(
        nav_disabled_vars(Some(0.25)),
        "--dx-carousel-nav-disabled-opacity:0.25;"
    );
    assert_eq!(
        nav_disabled_vars(Some(1.0)),
        "--dx-carousel-nav-disabled-opacity:1;"
    );
    assert_eq!(
        nav_disabled_vars(Some(7.0)),
        "--dx-carousel-nav-disabled-opacity:1;"
    );
    // Below the range clamps to 0, which is the "hidden" case.
    assert_eq!(nav_disabled_vars(Some(-3.0)), nav_disabled_vars(Some(0.0)));
}

#[test]
fn nav_disabled_vars_hide_the_arrow_only_at_exactly_zero() {
    assert_eq!(
        nav_disabled_vars(Some(0.0)),
        "--dx-carousel-nav-disabled-opacity:0;--dx-carousel-nav-disabled-visibility:hidden;"
    );
    // A tiny but visible opacity is still a visible, focusable-looking
    // arrow: hiding it would be a lie.
    assert!(!nav_disabled_vars(Some(0.01)).contains("visibility"));
}

#[test]
fn nav_disabled_vars_ignore_none_and_non_finite() {
    assert_eq!(nav_disabled_vars(None), "");
    assert_eq!(nav_disabled_vars(Some(f32::NAN)), "");
    assert_eq!(nav_disabled_vars(Some(f32::INFINITY)), "");
    assert_eq!(nav_disabled_vars(Some(f32::NEG_INFINITY)), "");
}

#[component]
fn NavOpacityCarousel(opacity: Option<f32>, caller_style: bool) -> Element {
    let style = caller_style.then_some("--dx-carousel-nav-disabled-opacity:0.9;");
    rsx! {
        Carousel {
            aria_label: "Featured photos",
            nav_disabled_opacity: opacity,
            style,
            CarouselPrevious { "Prev" }
            CarouselNext { "Next" }
            CarouselContent {
                CarouselItem { index: 0usize, "One" }
                CarouselItem { index: 1usize, "Two" }
            }
        }
    }
}

/// The opening tag of the root `<div role="region"`.
fn root_tag(html: &str) -> &str {
    let at = html.find("role=\"region\"").unwrap();
    let tag_start = html[..at].rfind('<').unwrap();
    &html[tag_start..html[tag_start..].find('>').unwrap() + tag_start]
}

#[test]
fn nav_disabled_opacity_sets_the_variable_on_the_root_only() {
    let html = render_props(
        NavOpacityCarousel,
        NavOpacityCarouselProps {
            opacity: Some(0.0),
            caller_style: false,
        },
    );
    let root = root_tag(&html);
    assert!(
        root.contains("--dx-carousel-nav-disabled-opacity:0;"),
        "{root}"
    );
    assert!(
        root.contains("--dx-carousel-nav-disabled-visibility:hidden;"),
        "{root}"
    );
    // Inherited by both arrows: neither button carries its own copy.
    assert_eq!(
        html.matches("--dx-carousel-nav-disabled-opacity").count(),
        1
    );
}

#[test]
fn nav_disabled_opacity_none_adds_no_style_to_the_root() {
    // A carousel that does not use the prop renders no `style` attribute
    // at all, and none of the variables anywhere.
    let html = render_props(
        NavOpacityCarousel,
        NavOpacityCarouselProps {
            opacity: None,
            caller_style: false,
        },
    );
    assert!(!root_tag(&html).contains("style="), "{}", root_tag(&html));
    assert!(!html.contains("--dx-carousel-nav-disabled"));
}

#[test]
fn caller_style_overrides_the_nav_disabled_opacity_prop() {
    let html = render_props(
        NavOpacityCarousel,
        NavOpacityCarouselProps {
            opacity: Some(0.3),
            caller_style: true,
        },
    );
    let root = root_tag(&html);
    let prop_at = root
        .find("--dx-carousel-nav-disabled-opacity:0.3;")
        .unwrap();
    let caller_at = root
        .find("--dx-carousel-nav-disabled-opacity:0.9;")
        .unwrap();
    assert!(prop_at < caller_at, "{root}");
    // One merged `style` attribute, never two.
    assert_eq!(root.matches(" style=").count(), 1, "{root}");
}

#[test]
fn every_slide_renderer_declares_border_box_so_the_gap_padding_stays_inside_the_basis() {
    // `item_basis_style`'s own doc: without it, a slide is one gap wider
    // than the snapport and the browser stops snapping it.
    for html in [render(ThreeSlideCarousel), render(VirtualCarousel12Loop)] {
        assert!(html.contains("box-sizing:border-box;flex:0 0 calc("));
    }
}

// -- `align` ---------------------------------------------------------

#[test]
fn item_scroll_snap_align_defaults_to_start() {
    // Matches shadcn's own default (`opts={{ align: "start" }}` in
    // every demo that sets it explicitly) -- the same start alignment a
    // carousel has with no `align` prop at all.
    let html = render(ThreeSlideCarousel);
    assert!(html.contains("scroll-snap-align:start;"));
}

#[component]
fn CenterAlignedCarousel() -> Element {
    rsx! {
        Carousel { aria_label: "Featured photos", align: CarouselAlign::Center,
            CarouselContent {
                CarouselItem { index: 0usize, "One" }
                CarouselItem { index: 1usize, "Two" }
            }
        }
    }
}

#[component]
fn EndAlignedCarousel() -> Element {
    rsx! {
        Carousel { aria_label: "Featured photos", align: CarouselAlign::End,
            CarouselContent {
                CarouselItem { index: 0usize, "One" }
                CarouselItem { index: 1usize, "Two" }
            }
        }
    }
}

#[test]
fn align_center_sets_scroll_snap_align_center_on_every_item() {
    let html = render(CenterAlignedCarousel);
    assert!(!html.contains("scroll-snap-align:start;"));
    assert_eq!(html.matches("scroll-snap-align:center;").count(), 2);
}

#[test]
fn align_end_sets_scroll_snap_align_end_on_every_item() {
    let html = render(EndAlignedCarousel);
    assert!(!html.contains("scroll-snap-align:start;"));
    assert_eq!(html.matches("scroll-snap-align:end;").count(), 2);
}

#[test]
fn align_as_str_matches_the_scroll_snap_align_keywords() {
    assert_eq!(CarouselAlign::Start.as_str(), "start");
    assert_eq!(CarouselAlign::Center.as_str(), "center");
    assert_eq!(CarouselAlign::End.as_str(), "end");
}

#[test]
fn align_default_is_start() {
    assert_eq!(CarouselAlign::default(), CarouselAlign::Start);
}

#[component]
fn CarouselWithBasisOverride() -> Element {
    rsx! {
        Carousel { aria_label: "Featured photos",
            CarouselContent {
                CarouselItem { index: 0usize, style: "flex-basis: 40%;", "One" }
            }
        }
    }
}

#[test]
fn an_inline_flex_basis_override_wins_over_the_default_calc() {
    // The "override flex-basis per item via inline style"
    // escape hatch (docs.md's own "Sizing slides" section)
    // must keep working: a caller-supplied
    // `flex-basis` simply appears later in the same `style`
    // attribute and wins.
    let html = render(CarouselWithBasisOverride);
    assert!(html.contains("flex-basis: 40%;"));
}

// -- `loop` / `loop_mode` -------------------------------------------

// Uses an explicit `loop_mode: LoopMode::Rewind` -- see `LoopMode`'s own
// doc: for the plain children API, `r#loop: true` alone (the default
// `LoopMode::Seamless`) is a no-op, and `SeamlessLoopCarousel`/its own
// tests just below pin exactly that. Only the explicit opt-in wraps,
// which is what this fixture needs.
#[component]
fn RewindLoopCarousel() -> Element {
    rsx! {
        Carousel { aria_label: "Featured photos", r#loop: true, loop_mode: LoopMode::Rewind,
            CarouselPrevious { "Previous" }
            CarouselNext { "Next" }
            CarouselContent {
                CarouselItem { index: 0usize, "One" }
                CarouselItem { index: 1usize, "Two" }
                CarouselItem { index: 2usize, "Three" }
            }
        }
    }
}

#[test]
fn loop_enabled_previous_is_never_disabled_even_before_registration() {
    // Even on the very first, pre-effect render (`count == 0`, the
    // same moment `next_button_is_enabled_before_registration_effects_run`
    // asserts the NON-loop `CarouselNext` is already enabled) --
    // explicit `LoopMode::Rewind` makes both buttons unconditionally
    // not-disabled (backlog row 91): they
    // wrap instead of ever reaching a real boundary.
    let html = render(RewindLoopCarousel);
    let previous_tag = button_tag(&html, 0);
    let next_tag = button_tag(&html, 1);
    assert!(!previous_tag.contains("disabled"));
    assert!(!next_tag.contains("disabled"));
}

#[test]
fn loop_enabled_previous_stays_enabled_once_items_have_registered() {
    let mut dom = VirtualDom::new(RewindLoopCarousel);
    dom.rebuild_in_place();
    for _ in 0..4 {
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
    }
    let html = dioxus_ssr::render(&dom);
    let previous_tag = button_tag(&html, 0);
    let next_tag = button_tag(&html, 1);
    assert!(!previous_tag.contains("disabled"));
    assert!(!next_tag.contains("disabled"));
}

// `r#loop: true` with NO explicit
// `loop_mode` (so the default, `LoopMode::Seamless`) on the plain
// children API -- which has no seamless path available to it at all
// (see `LoopMode`'s own doc) -- must degrade to a real no-op, not
// silently fall back to always-rewind.
#[component]
fn SeamlessDefaultLoopCarousel() -> Element {
    rsx! {
        Carousel { aria_label: "Featured photos", r#loop: true,
            CarouselPrevious { "Previous" }
            CarouselNext { "Next" }
            CarouselContent {
                CarouselItem { index: 0usize, "One" }
                CarouselItem { index: 1usize, "Two" }
                CarouselItem { index: 2usize, "Three" }
            }
        }
    }
}

#[test]
fn seamless_default_loop_mode_does_not_wrap_the_plain_children_api() {
    let mut dom = VirtualDom::new(SeamlessDefaultLoopCarousel);
    dom.rebuild_in_place();
    for _ in 0..4 {
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
    }
    let html = dioxus_ssr::render(&dom);
    let previous_tag = button_tag(&html, 0);
    assert!(previous_tag.contains("disabled"));
}

// -- Autoplay + rotation control -----------------------------------

#[component]
fn AutoplayCarousel() -> Element {
    rsx! {
        Carousel { aria_label: "Featured photos",
            CarouselRotationControl { "Toggle" }
            CarouselAutoplay {}
            CarouselContent {
                CarouselItem { index: 0usize, "One" }
                CarouselItem { index: 1usize, "Two" }
            }
        }
    }
}

#[test]
fn no_aria_live_without_a_carousel_autoplay() {
    // A `Carousel` with no `CarouselAutoplay` at all renders no
    // `aria-live` anywhere (APG's own "optional" framing: a
    // manually-paged carousel has nothing to announce around).
    let html = render(ThreeSlideCarousel);
    assert!(!html.contains("aria-live"));
}

#[test]
fn no_aria_live_before_carousel_autoplay_has_mounted() {
    // `CarouselAutoplay` publishes `autoplay.present` via its own
    // effect, which (like every other registration effect in this
    // module) never runs during a bare `rebuild_in_place` -- so SSR
    // and the client's pre-hydration first render both still agree on
    // "no aria-live yet," the same hydration-parity shape
    // `item_has_no_default_label_on_first_render_before_registration_effects_run`
    // already establishes for "N of M" labels.
    let html = render(AutoplayCarousel);
    assert!(!html.contains("aria-live"));
}

#[component]
fn AutoplayCarouselNotPlaying() -> Element {
    // `default_playing: false` -- deliberately never `true` in this
    // fixture. The `rotating=true` (ticking) case needs a real
    // `sleep().await` registration, which `crate::sleep::sleep`
    // asserts requires "a Tokio 1.x runtime" (`render_immediate`
    // panics under a plain `#[test]`, with no reactor
    // present) -- a synchronous SSR unit test has none, and adding one
    // just to spawn-and-immediately-abandon a timer would test the
    // runtime, not this component. The actual tick -> advance ->
    // aria-live="off" round trip is exercised in a browser instead,
    // `playwright/carousel-rewind-autoplay.spec.ts`'s own "autoplay" describe block,
    // where a real browser's own JS timer backs it.
    rsx! {
        Carousel { aria_label: "Featured photos",
            CarouselRotationControl { "Toggle" }
            CarouselAutoplay { default_playing: false }
            CarouselContent {
                CarouselItem { index: 0usize, "One" }
                CarouselItem { index: 1usize, "Two" }
            }
        }
    }
}

#[test]
fn aria_live_polite_and_rotation_label_once_autoplay_has_mounted_and_is_not_playing() {
    let mut dom = VirtualDom::new(AutoplayCarouselNotPlaying);
    dom.rebuild_in_place();
    for _ in 0..4 {
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
    }
    let html = dioxus_ssr::render(&dom);
    // `present` is `true` (published by `CarouselAutoplay`'s own
    // effect) but `playing` is `false` (`default_playing: false`), so
    // `rotating` is `false` and the live region is `"polite"` -- and
    // the timer never spawns at all (this component's own doc: the
    // ticking effect's `if !active { return; }` branch), which is
    // exactly why this fixture -- unlike a default-playing one -- is
    // safe to drive through `render_immediate` in a plain `#[test]`.
    assert!(html.contains(r#"aria-live="polite""#));
    assert!(html.contains(r#"aria-label="Start automatic slide show""#));
}

// -- Tablist variant ------------------------------------------------

#[component]
fn TablistCarousel() -> Element {
    rsx! {
        Carousel { aria_label: "Featured photos",
            CarouselIndicators {
                CarouselIndicator { index: 0usize, "1" }
                CarouselIndicator { index: 1usize, "2" }
            }
            CarouselContent {
                CarouselItem { index: 0usize, "One" }
                CarouselItem { index: 1usize, "Two" }
            }
        }
    }
}

#[test]
fn item_is_group_role_before_a_carousel_tab_list_has_mounted() {
    // Same hydration-parity shape as autoplay's own "before mount"
    // test above: `CarouselIndicators`'s presence publish is effect-driven,
    // so a bare `rebuild_in_place` (SSR, and the client's own
    // pre-hydration first render) still sees the ordinary `group` role.
    let html = render(ThreeSlideCarousel);
    assert!(html.contains(r#"role="group""#));
    assert!(!html.contains(r#"role="tabpanel""#));
}

#[test]
fn tablist_wires_role_tablist_tab_and_tabpanel_once_mounted() {
    let mut dom = VirtualDom::new(TablistCarousel);
    dom.rebuild_in_place();
    for _ in 0..4 {
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
    }
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains(r#"role="tablist""#));
    assert!(html.contains(r#"role="tab""#));
    assert!(html.contains(r#"role="tabpanel""#));
    // `aria-roledescription="slide"` stays on the tabpanel too --
    // deliberately following the tested example over the pattern
    // page's own contradicting prose (see `CarouselItem`'s own doc).
    assert!(html.contains(r#"aria-roledescription="slide""#));
    // Unquoted `true`/`false`, not `"true"`/`"false"` -- a plain
    // `bool` value renders that way regardless of attribute name
    // (matches `data-draggable=true`'s own
    // established convention elsewhere in this file's SSR tests, not
    // an aria-specific special case).
    assert!(html.contains("aria-selected=true"));
    assert!(html.contains("aria-selected=false"));
    assert!(html.contains(r#"aria-label="Slide 1""#));
    assert!(html.contains(r#"aria-label="Slide 2""#));
    // Roving tabindex: only the selected tab is a page tab stop.
    assert!(html.contains(r#"tabindex="0""#));
    assert!(html.contains(r#"tabindex="-1""#));
}

// -- CarouselVirtualContent ------------------------------------------

fn string_items(n: usize) -> Vec<String> {
    (0..n).map(|i| format!("Slide {i}")).collect()
}

#[component]
fn VirtualCarousel12Loop() -> Element {
    rsx! {
        Carousel { aria_label: "Featured photos", r#loop: true,
            CarouselPrevious { "Previous" }
            CarouselNext { "Next" }
            CarouselVirtualContent::<String> {
                items: string_items(12),
                render_item: move |(_idx, value): (usize, String)| rsx! { span { "{value}" } },
            }
        }
    }
}

#[component]
fn VirtualCarouselN(n: usize) -> Element {
    rsx! {
        Carousel { aria_label: "Featured photos",
            CarouselVirtualContent::<String> {
                items: string_items(n),
                render_item: move |(_idx, value): (usize, String)| rsx! { span { "{value}" } },
            }
        }
    }
}

#[test]
fn virtual_content_ssr_renders_every_item_in_order_before_any_effect_has_run() {
    // First render (SSR, and a client's own pre-hydration paint) never
    // windows -- see `CarouselVirtualContent`'s own "SSR / first
    // render" doc. All 12 slides, in true order, must be present.
    let html = render(VirtualCarousel12Loop);
    for i in 0..12 {
        assert!(
            html.contains(&format!("Slide {i}")),
            "missing Slide {i} in SSR output"
        );
    }
    // "k of 12" labels are correct at first render too (unlike the
    // plain children API, `CarouselVirtualContent` already knows `N`
    // from `items.len()` without waiting for per-child registration
    // effects).
    assert!(html.contains(r#"aria-label="1 of 12""#));
    assert!(html.contains(r#"aria-label="12 of 12""#));
}

#[test]
fn virtual_content_ssr_only_slide_zero_is_visible() {
    let html = render(VirtualCarousel12Loop);
    let group_tags: Vec<&str> = html
        .match_indices(r#"role="group""#)
        .map(|(tag_start, _)| {
            let open_start = html[..tag_start].rfind("<div").unwrap();
            let open_end = html[open_start..].find('>').unwrap() + open_start;
            &html[open_start..=open_end]
        })
        .collect();
    assert_eq!(group_tags.len(), 12);
    assert!(!group_tags[0].contains("inert"));
    for tag in &group_tags[1..] {
        assert!(tag.contains("inert"));
    }
}

#[test]
fn virtual_content_position_equals_data_index_before_any_window_is_active() {
    let html = render(VirtualCarousel12Loop);
    for i in 0..12 {
        assert!(
            html.contains(&format!(r#""data-index":{i}"#))
                || html.contains(&format!("data-index={i}"))
        );
    }
}

#[component]
fn VirtualCarouselZero() -> Element {
    rsx! {
        Carousel { aria_label: "Featured photos",
            CarouselVirtualContent::<String> {
                items: Vec::<String>::new(),
                render_item: move |(_idx, value): (usize, String)| rsx! { span { "{value}" } },
            }
        }
    }
}

#[test]
fn virtual_content_n_zero_renders_no_slides() {
    let html = render(VirtualCarouselZero);
    assert!(!html.contains(r#"role="group""#));
}

#[test]
fn virtual_content_n_one_renders_one_slide_labeled_1_of_1() {
    let html = dioxus_ssr::render(&{
        let mut dom = VirtualDom::new_with_props(VirtualCarouselN, VirtualCarouselNProps { n: 1 });
        dom.rebuild_in_place();
        dom
    });
    assert!(html.contains(r#"aria-label="1 of 1""#));
    assert_eq!(html.matches(r#"role="group""#).count(), 1);
}

#[test]
fn virtual_content_n_two_renders_two_distinct_slides() {
    let mut dom = VirtualDom::new_with_props(VirtualCarouselN, VirtualCarouselNProps { n: 2 });
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains(r#"aria-label="1 of 2""#));
    assert!(html.contains(r#"aria-label="2 of 2""#));
}

#[test]
fn virtual_content_n_three_renders_three_distinct_slides() {
    let mut dom = VirtualDom::new_with_props(VirtualCarouselN, VirtualCarouselNProps { n: 3 });
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains(r#"aria-label="1 of 3""#));
    assert!(html.contains(r#"aria-label="2 of 3""#));
    assert!(html.contains(r#"aria-label="3 of 3""#));
}

#[test]
fn window_math_small_n_duplicates_data_index_with_unique_positions() {
    // N=3 < 2R+1 (radius 2 -> window width 5): the shared window math
    // (`primitives/src/virtual/window.rs`) is
    // what `CarouselVirtualContent` reuses, and it already guarantees
    // unique positions with a repeating data index -- pinned here
    // against this component's own actual radius default (2) rather
    // than only against that module's own generic tests.
    let items = crate::r#virtual::window(3, 0, 0, 2, true);
    let mut positions: Vec<isize> = items.iter().map(|i| i.position).collect();
    positions.sort_unstable();
    positions.dedup();
    assert_eq!(positions.len(), items.len());
    let data_indices: Vec<usize> = items.iter().map(|i| i.data_index).collect();
    assert_eq!(data_indices, vec![1, 2, 0, 1, 2]);
}

#[component]
fn VirtualCarouselCenterAligned() -> Element {
    rsx! {
        Carousel { aria_label: "Featured photos", align: CarouselAlign::Center,
            CarouselVirtualContent::<String> {
                items: string_items(12),
                render_item: move |(_idx, value): (usize, String)| rsx! { span { "{value}" } },
            }
        }
    }
}

#[test]
fn virtual_content_item_scroll_snap_align_follows_the_align_prop() {
    let html = render(VirtualCarouselCenterAligned);
    assert!(!html.contains("scroll-snap-align:start;"));
    assert!(html.contains("scroll-snap-align:center;"));
}

#[test]
fn virtual_content_slides_reserve_gap_the_same_way_plain_items_do() {
    // `CarouselVirtualContent`'s own rendered slide markup duplicates
    // (rather than shares) `CarouselItem`'s inline style construction --
    // see that component's own doc for why a fresh string is built here
    // instead of calling into `CarouselItem` -- so the gap model must be
    // pinned on this path too, not just the plain children API.
    let html = render(VirtualCarousel12Loop);
    assert!(html.contains("padding-inline-start:var(--dx-carousel-gap, 0px)"));
    assert!(html.contains("margin-inline-start:calc(var(--dx-carousel-gap, 0px) * -1)"));
}
