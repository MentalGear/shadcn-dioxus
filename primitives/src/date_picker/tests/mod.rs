use super::*;
use time::macros::date;
mod key_effects;
mod state;

#[component]
fn ControlledDatePicker() -> Element {
    rsx! {
        DatePicker {
            selected_date: Some(date!(2026 - 05 - 07)),
            DatePickerInput {}
        }
    }
}

#[component]
fn ControlledDateRangePicker() -> Element {
    rsx! {
        DateRangePicker {
            selected_range: Some(DateRange::new(date!(2026 - 05 - 07), date!(2026 - 05 - 11))),
            DateRangePickerInput {}
        }
    }
}

#[test]
fn date_picker_input_renders_controlled_date_on_first_render() {
    let mut dom = VirtualDom::new(ControlledDatePicker);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    assert!(html.contains("2026"));
    assert!(html.contains("05"));
    assert!(html.contains("07"));
    assert!(!html.contains("YYYY"));
    assert!(!html.contains("MM"));
    assert!(!html.contains("DD"));
}

#[component]
fn UntouchedDatePicker() -> Element {
    rsx! {
        DatePicker {
            DatePickerInput {}
        }
    }
}

#[test]
fn untouched_date_segments_omit_aria_valuenow_and_aria_valuetext() {
    // backlog row 84 findings 2/3/6: before any date is chosen, every
    // segment's `props.value` is `None` -- `aria-valuenow` (and the
    // month segment's `aria-valuetext`) must be entirely absent
    // rather than substituting `default`/a formatted default (with
    // `now_value` as `(props.value)().unwrap_or(props.default)`,
    // `aria-valuenow="<today's year>"` etc. would always be present).
    let mut dom = VirtualDom::new(UntouchedDatePicker);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    assert!(
        !html.contains("aria-valuenow"),
        "an untouched segment must not report any aria-valuenow: {html}"
    );
    assert!(
        !html.contains("aria-valuetext"),
        "an untouched month segment must not report any aria-valuetext: {html}"
    );
    // Sanity: the segments really did render (the assertions above
    // aren't vacuously true because nothing rendered at all).
    assert!(html.contains(r#"aria-label="year""#));
    assert!(html.contains("YYYY"));
}

#[test]
fn touched_date_segments_report_aria_valuenow_and_month_reports_a_locale_aware_aria_valuetext() {
    // Counterpart to `untouched_date_segments_omit_aria_valuenow_and_
    // aria_valuetext` above: once a segment actually holds a value
    // (here, `ControlledDatePicker`'s fixed 2026-05-07), `aria-valuenow`
    // must report it, and the month segment's `aria-valuetext` must be
    // the default formatter's locale-aware month name (backlog row 84
    // finding 2 -- `Month::to_string()` by default, "May" for month 5),
    // not merely "some non-empty string": the test fails if
    // `aria-valuetext` is not rendered at all, since
    // `html.contains(r#"aria-valuetext="May""#)` is then false.
    let mut dom = VirtualDom::new(ControlledDatePicker);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    assert!(
        html.contains(r#"aria-valuenow="2026""#),
        "year segment must report its value: {html}"
    );
    assert!(
        html.contains(r#"aria-valuenow="5""#),
        "month segment must report its raw numeric value too: {html}"
    );
    assert!(
        html.contains(r#"aria-valuenow="7""#),
        "day segment must report its value: {html}"
    );
    assert!(
        html.contains(r#"aria-valuetext="May""#),
        "month segment must report a locale-aware name via aria-valuetext: {html}"
    );
    // Year and day are plain numbers -- no aria-valuetext for either
    // (backlog row 84 finding 2 is scoped to the month segment only,
    // per the APG spinbutton pattern's own rationale: a bare number is
    // "user-friendly" for a day-of-month or a year, unlike a month
    // index). `year value_text` is fixed structurally (only
    // `DatePickerMonthSegment` ever sets `DateSegmentProps::value_text`)
    // rather than by an attribute count here, since `aria-valuetext`
    // appearing exactly once total already proves it's not also on
    // year/day.
    assert_eq!(html.matches("aria-valuetext").count(), 1, "{html}");
}

#[test]
fn date_segment_renders_aria_label_and_no_dangling_aria_labelledby() {
    // Guard for the `aria-labelledby` shape -- SSR-level
    // rather than the browser-level `playwright/date-picker.spec.ts`
    // check, so it runs without needing a `dx serve --web` build; the
    // browser-level test is the fuller companion.
    let mut dom = VirtualDom::new(ControlledDatePicker);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    assert!(
        html.contains(r#"aria-label="year""#),
        "aria-label must still be present: {html}"
    );
    assert!(
            !html.contains("aria-labelledby"),
            "aria-labelledby must not be rendered at all (no visible label element exists to reference): {html}"
        );
}

#[test]
fn date_range_picker_input_renders_controlled_range_on_first_render() {
    let mut dom = VirtualDom::new(ControlledDateRangePicker);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    assert!(html.contains("2026"));
    assert!(html.contains("05"));
    assert!(html.contains("07"));
    assert!(html.contains("11"));
    assert!(!html.contains("YYYY"));
    assert!(!html.contains("MM"));
    assert!(!html.contains("DD"));
}
