use super::super::component::*;
use dioxus::prelude::*;

// (title, description, cover gradient from, cover gradient to)
const MODELS: &[(&str, &str, &str, &str)] = &[
    (
        "Starter 1.5",
        "Fast and lightweight for everyday tasks.",
        "1e1b4b",
        "f472b6",
    ),
    (
        "Pro 1.5",
        "Balanced quality for longer, complex work.",
        "064e3b",
        "67e8f9",
    ),
];

/// A self-contained gradient banner (an inline SVG), so the demo needs no
/// network and hot-links no remote image.
fn cover(from: &str, to: &str) -> String {
    format!(
        "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 128 128'%3E%3Cdefs%3E%3ClinearGradient id='g' x1='0' y1='0' x2='1' y2='1'%3E%3Cstop offset='0' stop-color='%23{from}'/%3E%3Cstop offset='1' stop-color='%23{to}'/%3E%3C/linearGradient%3E%3C/defs%3E%3Crect width='128' height='128' fill='url(%23g)'/%3E%3C/svg%3E"
    )
}

#[component]
pub fn Demo() -> Element {
    rsx! {
        div {
            display: "grid",
            grid_template_columns: "repeat(2, minmax(0, 1fr))",
            gap: "1rem",
            width: "100%",
            max_width: "28rem",

            for (title , description , from , to) in MODELS.iter() {
                Item { variant: ItemVariant::Outline,
                    ItemHeader {
                        img {
                            src: cover(from, to),
                            alt: "{title}",
                            width: "128",
                            height: "128",
                            style: "aspect-ratio: 1; width: 100%; border-radius: var(--dx-radius-sm); object-fit: cover",
                        }
                    }
                    ItemContent {
                        ItemTitle { "{title}" }
                        ItemDescription { "{description}" }
                    }
                }
            }
        }
    }
}
