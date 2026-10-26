use super::*;
use crate::test_support::{find_element, with_runtime};
use time::macros::date;

#[test]
fn horizontal_day_step_ltr_matches_arrow_direction() {
    assert_eq!(
        horizontal_day_step(&Key::ArrowRight, Direction::Ltr),
        Some(1)
    );
    assert_eq!(
        horizontal_day_step(&Key::ArrowLeft, Direction::Ltr),
        Some(-1)
    );
}

#[test]
fn horizontal_day_step_rtl_swaps_left_and_right() {
    assert_eq!(
        horizontal_day_step(&Key::ArrowRight, Direction::Rtl),
        Some(-1)
    );
    assert_eq!(
        horizontal_day_step(&Key::ArrowLeft, Direction::Rtl),
        Some(1)
    );
}

#[test]
fn horizontal_day_step_ignores_vertical_keys_both_directions() {
    for direction in [Direction::Ltr, Direction::Rtl] {
        assert_eq!(horizontal_day_step(&Key::ArrowUp, direction), None);
        assert_eq!(horizontal_day_step(&Key::ArrowDown, direction), None);
        assert_eq!(horizontal_day_step(&Key::Home, direction), None);
        assert_eq!(horizontal_day_step(&Key::End, direction), None);
    }
}

#[test]
fn week_numbers_follow_react_day_picker_defaults() {
    // Monday start: ISO 8601. 2026-01-01 is a Thursday, so ISO week 1 starts Mon 2025-12-29.
    assert_eq!(week_number(date!(2025 - 12 - 29), Weekday::Monday), 1);
    assert_eq!(week_number(date!(2026 - 10 - 12), Weekday::Monday), 42);
    // 2020 had 53 ISO weeks; its last Monday-start week is week 53.
    assert_eq!(week_number(date!(2020 - 12 - 28), Weekday::Monday), 53);
    // 2021-01-01 is a Friday: the week of 2020-12-28 is week 53 of 2020, and 2021's week 1
    // starts Mon 2021-01-04.
    assert_eq!(week_number(date!(2021 - 01 - 04), Weekday::Monday), 1);

    // Sunday start (en-US): the week that holds January 1st is week 1, even when most of it is
    // in the old year. 2026-01-01 is a Thursday, so Sun 2025-12-28 starts week 1.
    assert_eq!(week_number(date!(2025 - 12 - 28), Weekday::Sunday), 1);
    assert_eq!(week_number(date!(2026 - 01 - 04), Weekday::Sunday), 2);
    assert_eq!(week_number(date!(2026 - 10 - 11), Weekday::Sunday), 42);
    // 2021-01-01 is a Friday, so the week starting Sun 2020-12-27 is week 1 of 2021.
    assert_eq!(week_number(date!(2020 - 12 - 27), Weekday::Sunday), 1);
    assert_eq!(week_number(date!(2020 - 12 - 20), Weekday::Sunday), 52);
}

#[test]
fn week_number_accepts_any_day_of_the_week() {
    assert_eq!(week_number(date!(2026 - 10 - 14), Weekday::Monday), 42);
    assert_eq!(week_number(date!(2026 - 10 - 17), Weekday::Sunday), 42);
}

#[component]
fn CalendarWithWeekNumbers() -> Element {
    rsx! {
        Calendar {
            view_date: date!(2026 - 05 - 15),
            first_day_of_week: Weekday::Monday,
            show_week_number: true,
            CalendarView {
                CalendarGrid {}
            }
        }
    }
}

#[component]
fn CalendarWithoutWeekNumbers() -> Element {
    rsx! {
        Calendar {
            view_date: date!(2026 - 05 - 15),
            CalendarView {
                CalendarGrid {}
            }
        }
    }
}

#[test]
fn show_week_number_renders_a_labelled_row_header_per_week() {
    let mut dom = VirtualDom::new(CalendarWithWeekNumbers);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    // May 2026 starting Monday spans 2026-04-27 .. 2026-05-31: ISO weeks 18 to 22.
    for week in 18..=22 {
        assert!(
            html.contains(&format!("aria-label=\"Week {week}\"")),
            "week {week} missing in {html}"
        );
    }
    assert_eq!(html.matches("role=\"rowheader\"").count(), 5);
    assert_eq!(html.matches("aria-label=\"Week number\"").count(), 1);
}

#[test]
fn week_numbers_are_absent_by_default() {
    let mut dom = VirtualDom::new(CalendarWithoutWeekNumbers);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    assert!(!html.contains("rowheader"));
    assert!(!html.contains("Week number"));
}

#[component]
fn ConsecutiveCalendarViews() -> Element {
    rsx! {
        Calendar {
            view_date: date!(2026 - 05 - 15),
            CalendarView {
                CalendarMonthTitle {}
            }
            CalendarView {
                CalendarMonthTitle {}
            }
            CalendarView {
                CalendarMonthTitle {}
            }
        }
    }
}

#[component]
fn CalendarDayWithCustomChild() -> Element {
    rsx! {
        Calendar {
            view_date: date!(2026 - 05 - 15),
            CalendarView {
                CalendarDay {
                    date: date!(2026 - 05 - 15),
                    "Custom day"
                }
            }
        }
    }
}

#[component]
fn RangeCalendarDayWithCustomChild() -> Element {
    rsx! {
        RangeCalendar {
            view_date: date!(2026 - 05 - 15),
            CalendarView {
                CalendarDay {
                    date: date!(2026 - 05 - 15),
                    "Custom range day"
                }
            }
        }
    }
}

#[test]
fn implicit_calendar_views_render_consecutive_months_on_first_render() {
    let mut dom = VirtualDom::new(ConsecutiveCalendarViews);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    assert!(html.contains("May 2026"));
    assert!(html.contains("June 2026"));
    assert!(html.contains("July 2026"));
}

#[test]
fn calendar_day_forwards_custom_children() {
    let mut dom = VirtualDom::new(CalendarDayWithCustomChild);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    assert!(html.contains("Custom day"));
    assert!(!html.contains(">15</button>"));
}

#[test]
fn range_calendar_day_forwards_custom_children() {
    let mut dom = VirtualDom::new(RangeCalendarDayWithCustomChild);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    assert!(html.contains("Custom range day"));
    assert!(!html.contains(">15</button>"));
}

