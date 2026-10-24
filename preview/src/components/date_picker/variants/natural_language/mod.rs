use crate::components::calendar::{Calendar, CaptionLayout};
use crate::components::field::{Field, FieldLabel};
use crate::components::input_group::{InputGroup, InputGroupAddon, InputGroupAlign, InputGroupInput};
use crate::components::popover::{PopoverContent, PopoverRoot};
use dioxus::prelude::*;
use dioxus_icons::lucide::CalendarDays;
use dioxus_primitives::{dioxus_attributes::attributes, popover::PopoverTrigger, ContentAlign};
use time::{macros::date, Date, Duration, Month, UtcDateTime, Weekday};

/// shadcn's "Natural Language Picker" (`date-picker-natural-language`): a text field that
/// understands phrases, with the calendar button INSIDE the field.
///
/// - the field starts as "In 2 days" and the sentence below it shows the date that parses to;
/// - every keystroke is parsed; a hit updates the date, text that does not parse KEEPS the
///   previous date (the sentence does not blank out while you are mid-word);
/// - ArrowDown in the field opens the calendar;
/// - the calendar button is a ghost icon button in the field's trailing addon, labelled
///   "Select date"; its popover is end-aligned;
/// - picking a day writes it into the field (`July 16, 2026`) and closes the popover.
///
/// shadcn parses with `chrono-node`; there is no Rust twin, so [`parse_phrase`] is a small
/// hand-written subset (see its doc). `confirm` is not involved: this is a plain `Calendar` in a
/// `Popover`, exactly as in shadcn, which closes on select.
#[component]
pub fn Demo() -> Element {
    let mut text = use_signal(|| "In 2 days".to_string());
    // The clock is read after hydration, never while rendering: a render-time date would be baked into the
    // prerendered page as the BUILD day, and hydration keeps the server's text (backlog row 377).
    let mut date = use_signal(|| None::<Date>);
    let mut view_date = use_signal(|| date!(2026 - 05 - 15));
    let mut open = use_signal(|| false);
    use_effect(move || {
        let start = parse_phrase("In 2 days", utc_today());
        date.set(start);
        view_date.set(start.unwrap_or_else(utc_today));
    });

    // The trigger is the popover primitive's bare button wearing the real `Button` recipe
    // (ghost, icon-xs), so no second `<button>` nests inside it.
    let trigger_attributes = attributes!(button {
        class: "dx-button",
        "data-style": "ghost",
        "data-size": "icon-xs",
        r#type: "button",
        aria_label: "Select date",
    });

    // `InputGroupInput` has no `onkeydown` prop, so the key listener travels with the field's other
    // attributes: ArrowDown opens the calendar, as in shadcn.
    let mut field_attributes = attributes!(input {
        id: "date-picker-natural-language-input",
        placeholder: "Tomorrow or next week",
        value: text(),
    });
    field_attributes.push(dioxus::html::events::onkeydown(move |e: KeyboardEvent| {
        if e.key() == Key::ArrowDown {
            e.prevent_default();
            open.set(true);
        }
    }));

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/button/style.css") }
        // The date shares one grid cell with a hidden copy of the widest date text, so its span is as wide as that text
        // whatever the date is, and while the clock has not been read yet: the sentence wraps the same at first paint,
        // hydrated and for every date (backlog row 377). `visibility: hidden` keeps the copy out of the accessibility tree.
        style {
            "[data-widest-date] {{ display: inline-grid; font-weight: 500; }} [data-widest-date]::before {{ content: attr(data-widest-date); grid-area: 1 / 1; visibility: hidden; }} [data-widest-date] > span {{ grid-area: 1 / 1; }}"
        }
        Field { style: "width: 100%; max-width: 20rem;",
            FieldLabel { html_for: "date-picker-natural-language-input", "Schedule Date" }
            InputGroup {
                InputGroupInput {
                    oninput: move |e: FormEvent| {
                        let value = e.value();
                        if let Some(parsed) = parse_phrase(&value, utc_today()) {
                            view_date.set(parsed);
                            date.set(Some(parsed));
                        }
                        text.set(value);
                    },
                    attributes: field_attributes,
                }
                InputGroupAddon { align: InputGroupAlign::End,
                    PopoverRoot {
                        is_modal: false,
                        open: Some(open()),
                        on_open_change: move |v| open.set(v),
                        PopoverTrigger { attributes: trigger_attributes,
                            CalendarDays {}
                        }
                        // One surface, as in `DatePicker`'s own popover (date_picker/style.css): the
                        // panel is the card, so it hugs the calendar and the calendar drops its own
                        // border and background (calendar/style.css reads these two).
                        PopoverContent {
                            align: ContentAlign::End,
                            style: "width: max-content; padding: 0; --dx-calendar-border: none; --dx-calendar-background: transparent;",
                            Calendar {
                                selected_date: date(),
                                on_date_change: move |picked: Option<Date>| {
                                    text.set(picked.map(format_date).unwrap_or_default());
                                    date.set(picked);
                                    open.set(false);
                                },
                                view_date: view_date(),
                                on_view_change: move |new_view: Date| view_date.set(new_view),
                                // shadcn: `captionLayout="dropdown"`, so a far-off date is two picks.
                                caption_layout: CaptionLayout::Dropdown,
                            }
                        }
                    }
                }
            }
            div { style: "padding-inline: 0.25rem; font-size: 0.875rem; color: var(--dx-muted-foreground);",
                "Your post will be published on "
                span { "data-widest-date": WIDEST_DATE,
                    span { {date().map(format_date).unwrap_or_default()} }
                }
                "."
            }
        }
    }
}

