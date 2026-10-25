use super::*;

pub(crate) struct DemoEntry {
    pub(crate) tag: &'static str,
    pub(crate) title: &'static str,
    pub(crate) description: &'static str,
    pub(crate) route: fn() -> Route,
    pub(crate) thumb: fn() -> Element,
}

pub(crate) fn email_client_thumb() -> Element {
    rsx! {
        Mail { size: "56", stroke_width: "1.4" }
    }
}

pub(crate) const DEMO_ENTRIES: &[DemoEntry] = &[DemoEntry {
    tag: "Dashboard",
    title: "Email client",
    description:
        "Multi-pane mail app composed from the sidebar, item list, reading pane, and compose modal.",
    route: || Route::EmailClientDashboard {
        dark_mode: Route::in_dark_mode(),
    },
    thumb: email_client_thumb,
}];

#[component]
pub(crate) fn Demos(dark_mode: Option<bool>) -> Element {
    rsx! {
        // The same docs shell as every other page (backlog row 303): its `SidebarInset`
        // renders the page's one `<main>`, so the content is a plain `div` here.
        DocsLayout { active: DocsNavActive::Demos,
            div { class: "dx-home-page",
                section { class: "dx-home-section",
                    header { class: "dx-section-header",
                        span { class: "dx-section-eyebrow", "Demos" }
                        h1 { class: "dx-section-title", "Demo apps" }
                        p { class: "dx-section-summary",
                            "End-to-end app demos assembled from these primitives. Open one to explore the layout and try it live."
                        }
                    }
                    ul { class: "dx-demos-grid",
                        for entry in DEMO_ENTRIES {
                            li { class: "dx-demos-item",
                                // link-ok: a demo-app card on `/demos`, never `/demos` itself
                                Link {
                                    to: (entry.route)(),
                                    class: "dx-demos-card",
                                    div { class: "dx-demos-card-thumb", {(entry.thumb)()} }
                                    div { class: "dx-demos-card-meta",
                                        span { class: "dx-demos-card-tag", "{entry.tag}" }
                                        h2 { class: "dx-demos-card-title", "{entry.title}" }
                                        p { class: "dx-demos-card-description", "{entry.description}" }
                                        span { class: "dx-demos-card-cta",
                                            "Open demo"
                                            ArrowRight { size: "16", stroke_width: "1.6" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // The header and the card grid are the page's top group.
                    FirstPaintGate {}
                }
            }
        }
    }
}

/// Legacy query-string deep link (`/component/?name=<x>&`, dev-docs/
/// backlog.md row 46). Renders the SAME markup regardless of `name` -- a
/// generic, name-agnostic loading shell -- on every platform, so there is
/// nothing for a server prerender and a client hydration to ever disagree
/// on structurally (resolving `name` against `components::DEMOS` here would
/// render the "Component not found" shell server-side for every X, since
/// `dx build --ssg` only ever prerenders this route with an EMPTY query --
/// see `Route::ComponentDemo`'s own doc comment). Once mounted client-side (in
/// a real browser, where `name` IS available from the real URL), redirects
/// to the canonical `ComponentDemoPath` route so old links/bookmarks/specs
/// keep landing on the right component.
#[component]
pub(crate) fn ComponentDemo(
    iframe: Option<bool>,
    dark_mode: Option<bool>,
    name: String,
) -> Element {
    let nav = navigator();
    use_effect(move || {
        nav.replace(Route::ComponentDemoPath {
            name: name.clone(),
            iframe,
            dark_mode,
        });
    });

    rsx! {
        Navbar {}
        main {
            id: MAIN_ID,
            tabindex: "-1",
            class: "dx-component-demo-redirect",
            role: "main",
            p { "Loading component…" }
        }
    }
}

/// Canonical, SSG-enumerable component page (`/component/<name>/`, row 46).
/// `name` is a PATH segment here, so the server prerender for a given `name`
/// and the client's initial render for that same URL always resolve to the
/// same branch below -- unlike the legacy `ComponentDemo` query route, a
/// genuinely nonexistent `name` shows "Component not found" identically on
/// both sides rather than on every name unconditionally.
#[component]
pub(crate) fn ComponentDemoPath(
    iframe: Option<bool>,
    dark_mode: Option<bool>,
    name: String,
) -> Element {
    let route = router().current::<Route>();
    tracing::info!("route: {route}");
    let Some(demo) = components::DEMOS
        .iter()
        .find(|demo| demo.name == name)
        .cloned()
    else {
        // Bypasses `ComponentHighlight`/`DocsLayout` entirely, so -- like
        // `Demos` -- there is no `SidebarProvider` ancestor here either;
        // `Navbar {}` renders its trigger-less fallback (see its own doc
        // comment) rather than being left off this page altogether.
        return rsx! {
            Navbar {}
            main { id: MAIN_ID, tabindex: "-1", class: "dx-component-demo-not-found",
                h3 { "Component not found" }
                p { "The requested component does not exist." }
            }
        };
    };
    rsx! {
        ComponentHighlight { demo }
    }
}

/// The catch-all (`Route::NotFound`): a URL no route matches. Same markup and
/// class as `ComponentDemoPath`'s "Component not found" shell (no sidebar, so
/// `Navbar {}` renders its trigger-less form), plus a way home. Client-only in
/// practice: it is never prerendered (see the route's comment).
#[component]
pub(crate) fn NotFound(segments: Vec<String>) -> Element {
    let path = segments.join("/");
    rsx! {
        Navbar {}
        main { id: MAIN_ID, tabindex: "-1", class: "dx-component-demo-not-found",
            h3 { "Page not found" }
            p { "There is no page at \"/{path}\"." }
            // link-ok: content link; the catch-all page is never `/`
            Link { to: Route::home(), "Back to the home page" }
        }
    }
}
