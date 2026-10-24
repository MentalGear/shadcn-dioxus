use dioxus::prelude::*;
use dioxus_primitives::{dioxus_attributes::attributes, merge_attributes, toggle};

/// The look of a [`Toggle`] (shadcn's `variant`).
#[derive(Copy, Clone, PartialEq, Default)]
#[non_exhaustive]
pub enum ToggleVariant {
    /// No edge: the control is only visible by its content until hovered or pressed.
    #[default]
    Default,
    /// A 1px `--dx-input` edge around a transparent fill.
    Outline,
}

impl ToggleVariant {
    /// The `data-variant` value the stylesheet keys on.
    pub fn class(&self) -> &'static str {
        match self {
            ToggleVariant::Default => "default",
            ToggleVariant::Outline => "outline",
        }
    }
}

/// The size of a [`Toggle`] (shadcn's `size`): the height follows the theme's
/// `--dx-control-height-*` ladder, the same 28 / 32 / 36px as `Button`.
#[derive(Copy, Clone, PartialEq, Default)]
#[non_exhaustive]
pub enum ToggleSize {
    /// 28px high, `0.8rem` text, a 14px icon.
    Sm,
    /// 32px high.
    #[default]
    Default,
    /// 36px high.
    Lg,
}

impl ToggleSize {
    /// The `data-size` value the stylesheet keys on.
    pub fn class(&self) -> &'static str {
        match self {
            ToggleSize::Sm => "sm",
            ToggleSize::Default => "default",
            ToggleSize::Lg => "lg",
        }
    }
}

/// The props for the themed [`Toggle`]: the primitive's, plus `variant` and `size`.
#[derive(Props, Clone, PartialEq)]
pub struct ToggleProps {
    /// The controlled pressed state of the toggle.
    pub pressed: ReadSignal<Option<bool>>,

    /// The default pressed state when uncontrolled.
    #[props(default)]
    pub default_pressed: bool,

    /// Whether the toggle is disabled.
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// The look of the toggle: no edge (`Default`) or a 1px edge (`Outline`).
    #[props(default)]
    pub variant: ToggleVariant,

    /// The size of the toggle: `Sm` (28px), `Default` (32px) or `Lg` (36px).
    #[props(default)]
    pub size: ToggleSize,

    /// Extra class for the toggle element, appended to the theme's `dx-toggle`.
    pub class: Option<String>,

    /// Callback fired when the pressed state changes.
    #[props(default)]
    pub on_pressed_change: Callback<bool>,

    /// Callback fired when the toggle is mounted.
    #[props(default)]
    pub onmounted: Callback<Event<MountedData>>,
    /// Callback fired when the toggle receives focus.
    #[props(default)]
    pub onfocus: Callback<Event<FocusData>>,
    /// Callback fired when a key is pressed on the toggle.
    #[props(default)]
    pub onkeydown: Callback<Event<KeyboardData>>,

    /// Additional attributes to apply to the toggle element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the toggle component.
    pub children: Element,
}

#[component]
pub fn Toggle(props: ToggleProps) -> Element {
    // Merges `class` the same way `../popover/component.rs`'s
    // `PopoverContent` and `../radio_group/component.rs`'s `RadioItem` do:
    // a caller's own class should extend, not replace, this theme's
    // `"dx-toggle"` -- this shape avoids the clobber bug (backlog row 12).
    let class = if let Some(class) = props.class {
        format!("{} {}", "dx-toggle", class)
    } else {
        "dx-toggle".to_string()
    };
    // `variant` and `size` only select slot variables in the stylesheet
    // (`data-variant` / `data-size`); they are theme state, so they win over a
    // caller's attribute of the same name.
    let owned = attributes!(button {
        "data-dx-root": "",
        "data-variant": props.variant.class(),
        "data-size": props.size.class(),
    });
    let merged = merge_attributes(vec![props.attributes, owned]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/toggle/style.css") }
        toggle::Toggle {
            class,
            pressed: props.pressed,
            default_pressed: props.default_pressed,
            disabled: props.disabled,
            on_pressed_change: props.on_pressed_change,
            onmounted: props.onmounted,
            onfocus: props.onfocus,
            onkeydown: props.onkeydown,
            attributes: merged,
            {props.children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[component]
    fn ToggleWithCallerClass() -> Element {
        rsx! {
            Toggle {
                pressed: None,
                class: "caller-class".to_string(),
                "B"
            }
        }
    }

    #[test]
    fn caller_class_and_theme_class_both_render_on_the_button() {
        let mut dom = VirtualDom::new(ToggleWithCallerClass);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);

        // A caller's own class must extend, not replace, the theme's
        // "dx-toggle" (backlog row 12).
        assert!(html.contains(r#"class="dx-toggle caller-class""#));
    }

    #[test]
    fn theme_class_alone_renders_when_caller_sets_no_class() {
        let mut dom = VirtualDom::new(|| {
            rsx! {
                Toggle { pressed: None, "B" }
            }
        });
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);

        assert!(html.contains(r#"class="dx-toggle""#));
    }

    #[test]
    fn variant_and_size_render_as_data_attributes() {
        let mut dom = VirtualDom::new(|| {
            rsx! {
                Toggle {
                    pressed: None,
                    variant: ToggleVariant::Outline,
                    size: ToggleSize::Sm,
                    "B"
                }
            }
        });
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);

        assert!(html.contains(r#"data-variant="outline""#));
        assert!(html.contains(r#"data-size="sm""#));
    }

    #[test]
    fn variant_and_size_default_to_default() {
        let mut dom = VirtualDom::new(|| {
            rsx! {
                Toggle { pressed: None, "B" }
            }
        });
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);

        assert!(html.contains(r#"data-variant="default""#));
        assert!(html.contains(r#"data-size="default""#));
    }
}
