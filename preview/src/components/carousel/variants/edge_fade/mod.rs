use super::super::component::*;
use crate::components::card::{Card, CardContent};
use dioxus::prelude::*;
use dioxus_icons::lucide::{ChevronLeft, ChevronRight};

/// `class: "dx-carousel-fade"` on the root: the slides fade out toward the
/// track's edges to show there is more to scroll, and each fade is gone once
/// the track is at that end. Pure CSS (a scroll-driven mask on the stationary
/// viewport, see style.css), so it holds under RTL and vertical layouts, and
/// `--dx-carousel-fade-size` sets the fade length.
///
/// Three slides per view on a wide track so a fading neighbour is visible on
/// both sides mid-scroll. One carousel on purpose (every variant renders on
/// the same page).
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 30rem; margin: 0 auto; --dx-carousel-per-view: 3;",
            Carousel { aria_label: "Gallery with edge fade", class: "dx-carousel-fade",
                CarouselPrevious { ChevronLeft {} }
                CarouselNext { ChevronRight {} }
                CarouselContent {
                    for i in 0..8usize {
                        CarouselItem { key: "{i}", index: i,
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
