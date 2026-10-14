use super::super::component::*;
use dioxus::prelude::*;

/// `as` renders the button as a link. The anchor takes the button's attributes
/// and handlers, so the click count below moves when the link is clicked.
#[component]
pub fn Demo() -> Element {
    let mut clicks = use_signal(|| 0);
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: var(--dx-space-3); align-items: flex-start;",
            Button {
                variant: ButtonVariant::Secondary,
                size: ButtonSize::Sm,
                onclick: move |_| clicks += 1,
                r#as: move |attrs: Vec<Attribute>| rsx! {
                    a { href: "#", ..attrs, "Login" }
                },
            }
            p { style: "margin: 0; font-size: var(--dx-text-sm); color: var(--dx-muted-foreground)",
                "Clicked {clicks} time(s)"
            }
        }
    }
}
