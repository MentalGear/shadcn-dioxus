use super::super::component::*;
use crate::components::calendar::CaptionLayout;
use dioxus::prelude::*;
use time::{macros::date, Date};

/// shadcn's date-of-birth picker: `caption_layout` reaches the calendar inside the popover, so a
/// far-away date is one or two picks instead of a long run of arrow clicks. The three dropdown
/// layouts side by side; `Dropdown` is the shadcn demo's `captionLayout="dropdown"`.
#[component]
pub fn Demo() -> Element {
    let layouts = [
        ("Month and year", CaptionLayout::Dropdown),
        ("Month only", CaptionLayout::DropdownMonths),
        ("Year only", CaptionLayout::DropdownYears),
    ];
    rsx! {
        div { style: "display: flex; flex-wrap: wrap; gap: 24px;",
            for (name , layout) in layouts {
                BirthExample { key: "{name}", name, layout }
            }
        }
    }
}

#[component]
fn BirthExample(name: &'static str, layout: CaptionLayout) -> Element {
    let mut selected_date = use_signal(|| None::<Date>);
    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 8px; align-items: flex-start;",
            "data-caption-layout": name,
            span { style: "font-size: 0.875rem; color: var(--dx-muted-foreground);", "{name}" }
            DatePicker {
                selected_date: selected_date(),
                on_value_change: move |v| selected_date.set(v),
                caption_layout: layout,
                min_date: date!(1925 - 01 - 01),
                max_date: date!(2026 - 12 - 31),
            }
        }
    }
}
