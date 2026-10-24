use crate::components::button::{Button, ButtonVariant};

use super::super::component::{
    Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
};
use dioxus::prelude::*;

// dev-docs/backlog.md row 32: no `#[css_module]` of its own here, and no
// `document::Link` needed either -- this `Demo` always renders the `Dialog`
// themed wrapper below, which carries its own
// `document::Link` for `style.css`. `document::Link` dedupes on `(href,
// rel)`, so the link the wrapper inserts covers this page.
#[component]
pub fn Demo() -> Element {
    let mut open = use_signal(|| false);
    // Nested dialog, additive to this demo so the tier-3 scroll-lock oracle
    // (playwright/oracle/tier3-radix/scroll-lock.spec.ts) has a fixture for
    // the refcounted-lock case: two modals open at once, closing the inner
    // one must leave the page still locked (dev-docs/plan.md, scroll lock).
    let mut nested_open = use_signal(|| false);

    rsx! {
        Button {
            r#type: "button",
            variant: ButtonVariant::Outline,
            onclick: move |_| open.set(true),
            "Show Dialog"
        }
        // The panel's own close button (`show_close_button`, default on) is the X at the top end.
        Dialog { open: open(), on_open_change: move |v| open.set(v),
            DialogHeader {
                DialogTitle { "Item information" }
                DialogDescription { "Here is some additional information about the item." }
            }
            Button {
                r#type: "button",
                variant: ButtonVariant::Outline,
                onclick: move |_| nested_open.set(true),
                "Open Nested Dialog"
            }
            // Rendered as a child of the outer Dialog's content -- not a
            // sibling -- so a still-open outer dialog's scroll lock isn't
            // released when this one closes (dev-docs/plan.md, scroll lock).
            // No corner X: its dismissal is a footer `DialogClose`, labelled so the
            // oracles can address it apart from the outer dialog's "Close".
            Dialog {
                open: nested_open(),
                on_open_change: move |v| nested_open.set(v),
                show_close_button: false,
                DialogHeader {
                    DialogTitle { "Nested dialog" }
                    DialogDescription { "A second, independently-opened dialog nested inside the first." }
                }
                DialogFooter {
                    DialogClose {
                        r#as: |attributes| rsx! {
                            Button { variant: ButtonVariant::Outline, attributes, "Close Nested" }
                        },
                    }
                }
            }
        }
    }
}
