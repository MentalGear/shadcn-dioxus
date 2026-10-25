use super::super::component::*;
use dioxus::prelude::*;

/// A root-absolute site path with the app base path (`dx --base-path`, e.g.
/// `/shadcn-dioxus` on Pages) in front. `NavigationMenuLink` renders a
/// plain `<a>`, not a router `Link`, so it does not get the prefix for free
/// and a bare `"/docs"` would 404 when the site is served below a sub-path.
fn site_href(path: &str) -> String {
    let prefix = try_router()
        .and_then(|router| router.prefix())
        .unwrap_or_default();
    format!("{}{path}", prefix.trim_end_matches('/'))
}

/// One entry of a panel's link list -- shadcn's demo `ListItem`: a title
/// (`text-sm leading-none font-medium`) over a muted description clamped to
/// two lines. Every list link in this demo goes through it, so a link can
/// never render as bare text next to titled siblings (which would make
/// "Button"/"Input" larger and lighter than "Introduction").
#[component]
fn ListItem(href: String, title: String, description: String) -> Element {
    rsx! {
        NavigationMenuLink { href,
            div { class: "dx-navigation-menu-link-title", "{title}" }
            div { class: "dx-navigation-menu-link-description", "{description}" }
        }
    }
}

#[component]
pub fn Demo() -> Element {
    rsx! {
        div {
            // A distinct accessible name from the site's own top-level nav
            // avoids an axe `landmark-unique` collision on this demo page
            // -- same reasoning as `../../navbar/variants/main/mod.rs`'s
            // identical comment.
            NavigationMenu { aria_label: "Component navigation menu",
                NavigationMenuList {
                    NavigationMenuItem {
                        NavigationMenuTrigger { "Getting started" }
                        NavigationMenuContent {
                            div { class: "dx-navigation-menu-grid dx-navigation-menu-grid-2col",
                                NavigationMenuLink {
                                    href: site_href("/"),
                                    class: "dx-navigation-menu-featured",
                                    // Deliberately not "shadcn-dioxus" --
                                    // the site's own persistent chrome
                                    // (`main.rs`'s navbar brand link) already
                                    // renders an `<a>` with that exact
                                    // accessible name on every page, and
                                    // `getByRole('link', { name: ... })`
                                    // (both this demo's own oracle/smoke
                                    // specs and any real screen-reader user's
                                    // link-by-name navigation) would
                                    // otherwise hit two matches.
                                    // Same class of fix as this
                                    // component's own `aria_label` above.
                                    div { class: "dx-navigation-menu-link-title", "Component Library" }
                                    div { class: "dx-navigation-menu-link-description",
                                        "Beautifully designed, accessible primitives for Dioxus."
                                    }
                                }
                                ListItem {
                                    href: site_href("/docs"),
                                    title: "Introduction",
                                    description: "Re-usable primitives you can copy into your own project.",
                                }
                                ListItem {
                                    href: site_href("/component/button/"),
                                    title: "Button",
                                    description: "Displays a button or a component that looks like a button.",
                                }
                                ListItem {
                                    href: site_href("/component/input/"),
                                    title: "Input",
                                    description: "Displays a form input field or a component that looks like an input field.",
                                }
                            }
                        }
                    }
                    NavigationMenuItem {
                        NavigationMenuTrigger { "Components" }
                        NavigationMenuContent {
                            div { class: "dx-navigation-menu-grid",
                                ListItem {
                                    href: site_href("/component/accordion/"),
                                    title: "Accordion",
                                    description: "A vertically stacked set of collapsible panels.",
                                }
                                ListItem {
                                    href: site_href("/component/dialog/"),
                                    title: "Dialog",
                                    description: "A modal window layered above the page.",
                                }
                                ListItem {
                                    href: site_href("/component/tooltip/"),
                                    title: "Tooltip",
                                    description: "A short message shown on hover or focus.",
                                }
                                ListItem {
                                    href: site_href("/component/tabs/"),
                                    title: "Tabs",
                                    description: "Switch between panels of related content.",
                                }
                                ListItem {
                                    href: site_href("/component/select/"),
                                    title: "Select",
                                    description: "Pick one value from a list of options.",
                                }
                                ListItem {
                                    href: site_href("/component/progress/"),
                                    title: "Progress",
                                    description: "Displays the completion progress of a task.",
                                }
                            }
                        }
                    }
                    NavigationMenuItem {
                        NavigationMenuLink { href: site_href("/docs"), "Docs" }
                    }
                }
            }
        }
    }
}
