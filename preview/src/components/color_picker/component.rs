use dioxus::prelude::*;
use dioxus_primitives::color_picker::{
    self, Color, ColorAreaProps, ColorPickerContext,
};
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;
use dioxus_primitives::popover;
use dioxus_primitives::use_controlled;
use dioxus_primitives::label::Label;
use dioxus_primitives::slider::*;
use palette::{encoding, FromColor, Hsv, Hsva, IntoColor, RgbHue, Srgb, Srgba, WithAlpha};
use std::str::FromStr;

use crate::components::input::Input;

// dev-docs/backlog.md row 32: see checkbox/component.rs's
// header comment for the full delivery-mechanism rationale (asset!() +
// document::Link, embedded in the wrapper so a `shadcn-dioxus add
// color_picker`-copied component needs no extra wiring).
//
// Every class in this sheet carries the full `dx-color-picker-` namespace (e.g.
// `dx-color-picker-swatch`, not `dx-color-swatch`): with no hash suffix, a
// plain `dx-color-swatch` would be indistinguishable from any other
// component that might someday define the same short name -- see
// `scripts/check-dx-class-prefix.sh`, which enforces this repo-wide.

/// A color with its alpha as 8-bit channels, the form a hex string carries.
type Rgba8 = palette::rgb::Rgba<encoding::Srgb, u8>;

fn to_rgba8(color: Hsva<encoding::Srgb, f64>) -> Rgba8 {
    Srgba::<f64>::from_color(color).into_format()
}

/// `#RRGGBB`, or `#RRGGBBAA` when `with_alpha` is set.
fn format_hex(color: Hsva<encoding::Srgb, f64>, with_alpha: bool) -> String {
    let rgba = to_rgba8(color);
    if with_alpha {
        format!("#{rgba:X}")
    } else {
        format!("#{:X}", rgba.color)
    }
}

/// `#RRGGBB` for an opaque color and `#RRGGBBAA` for one with transparency: what the trigger and the
/// swatch announce and paint, whether or not the alpha editor is on.
fn format_hex_auto(color: Hsva<encoding::Srgb, f64>) -> String {
    format_hex(color, to_rgba8(color).alpha != u8::MAX)
}

/// The result of reading the hex field: the color and, when the text spelled one, its alpha.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ParsedHex {
    rgb: Color,
    /// `None` when the text carries no alpha and the current alpha is kept.
    alpha: Option<u8>,
}

/// Reads `#RGB` / `#RRGGBB` and, with `with_alpha`, `#RGBA` / `#RRGGBBAA`. In alpha mode a text
/// without an alpha digit pair means opaque (alpha 1); without it the alpha is left alone.
fn parse_hex(text: &str, with_alpha: bool) -> Option<ParsedHex> {
    let digits = text.strip_prefix('#').unwrap_or(text);
    match digits.len() {
        3 | 6 => Color::from_str(digits).ok().map(|rgb| ParsedHex {
            rgb,
            alpha: with_alpha.then_some(u8::MAX),
        }),
        4 | 8 if with_alpha => Rgba8::from_str(digits).ok().map(|c| ParsedHex {
            rgb: c.color,
            alpha: Some(c.alpha),
        }),
        _ => None,
    }
}

/// Applies a parsed hex to `current`: the new color, with the parsed alpha or the current one.
fn apply_hex(current: Hsva<encoding::Srgb, f64>, parsed: ParsedHex) -> Hsva<encoding::Srgb, f64> {
    let hsv: Hsv<encoding::Srgb, f64> = parsed.rgb.into_format::<f64>().into_color();
    let alpha = parsed
        .alpha
        .map_or(current.alpha, |a| f64::from(a) / f64::from(u8::MAX));
    hsv.with_alpha(alpha)
}

/// Whether the field's text already spells `external`.
fn hex_matches(parsed: ParsedHex, external: Hsva<encoding::Srgb, f64>) -> bool {
    let ext = to_rgba8(external);
    parsed.rgb == ext.color && parsed.alpha.is_none_or(|a| a == ext.alpha)
}

