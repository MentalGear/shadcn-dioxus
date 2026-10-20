//! A justified photo wall: 20,000 generated photos of mixed aspect ratios in rows that fill the
//! width edge to edge, every photo uncropped at its own aspect. A slider sets the target row
//! height; a switch turns the wall into a canvas six times as wide as the demo, scrolling both
//! ways (rows windowed by `y`, the tiles of each row by `x`).
use super::super::component::*;
use crate::components::field::{Field, FieldLabel, FieldOrientation};
use crate::components::slider::Slider;
use crate::components::switch::Switch;
use dioxus::prelude::*;

/// How many photos the wall holds.
const PHOTOS: usize = 20_000;
/// The wall's canvas width in wall mode, in multiples of the demo's width.
const WALL_FACTOR: f64 = 6.0;

/// Deterministic aspect ratio (width, height) of photo `index`: portrait 2:3 and 3:4, square,
/// landscape 4:3, 3:2 and 16:9, and the occasional 3:1 panorama.
fn aspect(index: usize) -> (u32, u32) {
    const SET: [(u32, u32); 10] = [
        (2, 3),
        (3, 2),
        (1, 1),
        (4, 3),
        (3, 4),
        (3, 2),
        (16, 9),
        (4, 3),
        (2, 3),
        (3, 1),
    ];
    let mixed = index.wrapping_mul(2_654_435_761) >> 7;
    SET[(mixed ^ (index / 7)) % SET.len()]
}

/// The `data:` URI of generated photo `index`: a sky gradient, a sun and two hills drawn at the
/// photo's own aspect ratio, colours and positions derived from the index. Nothing is fetched.
fn photo_src(index: usize) -> String {
    let (aw, ah) = aspect(index);
    let w = 100.0 * aw as f32 / ah as f32;
    let hue = (index * 47) % 360;
    let peak = 48 + (index * 29) % 22;
    let sun_x = w * (0.2 + ((index * 37) % 60) as f32 / 100.0);
    let sun_y = 16 + (index * 13) % 22;
    let (a, b, c) = (w * 0.3, w * 0.6, w * 0.55);
    let svg = format!(
        "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 {w:.1} 100' preserveAspectRatio='none'>\
         <defs><linearGradient id='g' x1='0' y1='0' x2='0' y2='1'>\
         <stop offset='0' stop-color='hsl({hue} 70% 72%)'/><stop offset='1' stop-color='hsl({h2} 75% 88%)'/>\
         </linearGradient></defs><rect width='{w:.1}' height='100' fill='url(#g)'/>\
         <circle cx='{sun_x:.1}' cy='{sun_y}' r='8' fill='hsl(48 95% 62%)'/>\
         <path d='M0 {p1} Q {a:.1} {p2} {b:.1} {p3} T {w:.1} {peak} V 100 H 0 Z' fill='hsl({h3} 35% 55%)'/>\
         <path d='M0 82 Q {a:.1} {p4} {c:.1} 80 T {w:.1} 78 V 100 H 0 Z' fill='hsl({h4} 45% 38%)'/></svg>",
        h2 = (hue + 40) % 360,
        h3 = (hue + 130) % 360,
        h4 = (hue + 150) % 360,
        p1 = peak + 12,
        p2 = peak - 8,
        p3 = peak + 6,
        p4 = peak + 4,
    );
    let encoded = svg
        .replace('%', "%25")
        .replace('#', "%23")
        .replace('<', "%3C")
        .replace('>', "%3E");
    format!("data:image/svg+xml,{encoded}")
}

#[component]
pub fn Demo() -> Element {
    let aspects = use_signal(|| {
        (0..PHOTOS)
            .map(|index| {
                let (w, h) = aspect(index);
                f64::from(w) / f64::from(h)
            })
            .collect::<Vec<f64>>()
    });
    let mut row_height = use_signal(|| 160.0_f64);
    let mut wall = use_signal(|| false);
    let mut demo_width = use_signal(|| 800.0_f64);
    let canvas_width = use_memo(move || wall().then(|| (demo_width() * WALL_FACTOR).round()));

    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: var(--dx-space-3); width: 100%; min-width: 0;",
            onresize: move |event: Event<ResizeData>| {
                if let Ok(size) = event.data().get_content_box_size() {
                    if size.width > 0.0 {
                        demo_width.set(size.width);
                    }
                }
            },
            div { style: "display: flex; flex-wrap: wrap; align-items: center; gap: var(--dx-space-3) var(--dx-space-6);",
                div { style: "display: flex; flex: 1 1 14rem; flex-direction: column; gap: var(--dx-space-2); max-width: 20rem;",
                    div { style: "display: flex; justify-content: space-between; font-size: var(--dx-text-sm); font-weight: 500;",
                        span { "Target row height" }
                        span { style: "color: var(--dx-muted-foreground); font-variant-numeric: tabular-nums;",
                            "{row_height:.0}px"
                        }
                    }
                    Slider {
                        label: "Target row height",
                        horizontal: true,
                        min: 80.0,
                        max: 320.0,
                        step: 10.0,
                        default_value: 160.0,
                        on_value_change: move |value: f64| row_height.set(value),
                    }
                }
                Field { orientation: FieldOrientation::Horizontal,
                    Switch {
                        id: "virtual-grid-justified-wall",
                        checked: wall(),
                        on_checked_change: move |value| wall.set(value),
                    }
                    FieldLabel { html_for: "virtual-grid-justified-wall", "Wall (both axes)" }
                }
            }
            VirtualJustifiedGrid {
                aspects,
                target_row_height: row_height,
                canvas_width,
                render_tile: |(index, _rect): (usize, TileRect)| {
                    let (w, h) = aspect(index);
                    rsx! {
                        img {
                            src: photo_src(index),
                            alt: "Generated photo {index + 1}, {w}:{h}",
                            draggable: "false",
                            style: "display: block; width: 100%; height: 100%; border-radius: inherit;",
                        }
                    }
                },
                "aria-label": "Justified photo wall, 20,000 generated photos",
                style: "height: 28rem;",
            }
        }
    }
}
