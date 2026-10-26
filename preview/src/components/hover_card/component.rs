use dioxus::prelude::*;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::hover_card::{
    self, HoverCardContentProps, HoverCardProps, HoverCardTriggerProps,
};
use dioxus_primitives::merge_attributes;

#[component]
pub fn HoverCard(props: HoverCardProps) -> Element {
    let base = attributes!(div { class: "dx-hover-card" });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/hover_card/style.css") }
        hover_card::HoverCard {
            attributes: merged,
            ..props
        }
    }
}

#[component]
pub fn HoverCardTrigger(props: HoverCardTriggerProps) -> Element {
    let base = attributes!(button { class: "dx-hover-card-trigger" });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/hover_card/style.css") }
        hover_card::HoverCardTrigger {
            attributes: merged,
            ..props
        }
    }
}

#[component]
pub fn HoverCardContent(props: HoverCardContentProps) -> Element {
    let base = attributes!(div {
        class: "dx-hover-card-content",
        "data-dx-surface": "floating",
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/hover_card/style.css") }
        hover_card::HoverCardContent {
            attributes: merged,
            ..props
        }
    }
}
