use super::super::component::*;
use crate::components::dialog::{Dialog, DialogDescription, DialogTitle};
use dioxus::prelude::*;
use dioxus_icons::lucide::{Copy, FileSearch, X};

/// The trigger fills the card and opens a real `Dialog`; the copy and remove actions stay
/// independently clickable above it. Opening is a plain controlled `open` signal: the trigger sets
/// it and the dialog's own dismissal (Escape, outside click, close button) clears it.
#[component]
pub fn Demo() -> Element {
    let mut open = use_signal(|| false);
    let mut removed = use_signal(|| false);
    let mut copied = use_signal(|| false);

    rsx! {
        div { style: "display: flex; width: 100%; max-width: 24rem; flex-direction: column; align-items: stretch; gap: var(--dx-space-3);",
            if !removed() {
                Attachment { style: "width: 100%;",
                    AttachmentMedia {
                        FileSearch {}
                    }
                    AttachmentContent {
                        AttachmentTitle { "research-summary.pdf" }
                        AttachmentDescription { "Open preview dialog" }
                    }
                    AttachmentActions {
                        AttachmentAction {
                            aria_label: "Copy link to research-summary.pdf",
                            onclick: move |_| copied.set(true),
                            Copy {}
                        }
                        AttachmentAction {
                            aria_label: "Remove research-summary.pdf",
                            onclick: move |_| removed.set(true),
                            X {}
                        }
                    }
                    AttachmentTrigger {
                        aria_label: "Preview research-summary.pdf",
                        "aria-haspopup": "dialog",
                        onclick: move |_| open.set(true),
                    }
                }
            }
            p { role: "status", style: "margin: 0; font-size: var(--dx-text-sm); color: var(--dx-muted-foreground)",
                if copied() {
                    "Link copied"
                } else if removed() {
                    "research-summary.pdf removed"
                }
            }
        }
        Dialog { open: open(), on_open_change: move |v| open.set(v),
            button {
                class: "dx-dialog-close",
                r#type: "button",
                aria_label: "Close",
                tabindex: if open() { "0" } else { "-1" },
                onclick: move |_| open.set(false),
                "×"
            }
            DialogTitle { "research-summary.pdf" }
            DialogDescription {
                "The attachment trigger fills the card and opens the dialog, while the actions stay independently clickable above it."
            }
        }
    }
}
