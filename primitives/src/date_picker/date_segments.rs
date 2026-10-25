use super::*;

#[derive(Clone, Copy)]
pub(super) struct DateElementContext {
    pub(super) start_index: usize,
    pub(super) year_value: Signal<Option<i32>>,
    pub(super) month_value: Signal<Option<u8>>,
    pub(super) day_value: Signal<Option<u8>>,
    pub(super) on_format_day_placeholder: Callback<(), String>,
    pub(super) on_format_month_placeholder: Callback<(), String>,
    pub(super) on_format_year_placeholder: Callback<(), String>,
    /// Locale-aware month name for the month segment's `aria-valuetext`
    /// (backlog row 84 finding 2) -- see `DateElementProps::on_format_month`.
    pub(super) on_format_month: Callback<Month, String>,
}

/// The props for the [`DatePickerYearSegment`] component.
#[derive(Props, Clone, PartialEq)]
pub struct DatePickerYearSegmentProps {
    /// Additional attributes for the year segment element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// The props for the [`DatePickerMonthSegment`] component.
#[derive(Props, Clone, PartialEq)]
pub struct DatePickerMonthSegmentProps {
    /// Additional attributes for the month segment element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// The props for the [`DatePickerDaySegment`] component.
#[derive(Props, Clone, PartialEq)]
pub struct DatePickerDaySegmentProps {
    /// Additional attributes for the day segment element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// The props for the [`DatePickerSeparator`] component.
#[derive(Props, Clone, PartialEq)]
pub struct DatePickerSeparatorProps {
    /// The separator symbol.
    #[props(default = '-')]
    pub symbol: char,

