use dioxus::prelude::*;
use dioxus_primitives::separator::{self, SeparatorProps};
use dioxus_primitives::{dioxus_attributes::attributes, merge_attributes};

#[component]
pub fn Separator(props: SeparatorProps) -> Element {
    let base = attributes!(div {
        class: "dx-separator", "data-dx-root": "",
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/separator/style.css") }
        separator::Separator {
            attributes: merged,
            ..props
        }
    }
}
