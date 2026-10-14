use super::super::component::*;
use dioxus::prelude::*;

use dioxus_primitives::calendar::DateRange;

/// shadcn's `date-picker-range` (`numberOfMonths={2}`): no footer and the popover STAYS OPEN, even
/// after both ends are picked.
#[component]
pub fn Demo() -> Element {
    let mut selected_range = use_signal(|| None::<DateRange>);

    rsx! {
        div {
            DateRangePicker {
                selected_range: selected_range(),
                on_range_change: move |range| {
                    tracing::info!("Selected range: {:?}", range);
                    selected_range.set(range);
                },
                month_count: 2,
                confirm: false,
                close_on_select: false,
            }
        }
    }
}