    /// Additional attributes for the separator element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// A year segment in a date input.
#[component]
pub fn DatePickerYearSegment(props: DatePickerYearSegmentProps) -> Element {
    let mut ctx = use_context::<DateElementContext>();
    let base_ctx = use_context::<BaseDatePickerContext>();
    let today = use_today();
    let min_year = base_ctx.enabled_date_range.start().year();
    let max_year = base_ctx.enabled_date_range.end().year();

    // `aria_label: "year"` is an overridable default folded into the same
    // `merge_attributes` call as `props.attributes`, not an ad-hoc keyed
    // prop beside a separately-threaded `attributes: props.attributes`:
    // `DateSegment` has no typed `aria_label` field, so both would land as
    // independent entries in the same forwarded `Vec<Attribute>`, the
    // component-forward shape of backlog row 93's duplicate-attribute
    // hazard. A caller's own `aria_label` wins, the same as at the
    // element-level sites in this file.
    let defaults = attributes!(div { aria_label: "year" });
    let merged = merge_attributes(vec![defaults, props.attributes.clone()]);

    rsx! {
        DateSegment {
            index: ctx.start_index,
            value: ctx.year_value,
            default: today().year(),
            on_value_change: move |value: Option<i32>| ctx.year_value.set(value),
            min: min_year,
            max: max_year,
            max_length: 4,
            // A year is not cyclical (unlike month/day) -- backlog row 84
            // finding 4, see `date_segment_key_effects`'s own doc for the
            // full calibration (native Chromium `<input type="date">` does
            // not wrap its year sub-field either).
            wrap: false,
            on_format_placeholder: ctx.on_format_year_placeholder,
            attributes: merged,
        }
    }
}

/// Compute the min/max month allowed for the given year, based on the
/// enabled date range's boundary months (only the boundary *year* clamps the
/// month; any other year allows the full January-December range).
pub(super) fn month_bounds_for_year(
    year_value: Option<i32>,
    min_year: i32,
    max_year: i32,
    min_date: Date,
    max_date: Date,
) -> (u8, u8) {
    let min_month = match year_value {
        Some(year) if year == min_year => min_date.month(),
        _ => Month::January,
    };
    let max_month = match year_value {
        Some(year) if year == max_year => max_date.month(),
        _ => Month::December,
    };
    (min_month as u8, max_month as u8)
}

/// A month segment in a date input.
#[component]
pub fn DatePickerMonthSegment(props: DatePickerMonthSegmentProps) -> Element {
    let mut ctx = use_context::<DateElementContext>();
    let base_ctx = use_context::<BaseDatePickerContext>();
    let today = use_today();
    let min_date = base_ctx.enabled_date_range.start();
    let max_date = base_ctx.enabled_date_range.end();
    let min_year = min_date.year();
    let max_year = max_date.year();
    let (min_month, max_month) =
        month_bounds_for_year((ctx.year_value)(), min_year, max_year, min_date, max_date);

    // backlog row 84 finding 2: a locale-aware month *name* for
    // `aria-valuetext`, not a bare 1-12 number -- `ctx.on_format_month` is
    // the same kind of caller-suppliable, i18n-capable callback the
    // calendar grid's `CalendarMonthTitle`/`CalendarSelectMonthValue` (and
    // `DatePickerCalendarProps::on_format_month`) already use, defaulting to
    // `Month::to_string()` (see `DateElementProps::on_format_month`) but
    // overridable with e.g. `dioxus_i18n`'s `tid!` for a translated name.
    // `None` while the segment has no value yet (`ctx.month_value()` is
    // `None`), matching `aria-valuenow`'s own untouched-is-absent rule
    // (finding 3) so the two stay in lockstep -- see `DateSegment`'s
    // `now_value` doc.
    let value_text = (ctx.month_value)()
        .and_then(|m| Month::try_from(m).ok())
        .map(|m| ctx.on_format_month.call(m));

    // Same reasoning as `DatePickerYearSegment` above.
    let defaults = attributes!(div {
        aria_label: "month"
    });
    let merged = merge_attributes(vec![defaults, props.attributes.clone()]);

    rsx! {
        DateSegment {
            index: ctx.start_index + 1usize,
            value: ctx.month_value,
            default: today().month() as u8,
            on_value_change: move |value: Option<u8>| ctx.month_value.set(value),
            min: min_month,
            max: max_month,
            max_length: 2,
            // Cyclical (December -> January) -- backlog row 84 finding 4,
            // matches native Chromium `<input type="date">`'s month
            // sub-field. `wrap: true` is also `DateSegment`'s own default;
            // stated explicitly here (and on the day segment below) so the
            // calibration decision is visible at every call site, not just
            // the one (year) that overrides it.
            wrap: true,
            on_format_placeholder: ctx.on_format_month_placeholder,
            value_text,
            attributes: merged,
        }
    }
}

/// Compute the min/max day allowed for the given year/month, based on the
/// enabled date range's boundary year+month. Falls back to the target
/// month's actual length (or 31, if the month value is out of range) when
/// not at a boundary.
pub(super) fn day_bounds_for_year_month(
    year_value: Option<i32>,
    month_value: Option<u8>,
    min_year: i32,
    max_year: i32,
    min_date: Date,
    max_date: Date,
) -> (u8, u8) {
    let min_day = match (year_value, month_value) {
        (Some(year), Some(month)) if year == min_year && month == min_date.month() as u8 => {
            min_date.day()
        }
        _ => 1,
    };
    let max_day = match (year_value, month_value) {
        (Some(year), Some(month)) if year == max_year && month == max_date.month() as u8 => {
            max_date.day()
        }
        (Some(year), Some(month)) => {
            if let Ok(month) = Month::try_from(month) {
                month.length(year)
            } else {
                31
            }
        }
        _ => 31,
    };
    (min_day, max_day)
}

/// A day segment in a date input.
#[component]
pub fn DatePickerDaySegment(props: DatePickerDaySegmentProps) -> Element {
    let mut ctx = use_context::<DateElementContext>();
    let base_ctx = use_context::<BaseDatePickerContext>();
    let today = use_today();
    let min_date = base_ctx.enabled_date_range.start();
    let max_date = base_ctx.enabled_date_range.end();
    let min_year = min_date.year();
    let max_year = max_date.year();
    let (min_day, max_day) = day_bounds_for_year_month(
        (ctx.year_value)(),
        (ctx.month_value)(),
        min_year,
        max_year,
        min_date,
        max_date,
    );

    // Same reasoning as `DatePickerYearSegment` above.
    let defaults = attributes!(div { aria_label: "day" });
    let merged = merge_attributes(vec![defaults, props.attributes.clone()]);

    rsx! {
        DateSegment {
            index: ctx.start_index + 2usize,
            value: ctx.day_value,
            default: today().day(),
            on_value_change: move |value: Option<u8>| ctx.day_value.set(value),
            min: min_day,
            max: max_day,
            max_length: 2,
            // Cyclical (the last day of a month -> the 1st) -- backlog row
            // 84 finding 4, matches native Chromium `<input type="date">`'s
            // day sub-field. See `DatePickerMonthSegment`'s identical note.
            wrap: true,
            on_format_placeholder: ctx.on_format_day_placeholder,
            attributes: merged,
        }
    }
}

/// A separator in a date input.
#[component]
pub fn DatePickerSeparator(props: DatePickerSeparatorProps) -> Element {
    let owned = attributes!(span {
        aria_hidden: "true",
        tabindex: "-1",
        "is-separator": true,
        "no-date": true,
    });
    let merged = merge_attributes(vec![props.attributes.clone(), owned]);

    rsx! {
        span { ..merged, "{props.symbol}" }
    }
}
