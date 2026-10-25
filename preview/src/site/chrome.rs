use super::*;

/// The outermost layout, applied to EVERY route (never popped by an
/// `#[end_layout]` anywhere in `Route`) -- this is deliberately where
/// `GlobalHead` renders (dev-docs/backlog.md row 46, below), not
/// `NavigationLayout`, precisely because it stays mounted across a
/// client-side transition between ANY two routes in the app, including two
/// routes that both sit outside `NavigationLayout`
/// (`ComponentBlockDemo`/`ComponentBlockDemoPath`/`EmailClientDashboard`).
///
/// **Why `GlobalHead` mounts here, once (row 46):** if each of those three routes called
/// `GlobalHead {}` itself, the first client-side navigation between two routes that each mount their
/// own independent `GlobalHead` instance (`ComponentBlockDemo`'s redirect to `ComponentBlockDemoPath`)
/// would silently drop `main.css`/`dx-components-theme.css`/the Google Fonts `<link>`s entirely
/// (present on a hard load of the destination route directly, absent after the redirect transition)
/// rather than erroring: a hydration-adjacent silent-CSS-loss defect of the same *class*
/// `oracle/tier2-html/global-stylesheet.spec.ts` exists to catch for the `@import` case. Root cause
/// (see `eager_head`'s module docs): `document::Link` records its href in a de-duplication set while
/// it RENDERS but only appends the `<link>` from an effect queued on its own scope, and a removed
/// scope's queued effects are dropped -- so the FIRST `GlobalHead` instance, torn down by the redirect
/// in the turn it first rendered, never inserts anything, and the SECOND instance's identical
/// `document::Link`s are skipped as duplicates of links that were never there. (Unmounting does not
/// remove head elements, so the sheets can only be absent because they were never inserted.) This
/// layout's once-only mount covers these four sheets; the class -- every other component's link on
/// any torn-down subtree -- is covered by `eager_head`, installed in `App`. `GlobalHead` therefore
/// mounts exactly once, here, and never unmounts for the lifetime of the app. No route
/// (`NavigationLayout`, `ComponentBlockDemo`/`ComponentBlockDemoPath`, `EmailClientDashboard`)
/// calls it;
/// `NavigationLayout`'s own `hero.css` link stays where it is, since nothing outside
/// `NavigationLayout` needs `hero.css`, so no cross-layout transition unmounts it under a sibling
/// that also renders it.
#[component]
pub(crate) fn AppLayout() -> Element {
    use_effect(move || {
        theme::theme_seed();
        if let Some(dark_mode) = Route::in_dark_mode() {
            theme::set_theme(dark_mode);
        }
    });

    // The ONE place that sets the document title (row 336): derived from the current route, so a
    // client-side navigation replaces it and no page can forget to. Pages never render their own
    // `document::Title`; `Route::page_title` is the table.
    let route = use_route::<Route>();
    let title = route.page_title();

    rsx! {
        document::Title { "{title}" }
        GlobalHead {}
        Outlet::<Route> {}
    }
}

#[component]
pub(crate) fn NavigationLayout() -> Element {
    // Send the route to the parent window if in an iframe
    let mut initial_route = use_hook(|| CopyValue::new(true));
    use_effect(move || {
        let route: Route = router().current();

        // Only send route changes, not the initial route
        if initial_route() || !Route::in_iframe().unwrap_or_default() {
            initial_route.set(false);
            return;
        }

        let eval = document::eval(
            "let route = await dioxus.recv();
            window.top.postMessage({ 'route': route }, '*');",
        );
        let _ = eval.send(route.to_string());
    });

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/hero.css") }
        Outlet::<Route> {}
        Footer {}
    }
}

