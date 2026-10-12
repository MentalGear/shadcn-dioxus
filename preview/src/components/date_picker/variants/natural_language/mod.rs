use crate::components::calendar::Calendar;
use crate::components::input::Input;
use crate::components::label::Label;
use crate::components::popover::{PopoverContent, PopoverRoot, PopoverTrigger};
use dioxus::prelude::*;
use dioxus_icons::lucide::CalendarDays;
use dioxus_primitives::ContentAlign;
use time::{macros::date, Date, Duration, UtcDateTime, Weekday};

/// A text `Input` that understands a handful of phrases ("today", "tomorrow", "in 3 days",
/// "next friday", ...), plus a `Popover` holding a `Calendar` for picking by hand. The two stay
/// in sync: typing selects the parsed date in the calendar, picking a day writes it into the
/// input. This is a small hand-written parser, not a natural-language library.
#[component]
pub fn Demo() -> Element {
    let mut text = use_signal(String::new);
    let mut selected_date = use_signal(|| None::<Date>);
    let mut view_date = use_signal(|| date!(2026 - 05 - 15));
    let mut open = use_signal(|| false);

    rsx! {
        div { style: "display: grid; gap: 0.5rem; width: 100%; max-width: 20rem;",
            Label { html_for: "date-picker-natural-language-input", "Schedule date" }
            div { style: "display: flex; gap: 0.5rem;",
                Input {
                    id: "date-picker-natural-language-input",
                    placeholder: "Tomorrow or next week",
                    value: text,
                    oninput: move |e: FormEvent| {
                        let value = e.value();
                        let parsed = parse_phrase(&value, UtcDateTime::now().date());
                        if let Some(date) = parsed {
                            view_date.set(date);
                        }
                        selected_date.set(parsed);
                        text.set(value);
                    },
                }
                PopoverRoot {
                    is_modal: false,
                    open: Some(open()),
                    on_open_change: move |v| open.set(v),
                    PopoverTrigger {
                        aria_label: "Choose the schedule date from a calendar",
                        CalendarDays { size: "16px" }
                    }
                    // One surface, as in `DatePicker`'s own popover (date_picker/style.css): the
                    // panel is the card, so it hugs the calendar and the calendar drops its own
                    // border, background and shadow (calendar/style.css reads these three).
                    PopoverContent {
                        align: ContentAlign::End,
                        style: "width: max-content; padding: 0; --dx-calendar-border: none; --dx-calendar-background: transparent; --dx-calendar-shadow: none;",
                        Calendar {
                            selected_date: selected_date(),
                            on_date_change: move |date: Option<Date>| {
                                if let Some(date) = date {
                                    text.set(format_long(date));
                                }
                                selected_date.set(date);
                                open.set(false);
                            },
                            view_date: view_date(),
                            on_view_change: move |new_view: Date| view_date.set(new_view),
                        }
                    }
                }
            }
            p { style: "margin: 0; font-size: 0.875rem; color: var(--dx-muted-foreground);",
                if let Some(date) = selected_date() {
                    "Your date is {format_long(date)}."
                } else {
                    "Try \"today\", \"tomorrow\", \"in 3 days\" or \"next friday\"."
                }
            }
        }
    }
}

/// `Friday, October 16, 2026`.
fn format_long(date: Date) -> String {
    format!(
        "{}, {} {}, {}",
        date.weekday(),
        date.month(),
        date.day(),
        date.year()
    )
}

/// Turns a short phrase into a date relative to `today`. Understands `today`, `tomorrow`,
/// `yesterday`, `in N day(s)/week(s)`, `in a week`, `next week` and `[next] <weekday>`
/// (the next such weekday strictly after today). Anything else is `None`.
fn parse_phrase(input: &str, today: Date) -> Option<Date> {
    let lowered = input.trim().to_lowercase();
    let words: Vec<&str> = lowered.split_whitespace().collect();
    match words.as_slice() {
        ["today"] => Some(today),
        ["tomorrow"] => Some(today + Duration::days(1)),
        ["yesterday"] => Some(today - Duration::days(1)),
        ["next", "week"] | ["in", "a", "week"] => Some(today + Duration::weeks(1)),
        ["in", n, unit] => {
            let n: i64 = n.parse().ok()?;
            match *unit {
                "day" | "days" => today.checked_add(Duration::days(n)),
                "week" | "weeks" => today.checked_add(Duration::weeks(n)),
                _ => None,
            }
        }
        ["next", day] | [day] => {
            let target = weekday_from_name(day)?;
            let ahead = (i64::from(target.number_days_from_monday())
                - i64::from(today.weekday().number_days_from_monday()))
            .rem_euclid(7);
            Some(today + Duration::days(if ahead == 0 { 7 } else { ahead }))
        }
        _ => None,
    }
}

fn weekday_from_name(name: &str) -> Option<Weekday> {
    Some(match name {
        "monday" | "mon" => Weekday::Monday,
        "tuesday" | "tue" => Weekday::Tuesday,
        "wednesday" | "wed" => Weekday::Wednesday,
        "thursday" | "thu" => Weekday::Thursday,
        "friday" | "fri" => Weekday::Friday,
        "saturday" | "sat" => Weekday::Saturday,
        "sunday" | "sun" => Weekday::Sunday,
        _ => return None,
    })
}
