use super::super::component::*;
use dioxus::prelude::*;

/// The two ends of a page range: on the first page Previous is unavailable, on the last page Next is. The
/// pagination has no `disabled` prop, so an unavailable link has no `href` (it is not focusable) and carries
/// `aria-disabled`, with the dimmed look of a disabled button.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 1rem; width: 100%;",
            div { style: "display: flex; flex-direction: column; align-items: center; gap: 0.5rem;",
                p { style: "margin: 0; font-size: var(--dx-text-sm); color: var(--dx-muted-foreground);", "First page" }
                Pagination {
                    "aria-label": "First page pagination",
                    PaginationContent {
                        PaginationItem {
                            PaginationPrevious {
                                "aria-disabled": "true",
                                style: "opacity: var(--dx-opacity-disabled); pointer-events: none;",
                            }
                        }
                        PaginationItem {
                            PaginationLink { href: "#", is_active: true, "1" }
                        }
                        PaginationItem {
                            PaginationLink { href: "#", "2" }
                        }
                        PaginationItem {
                            PaginationNext { href: "#" }
                        }
                    }
                }
            }
            div { style: "display: flex; flex-direction: column; align-items: center; gap: 0.5rem;",
                p { style: "margin: 0; font-size: var(--dx-text-sm); color: var(--dx-muted-foreground);", "Last page" }
                Pagination {
                    "aria-label": "Last page pagination",
                    PaginationContent {
                        PaginationItem {
                            PaginationPrevious { href: "#" }
                        }
                        PaginationItem {
                            PaginationLink { href: "#", "9" }
                        }
                        PaginationItem {
                            PaginationLink { href: "#", is_active: true, "10" }
                        }
                        PaginationItem {
                            PaginationNext {
                                "aria-disabled": "true",
                                style: "opacity: var(--dx-opacity-disabled); pointer-events: none;",
                            }
                        }
                    }
                }
            }
        }
    }
}
