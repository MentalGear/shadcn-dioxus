use super::super::component::*;
use crate::nav_link::NavLink;
use crate::Route;
use dioxus::prelude::*;
use dioxus_primitives::AsChild;

/// shadcn's `render={<Link />}`: `as` hands the crumb's attributes to the app's router link, so
/// the crumb keeps its look and a click navigates in-app instead of reloading the page.
#[component]
pub fn Demo() -> Element {
    rsx! {
        Breadcrumb { aria_label: "Path with router links",
            BreadcrumbList {
                BreadcrumbItem {
                    BreadcrumbLink {
                        r#as: move |el: AsChild| rsx! {
                            NavLink { to: Route::component("card"), attributes: el.attributes, {el.children} }
                        },
                        "Card"
                    }
                }
                BreadcrumbSeparator {}
                BreadcrumbItem {
                    BreadcrumbLink {
                        r#as: move |el: AsChild| rsx! {
                            NavLink { to: Route::component("separator"), attributes: el.attributes, {el.children} }
                        },
                        "Separator"
                    }
                }
                BreadcrumbSeparator {}
                BreadcrumbItem {
                    BreadcrumbPage { "Router link" }
                }
            }
        }
    }
}
