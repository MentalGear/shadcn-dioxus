use crate::components::separator::Separator;
use dioxus::prelude::*;
use dioxus_primitives::AsChild;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;

#[derive(Copy, Clone, PartialEq, Default)]
#[non_exhaustive]
pub enum ItemVariant {
    #[default]
    Default,
    Outline,
    Muted,
}

impl ItemVariant {
    pub fn class(&self) -> &'static str {
        match self {
            ItemVariant::Default => "default",
            ItemVariant::Outline => "outline",
            ItemVariant::Muted => "muted",
        }
    }
}

#[derive(Copy, Clone, PartialEq, Default)]
#[non_exhaustive]
pub enum ItemSize {
    #[default]
    Default,
    Sm,
    /// Nova's `xs`: tighter padding and gap, `text-xs` description.
    Xs,
}

impl ItemSize {
    pub fn class(&self) -> &'static str {
        match self {
            ItemSize::Default => "default",
            ItemSize::Sm => "sm",
            ItemSize::Xs => "xs",
        }
    }
}

#[derive(Copy, Clone, PartialEq, Default)]
#[non_exhaustive]
pub enum ItemMediaVariant {
    #[default]
    Default,
    Icon,
    Image,
}

impl ItemMediaVariant {
    pub fn class(&self) -> &'static str {
        match self {
            ItemMediaVariant::Default => "default",
            ItemMediaVariant::Icon => "icon",
            ItemMediaVariant::Image => "image",
        }
    }
}

/// Provided by [`ItemGroup`] so an [`Item`] knows it is a list's child. `ItemGroup` is `role="list"`,
/// which ARIA requires to own `listitem`s (axe `aria-required-children`), and an `Item` outside a group
/// must NOT claim `listitem` (an orphan one is the mirror failure, `aria-required-parent`). Context
/// makes both hold by construction: the role follows the structure, nobody has to remember it.
#[derive(Clone, Copy)]
struct InItemGroup;

