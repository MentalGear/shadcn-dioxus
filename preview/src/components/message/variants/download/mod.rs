use super::super::component::*;
use crate::components::attachment::{
    Attachment, AttachmentAction, AttachmentActions, AttachmentContent, AttachmentDescription,
    AttachmentMedia, AttachmentMediaVariant, AttachmentOrientation, AttachmentTitle,
};
use crate::components::bubble::{Bubble, BubbleContent, BubbleVariant};
use crate::components::button::{ButtonSize, ButtonVariant};
use dioxus::prelude::*;
use dioxus_icons::lucide::{Download, FileText};

const WORKSPACE: Asset = asset!("/src/components/attachment/variants/image_gallery/workspace.svg");

/// A picture goes out with the request and a document comes back. The reply's attachment carries
/// a secondary "Download" action instead of a remove action.
#[component]
pub fn Demo() -> Element {
    let mut downloads = use_signal(|| 0u32);

    rsx! {
        div { style: "display: flex; width: 100%; max-width: 24rem; min-width: 0; flex-direction: column; gap: var(--dx-space-8);",
            Message { align: MessageAlign::End,
                MessageContent {
                    Attachment { orientation: AttachmentOrientation::Vertical,
                        AttachmentMedia { variant: AttachmentMediaVariant::Image,
                            img { src: "{WORKSPACE}", alt: "Workspace" }
                        }
                    }
                    Bubble {
                        BubbleContent {
                            "Here's the image. Can you add it to the PDF? Use it for the cover page."
                        }
                    }
                }
            }
            Message {
                MessageContent {
                    Bubble { variant: BubbleVariant::Muted,
                        BubbleContent { "Done. Here's the PDF with the image added as the cover page." }
                    }
                    Attachment {
                        AttachmentMedia {
                            FileText {}
                        }
                        AttachmentContent {
                            AttachmentTitle { "sales-dashboard.pdf" }
                            AttachmentDescription { "PDF · 2.4 MB" }
                        }
                        AttachmentActions {
                            AttachmentAction {
                                variant: ButtonVariant::Secondary,
                                size: ButtonSize::IconSm,
                                r#type: "button",
                                title: "Download",
                                aria_label: "Download sales-dashboard.pdf",
                                onclick: move |_| downloads += 1,
                                Download {}
                            }
                        }
                    }
                }
            }
            Message { align: MessageAlign::End,
                MessageContent {
                    Bubble {
                        BubbleContent { "Thanks. Looks good." }
                    }
                }
            }
            p { role: "status", style: "margin: 0; font-size: var(--dx-text-sm); color: var(--dx-muted-foreground)",
                if downloads() > 0 {
                    "Downloaded sales-dashboard.pdf ({downloads()}x)"
                }
            }
        }
    }
}
