use super::super::component::*;
use crate::components::card::{Card, CardContent};
use dioxus::prelude::*;
use dioxus_icons::lucide::{ChevronDown, ChevronUp};

/// `CarouselOrientation::Vertical` is the horizontal construction on the
/// other axis (see `primitives/src/carousel.rs`'s module doc). A vertical
/// carousel needs an explicit height on `CarouselContent` -- there is
/// nothing else to derive one from, the same way `ScrollArea` needs one.
///
/// Unlike the horizontal variants (`main`/`indicators`/`rtl`/`multiple`),
/// this wrapper needs no compensating size bump for `.dx-carousel`'s own
/// arrow-reservation padding (style.css). That reservation
/// costs a horizontal wrapper track width because `CarouselContent` has no
/// explicit width of its own there -- it fills whatever's left of
/// `.dx-carousel`'s fixed `width: 100%` after `padding-inline` is carved
/// out of it (`box-sizing: border-box`). Here `CarouselContent` instead
/// gets an explicit, literal `height: 12rem` below, wired to nothing else
/// -- so the reservation's `padding-block` has no fixed budget to be
/// carved out of; `.dx-carousel` (a plain block box, no explicit height)
/// just grows 6rem taller to fit both, and the visible slide stays 12rem.
/// The slide stays 192px across every viewport and theme, with or without
/// the reservation.
///
/// This wrapper carries no `padding-block` of its own: Previous/Next sit
/// *inside* `.dx-carousel`'s own (self-expanding) box, so nothing here needs
/// external headroom for them, and no button straddles
/// `.dx-component-preview-frame`'s border (style.css's vertical
/// `padding-block` rule says the same).
///
/// Sizing follows shadcn's own `carousel-orientation.tsx` (two slides per
/// view, not a single slide filling the box): `--dx-carousel-per-view: 2`
/// (`basis-1/2`) against a `16.875rem` (270px) `CarouselContent` height --
/// so each of the 2 visible slides gets half that height, split across the
/// block axis, rather than one slide filling the whole box. Deliberately
/// no `aspect-ratio` on the slide's own `CardContent` here, unlike every
/// other demo: shadcn's own orientation example has none either (height
/// comes from `CarouselContent`'s explicit height, not from the slide's
/// own aspect ratio).
///
/// **The `Card` must be `box-sizing: border-box`** -- and is, from the
/// theme (`dx-components-theme.css`'s box-model rule), not from this demo.
/// `Card` (`.dx-card`) carries `padding: 24px 0` plus a 1px border, so
/// under the browser's `content-box` default `height: 100%` means "the
/// slide's usable block size *plus* 50px of padding/border on top".
/// Without it each item is 135px (half of the 270px content box), 119px of it
/// usable after the 16px leading gap padding, and the Card would render
/// taller than that (119 + the padding and border) -- card 2
/// overlaps card 1 and the bottom card runs past the clip viewport. `border-box` makes the
/// Card exactly the slide's usable size; `CardContent` then takes the Card's
/// remaining height with `flex: 1` (the Card is a flex column) instead of a
/// second, again-overflowing `height: 100%`. The gap is the default
/// spacing (`--dx-space-4`, 16px), the same as the horizontal demos, so the
/// space between the two visible cards reads the same on both axes.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "width: 100%; max-width: 12rem; margin: 0 auto;",
            Carousel {
                aria_label: "Vertical scrolling demo",
                orientation: CarouselOrientation::Vertical,
                CarouselPrevious { ChevronUp {} }
                CarouselNext { ChevronDown {} }
                CarouselContent { style: "height: 16.875rem; --dx-carousel-per-view: 2;",
                    for i in 0..4usize {
                        CarouselItem { key: "{i}", index: i,
                            Card { style: "height: 100%;",
                                CardContent {
                                    style: "display: flex; flex: 1; min-height: 0; align-items: center; justify-content: center; font-size: var(--dx-text-3xl); font-weight: 600;",
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