#[derive(Clone, Copy)]
struct ColorPickerRootContext {
    open: Memo<bool>,
    disabled: ReadSignal<bool>,
    color: ReadSignal<Hsva<encoding::Srgb, f64>>,
}

/// The props for the [`ColorPickerRoot`] component.
#[derive(Props, Clone, PartialEq)]
pub struct ColorPickerRootProps {
    /// The selected color
    #[props(default)]
    pub color: ReadSignal<Hsva<encoding::Srgb, f64>>,

    /// Callback when color changes
    #[props(default)]
    pub on_color_change: Callback<Hsva<encoding::Srgb, f64>>,

    /// Whether the color picker is disabled
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// The controlled open state of the popover.
    pub open: ReadSignal<Option<bool>>,

    /// The default open state when uncontrolled.
    #[props(default)]
    pub default_open: bool,

    /// Callback fired when the open state changes.
    #[props(default)]
    pub on_open_change: Callback<bool>,

    /// Additional attributes to extend the color picker element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the color picker element
    pub children: Element,
}

#[component]
pub fn ColorPickerRoot(props: ColorPickerRootProps) -> Element {
    let (open, set_open) = use_controlled(props.open, props.default_open, props.on_open_change);

    use_context_provider(|| ColorPickerRootContext {
        open,
        disabled: props.disabled,
        color: props.color,
    });

    let base = attributes!(div { class: "dx-color-picker", "data-dx-root": "" });
    let merged = merge_attributes(vec![base, props.attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/color_picker/style.css") }
        // Eager: the hex field in the popover is the themed `Input`, whose own `Link` would only be
        // inserted when the popover first mounts its content -- late and not render-blocking.
        document::Link { rel: "stylesheet", href: asset!("/src/components/input/style.css") }
        color_picker::ColorPicker {
            color: props.color,
            on_color_change: props.on_color_change,
            disabled: props.disabled,
            attributes: merged,
            popover::PopoverRoot {
                is_modal: false,
                open: Some(open()),
                on_open_change: move |v| set_open.call(v),
                {props.children}
            }
        }
    }
}

/// The props for the [`ColorPicker`] component.
#[derive(Props, Clone, PartialEq)]
pub struct ColorPickerProps {
    /// The selected color
    #[props(default)]
    pub color: ReadSignal<Hsva<encoding::Srgb, f64>>,

    /// Callback when color changes
    #[props(default)]
    pub on_color_change: Callback<Hsva<encoding::Srgb, f64>>,

    /// Whether the color picker is disabled
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Optional label on the trigger button
    #[props(default)]
    pub label: Option<String>,

    /// Let the user edit the alpha channel: adds an "Alpha" slider and makes the hex field show and
    /// accept `#RRGGBBAA`. Off (the default) the UI is unchanged and the alpha of `color` is kept.
    #[props(default)]
    pub alpha: ReadSignal<bool>,

    /// The controlled open state of the popover.
    pub open: ReadSignal<Option<bool>>,

    /// The default open state when uncontrolled.
    #[props(default)]
    pub default_open: bool,

    /// Callback fired when the open state changes.
    #[props(default)]
    pub on_open_change: Callback<bool>,

    /// Additional attributes to extend the color picker element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// Additional content to append to the default color picker popover
    pub children: Element,
}

#[component]
pub fn ColorPicker(props: ColorPickerProps) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/color_picker/style.css") }
        // Eager: the hex field in the popover is the themed `Input`, whose own `Link` would only be
        // inserted when the popover first mounts its content -- late and not render-blocking.
        document::Link { rel: "stylesheet", href: asset!("/src/components/input/style.css") }
        ColorPickerRoot {
            color: props.color,
            on_color_change: props.on_color_change,
            disabled: props.disabled,
            open: props.open,
            default_open: props.default_open,
            on_open_change: props.on_open_change,
            attributes: props.attributes,
            ColorPickerTrigger {
                label: props.label,
            }
            ColorPickerPopover {
                ColorPickerSelect { alpha: props.alpha }
                {props.children}
            }
        }
    }
}