// --- CalendarDayState / use_calendar_day_state / calendar_day_attributes
// (dev-docs/backlog.md row 12c, upstream DioxusLabs/components#199) ---

#[test]
fn day_state_computes_today_disabled_unavailable_and_relative_month() {
    with_runtime(|| {
        let disabled_ranges = [DateRange::new(date!(2024 - 06 - 10), date!(2024 - 06 - 20))];
        let mut base_ctx = make_base_ctx_for_relative_month(DateRange::new(
            date!(2024 - 01 - 01),
            date!(2024 - 12 - 31),
        ));
        base_ctx.available_ranges = use_memo(move || AvailableRanges::new(&disabled_ranges));
        let view_date = date!(2024 - 06 - 01); // base_ctx.today == 2024-06-15

        // A plain, available, current-month day.
        let state = day_state(
            date!(2024 - 06 - 01),
            &base_ctx,
            view_date,
            false,
            DaySelection::default(),
        );
        assert!(!state.today);
        assert!(!state.disabled);
        assert!(!state.unavailable);
        assert!(!state.outside_month);
        assert_eq!(state.relative_month, RelativeMonth::Current);

        // Today.
        let state = day_state(
            date!(2024 - 06 - 15),
            &base_ctx,
            view_date,
            false,
            DaySelection::default(),
        );
        assert!(state.today);

        // Inside the disabled range: unavailable, and therefore disabled
        // too even though the calendar itself is not.
        let state = day_state(
            date!(2024 - 06 - 12),
            &base_ctx,
            view_date,
            false,
            DaySelection::default(),
        );
        assert!(state.unavailable);
        assert!(state.disabled);

        // A day from the previous/next month shown in the grid.
        let state = day_state(
            date!(2024 - 05 - 28),
            &base_ctx,
            view_date,
            false,
            DaySelection::default(),
        );
        assert!(state.outside_month);
        assert_eq!(state.relative_month, RelativeMonth::Last);
        let state = day_state(
            date!(2024 - 07 - 03),
            &base_ctx,
            view_date,
            false,
            DaySelection::default(),
        );
        assert!(state.outside_month);
        assert_eq!(state.relative_month, RelativeMonth::Next);

        // `focused`/`selected`/`range_*` pass through as given.
        let state = day_state(
            date!(2024 - 06 - 01),
            &base_ctx,
            view_date,
            true,
            DaySelection {
                selected: true,
                range_start: true,
                range_end: true,
                in_range: true,
            },
        );
        assert!(state.focused);
        assert!(state.selected);
        assert!(state.range_start);
        assert!(state.range_end);
        assert!(state.in_range);
    });
}

fn day_state_fixture(overrides: impl FnOnce(&mut CalendarDayState)) -> CalendarDayState {
    let mut state = CalendarDayState {
        date: date!(2024 - 06 - 15),
        selected: false,
        range_start: false,
        range_end: false,
        in_range: false,
        focused: false,
        today: false,
        disabled: false,
        unavailable: false,
        outside_month: false,
        relative_month: RelativeMonth::Current,
    };
    overrides(&mut state);
    state
}

fn find_attr<'a>(attrs: &'a [Attribute], name: &str) -> Option<&'a Attribute> {
    attrs.iter().find(|a| a.name == name)
}

#[test]
fn calendar_day_attributes_always_includes_aria_label_selected_and_disabled() {
    let state = day_state_fixture(|_| {});
    let attrs = calendar_day_attributes(&state);

    assert_eq!(
        find_attr(&attrs, "aria-label").unwrap().value,
        Text(aria_label(&state.date))
    );
    assert_eq!(
        find_attr(&attrs, "data-selected").unwrap().value,
        Bool(false)
    );
    assert_eq!(
        find_attr(&attrs, "data-disabled").unwrap().value,
        Bool(false)
    );
    // backlog row 84: `aria-disabled` mirrors `data-disabled`
    // exactly (same source value, same unconditional presence) -- see
    // the comment in `calendar_day_attributes`.
    assert_eq!(
        find_attr(&attrs, "aria-disabled").unwrap().value,
        Bool(false)
    );
    assert_eq!(
        find_attr(&attrs, "data-month").unwrap().value,
        Text("current".to_string())
    );
    // Booleans that are false are omitted entirely, matching the
    // built-in cell's `if cond { true }` (no-else) attributes.
    assert!(find_attr(&attrs, "data-today").is_none());
    assert!(find_attr(&attrs, "data-unavailable").is_none());
    assert!(find_attr(&attrs, "data-selection-start").is_none());
    assert!(find_attr(&attrs, "data-selection-between").is_none());
    assert!(find_attr(&attrs, "data-selection-end").is_none());
}

#[test]
fn calendar_day_attributes_renders_true_valued_booleans_unquoted() {
    // `AttributeValue::Bool` (not `Text`) renders unquoted
    // (`dioxus-ssr`'s `write_attribute`) -- these must stay `Bool`, not
    // a `Text("true")` lookalike, to keep the built-in cell's rendered
    // HTML byte-identical.
    let state = day_state_fixture(|s| {
        s.today = true;
        s.selected = true;
        s.unavailable = true;
        s.disabled = true;
        s.range_start = true;
        s.range_end = true;
        s.in_range = true;
    });
    let attrs = calendar_day_attributes(&state);

    for name in [
        "data-today",
        "data-selected",
        "data-unavailable",
        "data-disabled",
        "aria-disabled",
        "data-selection-start",
        "data-selection-between",
        "data-selection-end",
    ] {
        assert_eq!(
            find_attr(&attrs, name)
                .unwrap_or_else(|| panic!("missing {name}"))
                .value,
            Bool(true),
            "{name} must be present and true"
        );
    }
}

#[test]
fn calendar_day_attributes_reports_last_and_next_relative_month() {
    let state = day_state_fixture(|s| {
        s.outside_month = true;
        s.relative_month = RelativeMonth::Last;
    });
    assert_eq!(
        find_attr(&calendar_day_attributes(&state), "data-month")
            .unwrap()
            .value,
        Text("last".to_string())
    );

    let state = day_state_fixture(|s| {
        s.outside_month = true;
        s.relative_month = RelativeMonth::Next;
    });
    assert_eq!(
        find_attr(&calendar_day_attributes(&state), "data-month")
            .unwrap()
            .value,
        Text("next".to_string())
    );
}

