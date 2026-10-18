// shadcn's base-nova `button-demo`: one outline button and an icon-only one
// (ui.shadcn.com/docs/components/base/button). The six variants moved to the
// `all_variants` demo below it.
use super::super::component::*;
use dioxus::prelude::*;
use dioxus_icons::lucide::ArrowUp;

#[component]
pub fn Demo() -> Element {
    rsx! {
        div {
            display: "flex",
            flex_direction: "row",
            flex_wrap: "wrap",
            align_items: "center",
            gap: "0.5rem",

            Button { variant: ButtonVariant::Outline, "Button" }

            // An icon-only button has no text, so it needs an accessible name (axe `button-name`).
            Button {
                variant: ButtonVariant::Outline,
                size: ButtonSize::Icon,
                aria_label: "Submit",
                ArrowUp { size: "16px" }
            }
        }
    }
}
