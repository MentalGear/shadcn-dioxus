use dioxus::prelude::*;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;
use dioxus_primitives::tooltip::{self, TooltipContentProps, TooltipTriggerProps};

/// The props for the styled [`Tooltip`]: the primitive's, except that `delay` and `close_delay`
/// default to 0 (the primitive's own defaults are Base UI's: 600 / 0).
///
/// shadcn's styled `tooltip.tsx` (base: `TooltipProvider delay = 0`, radix: `delayDuration = 0`)
/// ships a tooltip that opens on contact, so the styled component does too. Pass `delay` /
/// `close_delay` to ask for a rest-to-open tooltip (`delay: 600` is the primitive's default).
#[derive(Props, Clone, PartialEq)]
pub struct TooltipProps {
    /// Whether the tooltip is open
    pub open: ReadSignal<Option<bool>>,

    /// Default open state when uncontrolled
    #[props(default)]
    pub default_open: bool,

    /// Callback when open state changes
    #[props(default)]
    pub on_open_change: Callback<bool>,

    /// Whether the tooltip is disabled
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Milliseconds the pointer must rest on the trigger before the tooltip opens. Defaults to 0
    /// (shadcn's `TooltipProvider` default); keyboard focus always opens at once.
    #[props(default = ReadSignal::new(Signal::new(0)))]
    pub delay: ReadSignal<u64>,

    /// Milliseconds the tooltip stays open after the pointer leaves the trigger. Defaults to 0.
    #[props(default = ReadSignal::new(Signal::new(0)))]
    pub close_delay: ReadSignal<u64>,

    /// Additional attributes for the tooltip
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children: a [`TooltipTrigger`] and a [`TooltipContent`].
    pub children: Element,
}

#[component]
pub fn Tooltip(props: TooltipProps) -> Element {
    let base = attributes!(div {
        class: "dx-tooltip",
    });
    let merged = merge_attributes(vec![base, props.attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/tooltip/style.css") }
        tooltip::Tooltip {
            disabled: props.disabled,
            delay: props.delay,
            close_delay: props.close_delay,
            open: props.open,
            default_open: props.default_open,
            on_open_change: props.on_open_change,
            attributes: merged,
            {props.children}
        }
    }
}

#[component]
pub fn TooltipTrigger(props: TooltipTriggerProps) -> Element {
    let base = attributes!(button {
        class: "dx-tooltip-trigger",
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/tooltip/style.css") }
        tooltip::TooltipTrigger {
            attributes: merged,
            ..props
        }
    }
}

#[component]
pub fn TooltipContent(props: TooltipContentProps) -> Element {
    let base = attributes!(div {
        class: "dx-tooltip-content",
        "data-dx-surface": "floating",
        "data-dx-tone": "inverted",
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/tooltip/style.css") }
        tooltip::TooltipContent {
            attributes: merged,
            ..props
        }
    }
}
