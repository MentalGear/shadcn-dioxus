use dioxus::prelude::*;
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
    #[props(default)] variant: ButtonVariant,
    #[props(default)] size: ButtonSize,
    /// The corner radius. `Full` makes a pill (or, on an icon size, a circle).
    #[props(default)]
    radius: ButtonRadius,
    #[props(extends=GlobalAttributes)]
    #[props(extends=button)]
    attributes: Vec<Attribute>,
    onclick: Option<EventHandler<MouseEvent>>,
    onmousedown: Option<EventHandler<MouseEvent>>,
    onmouseup: Option<EventHandler<MouseEvent>>,
    onkeydown: Option<EventHandler<KeyboardEvent>>,
    /// Renders the button as the element the callback returns, given the
    /// button's attributes (shadcn's "As Link": an anchor, or a router link).
    /// The callback then owns the children, and the handlers ride on the
    /// element it renders, so a click on that element still calls `onclick`.
    r#as: Option<Callback<Vec<Attribute>, Element>>,
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
            {dynamic.call(merged)}
        } else {
            button { ..merged, {children} }
        }
    }
}
