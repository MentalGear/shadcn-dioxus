use super::super::component::*;
use dioxus::prelude::*;
use dioxus_icons::lucide::Slash;

#[component]
pub fn Demo() -> Element {
    rsx! {
        Breadcrumb { aria_label: "Path with custom separator",
            BreadcrumbList {
                BreadcrumbItem {
                    BreadcrumbLink { href: "#", "Home" }
                }
                BreadcrumbSeparator {
                    Slash {}
                }
                BreadcrumbItem {
                    BreadcrumbLink { href: "#", "Components" }
                }
                BreadcrumbSeparator {
                    Slash {}
                }
                BreadcrumbItem {
                    BreadcrumbPage { "Button" }
                }
            }
        }
    }
}