/// The widest text [`format_date`] produces in the page font: the longest month name, a two-digit day and a four-digit
/// year (`date-picker.spec.ts` checks all twelve months against it).
const WIDEST_DATE: &str = "September 30, 2026";

/// The current UTC day, read when called (an effect or an event handler, see [`Demo`]).
fn utc_today() -> Date {
    UtcDateTime::now().date()
}

/// shadcn's `formatDate`: `toLocaleDateString("en-US", { day: "2-digit", month: "long", year:
/// "numeric" })`, i.e. `July 16, 2026`.
fn format_date(date: Date) -> String {
    format!("{} {:02}, {}", date.month(), date.day(), date.year())
}

/// Turns typed text into a date relative to `today`, or `None` when it does not parse.
///
/// A deliberately small subset of what `chrono-node` understands (case-insensitive, extra
/// whitespace and commas ignored):
///
/// - `today`, `now`, `tomorrow`, `yesterday`;
/// - `in N day(s)/week(s)/month(s)`, `in a day/week/month`, and `N day(s)/week(s)/month(s) ago`;
/// - `next week` (+7 days), `next month`, `last week`, `last month`;
/// - `[this|next] <weekday>` (also `mon`, `tue`, ...): the next such weekday strictly after today;
/// - explicit dates: `2026-07-20`, `July 20`, `July 20, 2026`, `Jul 20th 2026`,
///   `20 July 2026` (a missing year is the current one) -- which includes what the calendar
///   writes back into the field, so a picked date parses to itself.
fn parse_phrase(input: &str, today: Date) -> Option<Date> {
    let lowered = input.replace(',', " ").to_lowercase();
    let words: Vec<&str> = lowered.split_whitespace().collect();
    match words.as_slice() {
        [] => None,
        ["today"] | ["now"] => Some(today),
        ["tomorrow"] => Some(today + Duration::days(1)),
        ["yesterday"] => Some(today - Duration::days(1)),
        ["next", "week"] => Some(today + Duration::weeks(1)),
        ["last", "week"] => Some(today - Duration::weeks(1)),
        ["next", "month"] => add_months(today, 1),
        ["last", "month"] => add_months(today, -1),
        ["in", n, unit] => shift(today, count(n)?, unit),
        [n, unit, "ago"] => shift(today, -count(n)?, unit),
        ["next" | "this", day] | [day] if weekday_from_name(day).is_some() => {
            let target = weekday_from_name(day)?;
            let ahead = (i64::from(target.number_days_from_monday())
                - i64::from(today.weekday().number_days_from_monday()))
            .rem_euclid(7);
            Some(today + Duration::days(if ahead == 0 { 7 } else { ahead }))
        }
        [iso] if iso.contains('-') => parse_iso(iso),
        [month, day] => explicit(today.year(), month, day),
        [month, day, year] => explicit(year.parse().ok()?, month, day),
        _ => None,
    }
}

/// `N` as a number, `a`/`an` as one.
fn count(word: &str) -> Option<i64> {
    match word {
        "a" | "an" => Some(1),
        _ => word.parse().ok(),
    }
}

fn shift(today: Date, n: i64, unit: &str) -> Option<Date> {
    match unit {
        "day" | "days" => today.checked_add(Duration::days(n)),
        "week" | "weeks" => today.checked_add(Duration::weeks(n)),
        "month" | "months" => add_months(today, i32::try_from(n).ok()?),
        _ => None,
    }
}

/// `date` moved by `months` calendar months, the day clamped to the target month's length.
fn add_months(date: Date, months: i32) -> Option<Date> {
    let index = date.year() * 12 + i32::from(date.month() as u8) - 1 + months;
    let year = index.div_euclid(12);
    let month = Month::try_from(u8::try_from(index.rem_euclid(12) + 1).ok()?).ok()?;
    Date::from_calendar_date(year, month, date.day().min(month.length(year))).ok()
}

fn parse_iso(text: &str) -> Option<Date> {
    let mut parts = text.split('-');
    let year: i32 = parts.next()?.parse().ok()?;
    let month: u8 = parts.next()?.parse().ok()?;
    let day: u8 = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Date::from_calendar_date(year, Month::try_from(month).ok()?, day).ok()
}

