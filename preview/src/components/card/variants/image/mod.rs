use super::super::component::*;
use crate::components::badge::{Badge, BadgeVariant};
use crate::components::button::Button;
use dioxus::prelude::*;

// Bundled abstract illustration (`preview/assets/avatars`); the card's own CSS
// rounds a first-child `img`, and `object-fit: cover` crops it to the banner.
const EVENT_COVER: Asset = asset!("/assets/avatars/nina-hart.svg");

#[component]
pub fn Demo() -> Element {
    rsx! {
        Card { style: "width: 100%; max-width: 24rem;",
            img {
                src: EVENT_COVER,
                alt: "Event cover illustration",
                style: "width: 100%; aspect-ratio: 16 / 9; object-fit: cover;",
            }
            CardHeader {
                CardAction {
                    Badge { variant: BadgeVariant::Secondary, "Featured" }
                }
                CardTitle { "Design systems meetup" }
                CardDescription {
                    "A practical talk on keeping component libraries consistent as your team grows."
                }
            }
            CardFooter {
                Button { "View Event" }
            }
        }
    }
}
