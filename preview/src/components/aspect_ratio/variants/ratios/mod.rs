use super::super::component::AspectRatio;
use dioxus::prelude::*;

const COVER: Asset = asset!("/assets/avatars/lena-park.svg");

fn ratio_box(ratio: f64, label: &'static str) -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 0.5rem; width: 100%; max-width: 24rem;",
            span { style: "font-size: var(--dx-text-sm); color: var(--dx-muted-foreground);", "{label}" }
            div { class: "dx-aspect-ratio-container",
                AspectRatio { ratio,
                    img {
                        class: "dx-aspect-ratio-image",
                        src: COVER,
                        alt: "Abstract cover illustration",
                    }
                }
            }
        }
    }
}

#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 1.5rem; width: 100%; max-width: 24rem;",
            {ratio_box(1.0, "1:1")}
            {ratio_box(4.0 / 3.0, "4:3")}
            {ratio_box(16.0 / 9.0, "16:9")}
        }
    }
}
