use dioxus::prelude::*;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;

/// The card's spacing and title scale, shadcn's `size="default" | "sm"`.
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
#[non_exhaustive]
pub enum CardSize {
    #[default]
    Default,
    Sm,
}

impl CardSize {
    /// The `data-size` value the stylesheet keys on.
    pub fn class(&self) -> &'static str {
        match self {
            CardSize::Default => "default",
            CardSize::Sm => "sm",
        }
    }
}

#[component]
pub fn Card(
    /// Spacing and title scale: 16px / `text-base` title (default) or 12px / `text-sm` (`Sm`).
    #[props(default)]
    size: CardSize,
    #[props(extends=GlobalAttributes)] attributes: Vec<Attribute>,
    /// The card's parts: a header, content and footer.
    children: Element,
) -> Element {
    let base = attributes!(div { class: "dx-card", "data-dx-root": "", "data-slot": "card" });
    // `data-size` reflects the typed prop, not a caller default -- owned-wins.
    let owned = attributes!(div { "data-size": size.class() });
    let merged = merge_attributes(vec![base, attributes, owned]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/card/style.css") }
        div { ..merged, {children} }
    }
}

#[component]
pub fn CardHeader(
    #[props(extends=GlobalAttributes)] attributes: Vec<Attribute>,
    /// The title, description and any action.
    children: Element,
) -> Element {
    let base = attributes!(div { class: "dx-card-header", "data-slot": "card-header" });
    let merged = merge_attributes(vec![base, attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/card/style.css") }
        div { ..merged, {children} }
    }
}

#[component]
pub fn CardTitle(
    #[props(extends=GlobalAttributes)] attributes: Vec<Attribute>,
    /// The card's heading text.
    children: Element,
) -> Element {
    let base = attributes!(div { class: "dx-card-title", "data-slot": "card-title" });
    let merged = merge_attributes(vec![base, attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/card/style.css") }
        div { ..merged, {children} }
    }
}

#[component]
pub fn CardDescription(
    #[props(extends=GlobalAttributes)] attributes: Vec<Attribute>,
    /// The supporting text under the title.
    children: Element,
) -> Element {
    let base = attributes!(div { class: "dx-card-description", "data-slot": "card-description" });
    let merged = merge_attributes(vec![base, attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/card/style.css") }
        div { ..merged, {children} }
    }
}

#[component]
pub fn CardAction(
    #[props(extends=GlobalAttributes)] attributes: Vec<Attribute>,
    /// The control to show in the header, such as a `Button`.
    children: Element,
) -> Element {
    let base = attributes!(div { class: "dx-card-action", "data-slot": "card-action" });
    let merged = merge_attributes(vec![base, attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/card/style.css") }
        div { ..merged, {children} }
    }
}

#[component]
pub fn CardContent(
    #[props(extends=GlobalAttributes)] attributes: Vec<Attribute>,
    /// The card's main content.
    children: Element,
) -> Element {
    let base = attributes!(div { class: "dx-card-content", "data-slot": "card-content" });
    let merged = merge_attributes(vec![base, attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/card/style.css") }
        div { ..merged, {children} }
    }
}

#[component]
pub fn CardFooter(
    #[props(extends=GlobalAttributes)] attributes: Vec<Attribute>,
    /// The footer's actions or details.
    children: Element,
) -> Element {
    let base = attributes!(div { class: "dx-card-footer", "data-slot": "card-footer" });
    let merged = merge_attributes(vec![base, attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/card/style.css") }
        div { ..merged, {children} }
    }
}