/// The site's persistent top nav. Rendered as a descendant of `DocsLayout`'s
/// `SidebarProvider` on every route that has a sidebar (`Docs`, `Home`,
/// `ComponentDemo`/`ComponentHighlight`) so its leading `SidebarTrigger` can
/// sit inside this same row instead of a separate strip below it (a `Navbar {}`
/// rendered as a sibling of `Outlet::<Route> {}` would not be a descendant of
/// `DocsLayout`'s `SidebarProvider`, and `DocsLayout` would need its own
/// separate `header { class: "dx-docs-inset-topbar", SidebarTrigger {} }`
/// strip). `Demos` has no sidebar and renders `Navbar {}` directly (as does
/// `ComponentDemo`'s own "not found" branch, which also bypasses
/// `DocsLayout`) -- `try_consume_context` (not `consume_context`, same
/// defensive-read pattern as `HomeSidebarControls` above) is what makes
/// omitting the trigger there safe rather than a panic, since neither page
/// has a `SidebarProvider` ancestor.
/// The `id` of every page's `<main>`: the target of the "Skip to content" link `Navbar` renders. A page
/// that renders a `<main>` passes `id: MAIN_ID, tabindex: "-1"` (the `tabindex` lets the fragment
/// navigation move focus into it, not only scroll), so the link has the same target everywhere;
/// `playwright/all-routes.spec.ts` checks it on every route.
pub(crate) const MAIN_ID: &str = "main-content";

#[component]
pub(crate) fn Navbar() -> Element {
    let in_iframe = Route::in_iframe().unwrap_or_default();
    let in_component = matches!(
        router().current(),
        Route::ComponentDemo { .. } | Route::ComponentDemoPath { .. }
    );
    let has_sidebar = try_consume_context::<SidebarCtx>().is_some();
    if in_iframe {
        return rsx! {
            nav {
                class: "dx-preview-navbar",
                aria_label: "Primary",
                border: "none",
                padding: "1rem",
                justify_content: "flex-start",
                if has_sidebar {
                    SidebarTrigger {
                        class: "dx-navbar-sidebar-trigger",
                        icon: rsx! {
                            Menu { class: "dx-sidebar-trigger-icon", size: "1rem" }
                        },
                    }
                }
                if in_component {
                    NavLink {
                        to: Route::home(),
                        class: "dx-navbar-brand",
                        aria_label: "Back",
                        ChevronLeft {
                            size: "2rem",
                            stroke: "var(--dx-foreground)",
                        }
                    }
                }
            }
        };
    }

    rsx! {
        // WCAG 2.4.1: the first focusable element on the page. Without it a keyboard user tabs through
        // the whole docs sidebar (88 stops on `/component/button/`) before reaching the content. It is
        // visually hidden until focused (`main.css`, `.dx-skip-link`). The block-demo iframe branch
        // above omits it: that page is one demo, embedded, with no nav to skip.
        a { class: "dx-skip-link", href: "#{MAIN_ID}", "Skip to content" }
        nav { class: "dx-preview-navbar", aria_label: "Primary",
            div { class: "dx-navbar-inner",
                div { class: "dx-navbar-primary",
                    if has_sidebar {
                        SidebarTrigger {
                        class: "dx-navbar-sidebar-trigger",
                        icon: rsx! {
                            Menu { class: "dx-sidebar-trigger-icon", size: "1rem" }
                        },
                    }
                    }
                    NavLink { to: Route::home(), class: "dx-navbar-brand",
                        img {
                            src: asset!("/assets/dioxus_color.svg"),
                            alt: "Dioxus Logo",
                            width: "18",
                            height: "18",
                        }
                        span { "{SITE_NAME}" }
                    }
                    NavLink { to: Route::docs(), class: "dx-navbar-link", "Docs" }
                    NavLink { to: Route::demos(), class: "dx-navbar-link", "Demos" }
                    NavLink { to: Route::charts_index(), class: "dx-navbar-link", "Charts" }
                }
                div { class: "dx-navbar-utilities",
                    // TODO: restore once the primitives crate is published
                    // Link {
                    //     to: "https://crates.io/crates/dioxus-components",
                    //     class: "dx-navbar-link",
                    //     aria_label: "Dioxus-Components crates.io",
                    //     Icon {
                    //         width: "24px",
                    //         height: "24px",
                    //         viewBox: ViewBox::new(0, 0, 576, 512),
                    //         path {
                    //             d: "M290.8 48.6l78.4 29.7L288 109.5 206.8 78.3l78.4-29.7c1.8-.7 3.8-.7 5.7 0zM136 92.5l0 112.2c-1.3 .4-2.6 .8-3.9 1.3l-96 36.4C14.4 250.6 0 271.5 0 294.7L0 413.9c0 22.2 13.1 42.3 33.5 51.3l96 42.2c14.4 6.3 30.7 6.3 45.1 0L288 457.5l113.5 49.9c14.4 6.3 30.7 6.3 45.1 0l96-42.2c20.3-8.9 33.5-29.1 33.5-51.3l0-119.1c0-23.3-14.4-44.1-36.1-52.4l-96-36.4c-1.3-.5-2.6-.9-3.9-1.3l0-112.2c0-23.3-14.4-44.1-36.1-52.4l-96-36.4c-12.8-4.8-26.9-4.8-39.7 0l-96 36.4C150.4 48.4 136 69.3 136 92.5zM392 210.6l-82.4 31.2 0-89.2L392 121l0 89.6zM154.8 250.9l78.4 29.7L152 311.7 70.8 280.6l78.4-29.7c1.8-.7 3.8-.7 5.7 0zm18.8 204.4l0-100.5L256 323.2l0 95.9-82.4 36.2zM421.2 250.9c1.8-.7 3.8-.7 5.7 0l78.4 29.7L424 311.7l-81.2-31.1 78.4-29.7zM523.2 421.2l-77.6 34.1 0-100.5L528 323.2l0 90.7c0 3.2-1.9 6-4.8 7.3z",
                    //             fill: "currentColor",
                    //             fill_rule: "nonzero",
                    //         }
                    //     }
                    // }
                    // link-ok: external target, never the current page
                    Link {
                        to: REGISTRY_GIT_URL,
                        class: "dx-navbar-link",
                        img {
                            class: "dx-light-mode-only",
                            src: asset!("/assets/github-mark/github-mark.svg"),
                            alt: "GitHub",
                            width: "22",
                            height: "22",
                        }
                        img {
                            class: "dx-dark-mode-only",
                            src: asset!("/assets/github-mark/github-mark-white.svg"),
                            alt: "GitHub",
                            width: "22",
                            height: "22",
                        }
                    }
                    theme::ThemePicker {}
                    theme::DarkModeToggle {}
                }
            }
        }
    }
}

