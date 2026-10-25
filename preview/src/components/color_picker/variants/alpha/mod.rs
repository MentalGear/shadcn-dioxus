use dioxus::prelude::*;

use super::super::component::*;
use dioxus_primitives::color_picker::Color;
use palette::{encoding, Hsv, Hsva, IntoColor, WithAlpha};

#[component]
pub fn Demo() -> Element {
    let mut color = use_signal(|| -> Hsva<encoding::Srgb, f64> {
        let opaque: Hsv<encoding::Srgb, f64> =
            Color::new(236, 72, 153).into_format::<f64>().into_color();
        opaque.with_alpha(0.5)
    });

    rsx! {
        ColorPicker {
            label: "Pick a color with alpha",
            alpha: true,
            color: color(),
            on_color_change: move |c| color.set(c),
        }
    }
}
