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
                    NavigationMenuItem {
                        NavigationMenuTrigger { "Learn" }
                        NavigationMenuContent {
                            div { class: "dx-navigation-menu-grid",
                                NavigationMenuLink { href: "#learn", "Tutorials" }
                                NavigationMenuLink { href: "#learn", "Guides" }
                            }
                        }
                    }
                    NavigationMenuItem {
                        NavigationMenuTrigger { "Build with the library" }
                        NavigationMenuContent {
                            div { class: "dx-navigation-menu-grid",
                                NavigationMenuLink { href: "#build", "Recipes" }
                            }
                        }
                    }
                    NavigationMenuIndicator {}
                }
            }
        }
    }
}
