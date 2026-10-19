use super::super::component::*;
use dioxus::prelude::*;
use time::{macros::date, Date};

/// shadcn's `calendar-caption` (`captionLayout="dropdown"`), plus the other three layouts of
/// react-day-picker's `captionLayout` side by side: `label` (the default), `dropdown`,
/// `dropdown-months` and `dropdown-years`.
#[component]
pub fn Demo() -> Element {
    let layouts = [
        ("Label", CaptionLayout::Label),
        ("Dropdown", CaptionLayout::Dropdown),
        ("Dropdown months", CaptionLayout::DropdownMonths),
        ("Dropdown years", CaptionLayout::DropdownYears),
    ];
    rsx! {
        div { style: "display: flex; flex-wrap: wrap; gap: 24px;",
            for (name , layout) in layouts {
                CaptionExample { key: "{name}", name, layout }
            }
        }
    }
}

#[component]
fn CaptionExample(name: &'static str, layout: CaptionLayout) -> Element {
    let mut selected_date = use_signal(|| None::<Date>);
    let mut view_date = use_signal(|| date!(2026 - 05 - 15));
    rsx! {
        div {
            style: "display: flex; flex-direction: column; gap: 8px; align-items: flex-start;",
            "data-caption-layout": name,
            span { style: "font-size: 0.875rem; color: var(--dx-muted-foreground);", "{name}" }
            Calendar {
                selected_date: selected_date(),
                on_date_change: move |date| selected_date.set(date),
                view_date: view_date(),
                on_view_change: move |new_view: Date| view_date.set(new_view),
                caption_layout: layout,
                min_date: date!(1995 - 07 - 21),
                max_date: date!(2035 - 09 - 11),
            }
        }
    }
}
