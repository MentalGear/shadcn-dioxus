use super::super::component::*;
use dioxus::prelude::*;

use time::Date;

/// shadcn's `date-picker-demo` / `date-picker-basic`: no footer (`confirm: false`) and the popover
/// STAYS OPEN after a date is picked (`close_on_select: false`), so several can be tried without
/// reopening it. Escape, an outside click and the trigger still close it. `DateRangePicker` takes
/// the same props (`close_on_select` applies once the range's end date is picked).
#[component]
pub fn Demo() -> Element {
    let mut selected_date = use_signal(|| None::<Date>);

    rsx! {
        div {
            DatePicker {
                selected_date: selected_date(),
                on_value_change: move |v| {
                    tracing::info!("Selected date changed: {:?}", v);
                    selected_date.set(v);
                },
                confirm: false,
                close_on_select: false,
            }
        }
    }
}
