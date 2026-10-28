use super::super::component::*;
use dioxus::prelude::*;

/// shadcn's "With Radio Group" example: two radio groups ("People" and
/// "Theme"), each with its own checked item, controlled (`value` +
/// `on_value_change`) and keeping the menu open after a choice
/// (the default) so the checked item can be seen to move. Each
/// group is named by its heading (`aria-labelledby`). A radio item falls
/// back to its `value` as its typeahead label.
///
/// Labels ("Right click for radio group"/"Pedro Duarte"/...) differ from
/// `main`'s and `rtl`'s on purpose -- see the sibling `checkboxes` variant.
#[component]
pub fn Demo() -> Element {
    let mut person = use_signal(|| "pedro".to_string());
    let mut theme = use_signal(|| "light".to_string());

    rsx! {
        ContextMenu {
            ContextMenuTrigger { "Right click for radio group" }
            ContextMenuContent {
                ContextMenuLabel { id: "dx-cm-people-label", "People" }
                ContextMenuRadioGroup {
                    aria_labelledby: "dx-cm-people-label",
                    value: Some(person()),
                    on_value_change: move |value| person.set(value),
                    ContextMenuRadioItem {
                        value: "pedro".to_string(),
                        "Pedro Duarte"
                    }
                    ContextMenuRadioItem {
                        value: "colm".to_string(),
                        "Colm Tuite"
                    }
                }
                ContextMenuLabel { id: "dx-cm-theme-label", "Theme" }
                ContextMenuRadioGroup {
                    aria_labelledby: "dx-cm-theme-label",
                    value: Some(theme()),
                    on_value_change: move |value| theme.set(value),
                    ContextMenuRadioItem {
                        value: "light".to_string(),
                        "Light"
                    }
                    ContextMenuRadioItem {
                        value: "dark".to_string(),
                        "Dark"
                    }
                    ContextMenuRadioItem {
                        value: "system".to_string(),
                        "System"
                    }
                }
            }
        }
    }
}
