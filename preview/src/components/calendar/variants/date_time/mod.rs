use super::super::component::*;
use crate::components::input::Input;
use crate::components::label::Label;
use dioxus::prelude::*;
use time::{macros::date, Date};

/// A `Calendar` for the date and an `Input` of `type="time"` for the time, with the combined
/// value shown underneath. The calendar starts on a fixed month so the static HTML and the
/// hydrated DOM match whatever day it is.
#[component]
pub fn Demo() -> Element {
    let mut selected_date = use_signal(|| None::<Date>);
    let mut view_date = use_signal(|| date!(2026 - 05 - 15));
    let mut time = use_signal(|| "10:30".to_string());

    rsx! {
        div { style: "display: grid; gap: 0.75rem; justify-items: start; padding: 20px;",
            Calendar {
                selected_date: selected_date(),
                on_date_change: move |date| selected_date.set(date),
                view_date: view_date(),
                on_view_change: move |new_view: Date| view_date.set(new_view),
            }
            div { style: "display: flex; align-items: center; gap: 0.75rem;",
                Label { html_for: "calendar-date-time-time", "Time" }
                Input {
                    id: "calendar-date-time-time",
                    r#type: "time",
                    value: time,
                    oninput: move |e: FormEvent| time.set(e.value()),
                }
            }
            p { style: "margin: 0; font-size: 0.875rem; color: var(--dx-muted-foreground);",
                if let Some(date) = selected_date() {
                    "Selected: {date} {time}"
                } else {
                    "Pick a date, then a time."
                }
            }
        }
    }
}
