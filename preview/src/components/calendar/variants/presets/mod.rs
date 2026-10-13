use super::super::component::*;
use crate::components::button::Button;
use dioxus::prelude::*;
use time::{macros::date, Date, Duration, UtcDateTime};

/// Preset buttons drive the controlled `selected_date` (and `view_date`) of a `Calendar`.
///
/// The presets are relative to the real current date, read when a button is clicked, never while
/// rendering: the calendar starts on a fixed month with nothing selected, so the static HTML and
/// the hydrated DOM are identical whatever day the page is built or opened.
#[component]
pub fn Demo() -> Element {
    let mut selected_date = use_signal(|| None::<Date>);
    let mut view_date = use_signal(|| date!(2026 - 05 - 15));

    let mut pick = move |days: i64| {
        let target = UtcDateTime::now().date() + Duration::days(days);
        selected_date.set(Some(target));
        view_date.set(target);
    };

    rsx! {
        div { style: "display: grid; gap: 0.75rem; justify-items: start; padding: 20px;",
            div { style: "display: flex; flex-wrap: wrap; gap: 0.5rem;",
                Button {
                    r#type: "button",
                    "data-style": "outline",
                    onclick: move |_| pick(0),
                    "Today"
                }
                Button {
                    r#type: "button",
                    "data-style": "outline",
                    onclick: move |_| pick(1),
                    "Tomorrow"
                }
                Button {
                    r#type: "button",
                    "data-style": "outline",
                    onclick: move |_| pick(7),
                    "In a week"
                }
            }
            Calendar {
                selected_date: selected_date(),
                on_date_change: move |date| selected_date.set(date),
                view_date: view_date(),
                on_view_change: move |new_view: Date| view_date.set(new_view),
            }
            p { style: "margin: 0; font-size: 0.875rem; color: var(--dx-muted-foreground);",
                if let Some(date) = selected_date() {
                    "Selected: {date}"
                } else {
                    "Pick a date or use a preset."
                }
            }
        }
    }
}