/// The props for the [`ColorPickerTrigger`] component.
#[derive(Props, Clone, PartialEq)]
pub struct ColorPickerTriggerProps {
    /// Optional label on the trigger button
    #[props(default)]
    pub label: Option<String>,

    /// Additional attributes to extend the trigger button
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// Additional content to render inside the trigger button
    pub children: Element,
}

#[component]
pub fn ColorPickerTrigger(props: ColorPickerTriggerProps) -> Element {
    let ctx = use_context::<ColorPickerRootContext>();
    let aria_hex = use_memo(move || format_hex_auto((ctx.color)()));
    let base = attributes!(button {
        class: "dx-color-picker-button",
        aria_label: format!("Color picker {aria_hex}"),
    });
    // `aria-expanded` mirrors the popover's real open state, not a caller
    // default -- owned-wins, merged after the caller's own attributes.
    let owned = attributes!(button { aria_expanded: (ctx.open)() });
    let merged = merge_attributes(vec![base, props.attributes, owned]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/color_picker/style.css") }
        popover::PopoverTrigger {
            disabled: if (ctx.disabled)() { true },
            attributes: merged,
            ColorSwatch { color: ctx.color }
            if let Some(label) = props.label { span { {label} } }
            {props.children}
        }
    }
}

/// The props for the [`ColorPickerPopover`] component.
#[derive(Props, Clone, PartialEq)]
pub struct ColorPickerPopoverProps {
    /// Additional attributes to extend the popover content
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the color picker popover
    pub children: Element,
}

#[component]
pub fn ColorPickerPopover(props: ColorPickerPopoverProps) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/color_picker/style.css") }
        popover::PopoverContent {
            class: "dx-color-picker-popover".to_string(),
            "data-dx-surface": "floating",
            attributes: props.attributes,
            {props.children}
        }
    }
}

/// The props for the [`ColorField`] component.
#[derive(Props, Clone, PartialEq)]
pub struct ColorFieldProps {
    /// Optional label above the input field
    #[props(default)]
    pub label: Option<String>,

    /// Optional props for the text description element
    #[props(default)]
    pub description: Option<String>,

    /// Edit the alpha too: the field shows `#RRGGBBAA` and accepts `#RGB`, `#RGBA`, `#RRGGBB` and
    /// `#RRGGBBAA` (a text without alpha means opaque). Off, it shows `#RRGGBB` and keeps the alpha.
    #[props(default)]
    pub alpha: ReadSignal<bool>,

    /// Additional attributes to extend the color field element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the color field element
    pub children: Element,
}

/// # ColorField
///
/// The [`ColorField`] allows users to edit a hex color (with alpha when its `alpha` prop is set).
/// Reads and writes the current color through the surrounding [`ColorPickerContext`].
#[component]
fn ColorField(props: ColorFieldProps) -> Element {
    let ctx = use_context::<ColorPickerContext>();
    let alpha = props.alpha;
    let hex_of = move |color: Hsva<encoding::Srgb, f64>| format_hex(color, alpha());

    let mut value = use_signal(|| hex_of(ctx.color()));

    // Synchronize local text with external color changes. Only overwrite
    // when the field already holds a parseable hex — otherwise the user is
    // mid-edit and replacing their text would clobber the input.
    //
    // `value.peek()`, not `value()`: this effect's only real input is
    // `external` (the surrounding context's color) -- `value` is read
    // purely to check whether the field already shows it, so a write is
    // skipped when it's already correct. Reading it with tracked syntax
    // would subscribe this effect to the very signal it writes below,
    // the shape `scripts/check-self-subscribing-effects.sh` flags
    // (dev-docs/backlog.md row 73's class). The field-went-empty
    // recovery is `oninput` below, the place that actually knows the field
    // just emptied, instead of a self-triggered re-run noticing it.
    use_effect(move || {
        let external = ctx.color();
        let with_alpha = alpha();
        let current = value.peek().clone();
        if let Some(parsed) = parse_hex(&current, with_alpha) {
            if !hex_matches(parsed, external) {
                value.set(format_hex(external, with_alpha));
            }
        }
    });

    let base = attributes!(div { class: "dx-color-picker-field-container" });
    let merged = merge_attributes(vec![base, props.attributes]);

    rsx! {
        div {
            ..merged,
            if let Some(label) = props.label {
                Label {
                    html_for: "color_field",
                    class: "dx-color-picker-slider-title",
                    {label}
                }
            }
            Input {
                id: "color_field",
                placeholder: "Enter a color",
                value: "{value}",
                oninput: move |e: FormEvent| {
                    let mut input = e.value();

                    // Sanitize input: allow only '#' and hex digits, length limit.
                    input.retain(|c| c == '#' || c.is_ascii_hexdigit());

                    // Automatically prepend '#' if missing.
                    if !input.starts_with('#') && !input.is_empty() {
                        input.insert(0, '#');
                    }

                    // `#` plus up to six digits, or eight with alpha.
                    input.truncate(if alpha() { 9 } else { 7 });

                    if input.is_empty() {
                        // Restore immediately instead of leaving the field
                        // blank. Doing it right here, where the field
                        // actually just emptied, needs no read of `value`
                        // in the sync effect above (which would subscribe
                        // that effect to its own write).
                        value.set(hex_of(ctx.color()));
                        return;
                    }

                    value.set(input.to_uppercase());

                    if let Some(parsed) = parse_hex(&input, alpha()) {
                        ctx.set_color(apply_hex(ctx.color(), parsed));
                    }
                },
            }
            if let Some(text) = props.description {
                span { class: "dx-color-picker-field-description", {text} }
            }
            {props.children}
        }
    }
}

