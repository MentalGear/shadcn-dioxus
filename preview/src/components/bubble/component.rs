use dioxus::prelude::*;
use dioxus_primitives::AsChild;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;

/// The visual treatment of a [`Bubble`].
#[derive(Copy, Clone, PartialEq, Default)]
#[non_exhaustive]
pub enum BubbleVariant {
    /// A strong primary bubble, usually for the current user.
    #[default]
    Default,
    /// The standard neutral bubble for conversation content.
    Secondary,
    /// A lower-emphasis bubble for quiet supporting content.
    Muted,
    /// A subtle primary-tinted bubble.
    Tinted,
    /// A bordered bubble for secondary or rich content.
    Outline,
    /// Unframed content for assistant text or rich content; spans the full row.
    Ghost,
    /// A destructive bubble for errors or failed actions.
    Destructive,
}

impl BubbleVariant {
    pub fn class(&self) -> &'static str {
        match self {
            BubbleVariant::Default => "default",
            BubbleVariant::Secondary => "secondary",
            BubbleVariant::Muted => "muted",
            BubbleVariant::Tinted => "tinted",
            BubbleVariant::Outline => "outline",
            BubbleVariant::Ghost => "ghost",
            BubbleVariant::Destructive => "destructive",
        }
    }
}

/// The inline alignment of a [`Bubble`] (or of its [`BubbleReactions`]).
#[derive(Copy, Clone, PartialEq, Default)]
#[non_exhaustive]
pub enum BubbleAlign {
    #[default]
    Start,
    End,
}

impl BubbleAlign {
    pub fn class(&self) -> &'static str {
        match self {
            BubbleAlign::Start => "start",
            BubbleAlign::End => "end",
        }
    }
}

/// Which edge of the bubble its [`BubbleReactions`] row overlaps.
#[derive(Copy, Clone, PartialEq, Default)]
#[non_exhaustive]
pub enum BubbleSide {
    Top,
    #[default]
    Bottom,
}

impl BubbleSide {
    pub fn class(&self) -> &'static str {
        match self {
            BubbleSide::Top => "top",
            BubbleSide::Bottom => "bottom",
        }
    }
}

/// Stacks consecutive bubbles from the same sender. Set `align` on each
/// [`Bubble`], not on the group.
#[component]
pub fn BubbleGroup(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    /// The bubbles from one sender.
    children: Element,
) -> Element {
    let base = attributes!(div {
        class: "dx-bubble-group",
        "data-slot": "bubble-group",
    });
    let merged = merge_attributes(vec![base, attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/bubble/style.css") }
        div { ..merged, {children} }
    }
}

/// The message surface: framed conversational content that sizes to its text,
/// up to 80% of the row (`Ghost` spans it). Presentational -- it holds no ARIA
/// role, so keep the conversation's semantics on the surrounding container, and
/// pair a variant's tone with text or an icon rather than colour alone.
/// Avatars, names, timestamps and message-level actions belong in `Message`.
#[component]
pub fn Bubble(
    /// The visual treatment: `Default` (the default), `Secondary`, `Muted`, `Tinted`, `Outline`, `Ghost` or `Destructive`.
    #[props(default)] variant: BubbleVariant,
    /// Which side of the row the bubble sits on: `Start` (the default) or `End`.
    #[props(default)] align: BubbleAlign,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    /// The message content.
    children: Element,
) -> Element {
    let base = attributes!(div {
        class: "dx-bubble", "data-dx-root": "",
        "data-slot": "bubble",
    });
    // `data-variant` / `data-align` reflect this wrapper's own typed props, not
    // caller defaults -- owned-wins.
    let owned = attributes!(div {
        "data-variant": variant.class(),
        "data-align": align.class(),
    });
    let merged = merge_attributes(vec![base, attributes, owned]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/bubble/style.css") }
        div { ..merged, {children} }
    }
}

/// The bubble's content. To turn the bubble into a real link or button (so it
/// is focusable and exposes the right role), pass `as`: the callback gets the
/// content's attributes and children as an [`AsChild`] and renders them on and in
/// its own element; its accessible name is that text.
#[component]
pub fn BubbleContent(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    /// Renders the content as the element the callback returns, such as a link or a button. The callback gets an `AsChild` with the content's attributes and children, to render on and in that element.
    r#as: Option<Callback<AsChild, Element>>,
    /// The message text or markup.
    children: Element,
) -> Element {
    let base = attributes!(div {
        class: "dx-bubble-content",
        "data-slot": "bubble-content",
    });
    let merged = merge_attributes(vec![base, attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/bubble/style.css") }
        if let Some(dynamic) = r#as {
            {dynamic.call(AsChild::new(merged, children))}
        } else {
            div { ..merged, {children} }
        }
    }
}

/// Reactions or quick-action buttons that overlap one edge of the bubble.
/// A static row of emoji needs `role: "img"` and an `aria-label` that names
/// them, so it is announced once; interactive reactions are real buttons, each
/// with its own `aria-label` when icon-only. Reactions overlap the bubble, so
/// leave vertical room between rows.
#[component]
pub fn BubbleReactions(
    /// The bubble edge the row overlaps: `Bottom` (the default) or `Top`.
    #[props(default)] side: BubbleSide,
    /// The side of the bubble the row aligns to: `End` (the default) or `Start`.
    #[props(default = BubbleAlign::End)] align: BubbleAlign,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    /// The reactions or quick-action buttons.
    children: Element,
) -> Element {
    let base = attributes!(div {
        class: "dx-bubble-reactions",
        "data-slot": "bubble-reactions",
    });
    // Placement reflects this wrapper's own typed props -- owned-wins.
    let owned = attributes!(div {
        "data-side": side.class(),
        "data-align": align.class(),
    });
    let merged = merge_attributes(vec![base, attributes, owned]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/bubble/style.css") }
        div { ..merged, {children} }
    }
}
