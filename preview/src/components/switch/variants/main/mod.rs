use super::super::component::*;
use crate::components::field::{Field, FieldLabel, FieldOrientation};
use dioxus::prelude::*;

// dev-docs/backlog.md row 32: no `#[css_module]` of its own here, and no
// `document::Link` needed either -- this `Demo` always renders the `Switch`
// themed wrapper below, which carries its own
// `document::Link` for `style.css`. `document::Link` dedupes on `(href,
// rel)`, so the link the wrapper inserts covers this page.
#[component]
pub fn Demo() -> Element {
    let mut checked = use_signal(|| false);
    rsx! {
        div { class: "dx-switch-example",
            Field { orientation: FieldOrientation::Horizontal,
                Switch {
                    id: "switch-main-airplane-mode",
                    checked: checked(),
                    on_checked_change: move |new_checked| {
                        checked.set(new_checked);
                    },
                }
                FieldLabel { html_for: "switch-main-airplane-mode", "Airplane Mode" }
            }
        }
    }
}
