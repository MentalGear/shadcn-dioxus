use dioxus::prelude::*;
use dioxus_icons::lucide::ChevronDown;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;
use dioxus_primitives::navigation_menu::{
    self, NavigationMenuContentProps, NavigationMenuIndicatorProps, NavigationMenuItemProps, NavigationMenuLinkProps,
    NavigationMenuListProps, NavigationMenuProps, NavigationMenuTriggerProps,
};

#[component]
pub fn NavigationMenu(props: NavigationMenuProps) -> Element {
    let base = attributes!(nav { class: "dx-navigation-menu", "data-dx-root": "" });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/navigation_menu/style.css") }
        navigation_menu::NavigationMenu {
            attributes: merged,
            ..props
        }
    }
}

#[component]
pub fn NavigationMenuList(props: NavigationMenuListProps) -> Element {
    let base = attributes!(ul { class: "dx-navigation-menu-list" });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/navigation_menu/style.css") }
        navigation_menu::NavigationMenuList {
            attributes: merged,
            ..props
        }
    }
}

#[component]
pub fn NavigationMenuItem(props: NavigationMenuItemProps) -> Element {
    let base = attributes!(li { class: "dx-navigation-menu-item" });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/navigation_menu/style.css") }
        navigation_menu::NavigationMenuItem {
            attributes: merged,
            ..props
        }
    }
}

#[component]
pub fn NavigationMenuTrigger(props: NavigationMenuTriggerProps) -> Element {
    let base = attributes!(button { class: "dx-navigation-menu-trigger" });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    let children = props.children.clone();
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/navigation_menu/style.css") }
        navigation_menu::NavigationMenuTrigger {
            attributes: merged,
            ..props,
            {children}
            ChevronDown {
                class: "dx-navigation-menu-expand-icon",
                size: "16px",
                stroke: "currentColor",
            }
        }
    }
}

#[component]
pub fn NavigationMenuContent(props: NavigationMenuContentProps) -> Element {
    let base = attributes!(div {
        class: "dx-navigation-menu-content",
        "data-dx-surface": "floating",
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/navigation_menu/style.css") }
        navigation_menu::NavigationMenuContent {
            attributes: merged,
            ..props
        }
    }
}

#[component]
pub fn NavigationMenuLink(props: NavigationMenuLinkProps) -> Element {
    let base = attributes!(a { class: "dx-navigation-menu-link" });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/navigation_menu/style.css") }
        navigation_menu::NavigationMenuLink {
            attributes: merged,
            ..props
        }
    }
}

/// shadcn's `NavigationMenuIndicator`: the arrow under the open item's trigger. The theme draws
/// the arrow (`.dx-navigation-menu-indicator-arrow`, Nova `cn-navigation-menu-indicator-arrow`);
/// the primitive only measures where the open trigger is.
#[component]
pub fn NavigationMenuIndicator(props: NavigationMenuIndicatorProps) -> Element {
    let base = attributes!(li { class: "dx-navigation-menu-indicator" });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/navigation_menu/style.css") }
        navigation_menu::NavigationMenuIndicator {
            attributes: merged,
            ..props,
            div { class: "dx-navigation-menu-indicator-arrow" }
        }
    }
}
