use crate::components::button::{Button, ButtonVariant};
use crate::components::field::{Field, FieldGroup};
use crate::components::form::DemoForm;
use crate::components::input::Input;
use crate::components::label::Label;

use super::super::component::{
    Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
};
use dioxus::prelude::*;
use dioxus_primitives::AsChild;

/// shadcn's `dialog-demo`: a form in a `DialogHeader` / fields / `DialogFooter` panel. The X is
/// the last control in DOM order, so a keyboard open focuses the first field ("Name"). The form
/// is the panel's single child, so it carries the panel's column and gap itself.
#[component]
pub fn Demo() -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        Button {
            r#type: "button",
            variant: ButtonVariant::Outline,
            onclick: move |_| open.set(true),
            "Open Dialog"
        }
        Dialog { open: open(), on_open_change: move |v| open.set(v),
            DemoForm {
                show_status: false,
                onsubmit: move |_| open.set(false),
                style: "display: flex; flex-direction: column; gap: var(--dx-space-4);",
                DialogHeader {
                    DialogTitle { "Edit profile" }
                    DialogDescription {
                        "Make changes to your profile here. Click save when you're done."
                    }
                }
                FieldGroup {
                    Field {
                        Label { html_for: "dialog-edit-name", "Name" }
                        Input {
                            id: "dialog-edit-name",
                            name: "name",
                            initial_value: "Pedro Duarte",
                        }
                    }
                    Field {
                        Label { html_for: "dialog-edit-username", "Username" }
                        Input {
                            id: "dialog-edit-username",
                            name: "username",
                            initial_value: "@peduarte",
                        }
                    }
                }
                DialogFooter {
                    DialogClose {
                        r#as: |el: AsChild| rsx! {
                            Button { variant: ButtonVariant::Outline, attributes: el.attributes, {el.children} }
                        },
                        "Cancel"
                    }
                    Button { r#type: "submit", "Save changes" }
                }
            }
        }
    }
}