/// The props for the [`ColorSwatch`] component.
#[derive(Props, Clone, PartialEq)]
pub struct ColorSwatchProps {
    /// The selected color
    #[props(default)]
    pub color: ReadSignal<Hsva<encoding::Srgb, f64>>,

    /// Additional attributes to extend the color swatch element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the color swatch element
    pub children: Element,
}

/// # ColorSwatch
///
/// The [`ColorSwatch`] displays a preview of a selected color.
#[component]
fn ColorSwatch(props: ColorSwatchProps) -> Element {
    let hex_color = use_memo(move || format_hex_auto((props.color)()));

    let base = attributes!(div {
        class: "dx-color-picker-swatch",
        aria_label: format!("Selected color {hex_color}"),
    });
    // `role="img"` and the `--swatch-color` custom property are the
    // wrapper's own required semantics/visual state, not a caller default --
    // owned-wins, merged after the caller's own attributes.
    let owned = attributes!(div { role: "img", style: "--swatch-color: {hex_color}" });
    let merged = merge_attributes(vec![base, props.attributes, owned]);

    rsx! {
        div {
            ..merged,
            {props.children}
        }
    }
}

/// The props for the [`ColorSlider`] component.
#[derive(Props, Clone, PartialEq)]
pub struct ColorSliderProps {
    /// The slider's accessible name (for example "Hue" or "Alpha").
    pub title: ReadSignal<String>,

    /// Additional attributes to extend the color slider element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the color slider element
    pub children: Element,
}

