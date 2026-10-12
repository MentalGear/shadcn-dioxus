use super::super::component::*;
use crate::components::input::Input;
use crate::components::label::Label;
use dioxus::prelude::*;
use time::Date;

/// A `DatePicker` for the date next to an `Input` of `type="time"` for the time, with the
/// combined value shown underneath.
#[component]
pub fn Demo() -> Element {
    let mut selected_date = use_signal(|| None::<Date>);
    let mut time = use_signal(|| "10:30".to_string());

    rsx! {
        div { style: "display: grid; gap: 0.75rem; justify-items: start;",
            div { style: "display: flex; flex-wrap: wrap; align-items: center; gap: 0.75rem;",
                DatePicker {
                    selected_date: selected_date(),
                    on_value_change: move |v| selected_date.set(v),
                }
                Label { html_for: "date-picker-time-time", "Time" }
                Input {
                    id: "date-picker-time-time",
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
