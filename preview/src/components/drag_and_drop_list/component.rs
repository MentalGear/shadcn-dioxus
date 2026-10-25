use dioxus::prelude::*;
use dioxus_icons::lucide::{GripVertical, X};
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::drag_and_drop_list::{
    self, DragAndDropContext, DragAndDropDropIndicatorProps, DragAndDropItemContext,
    DragAndDropListItemProps, DragAndDropListItemsProps,
};
use dioxus_primitives::drag_and_drop_list::sortable_item_key;
use dioxus_primitives::merge_attributes;

// dev-docs/backlog.md row 32: see checkbox/component.rs's
// header comment for the full delivery-mechanism rationale (asset!() +
// document::Link, embedded in the wrapper so a `shadcn-dioxus add
// drag_and_drop_list`-copied component needs no extra wiring).
//
// Every class here lives under the full `dx-drag-and-drop-list-` namespace
// (list, item, icon, body, remove button, drop indicator), in this file,
// `style.css`, and the `variants/main` demo's own inline stylesheet (which
// hovers `.dx-drag-and-drop-list-item` to restyle a demo-only class). A short
// name such as `.dx-remove-button` would collide with `tag_group`'s
// differently-sized one (unsized vs this one's 26x26px + `margin-left: 10px`)
// and silently restyle whichever sheet lost the CSS load order (backlog row 32)
// -- see `scripts/check-dx-class-prefix.sh`.
#[derive(Props, Clone, PartialEq)]
pub struct DragAndDropListProps {
    /// The items to render. Each takes its identity from the `key:` on its
    /// root element (else its position here); key every item when the order
    /// is controlled.
    pub items: Vec<Element>,

    /// Controlled order: the keys of `items`, first to last. Set, the list
    /// shows exactly this order and changes it only when the owner writes a
    /// new one after `on_reorder`. Unset, the list remembers the order.
    #[props(default)]
    pub order: Option<ReadSignal<Vec<String>>>,

    /// Called with the complete new order (every key) after an item is moved
    /// by pointer or keyboard.
    #[props(default)]
    pub on_reorder: Callback<Vec<String>>,

    /// Called with the key of an item the user removed (`is_removable`).
    #[props(default)]
    pub on_remove: Callback<String>,

    /// Set if the list items should be removable
    #[props(default)]
    pub is_removable: bool,

    /// Accessible label for the list
    #[props(default)]
    pub aria_label: Option<String>,

    /// Opacity of the drag ghost (the dragged item, left in the list as a
    /// placeholder), `0.0` to `1.0`. Sets `--dx-dnd-ghost-opacity`. Unset: the
    /// stylesheet's `0.9`.
    #[props(default)]
    pub ghost_opacity: Option<f32>,

    /// The room the other items make at the drop slot: any CSS length
    /// (`"25px"`, `"1.5rem"`), or a bare number of pixels. Sets
    /// `--dx-dnd-drop-gap`. Unset: the stylesheet's `25px`.
    #[props(default)]
    pub drop_gap: Option<String>,

    /// The axis the items are laid out on. `Vertical` (the default) moves a
    /// lifted item with Up and Down; `Horizontal` uses Left and Right,
    /// mirrored under right-to-left, and the pointer picks the slot by x.
    #[props(default)]
    pub axis: dioxus_primitives::drag_and_drop_list::ReorderAxis,

    /// Additional attributes to apply to the list element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the list component.
    pub children: Element,
}