/// # ColorSlider
///
/// The [`ColorSlider`] allows users to adjust the hue of the color held by
/// the surrounding [`ColorPickerContext`].
#[component]
fn ColorSlider(props: ColorSliderProps) -> Element {
    let ctx = use_context::<ColorPickerContext>();
    let mut current_hue = use_signal(|| ctx.color().hue.into_positive_degrees());

    let thumb_color = use_memo(move || {
        Srgb::<f64>::from_color(Hsv::<encoding::Srgb, f64>::new(
            RgbHue::new(current_hue()),
            1.0,
            1.0,
        ))
        .into_format::<u8>()
    });

    // `current_hue.peek()`, not `current_hue()`: this effect's only real
    // input is `ctx.color()` above -- the only other place that writes
    // `current_hue` (`on_value_change` below) always writes `ctx`'s hue
    // in the same call, so tracking `ctx.color()` alone already re-runs
    // this effect on every real change. Reading `current_hue` with
    // tracked syntax here would subscribe this effect to the value it
    // writes below, the shape `scripts/check-self-subscribing-effects.sh`
    // flags (dev-docs/backlog.md row 73's class).
    use_effect(move || {
        let value = ctx.color().hue.into_positive_degrees();
        let current = *current_hue.peek();

        let is_wrap_around = (value - current).abs() > 350.0;

        // Update the signal only if this is an actual new position,
        // and not a "flip" of the circle by the palette library.
        if !is_wrap_around && value != current {
            current_hue.set(value);
        }
    });

    let display_value = {
        let value = current_hue();
        format!("{value:.2}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
            + "°"
    };

    let base = attributes!(div { class: "dx-color-picker-slider-container" });
    let merged = merge_attributes(vec![base, props.attributes]);

    rsx! {

        div {
            ..merged,
            label { class: "dx-color-picker-slider-title", {props.title} }
            output { class: "dx-color-picker-slider-output", "{display_value}" }
            Slider {
                class: "dx-color-picker-slider",
                label: "Color Slider",
                horizontal: true,
                max: 360.0,
                value: current_hue(),
                on_value_change: move |h: f64| {
                    // Allow the value to be exactly 360.0
                    // The palette will understand that 360.0 == 0.0, but the signal will remain 360.0 for the UI.
                    current_hue.set(h);
                    ctx.set_hue(h);
                },
                SliderTrack {
                    class: "dx-color-picker-slider-track",
                    SliderThumb {
                        class: "dx-color-picker-slider-thumb",
                        aria_label: "Hue",
                        aria_valuetext: format!("{:.0}°", current_hue()),
                        background_color: format!("#{:X}", thumb_color()),
                    }
                }
            }
            {props.children}
        }
    }
}

/// # ColorAlphaSlider
///
/// The alpha slider for the color held by the surrounding [`ColorPickerContext`]: a 0-100 %
/// [`Slider`] over a checkerboard track that fades from transparent to the current color.
#[component]
fn ColorAlphaSlider() -> Element {
    let ctx = use_context::<ColorPickerContext>();
    let percent = use_memo(move || (ctx.color().alpha * 100.0).round());
    let track_color = use_memo(move || format!("#{:X}", to_rgba8(ctx.color()).color));

    rsx! {
        div { class: "dx-color-picker-slider-container",
            label { class: "dx-color-picker-slider-title", "Alpha" }
            output { class: "dx-color-picker-slider-output", "{percent()}%" }
            Slider {
                class: "dx-color-picker-slider",
                label: "Alpha",
                horizontal: true,
                max: 100.0,
                value: percent(),
                on_value_change: move |a: f64| ctx.set_alpha(a / 100.0),
                SliderTrack {
                    class: "dx-color-picker-slider-track",
                    style: "--track-gradient: linear-gradient(to right, transparent, {track_color})",
                    SliderThumb {
                        class: "dx-color-picker-slider-thumb",
                        aria_label: "Alpha",
                        aria_valuetext: format!("{}%", percent()),
                    }
                }
            }
        }
    }
}

#[component]
fn ColorArea(props: ColorAreaProps) -> Element {
    let base = attributes!(div { class: "dx-color-picker-area-container" });
    let merged = merge_attributes(vec![base, props.attributes]);

    rsx! {
        color_picker::ColorArea {
            step: props.step,
            attributes: merged,
            color_picker::AreaTrack {
                class: "dx-color-picker-area-track",
                color_picker::AreaThumb {
                    class: "dx-color-picker-area-thumb",
                    color_picker::AreaThumbSaturationInput {
                        class: "dx-color-picker-area-input",
                    }
                    color_picker::AreaThumbValueInput {
                        class: "dx-color-picker-area-input",
                    }
                }
            }
            {props.children}
        }
    }
}

/// The props for the [`ColorPickerSelect`] component.
#[derive(Props, Clone, PartialEq)]
pub struct ColorPickerSelectProps {
    /// Show the "Alpha" slider and make the hex field edit `#RRGGBBAA`.
    #[props(default)]
    pub alpha: ReadSignal<bool>,

    /// Additional attributes to extend the color picker select element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the color picker select element
    pub children: Element,
}

#[component]
pub fn ColorPickerSelect(props: ColorPickerSelectProps) -> Element {
    let ctx = use_context::<ColorPickerContext>();
    let alpha = props.alpha;

    let base = attributes!(div { class: "dx-color-picker-dialog" });
    let merged = merge_attributes(vec![base, props.attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/color_picker/style.css") }
        div {
            ..merged,
            ColorArea {}
            ColorSlider { title: "Hue" }
            if alpha() {
                ColorAlphaSlider {}
            }
            div {
                class: "dx-color-picker-input",
                ColorField { label: "Hex", alpha }
                ColorSwatch { color: ctx.color() }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hsva(r: u8, g: u8, b: u8, a: f64) -> Hsva<encoding::Srgb, f64> {
        Hsva::from_color(Srgba::new(r, g, b, 255).into_format::<f64, f64>()).with_alpha(a)
    }

    #[test]
    fn format_hex_shows_alpha_only_when_asked() {
        let c = hsva(0x12, 0x34, 0x56, 0.5);
        assert_eq!(format_hex(c, false), "#123456");
        assert_eq!(format_hex(c, true), "#12345680");
        assert_eq!(format_hex(hsva(255, 0, 0, 1.0), true), "#FF0000FF");
        assert_eq!(format_hex(hsva(0, 0, 0, 0.0), true), "#00000000");
    }

    #[test]
    fn format_hex_auto_adds_alpha_only_for_transparency() {
        assert_eq!(format_hex_auto(hsva(0x12, 0x34, 0x56, 1.0)), "#123456");
        assert_eq!(format_hex_auto(hsva(0x12, 0x34, 0x56, 0.5)), "#12345680");
    }

    #[test]
    fn parse_hex_with_alpha() {
        let p = parse_hex("#11223380", true).unwrap();
        assert_eq!((p.rgb, p.alpha), (Color::new(0x11, 0x22, 0x33), Some(0x80)));
        // `#RRGGBB` and `#RGB` are opaque.
        assert_eq!(parse_hex("#112233", true).unwrap().alpha, Some(255));
        assert_eq!(parse_hex("#123", true).unwrap().alpha, Some(255));
        // `#RGBA` doubles each digit.
        let p = parse_hex("#1238", true).unwrap();
        assert_eq!((p.rgb, p.alpha), (Color::new(0x11, 0x22, 0x33), Some(0x88)));
        // Lengths that are not a hex form, and non-hex digits, are rejected.
        assert_eq!(parse_hex("#11223", true), None);
        assert_eq!(parse_hex("#1122334", true), None);
        assert_eq!(parse_hex("#GGGGGG", true), None);
        assert_eq!(parse_hex("", true), None);
    }

    #[test]
    fn parse_hex_without_alpha_keeps_the_alpha() {
        let p = parse_hex("#112233", false).unwrap();
        assert_eq!((p.rgb, p.alpha), (Color::new(0x11, 0x22, 0x33), None));
        assert_eq!(parse_hex("#11223380", false), None);
        assert_eq!(parse_hex("#1238", false), None);
    }

    #[test]
    fn apply_hex_sets_or_keeps_the_alpha() {
        let current = hsva(1, 2, 3, 0.25);
        let set = apply_hex(current, parse_hex("#FF000080", true).unwrap());
        assert_eq!(format_hex(set, true), "#FF000080");
        let kept = apply_hex(current, parse_hex("#FF0000", false).unwrap());
        assert_eq!(kept.alpha, 0.25);
        assert_eq!(format_hex(kept, false), "#FF0000");
        let opaque = apply_hex(current, parse_hex("#FF0000", true).unwrap());
        assert_eq!(opaque.alpha, 1.0);
    }

    #[test]
    fn hex_matches_compares_alpha_only_when_the_text_has_one() {
        let ext = hsva(0x12, 0x34, 0x56, 0.5);
        assert!(hex_matches(parse_hex("#12345680", true).unwrap(), ext));
        assert!(!hex_matches(parse_hex("#123456FF", true).unwrap(), ext));
        assert!(hex_matches(parse_hex("#123456", false).unwrap(), ext));
        assert!(!hex_matches(parse_hex("#123457", false).unwrap(), ext));
    }
}
