use super::super::component::*;
use dioxus::prelude::*;

use dioxus_primitives::calendar::DateRange;

/// The library default for a range: a Cancel / Apply footer, with Apply disabled until BOTH ends
/// are picked. (shadcn's range demo, which has no footer, is `multi_month`.)
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
            }
        }
    }
}
