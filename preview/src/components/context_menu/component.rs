use dioxus::prelude::*;
use dioxus_icons::lucide::{Check, ChevronRight};
use dioxus_primitives::context_menu::{
    self, ContextMenuCheckboxItemProps, ContextMenuContentProps, ContextMenuGroupProps, ContextMenuSeparatorProps, ContextMenuShortcutProps, ContextMenuItemProps,
    ContextMenuProps, ContextMenuRadioGroupProps, ContextMenuRadioItemProps,
    ContextMenuSubContentProps, ContextMenuSubItemProps, ContextMenuSubProps,
    ContextMenuSubTriggerProps, ContextMenuTriggerProps,
};
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;

/// `variant` of an item (`Default` or `Destructive`), re-exported so an app that installed this
/// component does not have to name `dioxus_primitives::menu_item`.
pub use dioxus_primitives::menu_item::MenuItemVariant;

#[component]
pub fn ContextMenu(props: ContextMenuProps) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/context_menu/style.css") }
        context_menu::ContextMenu {
            ..props
        }
    }
}

#[component]
pub fn ContextMenuTrigger(props: ContextMenuTriggerProps) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/context_menu/style.css") }
        context_menu::ContextMenuTrigger {
            id: props.id,
            padding: "20px",
            background: "var(--dx-background)",
            border: "1px dashed var(--dx-border)",
            border_radius: ".5rem",
            cursor: "context-menu",
            user_select: "none",
            text_align: "center",
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[component]
pub fn ContextMenuContent(props: ContextMenuContentProps) -> Element {
    let base = attributes!(div {
        class: "dx-context-menu-content",
        "data-dx-surface": "floating",
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/context_menu/style.css") }
        context_menu::ContextMenuContent {
            attributes: merged,
            ..props
        }
    }
}

#[component]
pub fn ContextMenuItem(props: ContextMenuItemProps) -> Element {
    let base = attributes!(div {
        class: "dx-context-menu-item",
        "data-dx-menu-item": "true",
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/context_menu/style.css") }
        context_menu::ContextMenuItem {
            attributes: merged,
            ..props
        }
    }
}

/// Themed wrapper for [`context_menu::ContextMenuSub`] -- row 68's nested
/// submenu. Renders no element itself (neither does the primitive it
/// wraps), so there is no `class`/`attributes` to attach.
#[component]
pub fn ContextMenuSub(props: ContextMenuSubProps) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/context_menu/style.css") }
        context_menu::ContextMenuSub {
            ..props
        }
    }
}

/// Themed wrapper for [`context_menu::ContextMenuSubTrigger`]. Carries both
/// `dx-context-menu-item` (the same hover/focus/disabled chrome every plain
/// item gets) and `dx-context-menu-sub-trigger` (the open-state highlight)
/// -- see `style.css`. Appends a `ChevronRight` icon after the caller's own
/// content, same as `dropdown_menu/component.rs`'s identical
/// `DropdownMenuSubTrigger` wrapper -- see that component's doc for why a
/// real icon element, not CSS-generated `content:` text, matters here. The
/// icon carries `dx-context-menu-item-indicator`, so it is positioned in the
/// item's reserved indicator slot (`style.css`) and a long label can never
/// meet it.
#[component]
pub fn ContextMenuSubTrigger(props: ContextMenuSubTriggerProps) -> Element {
    let base = attributes!(div {
        class: "dx-context-menu-item dx-context-menu-sub-trigger",
        "data-dx-menu-item": "true",
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    let children = props.children.clone();
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/context_menu/style.css") }
        context_menu::ContextMenuSubTrigger {
            attributes: merged,
            ..props,
            {children}
            ChevronRight {
                class: "dx-context-menu-item-indicator dx-context-menu-sub-trigger-icon",
                "data-dx-menu-item-indicator": "end",
                size: "16px",
            }
        }
    }
}

/// Themed wrapper for [`context_menu::ContextMenuSubContent`]. Carries both
/// `dx-context-menu-content` (the shared menu chrome) and
/// `dx-context-menu-sub-content` (the side-anchored, not click-point,
/// static fallback position) -- see `style.css`.
#[component]
pub fn ContextMenuSubContent(props: ContextMenuSubContentProps) -> Element {
    let base = attributes!(div {
        class: "dx-context-menu-content dx-context-menu-sub-content",
        "data-dx-surface": "floating",
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/context_menu/style.css") }
        context_menu::ContextMenuSubContent {
            attributes: merged,
            ..props
        }
    }
}

/// Themed wrapper for [`context_menu::ContextMenuSubItem`]. Reuses the
/// plain `dx-context-menu-item` chrome.
#[component]
pub fn ContextMenuSubItem(props: ContextMenuSubItemProps) -> Element {
    let base = attributes!(div {
        class: "dx-context-menu-item",
        "data-dx-menu-item": "true",
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/context_menu/style.css") }
        context_menu::ContextMenuSubItem {
            attributes: merged,
            ..props
        }
    }
}

/// Themed wrapper for [`context_menu::ContextMenuCheckboxItem`] -- shadcn's
/// `ContextMenuCheckboxItem`. Carries `dx-context-menu-item` (the same
/// hover/focus/disabled chrome every plain item gets) and
/// `dx-context-menu-checkable-item` (the reserved indicator slot, see
/// `style.css`), and appends a `Check` icon after the caller's own content,
/// same as `dropdown_menu/component.rs`'s identical
/// `DropdownMenuCheckboxItem` wrapper -- see that component's doc. The icon
/// carries `dx-context-menu-item-indicator`, so it sits in the item's
/// reserved inline-end slot and a long label can never meet it; it is drawn
/// only while the primitive reports `data-state="checked"`.
#[component]
pub fn ContextMenuCheckboxItem(props: ContextMenuCheckboxItemProps) -> Element {
    let base = attributes!(div {
        class: "dx-context-menu-item dx-context-menu-checkable-item",
        "data-dx-menu-item": "true",
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    let children = props.children.clone();
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/context_menu/style.css") }
        context_menu::ContextMenuCheckboxItem {
            attributes: merged,
            ..props,
            {children}
            Check {
                class: "dx-context-menu-item-indicator",
                "data-dx-menu-item-indicator": "end",
                size: "16px",
            }
        }
    }
}

/// Themed wrapper for [`context_menu::ContextMenuRadioGroup`] -- shadcn's
/// `ContextMenuRadioGroup`. A `role="group"` with no chrome of its own.
#[component]
pub fn ContextMenuRadioGroup(props: ContextMenuRadioGroupProps) -> Element {
    let base = attributes!(div { class: "dx-context-menu-radio-group" });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/context_menu/style.css") }
        context_menu::ContextMenuRadioGroup {
            attributes: merged,
            ..props
        }
    }
}

/// Themed wrapper for [`context_menu::ContextMenuRadioItem`] -- shadcn's
/// `ContextMenuRadioItem`. Identical chrome and indicator slot to
/// [`ContextMenuCheckboxItem`]; the `Check` marks the chosen item.
#[component]
pub fn ContextMenuRadioItem(props: ContextMenuRadioItemProps) -> Element {
    let base = attributes!(div {
        class: "dx-context-menu-item dx-context-menu-checkable-item",
        "data-dx-menu-item": "true",
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    let children = props.children.clone();
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/context_menu/style.css") }
        context_menu::ContextMenuRadioItem {
            attributes: merged,
            ..props,
            {children}
            Check {
                class: "dx-context-menu-item-indicator",
                "data-dx-menu-item-indicator": "end",
                size: "16px",
            }
        }
    }
}

/// A non-interactive heading for a group of items -- shadcn's
/// `ContextMenuLabel`. Pure presentation (the primitive has no label
/// component): give it an `id` and point a [`ContextMenuRadioGroup`]'s
/// `aria-labelledby` at it so the group is named.
#[component]
pub fn ContextMenuLabel(
    /// Reserves the leading indicator slot -- shadcn's `inset` -- so the heading
    /// lines up with inset items. Carries `data-inset="true"`.
    #[props(default)]
    inset: bool,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    /// The label text.
    children: Element,
) -> Element {
    let base = attributes!(div {
        class: "dx-context-menu-label",
        "data-dx-menu-label": "true",
    });
    let inset_attr = attributes!(div {
        "data-inset": inset.then_some("true"),
    });
    let merged = merge_attributes(vec![base, attributes, inset_attr]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/context_menu/style.css") }
        div { ..merged, {children} }
    }
}

/// Themed wrapper for [`context_menu::ContextMenuGroup`] -- shadcn's `ContextMenuGroup`. A
/// `role="group"` with no chrome of its own; give it an `aria-label` when it
/// has a visible ContextMenuLabel.
#[component]
pub fn ContextMenuGroup(props: ContextMenuGroupProps) -> Element {
    let base = attributes!(div {
        class: "dx-context-menu-group",
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/context_menu/style.css") }
        context_menu::ContextMenuGroup {
            attributes: merged,
            ..props
        }
    }
}

/// Themed wrapper for [`context_menu::ContextMenuSeparator`] -- shadcn's
/// `ContextMenuSeparator`: a hairline in the `border` colour that spans the menu's
/// padding edge to edge.
#[component]
pub fn ContextMenuSeparator(props: ContextMenuSeparatorProps) -> Element {
    let base = attributes!(div {
        class: "dx-context-menu-separator",
        "data-dx-menu-separator": "true",
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/context_menu/style.css") }
        context_menu::ContextMenuSeparator {
            attributes: merged,
            ..props
        }
    }
}

/// Themed wrapper for [`context_menu::ContextMenuShortcut`] -- shadcn's
/// `ContextMenuShortcut`: muted, small, letter-spaced text pushed to the inline
/// end of its item (it takes the item's ink while the item is highlighted).
#[component]
pub fn ContextMenuShortcut(props: ContextMenuShortcutProps) -> Element {
    let base = attributes!(span {
        class: "dx-context-menu-shortcut",
        "data-dx-menu-shortcut": "true",
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/context_menu/style.css") }
        context_menu::ContextMenuShortcut {
            attributes: merged,
            ..props
        }
    }
}
