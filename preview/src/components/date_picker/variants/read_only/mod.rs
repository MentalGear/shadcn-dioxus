use super::super::component::*;
use dioxus::prelude::*;
use dioxus_primitives::calendar::DateRange;
use time::{Date, Month};

/// `read_only` on `DatePicker` and `DateRangePicker`, like a native `readonly` input: the segments
/// keep focus and selection (the arrow keys still move between them), but typing, the Up and Down
/// steps, Backspace and the calendar popover change nothing. The counter shows that
/// `on_value_change` is never called.
#[component]
pub fn Demo() -> Element {
    let mut changes = use_signal(|| 0u32);
    let date = Date::from_calendar_date(2026, Month::May, 7).expect("valid date");
    let end = Date::from_calendar_date(2026, Month::May, 11).expect("valid date");

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 1rem; align-items: flex-start;",
            DatePicker {
                selected_date: Some(date),
                read_only: true,
                on_value_change: move |_| changes += 1,
            }
            DateRangePicker {
                selected_range: Some(DateRange::new(date, end)),
                read_only: true,
                on_range_change: move |_| changes += 1,
            }
            p { id: "read-only-changes", "data-changes": "{changes}", "Edits reported: {changes}" }
        }
    }
}
