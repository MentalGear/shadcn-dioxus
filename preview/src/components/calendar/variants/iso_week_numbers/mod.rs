use super::super::component::*;
use dioxus::prelude::*;
use time::{macros::date, Date, Weekday};

/// ISO 8601 week numbers: `first_day_of_week: Weekday::Monday` with `show_week_number`. January
/// 2027 is the case that tells the numberings apart: the first row (Dec 28 - Jan 3) is ISO week 53
/// of 2026 because January 1st is a Friday, and week 1 starts on Monday January 4th. With a
/// Sunday start (the `week_numbers` variant) that same row would be week 1.
#[component]
pub fn Demo() -> Element {
    let mut selected_date = use_signal(|| None::<Date>);
    let mut view_date = use_signal(|| date!(2027 - 01 - 01));
    rsx! {
        div { style: "display: flex; justify-content: center;",
            Calendar {
                selected_date: selected_date(),
                on_date_change: move |date| selected_date.set(date),
                view_date: view_date(),
                on_view_change: move |new_view: Date| view_date.set(new_view),
                first_day_of_week: Weekday::Monday,
                show_week_number: true,
            }
        }
    }
}
