use super::super::component::*;
use crate::components::button::{Button, ButtonVariant};
use dioxus::prelude::*;

/// The row counts the buttons switch between.
const COUNTS: [usize; 2] = [10_000, 100_000];

fn row_title(idx: usize, rows: usize) -> String {
    format!("Row {} of {rows}", idx + 1)
}

#[component]
pub fn Demo() -> Element {
    let mut rows = use_signal(|| COUNTS[0]);

    rsx! {
        div { class: "dx-virtual-list-demo", "data-vl-demo": "find-in-page",
            p { class: "dx-virtual-list-subtitle",
                "Experimental: a windowed list that keeps each unmounted row's text in the DOM as a hidden=\"until-found\" placeholder. Search the page (Ctrl+F) for \"Row 7777 of\": the browser reveals the placeholder, and the list scrolls the real row into view."
            }
            style { r#".dx-virtual-list-container {{
  position: relative;
  max-height: 24rem;
  contain: layout paint;
  overflow-y: auto;
}}

.dx-virtual-list-demo {{
  display: flex;
  flex-direction: column;
  margin: 0 auto;
  gap: 0.75rem;
}}

.dx-virtual-list-demo .dx-virtual-list-subtitle {{
  margin: 0;
  color: var(--dx-muted-foreground);
  font-size: 0.9rem;
}}

.dx-virtual-list-find-row {{
  box-sizing: border-box;
  height: 48px;
  padding: 0.75rem 0.9rem;
  border-bottom: 1px solid var(--dx-border);
  color: var(--dx-foreground);
  font-size: 0.875rem;
}}"# }
            VirtualList {
                count: rows,
                virtualize: VirtualizeMode::Windowed,
                buffer: 8usize,
                estimate_size: |_| 48,
                // The same text the row shows, so a find-in-page match lands on the real row.
                text_value: move |idx: usize| row_title(idx, rows()),
                "aria-label": "Searchable windowed list",
                render_item: move |idx: usize| rsx! {
                    div { class: "dx-virtual-list-find-row", "{row_title(idx, rows())}" }
                },
            }
            div { style: "display: flex; flex-wrap: wrap; align-items: center; gap: var(--dx-space-2);",
                for count in COUNTS {
                    Button {
                        key: "{count}",
                        variant: if rows() == count { ButtonVariant::Secondary } else { ButtonVariant::Outline },
                        "aria-pressed": (rows() == count).to_string(),
                        onclick: move |_| rows.set(count),
                        "{count} rows"
                    }
                }
            }
        }
    }
}