#[component]
pub fn ItemGroup(
    #[props(extends=GlobalAttributes)]
    #[props(extends=div)]
    attributes: Vec<Attribute>,
    /// The `Item`s of the list.
    children: Element,
) -> Element {
    use_context_provider(|| InItemGroup);
    let base = attributes!(div {
        class: "dx-item-group",
        role: "list",
        "data-slot": "item-group",
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/item/style.css") }
        div { ..merged,{children} }
    }
}

#[component]
pub fn ItemSeparator(
    #[props(extends=GlobalAttributes)]
    #[props(extends=div)]
    attributes: Vec<Attribute>,
) -> Element {
    let base = attributes!(div {
        class: "dx-item-separator",
        "data-slot": "item-separator",
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/item/style.css") }
        Separator { horizontal: true, decorative: true, attributes: merged }
    }
}

#[component]
pub fn Item(
    /// The item's look: `Default`, `Outline` (a visible border) or `Muted` (a tinted background).
    /// The default is `Default`.
    #[props(default)] variant: ItemVariant,
    /// The item's density: `Default`, `Sm`, or `Xs` (tighter padding and gap, smaller description
    /// text). The default is `Default`.
    #[props(default)] size: ItemSize,
    #[props(extends=GlobalAttributes)]
    #[props(extends=div)]
    attributes: Vec<Attribute>,
    /// Called on a click.
    onclick: Option<EventHandler<MouseEvent>>,
    /// Called on a key press while the item has focus.
    onkeydown: Option<EventHandler<KeyboardEvent>>,
    /// Renders the item as another element, such as a link. The callback receives the item's
    /// attributes and children as an `AsChild` and returns the element that carries them.
    r#as: Option<Callback<AsChild, Element>>,
    /// The item's parts, such as `ItemMedia`, `ItemContent` and `ItemActions`.
    children: Element,
) -> Element {
    let in_group = try_use_context::<InItemGroup>().is_some();
    let mut base = attributes!(div {
        class: "dx-item", "data-dx-root": "",
        "data-slot": "item",
        "data-variant": variant.class(),
        "data-size": size.class(),
    });
    // Inside an `ItemGroup` (`role="list"`) the item is a `listitem`; a caller's own `role` still wins.
    // With `as` the item is usually a link, and `role="listitem"` on an `<a>` would erase its link role,
    // so the list item is a transparent wrapper instead (`display: contents`: no box, the item stays the
    // group's flex child).
    if in_group && r#as.is_none() {
        base.extend(attributes!(div { role: "listitem" }));
    }
    // The handlers are attributes, so the `as` element receives them exactly as the `div` does
    // (row 395); `merge_attributes` chains one that also arrives through `attributes`.
    let mut handlers = Vec::new();
    if let Some(f) = onclick {
        handlers.extend(attributes!(div { onclick: move |event| f.call(event) }));
    }
    if let Some(f) = onkeydown {
        handlers.extend(attributes!(div { onkeydown: move |event| f.call(event) }));
    }
    let merged = merge_attributes(vec![base, handlers, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/item/style.css") }
        if let Some(dynamic) = r#as {
            if in_group {
                div { role: "listitem", display: "contents", {dynamic.call(AsChild::new(merged, children))} }
            } else {
                {dynamic.call(AsChild::new(merged, children))}
            }
        } else {
            div { ..merged, {children} }
        }
    }
}

#[component]
pub fn ItemMedia(
    /// The media's kind: `Default`, `Icon` (sized for an icon) or `Image` (a fixed square
    /// that crops its image). The default is `Default`.
    #[props(default)] variant: ItemMediaVariant,
    #[props(extends=GlobalAttributes)]
    #[props(extends=div)]
    attributes: Vec<Attribute>,
    /// An icon or an image.
    children: Element,
) -> Element {
    let base = attributes!(div {
        class: "dx-item-media",
        "data-slot": "item-media",
        "data-variant": variant.class(),
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/item/style.css") }
        div { ..merged,{children} }
    }
}

#[component]
pub fn ItemContent(
    #[props(extends=GlobalAttributes)]
    #[props(extends=div)]
    attributes: Vec<Attribute>,
    /// The item's text: its title and description.
    children: Element,
) -> Element {
    let base = attributes!(div {
        class: "dx-item-content",
        "data-slot": "item-content",
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/item/style.css") }
        div { ..merged,{children} }
    }
}

#[component]
pub fn ItemTitle(
    #[props(extends=GlobalAttributes)]
    #[props(extends=div)]
    attributes: Vec<Attribute>,
    /// The item's title.
    children: Element,
) -> Element {
    let base = attributes!(div {
        class: "dx-item-title",
        "data-slot": "item-title",
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/item/style.css") }
        div { ..merged,{children} }
    }
}

#[component]
pub fn ItemDescription(
    #[props(extends=GlobalAttributes)]
    #[props(extends=p)]
    attributes: Vec<Attribute>,
    /// The item's secondary text.
    children: Element,
) -> Element {
    let base = attributes!(p {
        class: "dx-item-description",
        "data-slot": "item-description",
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/item/style.css") }
        p { ..merged,{children} }
    }
}

#[component]
pub fn ItemActions(
    #[props(extends=GlobalAttributes)]
    #[props(extends=div)]
    attributes: Vec<Attribute>,
    /// The item's actions, such as buttons.
    children: Element,
) -> Element {
    let base = attributes!(div {
        class: "dx-item-actions",
        "data-slot": "item-actions",
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/item/style.css") }
        div { ..merged,{children} }
    }
}

#[component]
pub fn ItemHeader(
    #[props(extends=GlobalAttributes)]
    #[props(extends=div)]
    attributes: Vec<Attribute>,
    /// Content above the item's title.
    children: Element,
) -> Element {
    let base = attributes!(div {
        class: "dx-item-header",
        "data-slot": "item-header",
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/item/style.css") }
        div { ..merged,{children} }
    }
}

#[component]
pub fn ItemFooter(
    #[props(extends=GlobalAttributes)]
    #[props(extends=div)]
    attributes: Vec<Attribute>,
    /// Content below the item's main row.
    children: Element,
) -> Element {
    let base = attributes!(div {
        class: "dx-item-footer",
        "data-slot": "item-footer",
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/item/style.css") }
        div { ..merged,{children} }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    thread_local! {
        static NAMES: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    }

    /// With `as`, the item's children reach the caller's element and its `onclick`/`onkeydown`
    /// ride in the attributes that element spreads (row 395).
    #[test]
    fn as_item_keeps_children_and_handlers() {
        fn app() -> Element {
            rsx! {
                Item {
                    onclick: |_| {},
                    onkeydown: |_| {},
                    r#as: |el: AsChild| {
                        NAMES.with(|n| {
                            n.borrow_mut().extend(el.attributes.iter().map(|a| a.name.to_string()))
                        });
                        rsx! {
                            a { href: "#item", ..el.attributes, {el.children} }
                        }
                    },
                    ItemContent { "Visit our documentation" }
                }
            }
        }
        NAMES.with(|n| n.borrow_mut().clear());
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(
            html.contains(r##"href="#item""##) && html.contains("Visit our documentation"),
            "the item's children render inside the `as` element: {html}"
        );
        let names = NAMES.with(|n| n.borrow().clone());
        for handler in ["onclick", "onkeydown"] {
            assert!(names.iter().any(|n| n == handler), "{handler} is in `attributes`: {names:?}");
        }
    }
}
