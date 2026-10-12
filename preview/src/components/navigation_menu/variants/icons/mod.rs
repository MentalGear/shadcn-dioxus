use super::super::component::*;
use dioxus::prelude::*;
use dioxus_icons::lucide::{BookOpen, Calendar, Compass, Layers, LayoutGrid, Palette};

/// Same helper as `../main/mod.rs`: a plain `<a>` gets no `--base-path` prefix for free.
fn site_href(path: &str) -> String {
    let prefix = try_router()
        .and_then(|router| router.prefix())
        .unwrap_or_default();
    format!("{}{path}", prefix.trim_end_matches('/'))
}

/// One column, 25rem wide: the 3-link panels here have no featured card, so the stock 2-column,
/// 37.5rem grid (`.dx-navigation-menu-grid`, sized for shadcn's long lists) made the panel 608px
/// wide and the right-hand "Resources" panel ran 11px past the page content area. 25rem is the
/// stock grid's own `sm` tier; `max-width` keeps it inside the panel on a phone.
const PANEL_GRID: &str = "width: 25rem; max-width: 100%; grid-template-columns: minmax(0, 1fr);";

/// A panel link with a leading icon: the icon sits beside the title, the description stays below.
/// The trigger and link names differ from `main`'s and `click_only`'s on purpose, because
/// `navigation_menu.spec.ts` queries those words by name (substring) on the shared page.
#[component]
fn IconLink(
    href: String,
    index: usize,
    title: String,
    description: String,
    children: Element,
) -> Element {
    rsx! {
        NavigationMenuLink { href, content_index: index,
            div { style: "display: flex; align-items: center; gap: 0.5rem;",
                {children}
                div { class: "dx-navigation-menu-link-title", "{title}" }
            }
            div { class: "dx-navigation-menu-link-description", "{description}" }
        }
    }
}

#[component]
pub fn Demo() -> Element {
    rsx! {
        div {
            NavigationMenu { aria_label: "Navigation menu with icons",
                NavigationMenuList {
                    NavigationMenuItem { index: 0usize,
                        NavigationMenuTrigger { "Product" }
                        NavigationMenuContent {
                            div {
                                class: "dx-navigation-menu-grid",
                                style: PANEL_GRID,
                                IconLink {
                                    href: site_href("/component/calendar/"),
                                    index: 0usize,
                                    title: "Calendar",
                                    description: "Pick a date or a range of dates.",
                                    Calendar { size: "16px" }
                                }
                                IconLink {
                                    href: site_href("/component/card/"),
                                    index: 1usize,
                                    title: "Card",
                                    description: "A container for related content and actions.",
                                    Layers { size: "16px" }
                                }
                                IconLink {
                                    href: site_href("/component/table/"),
                                    index: 2usize,
                                    title: "Table",
                                    description: "Rows and columns of structured data.",
                                    LayoutGrid { size: "16px" }
                                }
                            }
                        }
                    }
                    NavigationMenuItem { index: 1usize,
                        NavigationMenuTrigger { "Resources" }
                        NavigationMenuContent {
                            div {
                                class: "dx-navigation-menu-grid",
                                style: PANEL_GRID,
                                IconLink {
                                    href: site_href("/docs"),
                                    index: 0usize,
                                    title: "Introduction",
                                    description: "Learn how the library fits into your project.",
                                    BookOpen { size: "16px" }
                                }
                                IconLink {
                                    href: site_href("/component/accordion/"),
                                    index: 1usize,
                                    title: "Tour",
                                    description: "A quick look at the most common building blocks.",
                                    Compass { size: "16px" }
                                }
                                IconLink {
                                    href: site_href("/component/badge/"),
                                    index: 2usize,
                                    title: "Look and feel",
                                    description: "Colors, radii and type that follow your theme.",
                                    Palette { size: "16px" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
