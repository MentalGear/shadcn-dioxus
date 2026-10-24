use super::super::component::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::dropdown_menu::{
    DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger,
};
use dioxus::prelude::*;
use dioxus_primitives::AsChild;
use dioxus_icons::lucide::ChevronDown;

const SECTIONS: &[&str] = &["Documentation", "Themes", "GitHub"];

#[component]
pub fn Demo() -> Element {
    let mut section = use_signal(|| "Documentation".to_string());

    rsx! {
        Breadcrumb { aria_label: "Path with dropdown",
            BreadcrumbList {
                BreadcrumbItem {
                    BreadcrumbLink { href: "#", "Home" }
                }
                BreadcrumbSeparator {}
                BreadcrumbItem {
                    DropdownMenu {
                        // `as` renders the trigger AS a compact ghost `Button` (shadcn: a
                        // flex item with `gap-1` and a 14px chevron).
                        DropdownMenuTrigger { aria_label: "Choose section: {section}",
                            as: move |el: AsChild| rsx! {
                                Button {
                                    variant: ButtonVariant::Ghost,
                                    size: ButtonSize::Sm,
                                    attributes: el.attributes,
                                    {el.children}
                                }
                            },
                            "{section}"
                            ChevronDown {}
                        }
                        DropdownMenuContent {
                            for (i , name) in SECTIONS.iter().enumerate() {
                                DropdownMenuItem::<String> {
                                    value: name.to_string(),
                                    index: i,
                                    text_value: *name,
                                    on_select: move |value: String| section.set(value),
                                    "{name}"
                                }
                            }
                        }
                    }
                }
                BreadcrumbSeparator {}
                BreadcrumbItem {
                    BreadcrumbPage { "Breadcrumb" }
                }
            }
        }
    }
}
