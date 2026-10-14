use super::super::component::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::dropdown_menu::{
    DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger,
};
use dioxus::prelude::*;

const SECTIONS: &[&str] = &["Documentation", "Themes", "GitHub"];

/// shadcn's `breadcrumb-demo`: the ellipsis as the face of a menu trigger. `BreadcrumbEllipsis` is a `span`,
/// so it sits validly inside the trigger button (a ghost `icon-sm` `Button`), and the button, not the
/// `aria-hidden` ellipsis, carries the accessible name.
#[component]
pub fn Demo() -> Element {
    let mut picked = use_signal(|| "Nothing picked yet".to_string());

    rsx! {
        div { style: "display: flex; flex-direction: column; align-items: flex-start; gap: 0.75rem;",
            Breadcrumb { aria_label: "Path with a collapsed menu",
                BreadcrumbList {
                    BreadcrumbItem {
                        BreadcrumbLink { href: "#", "Home" }
                    }
                    BreadcrumbSeparator {}
                    BreadcrumbItem {
                        DropdownMenu {
                            DropdownMenuTrigger { aria_label: "Toggle menu",
                                as: move |attributes: Vec<Attribute>| rsx! {
                                    Button {
                                        variant: ButtonVariant::Ghost,
                                        size: ButtonSize::IconSm,
                                        attributes,
                                        BreadcrumbEllipsis {}
                                    }
                                },
                            }
                            DropdownMenuContent {
                                for (i , name) in SECTIONS.iter().enumerate() {
                                    DropdownMenuItem::<String> {
                                        value: name.to_string(),
                                        index: i,
                                        text_value: *name,
                                        on_select: move |value: String| picked.set(value),
                                        "{name}"
                                    }
                                }
                            }
                        }
                    }
                    BreadcrumbSeparator {}
                    BreadcrumbItem {
                        BreadcrumbLink { href: "#", "Components" }
                    }
                    BreadcrumbSeparator {}
                    BreadcrumbItem {
                        BreadcrumbPage { "Breadcrumb" }
                    }
                }
            }
            p { style: "margin: 0; font-size: 0.875rem; color: var(--dx-muted-foreground);",
                "Picked: {picked}"
            }
        }
    }
}
