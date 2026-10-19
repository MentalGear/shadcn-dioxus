use super::super::component::*;
use dioxus::prelude::*;

const PHOTOS: usize = 5_000;

/// One generated "photo": a gradient sky, a sun and two hills whose colours and positions are
/// derived from the index, so the demo ships no image files and loads nothing remote.
#[component]
fn Photo(index: usize) -> Element {
    let hue = (index * 47) % 360;
    let sky_top = format!("hsl({hue} 70% 72%)");
    let sky_bottom = format!("hsl({} 75% 88%)", (hue + 40) % 360);
    let hill = format!("hsl({} 45% 38%)", (hue + 150) % 360);
    let far_hill = format!("hsl({} 35% 55%)", (hue + 130) % 360);
    let sun_x = 20 + (index * 37) % 60;
    let sun_y = 18 + (index * 13) % 20;
    let peak = 40 + (index * 29) % 25;
    let gradient = format!("sky-{index}");
    rsx! {
        svg {
            view_box: "0 0 100 75",
            role: "img",
            "aria-label": "Generated landscape {index + 1}",
            style: "display: block; width: 100%; height: auto; aspect-ratio: 4 / 3; border-radius: var(--dx-radius-md);",
            defs {
                linearGradient { id: "{gradient}", x1: "0", y1: "0", x2: "0", y2: "1",
                    stop { offset: "0", stop_color: "{sky_top}" }
                    stop { offset: "1", stop_color: "{sky_bottom}" }
                }
            }
            rect { width: "100", height: "75", fill: "url(#{gradient})" }
            circle { cx: "{sun_x}", cy: "{sun_y}", r: "7", fill: "hsl(48 95% 62%)" }
            path { d: "M0 {peak + 10} Q 30 {peak - 8} 60 {peak + 6} T 100 {peak} V 75 H 0 Z", fill: "{far_hill}" }
            path { d: "M0 62 Q 25 {peak + 2} 55 60 T 100 58 V 75 H 0 Z", fill: "{hill}" }
        }
    }
}

#[component]
pub fn Demo() -> Element {
    let mut columns = use_signal(|| 4usize);

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: var(--dx-space-2); width: 100%;",
            div { role: "group", "aria-label": "Columns", style: "display: flex; gap: var(--dx-space-2); align-items: center; font-size: var(--dx-text-sm);",
                span { "Columns:" }
                for n in [2usize, 3, 4, 6] {
                    label { key: "{n}", style: "display: inline-flex; gap: var(--dx-space-1); align-items: center;",
                        input {
                            r#type: "radio",
                            name: "virtual-grid-columns",
                            checked: columns() == n,
                            onchange: move |_| columns.set(n),
                        }
                        "{n}"
                    }
                }
            }
            VirtualWrapGrid {
                count: PHOTOS,
                columns,
                // A 4:3 tile plus its caption; the grid measures the real height as rows render.
                estimate_row_height: move |_| 140,
                render_item: |index: usize| rsx! {
                    figure { style: "margin: 0; display: flex; flex-direction: column; gap: var(--dx-space-1);",
                        Photo { index }
                        figcaption { style: "color: var(--dx-muted-foreground); font-size: var(--dx-text-xs);",
                            "Photo {index + 1}"
                        }
                    }
                },
                "aria-label": "Photo gallery, 5,000 generated images",
                style: "height: 28rem;",
            }
        }
    }
}
