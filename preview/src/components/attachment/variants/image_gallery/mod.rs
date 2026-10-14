use super::super::component::*;
use dioxus::prelude::*;
use dioxus_icons::lucide::X;

const WORKSPACE: Asset = asset!("/src/components/attachment/variants/image_gallery/workspace.svg");
const DESK: Asset = asset!("/src/components/attachment/variants/image_gallery/desk-reference.svg");
const OFFICE: Asset = asset!("/src/components/attachment/variants/image_gallery/office-reference.svg");

/// `(name, meta, alt, source)`. The previews are bundled with `asset!`, so the demo needs no network.
const IMAGES: [(&str, &str, &str, Asset); 3] = [
    ("workspace.svg", "SVG · 1 KB", "Workspace", WORKSPACE),
    ("desk-reference.svg", "SVG · 1 KB", "Desk", DESK),
    ("office-reference.svg", "SVG · 1 KB", "Office", OFFICE),
];

/// Each image card is a vertical attachment. Its `AttachmentTrigger` renders as an `<a>` (through
/// `as`) that opens the full image, and the remove action sits above it.
#[component]
pub fn Demo() -> Element {
    let mut removed = use_signal(Vec::<&'static str>::new);

    rsx! {
        div { style: "width: 100%; max-width: 24rem; min-width: 0;",
            AttachmentGroup { style: "width: 100%;",
                for (name , meta , alt , src) in IMAGES.into_iter().filter(|(name, ..)| !removed.read().contains(name)) {
                    Attachment { key: "{name}", orientation: AttachmentOrientation::Vertical,
                        AttachmentMedia { variant: AttachmentMediaVariant::Image,
                            img { src: "{src}", alt }
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
                        AttachmentTrigger {
                            r#as: move |attrs: Vec<Attribute>| rsx! {
                                a {
                                    href: "{src}",
                                    target: "_blank",
                                    rel: "noreferrer",
                                    "aria-label": "Open {name}",
                                    ..attrs,
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}
