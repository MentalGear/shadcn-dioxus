use crate::components::{
    button::{Button, ButtonVariant},
    dialog::{Dialog, DialogDescription, DialogHeader, DialogTitle},
    drawer::{
        Drawer, DrawerClose, DrawerContent, DrawerDescription, DrawerFooter, DrawerHeader,
        DrawerTitle,
    },
    field::{Field, FieldGroup},
    form::DemoForm,
    input::Input,
    label::Label,
    sidebar::use_is_mobile,
};
use dioxus::prelude::*;
use dioxus_primitives::AsChild;

/// shadcn's "Responsive Dialog": a `Dialog` on a desktop viewport and a `Drawer` below the mobile
/// breakpoint (`use_is_mobile`, the Sidebar's hook), with the same form in both.
#[component]
pub fn Demo() -> Element {
    let mut open = use_signal(|| false);
    let is_mobile = use_is_mobile();

    rsx! {
        Button {
            r#type: "button",
            variant: ButtonVariant::Outline,
            onclick: move |_| open.set(true),
            "Edit Profile"
        }
        if is_mobile() {
            Drawer { open: open(), on_open_change: move |v| open.set(v),
                DrawerContent {
                    DrawerHeader {
                        DrawerTitle { "Edit profile" }
                        DrawerDescription {
                            "Make changes to your profile here. Click save when you're done."
                        }
                    }
                    ProfileForm { on_save: move |_| open.set(false) }
                    DrawerFooter {
                        DrawerClose {
                            as: |el: AsChild| rsx! {
                                Button { variant: ButtonVariant::Outline, attributes: el.attributes, {el.children} }
                            },
                            "Cancel"
                        }
                    }
                }
            }
        } else {
            Dialog { open: open(), on_open_change: move |v| open.set(v),
                DialogHeader {
                    DialogTitle { "Edit profile" }
                    DialogDescription {
                        "Make changes to your profile here. Click save when you're done."
                    }
                }
                ProfileForm { on_save: move |_| open.set(false) }
            }
        }
    }
}

#[component]
fn ProfileForm(on_save: EventHandler<()>) -> Element {
    rsx! {
        DemoForm {
            show_status: false,
            onsubmit: move |_| on_save.call(()),
            style: "display: flex; flex-direction: column; gap: var(--dx-space-4);",
            FieldGroup {
                Field {
                    Label { html_for: "responsive-email", "Email" }
                    Input {
                        id: "responsive-email",
                        name: "email",
                        r#type: "email",
                        initial_value: "shadcn@example.com",
                    }
                }
                Field {
                    Label { html_for: "responsive-username", "Username" }
                    Input {
                        id: "responsive-username",
                        name: "username",
                        initial_value: "@shadcn",
                    }
                }
            }
            Button { r#type: "submit", "Save changes" }
        }
    }
}
