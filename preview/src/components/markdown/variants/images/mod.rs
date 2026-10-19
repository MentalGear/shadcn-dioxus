use super::super::component::*;
use dioxus::prelude::*;

// With images off (the default) nothing loads and the alt text stays; `allow_images` loads same-origin and
// web images only (a `javascript:` source never does).
#[component]
pub fn Demo() -> Element {
    let mut allow = use_signal(|| false);
    let logo: Asset = asset!("/assets/favicon.svg");
    let source = format!(
        "A same-origin image: ![the shadcn-dioxus logo]({logo})\n\nA script source is never an image: ![a script image](javascript:alert(1))"
    );
    rsx! {
        div { style: "display: flex; width: 100%; max-width: 36rem; flex-direction: column; gap: var(--dx-space-3);",
            label { style: "display: inline-flex; align-items: center; gap: var(--dx-space-2); font-size: var(--dx-text-sm);",
                input {
                    r#type: "checkbox",
                    checked: allow(),
                    onchange: move |event| allow.set(event.checked()),
                }
                "allow_images"
            }
            Markdown { source, allow_images: allow() }
        }
    }
}
