use dioxus::prelude::*;
use dioxus_primitives::AsChild;
use dioxus_icons::lucide::BadgeCheck;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;

#[derive(Copy, Clone, PartialEq, Default)]
#[non_exhaustive]
pub enum BadgeVariant {
    #[default]
    Primary,
    Secondary,
    Destructive,
    Outline,
    /// No fill at rest; a muted fill on hover (shadcn's `ghost`).
    Ghost,
    /// Primary-coloured text that underlines on hover (shadcn's `link`).
    Link,
}

impl BadgeVariant {
    pub fn class(&self) -> &'static str {
        match self {
            BadgeVariant::Primary => "primary",
            BadgeVariant::Secondary => "secondary",
            BadgeVariant::Destructive => "destructive",
            BadgeVariant::Outline => "outline",
            BadgeVariant::Ghost => "ghost",
            BadgeVariant::Link => "link",
        }
    }
}

/// The props for the [`Badge`] component.
#[derive(Props, Clone, PartialEq)]
pub struct BadgeProps {
    /// The badge's look: `Primary` (default), `Secondary`, `Destructive`, `Outline`, `Ghost` or `Link`.
    #[props(default)]
    pub variant: BadgeVariant,

    /// Additional attributes to extend the badge element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// Renders the badge as the element the callback returns (shadcn's `render`
    /// prop, as a link or a router link). The callback gets the badge's
    /// attributes and children as an [`AsChild`]; spread the one and render
    /// the other inside the element.
    pub r#as: Option<Callback<AsChild, Element>>,

    /// The children of the badge element
    pub children: Element,
}

#[component]
pub fn Badge(props: BadgeProps) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/badge/style.css") }
        BadgeElement {
            "padding": true,
            variant: props.variant,
            attributes: props.attributes,
            r#as: props.r#as,
            {props.children}
        }
    }
}

#[component]
fn BadgeElement(props: BadgeProps) -> Element {
    let base = attributes!(span { class: "dx-badge", "data-dx-root": "" });
    // `data-style` reflects this wrapper's own typed `variant` prop, not a
    // caller default -- owned-wins.
    let owned = attributes!(span { "data-style": props.variant.class() });
    let merged = merge_attributes(vec![base, props.attributes, owned]);
    if let Some(dynamic) = props.r#as {
        dynamic.call(AsChild::new(merged, props.children))
    } else {
        rsx! {
            span { ..merged, {props.children} }
        }
    }
}

#[component]
pub fn VerifiedIcon() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/badge/style.css") }
        BadgeCheck {
            size: "12px",
            stroke: "var(--dx-foreground)",
        }
    }
}
