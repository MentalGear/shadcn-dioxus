use super::*;

/// Display title for a variant's `h3` on a component page, derived from the
/// variant's identifier: underscores become spaces and a known acronym is
/// upper-cased, so `rtl` reads "RTL" rather than "Rtl". An acronym that
/// *qualifies* another variant name goes in parentheses, so
/// `virtual_loop_rtl` reads "virtual loop (RTL)". (`.dx-component-variant-title`
/// then title-cases the words via CSS `text-transform: capitalize`, which
/// leaves an already upper-case acronym alone.)
pub(crate) fn variant_title(name: &str) -> String {
    const ACRONYMS: &[&str] = &["api", "ltr", "otp", "rtl", "ssr", "ui", "url"];
    let words: Vec<&str> = name.split('_').filter(|w| !w.is_empty()).collect();
    let is_acronym = |w: &str| ACRONYMS.contains(&w);
    let render = |w: &str| {
        if is_acronym(w) {
            w.to_uppercase()
        } else {
            w.to_string()
        }
    };
    match words.as_slice() {
        [rest @ .., last] if !rest.is_empty() && is_acronym(last) => format!(
            "{} ({})",
            rest.iter().map(|w| render(w)).collect::<Vec<_>>().join(" "),
            render(last)
        ),
        _ => words
            .iter()
            .map(|w| render(w))
            .collect::<Vec<_>>()
            .join(" "),
    }
}

#[component]
pub(crate) fn ComponentVariantHighlight(
    variant: ComponentVariantDemoData,
    main_variant: bool,
    /// The component the variant belongs to (its folder name): the code tab
    /// shows the source as it reads once that component is installed.
    owner: &'static str,
) -> Element {
    let ComponentVariantDemoData {
        name,
        rs_highlighted,
        css_highlighted: _,
        component: Comp,
    } = variant;
    let highlighted = installed_source::installed(owner, name, &rs_highlighted);
    // Every variant of a "Normal" component renders on the SAME page (the
    // "Examples" section below loops over all of them, main included --
    // see `ComponentHighlight` above), so a literal `"component-preview-
    // frame"` id can't be shared across more than one of these `TabContent`s
    // without producing duplicate ids (dev-docs/backlog.md rows 19/7: a
    // second `popover` variant would make `popover.spec.ts`'s basic test
    // (`page.locator("#component-preview-frame")`, no `.first()`) fail with a
    // strict-mode "2 elements" violation). Made unique by construction, not
    // per-instance: `main_variant` (the exact signal distinguishing
    // the one call site above from every entry in the loop below) picks
    // exactly `component-preview-frame` for `main`, so every
    // single-variant component page and every spec that targets that
    // literal id works untouched, and
    // `component-preview-frame-<name>` for every other variant, unique per
    // variant name so several additional variants on the same page (e.g.
    // `calendar[simple, internationalized, range, multi_month,
    // unavailable_dates]`, which `calendar.spec.ts` tolerates via
    // `.first()`) never collide with each other either.
    let frame_id = if main_variant {
        "component-preview-frame".to_string()
    } else {
        format!("component-preview-frame-{name}")
    };
    rsx! {
        if !main_variant {
            h3 { class: "dx-component-variant-title", "{variant_title(name)}" }
        }
        Tabs {
            default_value: "Demo",
            border_bottom_left_radius: "0.5rem",
            border_bottom_right_radius: "0.5rem",
            horizontal: true,
            width: "100%",
            variant: TabsVariant::Ghost,
            div { class: "dx-component-tabs-header",
                TabList {
                    TabTrigger { value: "Demo", "DEMO" }
                    TabTrigger { value: "Code", "CODE" }
                }
            }
            div {
                width: "100%",
                height: "100%",
                display: "flex",
                flex_direction: "column",
                justify_content: "center",
                align_items: "center",
                TabContent {
                    class: "dx-component-preview-frame",
                    id: "{frame_id}",
                    value: "Demo",
                    width: "100%",
                    position: "relative",
                    // The page's LTR/RTL toggle (`PREVIEW_DIRECTION`): `dir` on the frame mirrors
                    // layout and logical-property CSS, the provider the keyboard/positioning logic.
                    dir: if PREVIEW_DIRECTION() == Direction::Rtl { Some("rtl") } else { None },
                    DirectionProvider { direction: PREVIEW_DIRECTION(), Comp {} }
                }
                TabContent {
                    class: "dx-component-preview-frame",
                    value: "Code",
                    width: "100%",
                    position: "relative",
                    CodeBlock { source: highlighted }
                }
            }
        }
        // The main demo is the end of a component page's top group.
        if main_variant {
            FirstPaintGate {}
        }
        VariantNotes { owner, name }
    }
}

