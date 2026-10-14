use super::super::component::*;
use dioxus::prelude::*;
use dioxus_icons::lucide::ArrowUpRight;

/// `as` renders the badge as a real link: the anchor takes the badge's
/// attributes, so it keeps the badge's look and the link's role.
#[component]
pub fn Demo() -> Element {
    rsx! {
        Badge {
            r#as: move |attrs: Vec<Attribute>| rsx! {
                a { href: "#link", ..attrs,
                    "Open Link"
                    ArrowUpRight {}
                }
            },
        }
    }
}
