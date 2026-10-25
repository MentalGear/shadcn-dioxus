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
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/menubar/style.css") }
        menubar::Menubar {
            dir: props.dir,
            disabled: props.disabled,
            roving_loop: props.roving_loop,
            open_on_hover: props.open_on_hover,
            attributes: merged,
            {props.children}
        }
    }
}

#[component]
pub fn MenubarMenu(props: MenubarMenuProps) -> Element {
    let base = attributes!(div { class: "dx-menubar-menu" });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/menubar/style.css") }
        menubar::MenubarMenu {
            index: props.index,
            disabled: props.disabled,
            attributes: merged,
            {props.children}
        }
    }
}

#[component]
pub fn MenubarTrigger(props: MenubarTriggerProps) -> Element {
    let base = attributes!(button { class: "dx-menubar-trigger" });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/menubar/style.css") }
        menubar::MenubarTrigger { id: props.id, attributes: merged, {props.children} }
    }
}

#[component]
pub fn MenubarContent(props: MenubarContentProps) -> Element {
    let base = attributes!(div {
        class: "dx-menubar-content",
        "data-dx-surface": "floating",
    });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/menubar/style.css") }
        menubar::MenubarContent { id: props.id, attributes: merged, {props.children} }
    }
}

#[component]
pub fn MenubarItem(props: MenubarItemProps) -> Element {
    let base = attributes!(div {
        class: "dx-menubar-item",
        "data-dx-menu-item": "true",
    });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/menubar/style.css") }
        menubar::MenubarItem {
            variant: props.variant,
            inset: props.inset,
            index: props.index,
            value: props.value,
            disabled: props.disabled,
            text_value: props.text_value,
            on_select: props.on_select,
            attributes: merged,
            {props.children}
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
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/menubar/style.css") }
        menubar::MenubarCheckboxItem {
            inset: props.inset,
            index: props.index,
            checked: props.checked,
            default_checked: props.default_checked,
            on_checked_change: props.on_checked_change,
            disabled: props.disabled,
            text_value: props.text_value,
            close_on_select: props.close_on_select,
            attributes: merged,
            {props.children}
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
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/menubar/style.css") }
        menubar::MenubarRadioGroup {
            value: props.value,
            default_value: props.default_value,
            on_value_change: props.on_value_change,
            attributes: merged,
            {props.children}
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
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/menubar/style.css") }
        menubar::MenubarRadioItem {
            inset: props.inset,
            index: props.index,
            value: props.value,
            disabled: props.disabled,
            text_value: props.text_value,
            close_on_select: props.close_on_select,
            attributes: merged,
            {props.children}
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
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/menubar/style.css") }
        menubar::MenubarGroup { attributes: merged, {props.children} }
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
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/menubar/style.css") }
        menubar::MenubarSeparator { attributes: merged }
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
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/menubar/style.css") }
        menubar::MenubarShortcut { attributes: merged, {props.children} }
    }
}
