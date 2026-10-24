use super::super::component::*;
use crate::components::card::{Card, CardContent};
use dioxus::prelude::*;

/// The APG "tabbed" carousel style: a `CarouselIndicators` of dot pickers
/// (`role="tablist"`, roving tabindex, automatic activation on
/// arrow/Home/End) in place of `CarouselPrevious`/`CarouselNext`, matching
/// the vendored reference's own structure (`carousel-2-tablist.html`) --
/// no separate Previous/Next buttons. `CarouselItem` renders
/// `role="tabpanel"` (instead of `group`) once a `CarouselIndicators` is
/// present. See `CarouselIndicators`' own doc for why this composes
/// `crate::collection` directly rather than `Tabs`/`TabTrigger` (manual
/// activation there is the wrong contract).
///
/// This is the one "Indicators" demo. A plain custom `use_carousel()`-built
/// dot picker is unrelated to this tablist component despite the shared
/// name; it lives in the `api` demo.
#[component]
pub fn Demo() -> Element {
    rsx! {
        // See `variants/main/mod.rs`'s own comment for the `26rem` derivation.
        div { style: "width: 100%; max-width: 26rem; margin: 0 auto;",
            Carousel { aria_label: "Tabbed slide gallery",
                CarouselIndicators {
                    for i in 0..5usize {
                        CarouselIndicator { key: "{i}", index: i }
                    }
                }
                CarouselContent {
                    for i in 0..5usize {
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
