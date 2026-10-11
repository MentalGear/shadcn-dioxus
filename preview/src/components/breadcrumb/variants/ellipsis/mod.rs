use super::super::component::*;
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    rsx! {
        Breadcrumb { aria_label: "Path with collapsed items",
            BreadcrumbList {
                BreadcrumbItem {
                    BreadcrumbLink { href: "#", "Home" }
                }
                BreadcrumbSeparator {}
                BreadcrumbItem {
                    BreadcrumbEllipsis {}
                }
                BreadcrumbSeparator {}
                BreadcrumbItem {
                    BreadcrumbLink { href: "#", "Settings" }
                }
                BreadcrumbSeparator {}
                BreadcrumbItem {
                    BreadcrumbPage { "Notifications" }
                }
            }
        }
    }
}
