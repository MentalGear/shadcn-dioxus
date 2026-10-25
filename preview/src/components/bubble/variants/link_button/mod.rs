use super::super::component::*;
use dioxus::prelude::*;
use dioxus_primitives::AsChild;

/// `as` on `BubbleContent` renders a real, focusable `button` or `a`.
#[component]
pub fn Demo() -> Element {
    let mut sent = use_signal(|| 0);
    rsx! {
        div { style: "display: flex; width: 100%; max-width: 28rem; flex-direction: column; gap: var(--dx-space-6);",
            Bubble { variant: BubbleVariant::Muted, align: BubbleAlign::End,
                BubbleContent {
                    r#as: move |el: AsChild| rsx! {
                        button { r#type: "button", onclick: move |_| sent += 1, ..el.attributes, {el.children} }
                    },
                    "I forgot my password"
                }
            }
            Bubble { variant: BubbleVariant::Outline,
                BubbleContent {
                    r#as: move |el: AsChild| rsx! {
                        a { href: "#", ..el.attributes, {el.children} }
                    },
                    "Open the reset guide"
                }
            }
            p { style: "margin: 0; font-size: var(--dx-text-sm); color: var(--dx-muted-foreground)",
                "Sent {sent} time(s)"
            }
        }
    }
}
