use super::super::component::*;
use dioxus::prelude::*;
use time::{macros::date, Date};

/// shadcn's `--cell-size`: the whole month (day cells, weekday row, caption height, the nav
/// buttons and the month width) reads `--dx-calendar-cell-size`, so one custom property retunes
/// it. The default is `--dx-space-7` (28px); the second calendar sets `2.5rem`.
#[component]
pub fn Demo() -> Element {
    let sizes = [("Default (28px)", None), ("Large (2.5rem)", Some("2.5rem"))];
    rsx! {
        div { style: "display: flex; flex-wrap: wrap; gap: 24px; justify-content: center;",
            for (name , size) in sizes {
                CellSizeExample { key: "{name}", name, size }
            }
        }
    }
}

#[component]
fn CellSizeExample(name: &'static str, size: Option<&'static str>) -> Element {
    let mut selected_date = use_signal(|| None::<Date>);
    let mut view_date = use_signal(|| date!(2026 - 10 - 15));
    let style = size.map(|s| format!("--dx-calendar-cell-size: {s};"));
    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 8px; align-items: flex-start;",
            "data-cell-size": name,
            span { style: "font-size: 0.875rem; color: var(--dx-muted-foreground);", "{name}" }
            Calendar {
                selected_date: selected_date(),
                on_date_change: move |date| selected_date.set(date),
                view_date: view_date(),
                on_view_change: move |new_view: Date| view_date.set(new_view),
                style,
            }
        }
    }
}
