use dioxus::prelude::*;
use dioxus_primitives::AsChild;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;

#[derive(Copy, Clone, PartialEq, Default)]
#[non_exhaustive]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Destructive,
    Outline,
    Ghost,
    Link,
}

impl ButtonVariant {
    pub fn class(&self) -> &'static str {
        match self {
            ButtonVariant::Primary => "primary",
            ButtonVariant::Secondary => "secondary",
            ButtonVariant::Destructive => "destructive",
            ButtonVariant::Outline => "outline",
            ButtonVariant::Ghost => "ghost",
            ButtonVariant::Link => "link",
        }
    }
}

#[derive(Copy, Clone, PartialEq, Default)]
#[non_exhaustive]
pub enum ButtonSize {
    Xs,
    Sm,
    #[default]
    Default,
    Lg,
    Icon,
    IconXs,
    IconSm,
    IconLg,
}

impl ButtonSize {
    pub fn class(&self) -> &'static str {
        match self {
            ButtonSize::Xs => "xs",
            ButtonSize::Sm => "sm",
            ButtonSize::Default => "default",
            ButtonSize::Lg => "lg",
            ButtonSize::Icon => "icon",
            ButtonSize::IconXs => "icon-xs",
            ButtonSize::IconSm => "icon-sm",
            ButtonSize::IconLg => "icon-lg",
        }
    }
}

/// The button's corner radius: the size's own (default) or fully rounded (shadcn's `rounded-full`).
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
#[non_exhaustive]
pub enum ButtonRadius {
    #[default]
    Default,
    Full,
}

impl ButtonRadius {
    /// The `data-radius` value the stylesheet keys on.
    pub fn class(&self) -> &'static str {
        match self {
            ButtonRadius::Default => "default",
            ButtonRadius::Full => "full",
        }
    }
}

#[component]
pub fn Button(
    /// The style: `Primary` (the default), `Secondary`, `Destructive`, `Outline`, `Ghost` or `Link`.
    #[props(default)] variant: ButtonVariant,
    /// The size: `Default` (the default), `Xs`, `Sm`, `Lg`, `Icon`, `IconXs`, `IconSm` or `IconLg`.
    #[props(default)] size: ButtonSize,
    /// The corner radius. `Full` makes a pill (or, on an icon size, a circle).
    #[props(default)]
    radius: ButtonRadius,
    #[props(extends=GlobalAttributes)]
    #[props(extends=button)]
    attributes: Vec<Attribute>,
    /// Called with the click event.
    onclick: Option<EventHandler<MouseEvent>>,
    /// Called with the mouse event when a mouse button is pressed on the button.
    onmousedown: Option<EventHandler<MouseEvent>>,
    /// Called with the mouse event when a mouse button is released on the button.
    onmouseup: Option<EventHandler<MouseEvent>>,
    /// Called with the keyboard event when a key is pressed while the button has focus.
    onkeydown: Option<EventHandler<KeyboardEvent>>,
    /// Renders the button as the element the callback returns, shadcn's "As
    /// Link": an anchor, or a router link. The callback gets an [`AsChild`]:
    /// the button's attributes with its handlers in them, so a click on that
    /// element still calls `onclick`, and the button's children.
    r#as: Option<Callback<AsChild, Element>>,
    /// The button's label or icon.
    children: Element,
) -> Element {
    let base = attributes!(button {
        class: "dx-button", "data-dx-root": "",
        "data-style": variant.class(),
        "data-size": size.class(),
        "data-radius": radius.class(),
    });
    // The handlers are attributes, not rsx on the `button`, so the `as`
    // element receives them the same way the `button` does. Only the ones the
    // caller gave are emitted (a no-op listener would stand in the way of
    // nothing, but there is no reason to attach one). `attributes` is the
    // later list, the caller's, and `merge_attributes` chains a listener that
    // also arrives through it (DialogClose, a menu trigger's `as` ...) with
    // the prop instead of replacing it: the caller's first, then the prop,
    // unless the caller prevented default.
    let mut handlers = Vec::new();
    if let Some(f) = onclick {
        handlers.extend(attributes!(button { onclick: move |event| f.call(event) }));
    }
    if let Some(f) = onmousedown {
        handlers.extend(attributes!(button { onmousedown: move |event| f.call(event) }));
    }
    if let Some(f) = onmouseup {
        handlers.extend(attributes!(button { onmouseup: move |event| f.call(event) }));
    }
    if let Some(f) = onkeydown {
        handlers.extend(attributes!(button { onkeydown: move |event| f.call(event) }));
    }
    let merged = merge_attributes(vec![base, handlers, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/button/style.css") }
        if let Some(dynamic) = r#as {
            {dynamic.call(AsChild::new(merged, children))}
        } else {
            button { ..merged, {children} }
        }
    }
}
