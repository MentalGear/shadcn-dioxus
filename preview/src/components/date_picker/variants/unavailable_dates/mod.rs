use super::super::component::*;
use dioxus::prelude::*;

use dioxus_primitives::calendar::DateRange;
use time::{ext::NumericalDuration, Date, UtcDateTime};

/// Three unavailable ranges inside the month the picker opens on (the current one).
///
/// Computed from today, never hard-coded: a fixed date goes stale the day the calendar stops
/// opening on its month, and the demo then shows no unavailable day at all. The ranges are anchored
/// on today and the anchor slides back when today is near the end of the month, so the farthest
/// range (anchor + 17 days) always still lands inside the month in view. Near the end of the month
/// that slide can put today inside a range; that is harmless (an unavailable day is only a day
/// that cannot be picked), and the alternative -- dropping a range -- would leave the demo with
/// fewer unavailable days exactly when it is checked.
fn unavailable_ranges(today: Date) -> Vec<DateRange> {
    let days_in_month = today.month().length(today.year());
    let anchor_day = today.day().min(days_in_month - 17);
    let anchor = today.replace_day(anchor_day).unwrap_or(today);
    vec![
        DateRange::new(anchor + 2.days(), anchor + 4.days()),
        DateRange::new(anchor + 9.days(), anchor + 11.days()),
        DateRange::new(anchor + 16.days(), anchor + 17.days()),
    ]
}

/// A range picker with unavailable dates and no footer (`confirm: false`): the popover closes once
/// the range's end date is picked, after the default 300 ms dwell. (The footer version of a range is
/// the `range` demo.)
#[component]
pub fn Demo() -> Element {
    let mut selected_range = use_signal(|| None::<DateRange>);

    // Read from the clock after hydration, never while rendering: a render-time read is baked into the prerendered
    // page as the build day, and hydration keeps the server's markup (backlog row 377).
    let mut disabled_ranges = use_signal(Vec::<DateRange>::new);
    use_effect(move || disabled_ranges.set(unavailable_ranges(UtcDateTime::now().date())));

    rsx! {
        div {
            DateRangePicker {
                selected_range: selected_range(),
                on_range_change: move |range| {
                    tracing::info!("Selected range: {:?}", range);
                    selected_range.set(range);
                },
                disabled_ranges: disabled_ranges,
                confirm: false,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    fn all_in_month_of(today: Date) {
        let ranges = unavailable_ranges(today);
        assert_eq!(ranges.len(), 3);
        for range in ranges {
            for day in [range.start(), range.end()] {
                assert_eq!(
                    (day.year(), day.month()),
                    (today.year(), today.month()),
                    "{day} is outside {today}'s month"
                );
            }
        }
    }

    #[test]
    fn unavailable_ranges_stay_in_the_month_in_view_whatever_today_is() {
        // Every day of a long, a short and a leap February month, plus the year boundary.
        for (year, month) in [
            (2026, 10),
            (2026, 2),
            (2028, 2),
            (2026, 12),
            (2027, 1),
            (2026, 4),
        ] {
            let first =
                Date::from_calendar_date(year, time::Month::try_from(month as u8).unwrap(), 1)
                    .unwrap();
            let days = first.month().length(first.year());
            for day in 1..=days {
                all_in_month_of(first.replace_day(day).unwrap());
            }
        }
        all_in_month_of(date!(2026 - 10 - 05));
    }
}
