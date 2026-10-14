use super::super::component::*;
use dioxus::prelude::*;

use time::Date;

/// The library default: selections are STAGED behind a Cancel / Apply footer. Picking a day
/// highlights it in the calendar but leaves the input alone; Apply commits it and closes, Cancel,
/// Escape and an outside click restore the previous value and close. (shadcn's date pickers have
/// no footer -- the other demos switch it off with `confirm: false` to mirror theirs.)
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
            }
        }
    }
}
