use dioxus::prelude::*;
use dioxus_icons::lucide::ChevronDown;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;
use dioxus_primitives::navbar::{
    self, NavbarContentProps, NavbarItemProps, NavbarNavProps, NavbarProps, NavbarTriggerProps,
};

#[component]
pub fn Navbar(props: NavbarProps) -> Element {
    let base = attributes!(nav { class: "dx-navbar", "data-dx-root": "" });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/navbar/style.css") }
        navbar::Navbar {
            attributes: merged,
            ..props
        }
    }
}

#[component]
pub fn NavbarNav(props: NavbarNavProps) -> Element {
    let base = attributes!(div { class: "dx-navbar-nav" });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/navbar/style.css") }
        navbar::NavbarNav {
            attributes: merged,
            ..props
        }
    }
}

#[component]
pub fn NavbarTrigger(props: NavbarTriggerProps) -> Element {
    let base = attributes!(button { class: "dx-navbar-trigger" });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    let children = props.children.clone();
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/navbar/style.css") }
        navbar::NavbarTrigger {
            attributes: merged,
            ..props,
            {children}
            ChevronDown {
                class: "dx-navbar-expand-icon",
                size: "20px",
                stroke: "var(--dx-foreground)",
            }
        }
    }
}

#[component]
pub fn NavbarContent(props: NavbarContentProps) -> Element {
    let base = attributes!(div {
        class: "dx-navbar-content",
        "data-dx-surface": "floating",
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/navbar/style.css") }
        navbar::NavbarContent {
            attributes: merged,
            ..props
        }
    }
}

#[component]
pub fn NavbarItem(props: NavbarItemProps) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/navbar/style.css") }
        navbar::NavbarItem {
            class: "dx-navbar-item".to_string(),
            ..props
        }
    }
}