/// The id of [`FirstPaintGate`]'s marker.
pub(crate) const FIRST_PAINT_ID: &str = "dx-first-paint";

/// Placed where a page's top group ends: the page's first paint waits until the parser has reached this point (a
/// render-blocking `expect` link in the SSR head for the empty marker after it; Chromium 124+, ignored elsewhere).
/// Painted earlier, the half-parsed top of the page is on screen and every later sibling re-lays it out: a centred
/// demo row re-centres as its items arrive, a right-aligned control slides as its label does, a sticky sidebar drops
/// once the column beside it is tall enough (backlog row 407, `layout-stability.spec.ts`). One per page. After a
/// client-side navigation the link is inert: render-blocking only applies before a document's first paint.
#[component]
pub(crate) fn FirstPaintGate() -> Element {
    rsx! {
        document::Link { rel: "expect", href: "#{FIRST_PAINT_ID}", blocking: "render" }
        span { id: FIRST_PAINT_ID, hidden: true }
    }
}

/// The prose about one demo (`variants/<variant>/notes.md`, rendered by `build.rs`), in a closed
/// `Collapsible` under that demo, so a long explanation costs one line until the reader asks for it, and the prose sits with its
/// demo, not in a section away (row 233).
/// `hidden_until_found` keeps the closed note in the server markup, searchable and reachable by a
/// `#fragment` (`use_open_section_for_hash`).
#[component]
pub(crate) fn VariantNotes(owner: &'static str, name: &'static str) -> Element {
    use components::collapsible::{Collapsible, CollapsibleContent, CollapsibleTrigger};
    let Some(notes) = variant_notes(owner, name) else {
        return VNode::empty();
    };
    rsx! {
        Collapsible { class: "dx-variant-notes dx-docs-prose", hidden_until_found: true,
            CollapsibleTrigger { class: "dx-docs-collapsible-trigger dx-variant-notes-trigger",
                "About this example"
            }
            CollapsibleContent {
                div {
                    class: "dx-component-description",
                    dangerous_inner_html: prefix_internal_hrefs(notes),
                }
            }
        }
    }
}

