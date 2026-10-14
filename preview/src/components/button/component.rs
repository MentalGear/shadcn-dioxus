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

#[component]
pub fn Button(
    #[props(default)] variant: ButtonVariant,
    #[props(default)] size: ButtonSize,
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
        class: "dx-button",
        "data-style": variant.class(),
        "data-size": size.class(),
    });
    // The handlers are attributes, not rsx on the `button`, so the `as`
    // element receives them the same way the `button` does.
    let handlers = attributes!(button {
        onclick: move |event| {
            if let Some(f) = &onclick {
                f.call(event);
            }
        },
        onmousedown: move |event| {
            if let Some(f) = &onmousedown {
                f.call(event);
            }
        },
        onmouseup: move |event| {
            if let Some(f) = &onmouseup {
                f.call(event);
            }
        },
        onkeydown: move |event| {
            if let Some(f) = &onkeydown {
                f.call(event);
            }
        },
    });
    let merged = merge_attributes(vec![base, attributes, handlers]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/button/style.css") }
        if let Some(dynamic) = r#as {
            {dynamic.call(merged)}
        } else {
            button { ..merged, {children} }
        }
    }
}
