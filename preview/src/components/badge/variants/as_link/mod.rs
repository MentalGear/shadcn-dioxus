use super::super::component::*;
use dioxus::prelude::*;
use dioxus_primitives::AsChild;
use dioxus_icons::lucide::ArrowUpRight;

/// `as` renders the badge as a real link: the anchor takes the badge's
/// attributes, so it keeps the badge's look and the link's role.
#[component]
pub fn Demo() -> Element {
    rsx! {
        Badge {
            r#as: move |el: AsChild| rsx! {
                a { href: "#link", ..el.attributes,
                    {el.children}
                }
            },
            "Open Link"
            ArrowUpRight {}
        }
    }
}
