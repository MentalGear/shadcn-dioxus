use dioxus::prelude::*;
use dioxus_icons::lucide::ChevronsUpDown;
use dioxus_primitives::collapsible::{
    self, CollapsibleContentProps, CollapsibleProps, CollapsibleTriggerProps,
};
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;

#[component]
pub fn Collapsible(props: CollapsibleProps) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/collapsible/style.css") }
        collapsible::Collapsible {
            ..props
        }
    }
}

#[component]
pub fn CollapsibleTrigger(props: CollapsibleTriggerProps) -> Element {
    let base = attributes!(button {
        class: "dx-collapsible-trigger",
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);

    let show_icon = props.r#as.is_none();

    let children = props.children.clone();
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/collapsible/style.css") }
        collapsible::CollapsibleTrigger {
            attributes: merged,
            ..props,
            {children}
            if show_icon {
                ChevronsUpDown {
                    size: "1rem",
                    stroke: "var(--dx-muted-foreground)",
                }
            }
        }
    }
}

#[component]
pub fn CollapsibleContent(props: CollapsibleContentProps) -> Element {
    let base = attributes!(div { class: "dx-collapsible-content" });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/collapsible/style.css") }
        collapsible::CollapsibleContent {
            attributes: merged,
            ..props
        }
    }
}

#[component]
pub fn CollapsibleItem(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    /// The item's trigger and content.
    children: Element,
) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/collapsible/style.css") }
        div {
            border: "1px solid var(--dx-border)",
            border_radius: "0.5rem",
            padding: "1rem",
            ..attributes,
            {children}
        }
    }
}

#[component]
pub fn CollapsibleList(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    /// The items, stacked in a column.
    children: Element,
) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/collapsible/style.css") }
        div {
            display: "flex",
            flex_direction: "column",
            gap: "0.5rem",
            max_width: "20rem",
            color: "var(--dx-foreground)",
            ..attributes,
            {children}
        }
    }
}
