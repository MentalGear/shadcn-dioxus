use super::super::component::*;
use dioxus::prelude::*;

#[component]
pub fn Demo() -> Element {
    rsx! {

        ToggleGroup { horizontal: true, allow_multiple_pressed: true,
            ToggleItem { index: 0usize, aria_label: "Toggle bold",
                b { "B" }
            }
            ToggleItem { index: 1usize, aria_label: "Toggle italic",
                i { "I" }
            }
            ToggleItem { index: 2usize, aria_label: "Toggle strikethrough",
                s { "S" }
            }
        }
    }
}