/// `July 20` / `20 July` (either order of the two words).
fn explicit(year: i32, first: &str, second: &str) -> Option<Date> {
    let (month, day) = match (month_from_name(first), month_from_name(second)) {
        (Some(month), None) => (month, day_number(second)?),
        (None, Some(month)) => (month, day_number(first)?),
        _ => return None,
    };
    Date::from_calendar_date(year, month, day).ok()
}

/// `20`, `20th`, `1st`, `2nd`, `3rd`.
fn day_number(word: &str) -> Option<u8> {
    let digits = word.trim_end_matches(|c: char| c.is_ascii_alphabetic());
    let suffix = &word[digits.len()..];
    if !matches!(suffix, "" | "st" | "nd" | "rd" | "th") {
        return None;
    }
    digits.parse().ok()
}

fn month_from_name(name: &str) -> Option<Month> {
    let name = name.trim_end_matches('.');
    Some(match name {
        "january" | "jan" => Month::January,
        "february" | "feb" => Month::February,
        "march" | "mar" => Month::March,
        "april" | "apr" => Month::April,
        "may" => Month::May,
        "june" | "jun" => Month::June,
        "july" | "jul" => Month::July,
        "august" | "aug" => Month::August,
        "september" | "sep" | "sept" => Month::September,
        "october" | "oct" => Month::October,
        "november" | "nov" => Month::November,
        "december" | "dec" => Month::December,
        _ => return None,
    })
}

fn weekday_from_name(name: &str) -> Option<Weekday> {
    Some(match name {
        "monday" | "mon" => Weekday::Monday,
        "tuesday" | "tue" | "tues" => Weekday::Tuesday,
        "wednesday" | "wed" => Weekday::Wednesday,
        "thursday" | "thu" | "thur" | "thurs" => Weekday::Thursday,
        "friday" | "fri" => Weekday::Friday,
        "saturday" | "sat" => Weekday::Saturday,
        "sunday" | "sun" => Weekday::Sunday,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    // A Wednesday.
    const TODAY: Date = date!(2026 - 10 - 14);

    fn parse(text: &str) -> Option<Date> {
        parse_phrase(text, TODAY)
    }

    #[test]
    fn relative_phrases() {
        assert_eq!(parse("today"), Some(TODAY));
        assert_eq!(parse("Tomorrow"), Some(date!(2026 - 10 - 15)));
        assert_eq!(parse("yesterday"), Some(date!(2026 - 10 - 13)));
        assert_eq!(parse("In 2 days"), Some(date!(2026 - 10 - 16)));
        assert_eq!(parse("in 3 weeks"), Some(date!(2026 - 11 - 04)));
        assert_eq!(parse("in a week"), Some(date!(2026 - 10 - 21)));
        assert_eq!(parse("2 days ago"), Some(date!(2026 - 10 - 12)));
        assert_eq!(parse("next week"), Some(date!(2026 - 10 - 21)));
        assert_eq!(parse("next month"), Some(date!(2026 - 11 - 14)));
        assert_eq!(parse("in 4 months"), Some(date!(2027 - 02 - 14)));
    }

    #[test]
    fn month_arithmetic_clamps_the_day() {
        assert_eq!(
            parse_phrase("next month", date!(2026 - 01 - 31)),
            Some(date!(2026 - 02 - 28))
        );
        assert_eq!(
            parse_phrase("last month", date!(2026 - 03 - 31)),
            Some(date!(2026 - 02 - 28))
        );
    }

    #[test]
    fn weekdays_are_the_next_one_strictly_after_today() {
        assert_eq!(parse("friday"), Some(date!(2026 - 10 - 16)));
        assert_eq!(parse("next Friday"), Some(date!(2026 - 10 - 16)));
        assert_eq!(parse("wed"), Some(date!(2026 - 10 - 21)));
        assert_eq!(parse("this monday"), Some(date!(2026 - 10 - 19)));
    }

    #[test]
    fn explicit_dates() {
        assert_eq!(parse("2026-10-20"), Some(date!(2026 - 10 - 20)));
        assert_eq!(parse("October 20"), Some(date!(2026 - 10 - 20)));
        assert_eq!(parse("Oct 5th, 2027"), Some(date!(2027 - 10 - 05)));
        assert_eq!(parse("20 october 2025"), Some(date!(2025 - 10 - 20)));
        assert_eq!(parse("Feb 30 2026"), None);
    }

    #[test]
    fn the_formatted_date_parses_back_to_itself() {
        for date in [TODAY, date!(2026 - 01 - 05), date!(2030 - 12 - 31)] {
            assert_eq!(parse(&format_date(date)), Some(date), "{}", format_date(date));
        }
        assert_eq!(format_date(date!(2026 - 10 - 05)), "October 05, 2026");
    }

    #[test]
    fn garbage_does_not_parse() {
        for text in ["", "   ", "asdf", "tomorr", "in x days", "in 2 fortnights", "oct", "13"] {
            assert_eq!(parse(text), None, "{text:?}");
        }
    }
}
