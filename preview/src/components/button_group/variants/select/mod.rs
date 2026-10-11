use super::super::component::*;
use crate::components::button::{Button, ButtonVariant};
use crate::components::input::Input;
use crate::components::native_select::NativeSelect;
use dioxus::prelude::*;
use dioxus_icons::lucide::ArrowRight;

/// shadcn's "Select" example: a currency select, an amount `Input` and a
/// submit-style button in one group.
///
/// The select is a `NativeSelect`, not the custom-listbox `Select`: a `Select`
/// has no options (so no selected label, no width) until it hydrates, which
/// makes the route's JS-off page differ from its hydrated one
/// (`oracle/tier2-html/first-paint-styles.spec.ts` allowlists only the routes
/// that are genuinely client-measured), while a native `<select>` is complete in
/// the static HTML. Its wrapper `<div>` is the group's child, so it takes the
/// seam through the `--dx-group-radius-*` variables (see `button_group/docs.md`).
#[component]
pub fn Demo() -> Element {
    let currencies = [
        ("USD", "$ US Dollar"),
        ("EUR", "EUR Euro"),
        ("GBP", "GBP British Pound"),
    ];
    rsx! {
        ButtonGroup { "aria-label": "Send money", style: "width: 100%;",
            div { style: "width: 9rem;",
                NativeSelect { "aria-label": "Currency", name: "currency",
                    for (code , label) in currencies {
                        option { value: code, selected: code == "USD", "{label}" }
                    }
                }
            }
            Input {
                id: "button-group-select-amount",
                placeholder: "10.00",
                "aria-label": "Amount",
                style: "flex: 1;",
            }
            Button { variant: ButtonVariant::Outline, "aria-label": "Send", ArrowRight {} }
        }
    }
}
