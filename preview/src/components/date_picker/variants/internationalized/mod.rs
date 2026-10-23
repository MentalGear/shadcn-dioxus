use super::super::component::*;
use crate::components::calendar::CaptionLayout;
use crate::i18n::{use_locale_root, LocaleDemo};
use dioxus::prelude::*;

use dioxus_i18n::tid;
use time::{Date, Month, Weekday};

/// shadcn's `date-picker-rtl` twin for the localisation side: no footer and the popover STAYS OPEN
/// after a pick, like every uncontrolled shadcn date picker.
#[component]
pub fn Demo() -> Element {
    use_locale_root();
    let mut selected_date = use_signal(|| None::<Date>);

    rsx! {
        LocaleDemo {
            DatePicker {
                selected_date: selected_date(),
                on_value_change: move |v| {
                    tracing::info!("Selected date changed: {:?}", v);
                    selected_date.set(v);
                },
                confirm: false,
                close_on_select: false,
                on_format_day_placeholder: || tid!("D_Abbr"),
                on_format_month_placeholder: || tid!("M_Abbr"),
                on_format_year_placeholder: || tid!("Y_Abbr"),
                // backlog row 84 finding 2: the month segment's
                // `aria-valuetext` needs a locale-aware month name, the same
                // way the standalone `Calendar` internationalized demo
                // already feeds its grid's month title
                // (`calendar/variants/internationalized/mod.rs`'s identical
                // `on_format_month: |month: Month| tid!(& month.to_string())`).
                on_format_month: |month: Month| tid!(& month.to_string()),
                // The popover's calendar takes the same locale: weekday headers, the caption, and
                // the month dropdown's short names from their own keys (French "Juin" and "Juil."
                // would both cut to "Jui" from the full name).
                on_format_weekday: |weekday: Weekday| tid!(& weekday.to_string()),
                on_format_month_short: |month: Month| tid!(& format!("{month}_Short")),
                caption_layout: CaptionLayout::Dropdown,
            }
        }
    }
}
