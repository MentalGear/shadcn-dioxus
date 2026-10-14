use super::super::component::*;
use dioxus::prelude::*;
use dioxus_icons::lucide::{FileCode, FileText, Table, X};

const WORKSPACE: Asset = asset!("/src/components/attachment/variants/image_gallery/workspace.svg");

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Pdf,
    Image,
    Csv,
    Code,
}

const ITEMS: [(&str, &str, Kind); 4] = [
    ("briefing-notes.pdf", "PDF · 1.4 MB", Kind::Pdf),
    ("workspace.svg", "SVG · 1 KB", Kind::Image),
    ("customers.csv", "CSV · 18 KB", Kind::Csv),
    ("renderer.rs", "Rust · 12 KB", Kind::Code),
];

/// A scrolling group of fixed-width cards with a working remove action on each: icon media for
/// files, image media for pictures.
#[component]
pub fn Demo() -> Element {
    let mut removed = use_signal(Vec::<&'static str>::new);
    let remaining = ITEMS.len() - removed.read().len();

    rsx! {
        div { style: "display: flex; width: 100%; max-width: 24rem; min-width: 0; flex-direction: column; gap: var(--dx-space-3);",
            AttachmentGroup { style: "width: 100%;",
                for (name , meta , kind) in ITEMS.into_iter().filter(|(name, ..)| !removed.read().contains(name)) {
                    Attachment { key: "{name}", style: "width: 16rem;",
                        match kind {
                            Kind::Image => rsx! {
                                AttachmentMedia { variant: AttachmentMediaVariant::Image,
                                    img { src: "{WORKSPACE}", alt: "{name}" }
                                }
                            },
                            Kind::Pdf => rsx! {
                                AttachmentMedia {
                                    FileText {}
                                }
                            },
                            Kind::Csv => rsx! {
                                AttachmentMedia {
                                    Table {}
                                }
                            },
                            Kind::Code => rsx! {
                                AttachmentMedia {
                                    FileCode {}
                                }
                            },
                        }
                        AttachmentContent {
                            AttachmentTitle { "{name}" }
                            AttachmentDescription { "{meta}" }
                        }
                        AttachmentActions {
                            AttachmentAction {
                                aria_label: "Remove {name}",
                                onclick: move |_| removed.write().push(name),
                                X {}
                            }
                        }
                    }
                }
            }
            p { role: "status", style: "margin: 0; font-size: var(--dx-text-sm); color: var(--dx-muted-foreground)",
                "{remaining} of {ITEMS.len()} attachments"
            }
        }
    }
}
