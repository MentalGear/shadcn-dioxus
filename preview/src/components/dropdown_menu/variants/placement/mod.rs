use super::super::component::{
    DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger,
};
use dioxus::prelude::*;
use dioxus_primitives::{ContentAlign, ContentSide};

/// `side`, `align` and `side_offset` on `DropdownMenuContent`: each trigger opens its menu on a
/// different side of itself. A side without room flips to the opposite one.
///
/// `dropdown-menu.spec.ts` queries "Top"/"Right"/"Bottom" as `menuitemradio` names on this page;
/// these triggers are buttons, so their names never match those queries.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-wrap: wrap; gap: 0.5rem;",
            PlacedMenu { label: "Opens above", side: ContentSide::Top, align: ContentAlign::Start }
            PlacedMenu {
                label: "Opens on the right",
                side: ContentSide::Right,
                align: ContentAlign::Start,
            }
            PlacedMenu {
                label: "Opens below, end-aligned",
                side: ContentSide::Bottom,
                align: ContentAlign::End,
                side_offset: 12,
            }
            PlacedMenu {
                label: "Opens on the left",
                side: ContentSide::Left,
                align: ContentAlign::Center,
            }
        }
    }
}

#[component]
fn PlacedMenu(
    label: String,
    side: ContentSide,
    align: ContentAlign,
    #[props(default = 4)] side_offset: u32,
) -> Element {
    rsx! {
        DropdownMenu {
            DropdownMenuTrigger { "{label}" }
            DropdownMenuContent { side, align, side_offset,
                for item in ["Archive", "Mute thread", "Report"] {
                    DropdownMenuItem::<String> {
                        value: item.to_string(),
                        text_value: item,
                        "{item}"
                    }
                }
            }
        }
    }
}
