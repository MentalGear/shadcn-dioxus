use dioxus::prelude::*;
use dioxus_primitives::tabs::{self, TabContentProps, TabListProps, TabTriggerProps};
use dioxus_primitives::{dioxus_attributes::attributes, merge_attributes};

/// The props for the [`Tabs`] component.
#[derive(Props, Clone, PartialEq)]
pub struct TabsProps {
    /// The class of the tabs component.
    #[props(default)]
    pub class: String,

    /// The controlled value of the active tab.
    pub value: ReadSignal<Option<String>>,

    /// The default active tab value when uncontrolled.
    #[props(default)]
    pub default_value: String,

    /// Callback fired when the active tab changes.
    #[props(default)]
    pub on_value_change: Callback<String>,

    /// Whether the tabs are disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Whether the tabs are horizontal (the default, shadcn's `orientation="horizontal"`).
    /// `false` lays the list out as a column beside the panels (shadcn's
    /// `orientation="vertical"`), moves focus with `ArrowUp`/`ArrowDown`, and sets
    /// `aria-orientation="vertical"` on the list.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub horizontal: ReadSignal<bool>,

    /// Whether focus should loop around when reaching the end.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub roving_loop: ReadSignal<bool>,

    /// Keep inactive panels mounted as `hidden="until-found"` so find-in-page and `#fragment`
    /// navigation can reach and open them. See `dioxus_primitives::tabs::TabsProps`.
    #[props(default)]
    pub hidden_until_found: ReadSignal<bool>,

    /// The variant of the tabs component.
    #[props(default)]
    pub variant: TabsVariant,

    /// The size of the tabs: the list's height follows the theme's shared control-height
    /// scale (`--dx-control-height-sm` / `-default` / `-lg`, 28 / 32 / 36px).
    #[props(default)]
    pub size: TabsSize,

    /// Additional attributes to apply to the tabs element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the tabs component.
    pub children: Element,
}

/// The variant of the tabs component.
#[derive(Clone, Copy, PartialEq, Default)]
pub enum TabsVariant {
    /// The default variant.
    #[default]
    Default,
    /// No pill: a transparent list, and the active tab carries a 2px bar (an underline
    /// when horizontal, a bar on the panel side when vertical): shadcn's `line` variant.
    Line,
    /// An alias of [`TabsVariant::Line`]: it renders identically
    /// (`data-variant="line"`).
    Ghost,
}

impl TabsVariant {
    /// Convert the variant to a string for use in class names
    fn to_class(self) -> &'static str {
        match self {
            TabsVariant::Default => "default",
            TabsVariant::Line | TabsVariant::Ghost => "line",
        }
    }
}

/// The size of the tabs component. `Default` is shadcn's 32px list and never changes.
#[derive(Clone, Copy, PartialEq, Default)]
pub enum TabsSize {
    /// A 28px list with `text-xs` triggers.
    Sm,
    /// The default 32px list.
    #[default]
    Default,
    /// A 36px list with roomier triggers.
    Lg,
}

impl TabsSize {
    /// Convert the size to a string for use in the `data-size` attribute
    fn to_class(self) -> &'static str {
        match self {
            TabsSize::Sm => "sm",
            TabsSize::Default => "default",
            TabsSize::Lg => "lg",
        }
    }
}

#[component]
pub fn Tabs(props: TabsProps) -> Element {
    let base = attributes!(div {
        class: format!("{} {}", props.class, "dx-tabs"),
        "data-variant": props.variant.to_class(),
        "data-size": props.size.to_class(),
    });
    let merged = merge_attributes(vec![base, props.attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/tabs/style.css") }
        tabs::Tabs {
            value: props.value,
            default_value: props.default_value,
            on_value_change: props.on_value_change,
            disabled: props.disabled,
            horizontal: props.horizontal,
            roving_loop: props.roving_loop,
            hidden_until_found: props.hidden_until_found,
            attributes: merged,
            {props.children}
        }
    }
}

#[component]
pub fn TabList(props: TabListProps) -> Element {
    let base = attributes!(div {
        class: "dx-tabs-list"
    });
    let merged = merge_attributes(vec![base, props.attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/tabs/style.css") }
        tabs::TabList { attributes: merged, {props.children} }
    }
}

#[component]
pub fn TabTrigger(props: TabTriggerProps) -> Element {
    let base = attributes!(button {
        class: format!(
            "{} {}",
            "dx-tabs-trigger",
            props.class.unwrap_or_default()
        )
    });
    let merged = merge_attributes(vec![base, props.attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/tabs/style.css") }
        tabs::TabTrigger {
            class: None,
            id: props.id,
            value: props.value,
            index: props.index,
            disabled: props.disabled,
            attributes: merged,
            {props.children}
        }
    }
}

#[component]
pub fn TabContent(props: TabContentProps) -> Element {
    let base = attributes!(div {
        class: format!(
            "{} {} {}",
            props.class.unwrap_or_default(),
            "dx-tabs-content",
            "dx-tabs-content-themed"
        )
    });
    let merged = merge_attributes(vec![base, props.attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/tabs/style.css") }
        tabs::TabContent {
            class: None,
            value: props.value,
            id: props.id,
            index: props.index,
            attributes: merged,
            {props.children}
        }
    }
}
