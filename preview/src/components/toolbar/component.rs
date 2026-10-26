use dioxus::prelude::*;
use dioxus_primitives::toolbar::{self, ToolbarButtonProps, ToolbarProps, ToolbarSeparatorProps};
use dioxus_primitives::{dioxus_attributes::attributes, merge_attributes};

#[component]
pub fn Toolbar(props: ToolbarProps) -> Element {
    let base = attributes!(div {
        class: "dx-toolbar", "data-dx-root": "",
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/toolbar/style.css") }
        toolbar::Toolbar {
            attributes: merged,
            ..props
        }
    }
}

#[component]
pub fn ToolbarButton(props: ToolbarButtonProps) -> Element {
    let base = attributes!(button {
        class: "dx-toolbar-button",
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/toolbar/style.css") }
        toolbar::ToolbarButton {
            attributes: merged,
            ..props
        }
    }
}

#[component]
pub fn ToolbarSeparator(props: ToolbarSeparatorProps) -> Element {
    let base = attributes!(div {
        class: "dx-toolbar-separator",
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/toolbar/style.css") }
        toolbar::ToolbarSeparator {
            attributes: merged,
            ..props
        }
    }
}

#[component]
pub fn ToolbarGroup(
    #[props(extends = GlobalAttributes)]
    #[props(extends = div)]
    attributes: Vec<Attribute>,
    /// The toolbar buttons or controls in the group.
    children: Element,
) -> Element {
    let base = attributes!(div {
        class: "dx-toolbar-group",
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/toolbar/style.css") }
        div { ..merged, {children} }
    }
}
