use super::super::component::*;
use crate::components::spinner::Spinner;
use dioxus::prelude::*;
use dioxus_icons::lucide::{FileCode, X};

const WORKSPACE: Asset = asset!("/src/components/attachment/variants/image_gallery/workspace.svg");
const DESK: Asset = asset!("/src/components/attachment/variants/image_gallery/desk-reference.svg");
const OFFICE: Asset = asset!("/src/components/attachment/variants/image_gallery/office-reference.svg");

/// A composer's whole attachment tray in one place: a group of image attachments (vertical), a
/// file that is still uploading (a spinner as its media and a "Cancel upload" action), and a
/// finished code file with a remove action.
#[component]
pub fn Demo() -> Element {
    let mut uploading = use_signal(|| true);
    let mut code_file = use_signal(|| true);

    rsx! {
        div { style: "display: flex; width: 100%; max-width: 24rem; min-width: 0; flex-direction: column; gap: var(--dx-space-3);",
            AttachmentGroup {
                for (name , meta , alt , src) in [
                    ("workspace.svg", "SVG · 1 KB", "Workspace", WORKSPACE),
                    ("desk-reference.svg", "SVG · 1 KB", "Desk", DESK),
                    ("office-reference.svg", "SVG · 1 KB", "Office", OFFICE),
                ] {
                    Attachment { key: "{name}", orientation: AttachmentOrientation::Vertical,
                        AttachmentMedia { variant: AttachmentMediaVariant::Image,
                            img { src: "{src}", alt }
                        }
                        AttachmentContent {
                            AttachmentTitle { "{name}" }
                            AttachmentDescription { "{meta}" }
                        }
                    }
                }
            }
            if uploading() {
                Attachment { state: AttachmentState::Uploading, style: "width: 100%;",
                    AttachmentMedia {
                        Spinner { label: "Uploading sales-dashboard.pdf" }
                    }
                    AttachmentContent {
                        AttachmentTitle { "sales-dashboard.pdf" }
                        AttachmentDescription { "Uploading · 64%" }
                    }
                    AttachmentActions {
                        AttachmentAction {
                            aria_label: "Cancel upload",
                            onclick: move |_| uploading.set(false),
                            X {}
                        }
                    }
                }
            }
            if code_file() {
                Attachment { style: "width: 100%;",
                    AttachmentMedia {
                        FileCode {}
                    }
                    AttachmentContent {
                        AttachmentTitle { "message-renderer.rs" }
                        AttachmentDescription { "Rust · 12 KB" }
                    }
                    AttachmentActions {
                        AttachmentAction {
                            aria_label: "Remove message-renderer.rs",
                            onclick: move |_| code_file.set(false),
                            X {}
                        }
                    }
                }
            }
        }
    }
}
