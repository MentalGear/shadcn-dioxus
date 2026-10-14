use dioxus::prelude::*;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;
use dioxus_primitives::switch;

/// The size of a [`Switch`]. Nova's `data-[size=default]:h-[18.4px] w-[32px]` and
/// `data-[size=sm]:h-[14px] w-[24px]`; the track is not a control-height-ladder
/// step (a switch is not a button-height field), so the sizes are slot variables
/// in `style.css`.
#[derive(Copy, Clone, PartialEq, Default)]
#[non_exhaustive]
pub enum SwitchSize {
    #[default]
    Default,
    Sm,
}

impl SwitchSize {
    pub fn class(&self) -> &'static str {
        match self {
            SwitchSize::Default => "default",
            SwitchSize::Sm => "sm",
        }
    }
}

/// Props for the themed [`Switch`]: the primitive's, plus `size`.
#[derive(Props, Clone, PartialEq)]
pub struct SwitchProps {
    /// The controlled checked state of the switch.
    pub checked: ReadSignal<Option<bool>>,

    /// The default checked state when uncontrolled.
    #[props(default = false)]
    pub default_checked: bool,

    /// Whether the switch is disabled.
    #[props(default = ReadSignal::new(Signal::new(false)))]
    pub disabled: ReadSignal<bool>,

    /// Whether the switch is required in a form.
    #[props(default)]
    pub required: ReadSignal<bool>,

    /// The name attribute for form submission.
    #[props(default)]
    pub name: ReadSignal<String>,

    /// The value attribute for form submission.
    #[props(default = ReadSignal::new(Signal::new(String::from("on"))))]
    pub value: ReadSignal<String>,

    /// Callback fired when the checked state changes.
    #[props(default)]
    pub on_checked_change: Callback<bool>,

    /// `Default` (32 x 18.4px) or `Sm` (24 x 14px), shadcn's `size` prop.
    #[props(default)]
    pub size: SwitchSize,

    /// Additional attributes to apply to the switch element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the switch component.
    pub children: Element,
}

#[component]
pub fn Switch(props: SwitchProps) -> Element {
    let base = attributes!(button {
        class: "dx-switch",
        "data-size": props.size.class(),
    });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/switch/style.css") }
        switch::Switch {
            checked: props.checked,
            default_checked: props.default_checked,
            disabled: props.disabled,
            required: props.required,
            name: props.name,
            value: props.value,
            on_checked_change: props.on_checked_change,
            attributes: merged,
            switch::SwitchThumb { class: "dx-switch-thumb" }
        }
    }
}
