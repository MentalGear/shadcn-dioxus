use super::super::component::*;
use dioxus::prelude::*;
use time::{Date, Month, UtcDateTime};

/// shadcn's `calendar-week-numbers` (`showWeekNumber`): a week-number column in front of every
/// row, on January 12th of the current year (a month whose first and last rows straddle the year
/// end). The week starts on Sunday, so week 1 is the week that holds January 1st;
/// `first_day_of_week: Weekday::Monday` switches to ISO 8601 numbering.
#[component]
pub fn Demo() -> Element {
    let year = UtcDateTime::now().date().year();
    let january_12 = Date::from_calendar_date(year, Month::January, 12).expect("valid date");
    let mut selected_date = use_signal(move || Some(january_12));
    let mut view_date = use_signal(move || january_12);
    rsx! {
        div { style: "display: flex; justify-content: center;",
            Calendar {
                selected_date: selected_date(),
                on_date_change: move |date| selected_date.set(date),
                view_date: view_date(),
                on_view_change: move |new_view: Date| view_date.set(new_view),
                show_week_number: true,
            }
        }
    }
}
