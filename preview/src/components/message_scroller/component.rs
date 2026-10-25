use crate::components::button::{Button, ButtonSize, ButtonVariant};
use dioxus::prelude::*;
use dioxus_primitives::AsChild;
use dioxus_icons::lucide::ArrowDown;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;
use dioxus_primitives::message_scroller::{
    self, MessageScrollerContentProps, MessageScrollerItemProps, MessageScrollerProps,
    MessageScrollerProviderProps, MessageScrollerRowsProps, MessageScrollerViewportProps,
};

// Re-exported so a demo (or a consumer's own page) can write
// `use crate::components::message_scroller::*;` and reach the commands hook,
// the scrollability signal and the option enums without a second import from
// `dioxus_primitives` -- the same convention `carousel`'s wrapper follows.
#[allow(unused_imports, reason = "a consumer-facing surface; this crate's own demos use a subset")]
pub use dioxus_primitives::message_scroller::{
    use_message_scroller, use_message_scroller_scrollable, use_message_scroller_visibility,
    DefaultScrollPosition, MessageScrollerApi, MessageScrollerDirection, ScrollAlign,
    ScrollBehavior, ScrollFade, ScrollToMessageOptions, Scrollable, Virtualization, Visibility,
};

/// The headless root: owns the scroll controller and renders no element.
#[component]
pub fn MessageScrollerProvider(props: MessageScrollerProviderProps) -> Element {
    rsx! {
        message_scroller::MessageScrollerProvider {
            ..props
        }
    }
}

/// The frame: fills its parent (place it in a height-constrained container).
#[component]
pub fn MessageScroller(props: MessageScrollerProps) -> Element {
    let base = attributes!(div {
        class: "dx-message-scroller", "data-dx-root": "",
        "data-slot": "message-scroller",
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/message_scroller/style.css") }
        message_scroller::MessageScroller {
            attributes: merged,
            ..props
        }
    }
}

/// The scrollable element. Its edge fade is the `fade` prop (default
/// [`ScrollFade::Bottom`], `dx-scroll-fade-b` from `dx-effects.css`; without that file the fade
/// is simply absent).
#[component]
pub fn MessageScrollerViewport(props: MessageScrollerViewportProps) -> Element {
    let base = attributes!(div {
        class: "dx-message-scroller-viewport",
        "data-slot": "message-scroller-viewport",
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/message_scroller/style.css") }
        message_scroller::MessageScrollerViewport {
            attributes: merged,
            ..props
        }
    }
}

/// The transcript column holding the rows.
#[component]
pub fn MessageScrollerContent(props: MessageScrollerContentProps) -> Element {
    let base = attributes!(div {
        class: "dx-message-scroller-content",
        "data-slot": "message-scroller-content",
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/message_scroller/style.css") }
        message_scroller::MessageScrollerContent {
            attributes: merged,
            ..props
        }
    }
}

/// One transcript row: a message, marker, typing indicator or separator.
#[component]
pub fn MessageScrollerItem(props: MessageScrollerItemProps) -> Element {
    let base = attributes!(div {
        class: "dx-message-scroller-item",
        "data-slot": "message-scroller-item",
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/message_scroller/style.css") }
        message_scroller::MessageScrollerItem {
            attributes: merged,
            ..props
        }
    }
}

/// A long transcript's rows, grouped in chunks of 20 that the browser skips as a unit
/// (one skippable element per row costs main-thread time per scroll frame in proportion to the
/// row count). Use it in place of a `for` loop of `MessageScrollerItem`s.
#[component]
pub fn MessageScrollerRows(props: MessageScrollerRowsProps) -> Element {
    let base = attributes!(div {
        class: "dx-message-scroller-chunk",
        "data-slot": "message-scroller-chunk",
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/message_scroller/style.css") }
        message_scroller::MessageScrollerRows {
            attributes: merged,
            ..props
        }
    }
}

/// The jump control: scrolls to the end (or start) of the transcript. Renders
/// the themed `Button` (a secondary icon button by default) with an arrow and a
/// screen-reader label unless `children` are given. The button sits in a `dx-message-scroller-jump`
/// wrapper that places it on the frame's edge and slides and fades it; the button's own look is
/// tuned through Button's slot variables.
#[component]
pub fn MessageScrollerButton(
    /// The transcript edge to jump to: `End` (the newest message, the default)
    /// or `Start` (the oldest).
    #[props(default)] direction: MessageScrollerDirection,
    /// How the jump scrolls: `Smooth` (the default, skipped under reduced motion),
    /// `Auto` (the viewport's own CSS `scroll-behavior`) or `Instant`.
    #[props(default = ScrollBehavior::Smooth)] behavior: ScrollBehavior,
    /// The button's look. Default `Secondary`.
    #[props(default = ButtonVariant::Secondary)] variant: ButtonVariant,
    /// The button's size. Default `IconSm`.
    #[props(default = ButtonSize::IconSm)] size: ButtonSize,
    /// Called on a click, before the jump. Calling `prevent_default()` on the
    /// event cancels the jump.
    #[props(default)] onclick: Option<EventHandler<MouseEvent>>,
    #[props(extends = GlobalAttributes)]
    #[props(extends = button)]
    attributes: Vec<Attribute>,
    /// Replaces the default arrow and screen-reader label.
    children: Option<Element>,
) -> Element {
    let base = attributes!(button {
        class: "dx-message-scroller-button",
        "data-slot": "message-scroller-button",
    });
    let merged = merge_attributes(vec![base, attributes]);
    let jump_edge = match direction {
        MessageScrollerDirection::End => "end",
        MessageScrollerDirection::Start => "start",
    };
    let label = match direction {
        MessageScrollerDirection::End => "Scroll to end",
        MessageScrollerDirection::Start => "Scroll to start",
    };

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/message_scroller/style.css") }
        message_scroller::MessageScrollerButton {
            direction,
            behavior,
            onclick,
            attributes: merged,
            r#as: move |el: AsChild| {
                rsx! {
                    span { class: "dx-message-scroller-jump", "data-direction": jump_edge,
                        Button { variant, size, attributes: el.attributes,
                            if let Some(children) = children.clone() {
                                {children}
                            } else {
                                ArrowDown { size: "16px", "aria-hidden": "true" }
                                span { class: "dx-message-scroller-sr-only", "{label}" }
                            }
                        }
                    }
                }
            },
            {}
        }
    }
}