#[component]
pub(crate) fn Footer() -> Element {
    if Route::in_iframe().unwrap_or_default() {
        return rsx! {};
    }

    rsx! {
        footer { class: "dx-preview-footer",
            div { class: "dx-footer-inner",
                div { class: "dx-footer-brand",
                    NavLink { to: Route::home(), class: "dx-footer-brand-link",
                        img {
                            src: asset!("/assets/dioxus_color.svg"),
                            alt: "Dioxus Logo",
                            width: "22",
                            height: "22",
                        }
                        span { "{SITE_NAME}" }
                    }
                    p { class: "dx-footer-tagline",
                        "Accessible, themeable interface pieces for Dioxus apps."
                    }
                }
                nav { class: "dx-footer-nav", aria_label: "Footer",
                    div { class: "dx-footer-nav-group",
                        span { class: "dx-footer-nav-heading", "Library" }
                        NavLink { to: Route::home(), class: "dx-footer-link", "Components" }
                        NavLink { to: Route::docs(), class: "dx-footer-link", "Docs" }
                        NavLink { to: Route::demos(), class: "dx-footer-link", "Demos" }
                        NavLink { to: Route::charts_index(), class: "dx-footer-link", "Charts" }
                    }
                    div { class: "dx-footer-nav-group",
                        span { class: "dx-footer-nav-heading", "Project" }
                        // link-ok: external target, never the current page
                        Link {
                            to: REGISTRY_GIT_URL,
                            class: "dx-footer-link",
                            "GitHub"
                        }
                        // link-ok: external target, never the current page
                        Link {
                            to: "https://dioxuslabs.com",
                            class: "dx-footer-link",
                            "Dioxus"
                        }
                    }
                }
            }
            div { class: "dx-footer-base",
                span { class: "dx-footer-copy", "Built with Dioxus." }
            }
        }
    }
}
