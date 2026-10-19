use super::super::component::*;
use crate::components::card::{Card, CardContent};
use dioxus::prelude::*;
use dioxus_icons::lucide::{ChevronLeft, ChevronRight};

/// Embla `dragFree` demo: `drag_free: true`, slides of different widths
/// (`flex: 0 0 auto`, shadcn's `basis-auto`). A released mouse drag coasts
/// without snapping; Previous/Next still page by slide.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 26rem; margin: 0 auto;",
            Carousel { aria_label: "Free scrolling gallery", drag_free: true,
                CarouselPrevious { ChevronLeft {} }
                CarouselNext { ChevronRight {} }
                CarouselContent {
                    for i in 0..8usize {
                        CarouselItem {
                            key: "{i}",
                            index: i,
                            style: format!("flex: 0 0 auto; width: {}rem;", 7 + (i % 3) * 3),
                            Card {
                                CardContent {
                                    style: "display: flex; align-items: center; justify-content: center; aspect-ratio: 1; font-size: var(--dx-text-3xl); font-weight: 600;",
                                    "{i + 1}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
