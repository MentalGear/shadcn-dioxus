use dioxus::prelude::*;
use dioxus_icons::lucide::Check;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::menubar::{
    self, MenubarCheckboxItemProps, MenubarContentProps, MenubarGroupProps, MenubarSeparatorProps, MenubarShortcutProps, MenubarItemProps, MenubarMenuProps,
    MenubarProps, MenubarRadioGroupProps, MenubarRadioItemProps, MenubarTriggerProps,
};
use dioxus_primitives::merge_attributes;

/// `variant` of an item (`Default` or `Destructive`), re-exported so an app that installed this
/// component does not have to name `dioxus_primitives::menu_item`.
pub use dioxus_primitives::menu_item::MenuItemVariant;

#[component]
pub fn Menubar(props: MenubarProps) -> Element {
    let base = attributes!(div { class: "dx-menubar", "data-dx-root": "" });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/menubar/style.css") }
        menubar::Menubar {
            attributes: merged,
            ..props
        }
    }
}

#[component]
pub fn MenubarMenu(props: MenubarMenuProps) -> Element {
    let base = attributes!(div { class: "dx-menubar-menu" });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/menubar/style.css") }
        menubar::MenubarMenu {
            attributes: merged,
            ..props
        }
    }
}

#[component]
pub fn MenubarTrigger(props: MenubarTriggerProps) -> Element {
    let base = attributes!(button { class: "dx-menubar-trigger" });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/menubar/style.css") }
        menubar::MenubarTrigger {
            attributes: merged,
            ..props
        }
    }
}

#[component]
pub fn MenubarContent(props: MenubarContentProps) -> Element {
    let base = attributes!(div {
        class: "dx-menubar-content",
        "data-dx-surface": "floating",
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/menubar/style.css") }
        menubar::MenubarContent {
            attributes: merged,
            ..props
        }
    }
}

#[component]
pub fn MenubarItem(props: MenubarItemProps) -> Element {
    let base = attributes!(div {
        class: "dx-menubar-item",
        "data-dx-menu-item": "true",
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/menubar/style.css") }
        menubar::MenubarItem {
            attributes: merged,
            ..props
        }
    }
}

/// Themed wrapper for [`menubar::MenubarCheckboxItem`] -- shadcn's
/// `MenubarCheckboxItem`. Carries `dx-menubar-item` (the same hover/focus/
/// disabled chrome every plain item gets) and `dx-menubar-checkable-item`
/// (the reserved indicator slot, see `style.css`), and appends a `Check` icon
/// after the caller's own content. Unlike the dropdown and context menus,
/// shadcn's menubar puts the check at the inline **start** (Nova's
/// `.cn-menubar-checkbox-item`: `pl-7`, indicator `left-1.5`), so this item
/// reserves `padding-inline-start`; the three
/// `--dx-menu-item-indicator-*` tokens and the construction are the same.
/// The icon carries `dx-menubar-item-indicator`; it is drawn only while the
/// primitive reports `data-state="checked"`.
#[component]
pub fn MenubarCheckboxItem(props: MenubarCheckboxItemProps) -> Element {
    let base = attributes!(div {
        class: "dx-menubar-item dx-menubar-checkable-item",
        "data-dx-menu-item": "true",
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    let children = props.children.clone();
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/menubar/style.css") }
        menubar::MenubarCheckboxItem {
            attributes: merged,
            ..props,
            {children}
            Check {
                class: "dx-menubar-item-indicator",
                "data-dx-menu-item-indicator": "start",
                size: "16px",
            }
        }
    }
}

/// Themed wrapper for [`menubar::MenubarRadioGroup`] -- shadcn's
/// `MenubarRadioGroup`. A `role="group"` with no chrome of its own.
#[component]
pub fn MenubarRadioGroup(props: MenubarRadioGroupProps) -> Element {
    let base = attributes!(div { class: "dx-menubar-radio-group" });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/menubar/style.css") }
        menubar::MenubarRadioGroup {
            attributes: merged,
            ..props
        }
    }
}

/// Themed wrapper for [`menubar::MenubarRadioItem`] -- shadcn's
/// `MenubarRadioItem`. Identical chrome and indicator slot to
/// [`MenubarCheckboxItem`]; the `Check` marks the chosen item.
#[component]
pub fn MenubarRadioItem(props: MenubarRadioItemProps) -> Element {
    let base = attributes!(div {
        class: "dx-menubar-item dx-menubar-checkable-item",
        "data-dx-menu-item": "true",
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    let children = props.children.clone();
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/menubar/style.css") }
        menubar::MenubarRadioItem {
            attributes: merged,
            ..props,
            {children}
            Check {
                class: "dx-menubar-item-indicator",
                "data-dx-menu-item-indicator": "start",
                size: "16px",
            }
        }
    }
}

/// A non-interactive heading for a group of items -- shadcn's
/// `MenubarLabel`. Pure presentation (the primitive has no label component):
/// give it an `id` and point a [`MenubarRadioGroup`]'s `aria-labelledby` at
/// it so the group is named.
#[component]
pub fn MenubarLabel(
    /// Reserves the leading indicator slot -- shadcn's `inset` -- so the heading
    /// lines up with inset items. Carries `data-inset="true"`.
    #[props(default)]
    inset: bool,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    /// The heading text.
    children: Element,
) -> Element {
    let base = attributes!(div {
        class: "dx-menubar-label",
        "data-dx-menu-label": "true",
    });
    let inset_attr = attributes!(div {
        "data-inset": inset.then_some("true"),
    });
    let merged = merge_attributes(vec![base, attributes, inset_attr]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/menubar/style.css") }
        div { ..merged, {children} }
    }
}

/// Themed wrapper for [`menubar::MenubarGroup`] -- shadcn's `MenubarGroup`. A
/// `role="group"` with no chrome of its own; give it an `aria-label` when it
/// has a visible MenubarLabel.
#[component]
pub fn MenubarGroup(props: MenubarGroupProps) -> Element {
    let base = attributes!(div {
        class: "dx-menubar-group",
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/menubar/style.css") }
        menubar::MenubarGroup {
            attributes: merged,
            ..props
        }
    }
}

/// Themed wrapper for [`menubar::MenubarSeparator`] -- shadcn's
/// `MenubarSeparator`: a hairline in the `border` colour that spans the menu's
/// padding edge to edge.
#[component]
pub fn MenubarSeparator(props: MenubarSeparatorProps) -> Element {
    let base = attributes!(div {
        class: "dx-menubar-separator",
        "data-dx-menu-separator": "true",
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/menubar/style.css") }
        menubar::MenubarSeparator {
            attributes: merged,
            ..props
        }
    }
}

/// Themed wrapper for [`menubar::MenubarShortcut`] -- shadcn's
/// `MenubarShortcut`: muted, small, letter-spaced text pushed to the inline
/// end of its item (it takes the item's ink while the item is highlighted).
#[component]
pub fn MenubarShortcut(props: MenubarShortcutProps) -> Element {
    let base = attributes!(span {
        class: "dx-menubar-shortcut",
        "data-dx-menu-shortcut": "true",
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/menubar/style.css") }
        menubar::MenubarShortcut {
            attributes: merged,
            ..props
        }
    }
}