#[component]
pub(crate) fn BlockComponentVariantHighlight(
    component_name: &'static str,
    variant: ComponentVariantDemoData,
    main_variant: bool,
) -> Element {
    let ComponentVariantDemoData {
        name,
        rs_highlighted,
        css_highlighted,
        component: _,
    } = variant;
    let highlighted = installed_source::installed(component_name, name, &rs_highlighted);

    // The canonical, SSG-enumerable path route (row 46) -- NOT the legacy
    // `Route::ComponentBlockDemo` query form, so every block demo's iframe
    // content this app renders itself is already a page `dx build --ssg`
    // can prerender per (name, variant) pair.
    let route_path = Route::component_block(component_name, name).to_string();

    let iframe_src = match router().prefix() {
        Some(prefix) => format!("{prefix}{route_path}"),
        None => route_path,
    };

    // Same construction as `ComponentVariantHighlight`'s identical
    // `frame_id` above (see its doc comment for the full rationale) --
    // this is the Block-kind sibling of that function, and
    // every "Block" component with more than one variant (e.g. `sidebar(
    // block)[floating, inset]`) renders all of them on this same
    // `/component/?name=<name>&` page (`ComponentHighlight` above doesn't
    // branch on kind for *that* -- only for which of these two functions
    // renders each variant), so it needs the identical duplicate-id
    // guard; `sidebar.spec.ts` only
    // drives the single-variant `/component/block/?name=sidebar&variant=
    // ...&` route (`ComponentBlockDemo`, a different function entirely,
    // one variant per page by construction), never this one.
    let frame_id = if main_variant {
        "component-preview-frame".to_string()
    } else {
        format!("component-preview-frame-{name}")
    };

    rsx! {
        if !main_variant {
            h3 { class: "dx-component-variant-title", "{variant_title(name)}" }
        }
        Tabs {
            default_value: "Preview",
            border_bottom_left_radius: "0.5rem",
            border_bottom_right_radius: "0.5rem",
            horizontal: true,
            width: "100%",
            variant: TabsVariant::Ghost,
            div { class: "dx-component-tabs-header",
                TabList {
                    TabTrigger { value: "Preview", "PREVIEW" }
                    TabTrigger { value: "Code", "CODE" }
                }
            }
            div {
                width: "100%",
                height: "100%",
                display: "flex",
                flex_direction: "column",
                justify_content: "center",
                align_items: "center",
                TabContent {
                    id: "{frame_id}",
                    // Nova tabs content has no padding (row 132); the Block frame keeps its 2rem.
                    padding: "var(--dx-space-8)",
                    value: "Preview",
                    width: "100%",
                    position: "relative",
                    // `overflow-x: clip` is set globally on `html`/`body`
                    // (`assets/main.css`) to keep the page itself from ever
                    // growing a horizontal scrollbar -- so an iframe wider
                    // than the space this page happens to leave it would
                    // otherwise just get silently clipped, not pushed into
                    // a scrollbar. This local `auto` opts *this* frame back
                    // into scrolling on its own, so `.dx-block-demo-iframe`'s
                    // desktop-only `min-width` (`assets/main.css`) stays
                    // reachable instead of being cut off.
                    overflow_x: "auto",
                    iframe {
                        // axe `frame-title`: every block demo is built here, so one title names them all.
                        title: "{component_name} {variant_title(name)} demo",
                        src: "{iframe_src}",
                        width: "100%",
                        // Block demos embed here at whatever width this
                        // page's own layout leaves them -- the site nav is a
                        // real `Sidebar` (`DocsLayout`, this file), so that
                        // can be under 768px on a normal desktop viewport
                        // (e.g. ~684px on the live `sidebar` demo). Any block component
                        // that (like `Sidebar` itself, `components/sidebar/
                        // component.rs`'s `MOBILE_BREAKPOINT`) switches to
                        // a different, closed-by-default layout below that
                        // breakpoint would silently render nothing in the
                        // preview: its own `window.innerWidth` is this
                        // iframe's content width, not the outer page's, so
                        // it has no way to know it's being squeezed rather
                        // than genuinely viewed on a narrow screen.
                        //
                        // A flat, always-on `min-width: 820px` here would
                        // keep the preview desktop-sized -- but it ALSO
                        // fires for a genuinely narrow real device/window,
                        // forcing an 820px-wide iframe into e.g. a 390px
                        // mobile viewport (the iframe's own
                        // `getBoundingClientRect()` reports `width: 822`
                        // inside a 390px viewport, clipped/garbled rather
                        // than scrolled to, since `overflow-x: clip` on
                        // `html`/`body` hides the escape without giving a
                        // scrollbar). A real mobile visitor SHOULD see
                        // this block's own mobile rendering (e.g.
                        // `Sidebar`'s Sheet-trigger button) -- forcing
                        // desktop width there is wrong, not just ugly.
                        //
                        // So the rule is `.dx-block-demo-iframe` in
                        // `assets/main.css`: the `min-width: 820px` rule
                        // lives behind `@media (min-width: 768px)` in THAT
                        // file, not this iframe's own document. A media
                        // query written inside the iframe's own stylesheet
                        // would evaluate against the iframe's own (possibly
                        // squeezed) viewport -- the exact ambiguous signal
                        // at the heart of the problem -- but `main.css` is
                        // loaded by the OUTER top-level page, so its media
                        // query evaluates against the outer page's real
                        // `window.innerWidth`, i.e. the actual visitor
                        // viewport/device width. That correctly tells apart
                        // "the outer desktop page's own sidebar nav is
                        // squeezing an otherwise-plenty-wide viewport"
                        // (outer viewport >= 768: force the comfortable
                        // desktop min-width; `overflow-x: auto` above makes
                        // the extra width reachable by scrolling within
                        // this card instead of overflowing the page) from
                        // "this actually is a narrow device/window" (outer
                        // viewport < 768: no forced min-width, so the
                        // iframe sizes to its real, narrow container and
                        // the embedded block component's own breakpoint
                        // check sees a genuinely narrow width and renders
                        // its real mobile layout, matching what a mobile
                        // visitor should see).
                        class: "dx-block-demo-iframe",
                        height: "600px",
                        border: "1px solid var(--dx-border)",
                        border_radius: "0.5em",
                    }
                }
                TabContent {
                    padding: "var(--dx-space-8)",
                    value: "Code",
                    width: "100%",
                    position: "relative",
                    if let Some(css) = css_highlighted {
                        ComponentCode {
                            rs_highlighted: highlighted,
                            css_highlighted: css,
                            component_type: ComponentType::Block,
                        }
                    } else {
                        CodeBlock { source: highlighted }
                    }
                }
            }
        }
        VariantNotes { owner: component_name, name }
    }
}
