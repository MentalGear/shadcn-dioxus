use super::super::component::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use dioxus::prelude::*;

/// Controlled `sizes`: the group paints the percentages it is given and reports every drag or key
/// step through `on_sizes_change`, so the app can read, show and reset them.
#[component]
pub fn Demo() -> Element {
    let mut sizes = use_signal(|| vec![50.0, 50.0]);

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 1rem; width: 100%; max-width: 28rem;",
            ResizablePanelGroup {
                direction: ResizableDirection::Horizontal,
                height: "8rem",
                border: "1px solid var(--dx-input)",
                border_radius: "var(--dx-radius-lg)",
                overflow: "hidden",
                sizes: Some(sizes()),
                on_sizes_change: move |next: Vec<f64>| sizes.set(next),
                ResizablePanel { index: 0usize, min_size: 20.0,
                    div { style: "display: flex; align-items: center; justify-content: center; height: 100%;",
                        span { style: "font-weight: 600;", "One" }
                    }
                }
                ResizableHandle {
                    index: 0usize,
                    aria_label: "Controlled",
                    span { class: "dx-resizable-handle-grip" }
                }
                ResizablePanel { index: 1usize, min_size: 20.0,
                    div { style: "display: flex; align-items: center; justify-content: center; height: 100%;",
                        span { style: "font-weight: 600;", "Two" }
                    }
                }
            }
            div { style: "display: flex; align-items: center; gap: 0.75rem; font-size: 0.875rem;",
                span { id: "resizable-controlled-sizes",
                    {sizes().iter().map(|s| format!("{s:.0}%")).collect::<Vec<_>>().join(" / ")}
                }
                Button {
                    id: "resizable-controlled-reset",
                    variant: ButtonVariant::Outline,
                    size: ButtonSize::Sm,
                    onclick: move |_| sizes.set(vec![50.0, 50.0]),
                    "Reset"
                }
            }
        }
    }
}
