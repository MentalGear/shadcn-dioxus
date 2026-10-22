use super::super::component::*;
use dioxus::prelude::*;

/// The same bar as a shadcn demo with a `NavigationMenuIndicator` after the items: the arrow
/// under the open trigger, following it as the open item changes. Labels differ from `main`'s so
/// locators on that variant never match this one.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div {
            NavigationMenu { aria_label: "Indicator navigation menu",
                NavigationMenuList {
                    NavigationMenuItem { index: 0usize,
                        NavigationMenuTrigger { "Learn" }
                        NavigationMenuContent {
                            div { class: "dx-navigation-menu-grid",
                                NavigationMenuLink { href: "#learn", content_index: 0usize, "Tutorials" }
                                NavigationMenuLink { href: "#learn", content_index: 1usize, "Guides" }
                            }
                        }
                    }
                    NavigationMenuItem { index: 1usize,
                        NavigationMenuTrigger { "Build with the library" }
                        NavigationMenuContent {
                            div { class: "dx-navigation-menu-grid",
                                NavigationMenuLink { href: "#build", content_index: 0usize, "Recipes" }
                            }
                        }
                    }
                    NavigationMenuIndicator {}
                }
            }
        }
    }
}
