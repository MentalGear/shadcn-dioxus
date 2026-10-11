use super::super::component::*;
use dioxus::prelude::*;
use dioxus_icons::lucide::ChevronLeft;
use dioxus_primitives::direction::{Direction, DirectionProvider};

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { dir: "rtl",
            DirectionProvider { direction: Direction::Rtl,
                Breadcrumb { aria_label: "مسار التنقل",
                    BreadcrumbList {
                        BreadcrumbItem {
                            BreadcrumbLink { href: "#", "الرئيسية" }
                        }
                        BreadcrumbSeparator {
                            ChevronLeft {}
                        }
                        BreadcrumbItem {
                            BreadcrumbEllipsis {}
                        }
                        BreadcrumbSeparator {
                            ChevronLeft {}
                        }
                        BreadcrumbItem {
                            BreadcrumbLink { href: "#", "المكونات" }
                        }
                        BreadcrumbSeparator {
                            ChevronLeft {}
                        }
                        BreadcrumbItem {
                            BreadcrumbPage { "مسار التنقل" }
                        }
                    }
                }
            }
        }
    }
}