#[component]
fn DayStateOutsideCalendar() -> Element {
    let state = use_calendar_day_state(date!(2024 - 06 - 15));
    rsx! {
        div { id: "result", "{state.is_none()}" }
    }
}

#[test]
fn use_calendar_day_state_returns_none_outside_a_calendar_context() {
    let mut dom = VirtualDom::new(DayStateOutsideCalendar);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    assert!(html.contains(r#"<div id="result">true</div>"#));
}

#[component]
fn DayStateInsideCalendarView() -> Element {
    // `CalendarGrid` isn't rendered -- proves `use_calendar_day_state`
    // only needs a `CalendarView`, matching `use_calendar_grid`'s own
    // context requirements (see `use_calendar_grid`).
    let state = use_calendar_day_state(date!(2024 - 06 - 15));
    rsx! {
        div { id: "result", "{state.is_none()}" }
    }
}

#[test]
fn use_calendar_day_state_is_some_inside_a_calendar_view() {
    #[component]
    fn Demo() -> Element {
        rsx! {
            Calendar {
                view_date: date!(2024 - 06 - 01),
                today: date!(2024 - 06 - 15),
                CalendarView { DayStateInsideCalendarView {} }
            }
        }
    }

    let mut dom = VirtualDom::new(Demo);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    assert!(html.contains(r#"<div id="result">false</div>"#));
}

#[test]
fn use_calendar_day_state_matches_day_state_for_a_single_calendar() {
    // The hook must compute the *same* `CalendarDayState` `day_state`
    // does for the built-in cell -- that's the "one source of truth"
    // this hook exists for.
    with_runtime(|| {
        let disabled_ranges = [DateRange::new(date!(2024 - 06 - 18), date!(2024 - 06 - 19))];
        let mut base_ctx = make_base_ctx_for_relative_month(DateRange::new(
            date!(2024 - 01 - 01),
            date!(2024 - 12 - 31),
        ));
        base_ctx.available_ranges = use_memo(move || AvailableRanges::new(&disabled_ranges));
        use_context_provider(|| base_ctx);
        use_context_provider(|| CalendarViewContext { offset: 0 });
        use_context_provider(|| CalendarContext {
            selected_date: Signal::new(Some(date!(2024 - 06 - 15))).into(),
            set_selected_date: Callback::new(|_: Option<Date>| {}),
        });

        let view_date = base_ctx.view_date();

        let state = use_calendar_day_state(date!(2024 - 06 - 15))
            .expect("inside a Calendar + CalendarView context");
        let expected = day_state(
            date!(2024 - 06 - 15),
            &base_ctx,
            view_date,
            state.focused,
            DaySelection {
                selected: true,
                ..Default::default()
            },
        );
        assert_eq!(state, expected);
        assert!(state.selected);
        assert!(state.today);
        assert!(!state.range_start && !state.range_end && !state.in_range);

        // An unavailable date outside the selection.
        let state = use_calendar_day_state(date!(2024 - 06 - 18)).unwrap();
        assert!(state.unavailable);
        assert!(state.disabled);
        assert!(!state.selected);
    });
}

#[test]
fn use_calendar_day_state_matches_day_state_for_a_range_calendar() {
    with_runtime(|| {
        let base_ctx = make_base_ctx_for_relative_month(DateRange::new(
            date!(2024 - 01 - 01),
            date!(2024 - 12 - 31),
        ));
        use_context_provider(|| base_ctx);
        use_context_provider(|| CalendarViewContext { offset: 0 });
        let range = DateRange::new(date!(2024 - 06 - 10), date!(2024 - 06 - 20));
        use_context_provider(|| RangeCalendarContext {
            anchor_date: Signal::new(None),
            highlighted_range: Signal::new(Some(range)),
            set_selected_range: Callback::new(|_: Option<DateRange>| {}),
        });

        let start = use_calendar_day_state(date!(2024 - 06 - 10)).unwrap();
        assert!(
            start.selected,
            "range endpoints are inclusive of `selected`"
        );
        assert!(start.range_start);
        assert!(!start.in_range);
        assert!(!start.range_end);

        let middle = use_calendar_day_state(date!(2024 - 06 - 15)).unwrap();
        assert!(middle.selected);
        assert!(middle.in_range);
        assert!(!middle.range_start && !middle.range_end);

        let end = use_calendar_day_state(date!(2024 - 06 - 20)).unwrap();
        assert!(end.selected);
        assert!(end.range_end);
        assert!(!end.in_range && !end.range_start);

        let outside = use_calendar_day_state(date!(2024 - 06 - 25)).unwrap();
        assert!(
            !outside.selected && !outside.range_start && !outside.range_end && !outside.in_range
        );

        let view_date = base_ctx.view_date();
        let expected_middle = day_state(
            date!(2024 - 06 - 15),
            &base_ctx,
            view_date,
            middle.focused,
            DaySelection {
                selected: true,
                range_start: false,
                range_end: false,
                in_range: true,
            },
        );
        assert_eq!(middle, expected_middle);
    });
}

#[test]
fn single_calendar_day_renders_today_selected_and_month_attributes() {
    #[component]
    fn Demo() -> Element {
        rsx! {
            Calendar {
                view_date: date!(2024 - 06 - 01),
                today: date!(2024 - 06 - 15),
                selected_date: Some(date!(2024 - 06 - 15)),
                CalendarView {
                    CalendarDay { date: date!(2024 - 06 - 15) }
                }
            }
        }
    }

    let mut dom = VirtualDom::new(Demo);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    assert!(html.contains(&format!(
        r#"aria-label="{}""#,
        aria_label(&date!(2024 - 06 - 15))
    )));
    assert!(html.contains("data-today=true"));
    assert!(html.contains("data-selected=true"));
    assert!(html.contains("data-disabled=false"));
    assert!(html.contains("aria-disabled=false"));
    assert!(html.contains(r#"data-month="current""#));
    assert!(!html.contains("data-unavailable"));
}

#[test]
fn unavailable_calendar_day_renders_aria_disabled_true_and_available_day_renders_aria_disabled_false(
) {
    // backlog row 84: an unavailable day carries `aria-disabled` --
    // an end-to-end check (full `Calendar`, real
    // `disabled_ranges`, `dioxus_ssr::render`) rather
    // than only the lower-level `calendar_day_attributes` fixture tests
    // above, so this also exercises `is_unavailable`/`day_state`'s own
    // wiring, not just the attribute-building function in isolation.
    #[component]
    fn Demo() -> Element {
        rsx! {
            Calendar {
                view_date: date!(2024 - 06 - 01),
                today: date!(2024 - 06 - 01),
                disabled_ranges: vec![DateRange::new(date!(2024 - 06 - 15), date!(2024 - 06 - 15))],
                CalendarView {
                    CalendarDay { date: date!(2024 - 06 - 15) }
                    CalendarDay { date: date!(2024 - 06 - 16) }
                }
            }
        }
    }

    let mut dom = VirtualDom::new(Demo);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    // Backlog row 108: rather than slicing the raw HTML string from an
    // `aria-label="..."` match's own position (fragile -- `merge_attributes`,
    // row 93, sorts attributes by name, so no other attribute's position
    // relative to `aria-label` is guaranteed), parse each `<button>` into
    // an order-independent attribute map and look each one up by
    // `aria-label` directly.
    let unavailable_label = aria_label(&date!(2024 - 06 - 15));
    let available_label = aria_label(&date!(2024 - 06 - 16));

    let unavailable = find_element(&html, |attrs| {
        attrs.get("aria-label").map(String::as_str) == Some(unavailable_label.as_str())
    })
    .expect("unavailable day must render");
    let available = find_element(&html, |attrs| {
        attrs.get("aria-label").map(String::as_str) == Some(available_label.as_str())
    })
    .expect("available day must render");

    assert_eq!(
            unavailable.attrs.get("data-unavailable").map(String::as_str),
            Some("true"),
            "sanity: the fixture's disabled_ranges must actually make this day unavailable: {unavailable:?}"
        );
    assert_eq!(
        unavailable.attrs.get("aria-disabled").map(String::as_str),
        Some("true"),
        "an unavailable day must carry aria-disabled=true: {unavailable:?}"
    );
    assert!(
        !available.attrs.contains_key("data-unavailable"),
        "sanity: the following day must be available: {available:?}"
    );
    assert_eq!(
        available.attrs.get("aria-disabled").map(String::as_str),
        Some("false"),
        "an available day must still carry an explicit aria-disabled=false: {available:?}"
    );
}

#[test]
fn range_calendar_day_renders_selection_start_between_and_end_attributes() {
    #[component]
    fn Demo() -> Element {
        rsx! {
            RangeCalendar {
                view_date: date!(2024 - 06 - 01),
                today: date!(2024 - 06 - 01),
                selected_range: Some(DateRange::new(date!(2024 - 06 - 10), date!(2024 - 06 - 20))),
                CalendarView {
                    CalendarDay { date: date!(2024 - 06 - 10) }
                    CalendarDay { date: date!(2024 - 06 - 15) }
                    CalendarDay { date: date!(2024 - 06 - 20) }
                }
            }
        }
    }

    let mut dom = VirtualDom::new(Demo);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    assert!(html.contains("data-selection-start=true"));
    assert!(html.contains("data-selection-between=true"));
    assert!(html.contains("data-selection-end=true"));
    assert_eq!(
        html.matches("data-selected=true").count(),
        3,
        "all three dates are within the inclusive selected range"
    );
}

#[test]
fn test_weekday_set() {
    let mut weekdays = WeekdaySet::single(Weekday::Monday);
    // Test contains
    assert!(weekdays.contains(Weekday::Monday));
    assert!(!weekdays.contains(Weekday::Tuesday));

    // Test remove
    assert!(weekdays.remove(Weekday::Monday));
    assert!(!weekdays.contains(Weekday::Monday));
    assert!(!weekdays.remove(Weekday::Monday)); // Already removed

    let all_days = WeekdaySet(0b111_1111); // All days
    let empty_set = WeekdaySet(0b000_0000); // Empty
    let single_set = WeekdaySet::single(Weekday::Friday); // Single day
    let part_size_set = WeekdaySet(0b010_1010); // Tu, Th, Sa

    // Test iterator
    let days: Vec<_> = all_days.iter(Weekday::Sunday).collect();
    assert_eq!(days.len(), 7);
    assert_eq!(days[0], Weekday::Sunday);

    let mut iter = all_days.iter(Weekday::Wednesday);
    assert_eq!(iter.next(), Some(Weekday::Wednesday));
    assert_eq!(iter.next(), Some(Weekday::Thursday));

    // Test first
    assert_eq!(empty_set.first(), None);
    assert_eq!(single_set.first(), Some(Weekday::Friday));
    assert_eq!(part_size_set.first(), Some(Weekday::Tuesday));
    assert_eq!(all_days.first(), Some(Weekday::Monday));

    // Test is_empty
    assert!(empty_set.is_empty());
    assert!(!part_size_set.is_empty());
    assert!(!single_set.is_empty());
    assert!(!all_days.is_empty());
}

#[test]
fn test_days_since() {
    // Test days since calculation
    let date = date!(2024 - 01 - 01); // Monday
    assert_eq!(days_since(date, Weekday::Monday), 0);
    assert_eq!(days_since(date, Weekday::Sunday), 1);
    assert_eq!(days_since(date, Weekday::Tuesday), 6);
}

#[test]
fn test_month_navigation() {
    let date = date!(2024 - 01 - 15);

    // Test next month
    let next = next_month(date);
    assert!(next.is_some());
    assert_eq!(next.unwrap().month(), Month::February);
    assert_eq!(next.unwrap().year(), 2024);
    assert_eq!(next.unwrap().day(), 15);

    // Test previous month
    let prev = previous_month(date);
    assert!(prev.is_some());
    assert_eq!(prev.unwrap().month(), Month::December);
    assert_eq!(prev.unwrap().year(), 2023);
    assert_eq!(prev.unwrap().day(), 15);
}

#[test]
fn test_calendar_grid_weeks() {
    // Calls the real `calendar_grid_weeks` (the function `CalendarGrid`/
    // `use_calendar_grid` actually use to build the rendered month grid) --
    // not a hand-copied reimplementation. See dev-docs/reference/mutants-baseline.md.

    // Test February 2021: starts on Monday, 28 days
    // When first day of week is Monday, should fit in exactly 4 weeks
    let feb_2021 = date!(2021 - 02 - 15);
    let feb_grid = calendar_grid_weeks(feb_2021, Weekday::Monday);
    assert_eq!(
        feb_grid.len(),
        4,
        "February 2021 should have exactly 4 weeks"
    );

    // Count days from February in the grid
    let feb_days: Vec<_> = feb_grid
        .iter()
        .flatten()
        .filter(|d| d.month() == Month::February && d.year() == 2021)
        .collect();
    assert_eq!(
        feb_days.len(),
        28,
        "Should have all 28 days of February 2021"
    );

    // Test May 2024: starts on Wednesday, 31 days
    // When first day of week is Sunday, should need 5 weeks
    let may_2024 = date!(2024 - 05 - 15);
    let may_grid = calendar_grid_weeks(may_2024, Weekday::Sunday);
    assert_eq!(
        may_grid.len(),
        5,
        "May 2024 should have exactly 5 weeks when starting from Sunday"
    );

    // Count days from May in the grid
    let may_days: Vec<_> = may_grid
        .iter()
        .flatten()
        .filter(|d| d.month() == Month::May && d.year() == 2024)
        .collect();
    assert_eq!(may_days.len(), 31, "Should have all 31 days of May 2024");

    // Test that we don't generate empty trailing weeks
    // December 2018: starts on Saturday, 31 days (when week starts on Sunday)
    // Should need exactly 6 weeks (30 days in November + 31 in December + 5 in January = 66/7 = 6)
    let dec_2018 = date!(2018 - 12 - 15);
    let dec_grid = calendar_grid_weeks(dec_2018, Weekday::Sunday);
    assert_eq!(
        dec_grid.len(),
        6,
        "December 2018 should have exactly 6 weeks"
    );

    // Verify no week is completely empty, and every week is contiguous
    // and every week's 7 dates are consecutive calendar days
    let mut prev: Option<Date> = None;
    for week in &dec_grid {
        assert!(!week.is_empty(), "No week should be empty");
        assert_eq!(week.len(), 7, "Each week should have exactly 7 days");
        for &d in week {
            if let Some(p) = prev {
                assert_eq!(p.next_day().unwrap(), d, "grid dates must be consecutive");
            }
            prev = Some(d);
        }
    }

    // The grid's first date is the correct number of days before the 1st
    // of the month for the given first_day_of_week (i.e. the leading
    // days from the previous month line up under the right weekday
    // column), and the grid's last date completes the final week.
    let first_of_dec = dec_2018.replace_day(1).unwrap();
    assert_eq!(dec_grid[0][0].weekday(), Weekday::Sunday);
    assert_eq!(
        first_of_dec - dec_grid[0][0],
        time::Duration::days(days_since(dec_2018, Weekday::Sunday))
    );
}

// -----------------------------------------------------------------
// Pure predicates (dev-docs/reference/mutants-baseline.md)
// -----------------------------------------------------------------

#[test]
fn test_nth_month_next_and_previous() {
    // NOTE: `nth_month_next`/`nth_month_previous` compute the target month via
    // `Month::nth_next`/`nth_prev`, which cycle *within* a 12-month wheel, and only
    // bump the year by at most 1 to account for that single wrap. They are only ever
    // called in this codebase with small offsets (multi-month calendar view offsets),
    // so this is exercised here with n in a realistic small range rather than n >= 12,
    // where the single `+1`/`-1` year adjustment would under/overshoot the real
    // number of year boundaries crossed.
    let date = date!(2024 - 01 - 15);

    assert_eq!(nth_month_next(date, 0), Some(date), "n=0 is a no-op");
    assert_eq!(nth_month_next(date, 1), Some(date!(2024 - 02 - 15)));

    // Crosses a year boundary within a single 12-month cycle.
    let nov = date!(2024 - 11 - 15);
    assert_eq!(
        nth_month_next(nov, 2),
        Some(date!(2025 - 01 - 15)),
        "wraps into the next year"
    );

    // Clamps the day when the target month is shorter.
    let jan_31 = date!(2024 - 01 - 31);
    assert_eq!(nth_month_next(jan_31, 1), Some(date!(2024 - 02 - 29))); // 2024 is a leap year

    assert_eq!(nth_month_previous(date, 0), Some(date), "n=0 is a no-op");
    assert_eq!(nth_month_previous(date, 1), Some(date!(2023 - 12 - 15)));
    assert_eq!(
        nth_month_previous(nov, 2),
        Some(date!(2024 - 09 - 15)),
        "no wrap needed for a same-year previous"
    );

    let mar_31 = date!(2024 - 03 - 31);
    assert_eq!(nth_month_previous(mar_31, 1), Some(date!(2024 - 02 - 29)));
}

#[test]
fn test_date_range_contains_and_contained_in_interval() {
    let range = DateRange::new(date!(2024 - 01 - 10), date!(2024 - 01 - 20));

    // `contains` is inclusive of both endpoints.
    assert!(range.contains(date!(2024 - 01 - 10)));
    assert!(range.contains(date!(2024 - 01 - 20)));
    assert!(range.contains(date!(2024 - 01 - 15)));
    assert!(!range.contains(date!(2024 - 01 - 09)));
    assert!(!range.contains(date!(2024 - 01 - 21)));

    // `contained_in_interval` is exclusive of both endpoints.
    assert!(!range.contained_in_interval(date!(2024 - 01 - 10)));
    assert!(!range.contained_in_interval(date!(2024 - 01 - 20)));
    assert!(range.contained_in_interval(date!(2024 - 01 - 15)));

    // `DateRange::new` normalizes a reversed (start, end) pair.
    let reversed = DateRange::new(date!(2024 - 01 - 20), date!(2024 - 01 - 10));
    assert_eq!(reversed.start(), date!(2024 - 01 - 10));
    assert_eq!(reversed.end(), date!(2024 - 01 - 20));
}

#[test]
fn test_date_range_display() {
    let range = DateRange::new(date!(2024 - 01 - 10), date!(2024 - 01 - 20));
    assert_eq!(
        range.to_string(),
        format!("{} - {}", date!(2024 - 01 - 10), date!(2024 - 01 - 20))
    );
    assert!(!range.to_string().is_empty());
}

#[test]
fn test_available_ranges_valid_interval_and_to_disabled_ranges() {
    let disabled = [
        DateRange::new(date!(2024 - 01 - 10), date!(2024 - 01 - 15)),
        DateRange::new(date!(2024 - 02 - 01), date!(2024 - 02 - 05)),
    ];
    let ranges = AvailableRanges::new(&disabled);

    // Inside a disabled range (inclusive endpoints) is invalid.
    assert!(!ranges.valid_interval(date!(2024 - 01 - 10)));
    assert!(!ranges.valid_interval(date!(2024 - 01 - 12)));
    assert!(!ranges.valid_interval(date!(2024 - 01 - 15)));
    // Outside any disabled range is valid.
    assert!(ranges.valid_interval(date!(2024 - 01 - 09)));
    assert!(ranges.valid_interval(date!(2024 - 01 - 16)));
    assert!(ranges.valid_interval(date!(2024 - 01 - 31)));
    assert!(!ranges.valid_interval(date!(2024 - 02 - 03)));

    // Round trips back out as the same (sorted, merged) disabled ranges.
    assert_eq!(ranges.to_disabled_ranges(), disabled.to_vec());
}

#[test]
fn test_available_ranges_merges_overlapping_disabled_ranges() {
    // Two overlapping disabled ranges collapse into one.
    let disabled = [
        DateRange::new(date!(2024 - 01 - 10), date!(2024 - 01 - 20)),
        DateRange::new(date!(2024 - 01 - 15), date!(2024 - 01 - 25)),
    ];
    let ranges = AvailableRanges::new(&disabled);

    assert_eq!(
        ranges.to_disabled_ranges(),
        vec![DateRange::new(date!(2024 - 01 - 10), date!(2024 - 01 - 25))]
    );
    assert!(!ranges.valid_interval(date!(2024 - 01 - 18)));
}

#[test]
fn test_available_ranges_available_range() {
    let disabled = [DateRange::new(date!(2024 - 01 - 10), date!(2024 - 01 - 20))];
    let ranges = AvailableRanges::new(&disabled);
    let bounds = DateRange::new(date!(2024 - 01 - 01), date!(2024 - 01 - 31));

    // A date between two disabled boundaries gets the gap between them.
    let available = ranges
        .available_range(date!(2024 - 01 - 05), bounds)
        .expect("date is not disabled");
    assert_eq!(available.start(), date!(2024 - 01 - 01));
    assert_eq!(available.end(), date!(2024 - 01 - 09));

    let available = ranges
        .available_range(date!(2024 - 01 - 25), bounds)
        .expect("date is not disabled");
    assert_eq!(available.start(), date!(2024 - 01 - 21));
    assert_eq!(available.end(), date!(2024 - 01 - 31));

    // A disabled date itself has no available range.
    assert_eq!(ranges.available_range(date!(2024 - 01 - 15), bounds), None);
}

#[test]
fn test_is_between_is_start_is_end() {
    let range = Some(DateRange::new(date!(2024 - 01 - 10), date!(2024 - 01 - 20)));

    assert!(is_start(date!(2024 - 01 - 10), range));
    assert!(!is_start(date!(2024 - 01 - 20), range));
    assert!(!is_start(date!(2024 - 01 - 15), range));

    assert!(is_end(date!(2024 - 01 - 20), range));
    assert!(!is_end(date!(2024 - 01 - 10), range));
    assert!(!is_end(date!(2024 - 01 - 15), range));

    assert!(is_between(date!(2024 - 01 - 15), range));
    assert!(
        !is_between(date!(2024 - 01 - 10), range),
        "endpoints are not 'between'"
    );
    assert!(
        !is_between(date!(2024 - 01 - 20), range),
        "endpoints are not 'between'"
    );

    // A single-day range (start == end) is neither a start nor an end per
    // the `date != r.end` / `date != r.start` guards.
    let single_day = Some(DateRange::new(date!(2024 - 01 - 10), date!(2024 - 01 - 10)));
    assert!(!is_start(date!(2024 - 01 - 10), single_day));
    assert!(!is_end(date!(2024 - 01 - 10), single_day));

    assert!(!is_start(date!(2024 - 01 - 10), None));
    assert!(!is_end(date!(2024 - 01 - 10), None));
    assert!(!is_between(date!(2024 - 01 - 10), None));
}

#[test]
fn test_aria_label() {
    let d = date!(2024 - 03 - 04); // A Monday
    assert_eq!(aria_label(&d), "Monday, March 4, 2024");
}

#[test]
fn test_relative_calendar_month() {
    with_runtime(|| {
        let enabled_range = DateRange::new(date!(2024 - 01 - 01), date!(2024 - 12 - 31));
        let base_ctx = make_base_ctx_for_relative_month(enabled_range);
        let current_month = Month::June;

        // Outside the enabled range entirely.
        assert_eq!(
            relative_calendar_month(date!(2023 - 12 - 15), &base_ctx, current_month),
            RelativeMonth::Last
        );
        assert_eq!(
            relative_calendar_month(date!(2025 - 01 - 15), &base_ctx, current_month),
            RelativeMonth::Next
        );

        // Inside the enabled range: compared against `current_month`.
        assert_eq!(
            relative_calendar_month(date!(2024 - 05 - 15), &base_ctx, current_month),
            RelativeMonth::Last
        );
        assert_eq!(
            relative_calendar_month(date!(2024 - 06 - 15), &base_ctx, current_month),
            RelativeMonth::Current
        );
        assert_eq!(
            relative_calendar_month(date!(2024 - 07 - 15), &base_ctx, current_month),
            RelativeMonth::Next
        );
    });
}

#[test]
fn test_relative_month_current_month() {
    assert!(RelativeMonth::Current.current_month());
    assert!(!RelativeMonth::Last.current_month());
    assert!(!RelativeMonth::Next.current_month());
}

#[test]
fn test_relative_month_display() {
    assert_eq!(RelativeMonth::Last.to_string(), "last");
    assert_eq!(RelativeMonth::Current.to_string(), "current");
    assert_eq!(RelativeMonth::Next.to_string(), "next");
}

// -----------------------------------------------------------------
// Context accessors (dev-docs/reference/mutants-baseline.md)
// -----------------------------------------------------------------

/// Build a minimal but real `BaseCalendarContext` just to exercise
/// `relative_calendar_month`'s `enabled_date_range` read; must run inside `with_runtime`.
fn make_base_ctx_for_relative_month(enabled_date_range: DateRange) -> BaseCalendarContext {
    BaseCalendarContext {
        focused_date: Signal::new(None),
        view_date: use_signal(|| date!(2024 - 06 - 01)).into(),
        available_ranges: use_memo(|| AvailableRanges::new(&[])),
        set_view_date: Callback::new(|_: Date| {}),
        format_weekday: Callback::new(|w: Weekday| format!("{w:?}")),
        format_month: Callback::new(|m: Month| m.to_string()),
        format_month_short: None,
        disabled: use_signal(|| false).into(),
        today: use_signal(|| date!(2024 - 06 - 15)).into(),
        first_day_of_week: Weekday::Monday,
        show_week_number: false,
        enabled_date_range,
        view_registrations: use_signal(Vec::new),
        direction: use_signal(|| Direction::Ltr).into(),
    }
}

/// Build a real, fully wired `BaseCalendarContext` (not a stand-in) for exercising the
/// signal-wiring accessor methods; must run inside `with_runtime`.
fn make_base_ctx(enabled_date_range: DateRange) -> BaseCalendarContext {
    let view_date_sig = use_signal(|| enabled_date_range.start());
    BaseCalendarContext {
        focused_date: Signal::new(None),
        view_date: view_date_sig.into(),
        available_ranges: use_memo(|| AvailableRanges::new(&[])),
        set_view_date: Callback::new(move |d: Date| {
            let mut view_date_sig = view_date_sig;
            view_date_sig.set(d);
        }),
        format_weekday: Callback::new(|w: Weekday| format!("{w:?}")),
        format_month: Callback::new(|m: Month| m.to_string()),
        format_month_short: None,
        disabled: use_signal(|| false).into(),
        today: use_signal(move || enabled_date_range.start()).into(),
        first_day_of_week: Weekday::Monday,
        show_week_number: false,
        enabled_date_range,
        view_registrations: use_signal(Vec::new),
        direction: use_signal(|| Direction::Ltr).into(),
    }
}

#[test]
fn test_base_calendar_context_focused_date() {
    with_runtime(|| {
        let mut ctx = make_base_ctx(DateRange::new(date!(2024 - 01 - 01), date!(2024 - 12 - 31)));

        assert_eq!(ctx.focused_date(), None);
        ctx.set_focused_date(Some(date!(2024 - 06 - 15)));
        assert_eq!(ctx.focused_date(), Some(date!(2024 - 06 - 15)));

        assert!(ctx.is_focused(date!(2024 - 06 - 15)));
        assert!(!ctx.is_focused(date!(2024 - 06 - 16)));
    });
}

#[test]
fn test_base_calendar_context_set_view_date_clamps_to_enabled_range() {
    with_runtime(|| {
        let ctx = make_base_ctx(DateRange::new(date!(2024 - 01 - 01), date!(2024 - 12 - 31)));

        ctx.set_view_date(date!(2024 - 06 - 15));
        assert_eq!(ctx.view_date(), date!(2024 - 06 - 15));

        // Out-of-range requests are clamped into the enabled range.
        ctx.set_view_date(date!(2025 - 03 - 01));
        assert_eq!(ctx.view_date(), date!(2024 - 12 - 31));

        ctx.set_view_date(date!(2020 - 01 - 01));
        assert_eq!(ctx.view_date(), date!(2024 - 01 - 01));
    });
}

#[test]
fn test_base_calendar_context_is_disabled_and_is_unavailable() {
    with_runtime(|| {
        let mut disabled_sig = use_signal(|| false);
        let mut ctx = make_base_ctx(DateRange::new(date!(2024 - 01 - 01), date!(2024 - 12 - 31)));
        ctx.disabled = disabled_sig.into();
        assert!(!ctx.is_disabled(), "false signal must read back as false");
        disabled_sig.set(true);
        assert!(ctx.is_disabled(), "true signal must read back as true");

        let disabled_ranges = [DateRange::new(date!(2024 - 06 - 10), date!(2024 - 06 - 20))];
        ctx.available_ranges = use_memo(move || AvailableRanges::new(&disabled_ranges));
        assert!(ctx.is_unavailable(date!(2024 - 06 - 15)));
        assert!(!ctx.is_unavailable(date!(2024 - 06 - 01)));
    });
}

#[test]
fn test_base_calendar_context_available_range_with_and_without_range_calendar() {
    with_runtime(|| {
        let ctx = make_base_ctx(DateRange::new(date!(2024 - 01 - 01), date!(2024 - 12 - 31)));
        // No `RangeCalendarContext` has been provided in this scope.
        assert_eq!(ctx.available_range(), None);

        // With a `RangeCalendarContext` whose anchor date is set, the base
        // context's `available_range` looks up the available span around it.
        let anchor_date = Signal::new(Some(date!(2024 - 06 - 15)));
        use_context_provider(|| RangeCalendarContext {
            anchor_date,
            highlighted_range: Signal::new(None),
            set_selected_range: Callback::new(|_: Option<DateRange>| {}),
        });
        assert_eq!(
            ctx.available_range(),
            Some(DateRange::new(date!(2024 - 01 - 01), date!(2024 - 12 - 31))),
            "with no disabled ranges, the full enabled range is available"
        );
    });
}

#[test]
fn test_base_calendar_context_visible_month_count_and_registrations() {
    with_runtime(|| {
        let ctx = make_base_ctx(DateRange::new(date!(2024 - 01 - 01), date!(2024 - 12 - 31)));
        let a = ScopeId(101);
        let b = ScopeId(102);

        assert_eq!(ctx.visible_month_count(), 1, "no views registered yet");
        assert_eq!(ctx.view_registrations.read().len(), 0);

        ctx.register_calendar_view(a, None);
        assert_eq!(
            ctx.calendar_view_offset(a, None),
            0,
            "first registration order"
        );
        assert_eq!(ctx.visible_month_count(), 1);
        assert_eq!(ctx.view_registrations.read().len(), 1);

        // Re-registering the exact same (id, offset) must be a no-op, not a duplicate push.
        ctx.register_calendar_view(a, None);
        assert_eq!(
            ctx.view_registrations.read().len(),
            1,
            "re-registering an identical (id, offset) must not add a duplicate entry"
        );

        ctx.register_calendar_view(b, Some(3));
        assert_eq!(ctx.calendar_view_offset(b, Some(3)), 3);
        assert_eq!(
            ctx.visible_month_count(),
            4,
            "highest offset (3) + 1 determines the visible month count"
        );

        ctx.unregister_calendar_view(b);
        assert_eq!(
            ctx.visible_month_count(),
            1,
            "removing the highest-offset view shrinks the count back down"
        );
        assert_eq!(ctx.view_registrations.read().len(), 1);

        // Unregistering the *only* remaining view must still remove it (guards
        // the "nothing to remove" short-circuit against misfiring when every
        // registered view happens to share the target id).
        ctx.unregister_calendar_view(a);
        assert!(
            ctx.view_registrations.read().is_empty(),
            "unregistering the last remaining view must empty the registrations"
        );

        // Unregistering an id that was never registered is a safe no-op.
        ctx.unregister_calendar_view(a);
        assert!(ctx.view_registrations.read().is_empty());
    });
}

#[test]
fn test_calendar_context_accessors() {
    with_runtime(|| {
        let mut selected = use_signal(|| None::<Date>);
        let last_set = use_signal(|| None::<Option<Date>>);
        let ctx = CalendarContext {
            selected_date: selected.into(),
            set_selected_date: Callback::new(move |d| {
                let mut last_set = last_set;
                last_set.set(Some(d));
            }),
        };

        assert_eq!(ctx.selected_date(), None);
        selected.set(Some(date!(2024 - 06 - 15)));
        assert_eq!(
            ctx.selected_date(),
            Some(date!(2024 - 06 - 15)),
            "selected_date must reflect the underlying signal, not a fixed value"
        );

        ctx.set_selected_date(Some(date!(2024 - 07 - 01)));
        assert_eq!(last_set(), Some(Some(date!(2024 - 07 - 01))));
    });
}

fn make_range_ctx() -> (RangeCalendarContext, Signal<Option<DateRange>>) {
    let anchor_date = Signal::new(None);
    let highlighted_range = Signal::new(None);
    let emitted = Signal::new(None::<DateRange>);
    let ctx = RangeCalendarContext {
        anchor_date,
        highlighted_range,
        set_selected_range: Callback::new(move |r: Option<DateRange>| {
            let mut emitted = emitted;
            emitted.set(r);
        }),
    };
    (ctx, highlighted_range)
}

#[test]
fn test_range_calendar_context_set_selected_date_two_click_flow() {
    with_runtime(|| {
        let (mut ctx, highlighted_range) = make_range_ctx();

        // First click sets the anchor and a same-day highlighted range.
        ctx.set_selected_date(Some(date!(2024 - 06 - 10)));
        assert_eq!(
            highlighted_range(),
            Some(DateRange::new(date!(2024 - 06 - 10), date!(2024 - 06 - 10)))
        );

        // Second click completes the range from the anchor.
        ctx.set_selected_date(Some(date!(2024 - 06 - 20)));
        assert_eq!(
            highlighted_range(),
            Some(DateRange::new(date!(2024 - 06 - 10), date!(2024 - 06 - 20)))
        );
    });
}

#[test]
fn test_range_calendar_context_set_hovered_date() {
    with_runtime(|| {
        let (mut ctx, highlighted_range) = make_range_ctx();

        // No anchor yet: hovering does nothing.
        ctx.set_hovered_date(date!(2024 - 06 - 15));
        assert_eq!(highlighted_range(), None);

        ctx.set_selected_date(Some(date!(2024 - 06 - 10)));
        ctx.set_hovered_date(date!(2024 - 06 - 25));
        assert_eq!(
            highlighted_range(),
            Some(DateRange::new(date!(2024 - 06 - 10), date!(2024 - 06 - 25)))
        );
    });
}

#[test]
fn test_range_calendar_context_reset_selection() {
    with_runtime(|| {
        let (mut ctx, highlighted_range) = make_range_ctx();

        ctx.set_selected_date(Some(date!(2024 - 06 - 10)));
        let reset_to = Some(DateRange::new(date!(2024 - 01 - 01), date!(2024 - 01 - 02)));
        ctx.reset_selection(reset_to);

        assert_eq!(highlighted_range(), reset_to);
        // The anchor was cleared, so a subsequent click starts a new anchor
        // rather than completing a range against the old one.
        ctx.set_selected_date(Some(date!(2024 - 03 - 01)));
        assert_eq!(
            highlighted_range(),
            Some(DateRange::new(date!(2024 - 03 - 01), date!(2024 - 03 - 01)))
        );
    });
}

#[test]
fn test_calendar_view_context_offset_view_date_round_trip() {
    with_runtime(|| {
        let base_ctx = make_base_ctx(DateRange::new(date!(2024 - 01 - 01), date!(2024 - 12 - 31)));
        use_context_provider(|| base_ctx);
        base_ctx.set_view_date(date!(2024 - 06 - 01));

        let view_ctx = CalendarViewContext { offset: 2 };
        assert_eq!(
            view_ctx.offset_view_date(),
            date!(2024 - 08 - 01),
            "offset view date is n months after the base view date"
        );

        // Setting a date on the offset view adjusts the base view date back by the offset.
        view_ctx.set_offset_view_date(date!(2024 - 09 - 15));
        assert_eq!(base_ctx.view_date(), date!(2024 - 07 - 15));
    });
}