#[component]
pub fn DragAndDropList(props: DragAndDropListProps) -> Element {
    let is_removable = props.is_removable;
    let aria_label = props
        .aria_label
        .clone()
        .unwrap_or_else(|| "Sortable list".to_string());
    // Keep a stable key per item so Dioxus moves keyed siblings instead of
    // swapping content between list items during reorder.
    let items: Vec<Element> = props
        .items
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            let key = sortable_item_key(item, idx);
            rsx! {
                DragIcon { key: "{key}" }
                div { class: "dx-drag-and-drop-list-item-body-div", {item} }
                if is_removable {
                    RemoveButton {}
                }
            }
        })
        .collect();

    let base = attributes!(ul { class: "dx-drag-and-drop-list", "data-dx-root": "" });
    let merged = merge_attributes(vec![base, props.attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/drag_and_drop_list/style.css") }
        drag_and_drop_list::DragAndDropList {
            axis: props.axis,
            items,
            order: props.order,
            on_reorder: props.on_reorder,
            on_remove: props.on_remove,
            aria_label: props.aria_label,
            ghost_opacity: props.ghost_opacity,
            drop_gap: props.drop_gap,
            attributes: merged,
            drag_and_drop_list::DragAndDropInstructions {}
            DragAndDropListItems {
                aria_label,
            }
            drag_and_drop_list::DragAndDropLiveRegion {}
            {props.children}
        }
    }
}

#[component]
pub fn DragAndDropListItem(props: DragAndDropListItemProps) -> Element {
    let base = attributes!(li { class: "dx-drag-and-drop-list-item" });
    let merged = merge_attributes(vec![base, props.attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/drag_and_drop_list/style.css") }
        drag_and_drop_list::DragAndDropListItem {
            index: props.index,
            // Forward the stable item key so the primitive tracks focus by
            // identity across reorders and removals instead of losing it.
            item_key: props.item_key.clone(),
            attributes: merged,
            {props.children}
        }
    }
}

#[component]
pub fn DragAndDropListItems(props: DragAndDropListItemsProps) -> Element {
    let base = attributes!(ul { class: "dx-drag-and-drop-list-ul" });
    let merged = merge_attributes(vec![base, props.attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/drag_and_drop_list/style.css") }
        drag_and_drop_list::DragAndDropListItems {
            aria_label: props.aria_label,
            attributes: merged,
            for item in drag_and_drop_list::use_drag_and_drop_list_items() {
                Fragment {
                    key: "{item.key}",
                    DragAndDropDropIndicator {
                        index: item.index,
                        position: "before",
                    }
                    DragAndDropListItem {
                        index: item.index,
                        item_key: item.key.clone(),
                        {item.children}
                    }
                    DragAndDropDropIndicator {
                        index: item.index,
                        position: "after",
                    }
                }
            }
        }
    }
}

#[component]
pub fn DragAndDropDropIndicator(props: DragAndDropDropIndicatorProps) -> Element {
    let base = attributes!(div { class: "dx-drag-and-drop-list-drop-indicator" });
    let merged = merge_attributes(vec![base, props.attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/drag_and_drop_list/style.css") }
        drag_and_drop_list::DragAndDropDropIndicator {
            index: props.index,
            position: props.position,
            attributes: merged,
        }
    }
}

#[component]
fn DragIcon() -> Element {
    rsx! {
        GripVertical {
            class: "dx-drag-and-drop-list-item-icon",
            "aria-hidden": "true",
            size: "16px",
        }
    }
}

#[component]
pub fn RemoveButton(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let mut ctx: DragAndDropContext = use_context();
    let item_ctx: DragAndDropItemContext = use_context();
    let index = item_ctx.index();
    let label = format!("Remove item {}", index + 1);
    let base = attributes!(button {
        class: "dx-drag-and-drop-list-remove-button",
        r#type: "button",
        aria_label: "{label}",
    });
    // `draggable: "false"` is required behavior -- this button must not
    // itself start a native drag inside a draggable list item -- not a
    // caller default, so it is owned-wins.
    let owned = attributes!(button { draggable: "false" });
    let merged = merge_attributes(vec![base, attributes, owned]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/drag_and_drop_list/style.css") }
        button {
            onpointerdown: move |event| event.stop_propagation(),
            onmousedown: move |event| event.stop_propagation(),
            onmouseup: move |event| event.stop_propagation(),
            ondragstart: move |event| {
                event.prevent_default();
                event.stop_propagation();
            },
            onkeydown: move |event| event.stop_propagation(),
            onclick: move |event| {
                event.stop_propagation();
                ctx.remove(index);
            },
            ..merged,
            {children}
            X { size: "14px" }
        }
    }
}
