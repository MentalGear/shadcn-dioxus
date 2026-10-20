use super::super::component::*;
use crate::components::avatar::{AvatarImageSize, ImageAvatar};
use dioxus::prelude::*;

const AVERY_LIN: Asset = asset!("/assets/avatars/avery-lin.svg");
const CASEY_PARK: Asset = asset!("/assets/avatars/casey-park.svg");
const PRIYA_NAIR: Asset = asset!("/assets/avatars/priya-nair.svg");
const MARCUS_WRIGHT: Asset = asset!("/assets/avatars/marcus-wright.svg");

struct Teammate {
    value: &'static str,
    name: &'static str,
    email: &'static str,
    avatar: Asset,
}

/// Options with an avatar and a second line of muted text. `ComboboxOption`'s
/// children are free-form, so this needs no extra part: `text_value` stays the
/// plain name (it is what the input shows once chosen and what the filter
/// matches), while the children draw the richer row.
///
/// Distinct from `main` for the page-wide locators in `combobox.spec.ts`
/// (`getByRole("combobox", { name: "Select framework" })` is a substring match):
/// the accessible name here is "Assign teammate", and no option reuses a
/// framework name.
#[component]
pub fn Demo() -> Element {
    let teammates = [
        Teammate {
            value: "avery",
            name: "Avery Lin",
            email: "avery.lin@example.com",
            avatar: AVERY_LIN,
        },
        Teammate {
            value: "casey",
            name: "Casey Park",
            email: "casey.park@example.com",
            avatar: CASEY_PARK,
        },
        Teammate {
            value: "priya",
            name: "Priya Nair",
            email: "priya.nair@example.com",
            avatar: PRIYA_NAIR,
        },
        Teammate {
            value: "marcus",
            name: "Marcus Wright",
            email: "marcus.wright@example.com",
            avatar: MARCUS_WRIGHT,
        },
    ];

    let names: Vec<(&'static str, &'static str)> =
        teammates.iter().map(|teammate| (teammate.value, teammate.name)).collect();

    rsx! {
        Combobox::<String> {
            item_to_label: move |value: String| {
                names
                    .iter()
                    .find(|(id, _)| *id == value)
                    .map_or(value.clone(), |(_, name)| name.to_string())
            },
            // Two-line rows with an avatar and an email are wider than the stock 200px
            // input/list, so the email wrapped mid-word; widen both through the slot variable.
            style: "--dx-combobox-width: 18rem;",
            placeholder: "Assign a teammate...",
            aria_label: "Assign teammate",
            list_aria_label: "Teammates",
            ComboboxEmpty { "No teammate found." }
            for (i , teammate) in teammates.iter().enumerate() {
                ComboboxOption::<String> {
                    index: i,
                    value: teammate.value.to_string(),
                    text_value: teammate.name.to_string(),
                    // Decorative: the name and email beside it already say who this is.
                    ImageAvatar {
                        size: AvatarImageSize::Small,
                        src: teammate.avatar.to_string(),
                        alt: "",
                        aria_hidden: "true",
                        ""
                    }
                    div { style: "display: flex; flex-direction: column; min-width: 0;",
                        span { "{teammate.name}" }
                        span { style: "color: var(--dx-muted-foreground); font-size: var(--dx-text-xs);",
                            "{teammate.email}"
                        }
                    }
                }
            }
        }
    }
}
