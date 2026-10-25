use super::super::component::*;
use dioxus::prelude::*;

/// A root-absolute site path with the app base path in front -- same helper
/// as `../main/mod.rs` (a plain `<a>`, not a router `Link`, gets no prefix
/// for free).
fn site_href(path: &str) -> String {
    let prefix = try_router()
        .and_then(|router| router.prefix())
        .unwrap_or_default();
    format!("{}{path}", prefix.trim_end_matches('/'))
}

/// Click-activation fixture (`open_on_hover: false`).
///
/// Same shape as `main`, but hovering a trigger does nothing and the pointer
/// leaving an open panel does not close it: a panel opens on click/Enter/
/// Space/ArrowDown and closes on a second click, Escape, choosing a link,
/// focus leaving the nav, or a press outside it.
///
/// Every label below ("Click-only navigation menu"/"Guides"/"Reference"/
/// "Installation"/"Theming"/"Primitives"/"Changelog") deliberately differs
/// from the `main` variant's ("Component navigation menu"/"Getting started"/
/// "Components"/"Docs"/...): `navigation_menu.spec.ts` and the oracle
/// `disclosure-navigation.spec.ts` query those words by name (substring, no
/// `exact`), and this variant renders alongside `main` on the same page, so
/// reusing them would make those locators match both.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div {
            NavigationMenu { aria_label: "Click-only navigation menu", open_on_hover: false,
                NavigationMenuList {
                    NavigationMenuItem {
                        NavigationMenuTrigger { "Guides" }
                        NavigationMenuContent {
                            div { class: "dx-navigation-menu-grid",
                                NavigationMenuLink {
                                    href: site_href("/docs"),
                                    "Installation"
                                }
                                NavigationMenuLink {
                                    href: site_href("/docs"),
                                    "Theming"
                                }
                            }
                        }
                    }
                    NavigationMenuItem {
                        NavigationMenuTrigger { "Reference" }
                        NavigationMenuContent {
                            div { class: "dx-navigation-menu-grid",
                                NavigationMenuLink {
                                    href: site_href("/component/button/"),
                                    "Primitives"
                                }
                            }
                        }
                    }
                    NavigationMenuItem {
                        NavigationMenuLink { href: site_href("/docs"), "Changelog" }
                    }
                }
            }
        }
    }
}
