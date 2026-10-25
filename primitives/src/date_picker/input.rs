use super::*;

/// The props for the [`DatePickerInputValue`] component.
#[derive(Props, Clone, PartialEq)]
pub struct DatePickerInputValueProps {
    /// Callback when display day placeholder
    #[props(default = Callback::new(|_| "D".to_string()))]
    pub on_format_day_placeholder: Callback<(), String>,

    /// Callback when display month placeholder
    #[props(default = Callback::new(|_| "M".to_string()))]
    pub on_format_month_placeholder: Callback<(), String>,

    /// Callback when display year placeholder
    #[props(default = Callback::new(|_| "Y".to_string()))]
    pub on_format_year_placeholder: Callback<(), String>,

    /// See `DateElementProps::on_format_month` (backlog row 84 finding 2).
    #[props(default = Callback::new(|month: Month| month.to_string()))]
    pub on_format_month: Callback<Month, String>,

    /// The children of the date value.
    #[props(default)]
    pub children: Option<Element>,
}

/// The props for the [`DateRangePickerInputValue`] component.
#[derive(Props, Clone, PartialEq)]
pub struct DateRangePickerInputValueProps {
    /// Callback when display day placeholder
    #[props(default = Callback::new(|_| "D".to_string()))]
    pub on_format_day_placeholder: Callback<(), String>,

    /// Callback when display month placeholder
    #[props(default = Callback::new(|_| "M".to_string()))]
    pub on_format_month_placeholder: Callback<(), String>,

    /// Callback when display year placeholder
    #[props(default = Callback::new(|_| "Y".to_string()))]
    pub on_format_year_placeholder: Callback<(), String>,

    /// See `DateElementProps::on_format_month` (backlog row 84 finding 2).
    #[props(default = Callback::new(|month: Month| month.to_string()))]
    pub on_format_month: Callback<Month, String>,

    /// The children of the date range value.
    #[props(default)]
    pub children: Option<Element>,
}

/// The props for the [`DateRangePickerStartValue`] component.
#[derive(Props, Clone, PartialEq)]
pub struct DateRangePickerStartValueProps {
    /// The children of the start date value.
    #[props(default)]
    pub children: Option<Element>,
}

/// The props for the [`DateRangePickerEndValue`] component.
#[derive(Props, Clone, PartialEq)]
pub struct DateRangePickerEndValueProps {
    /// The children of the end date value.
    #[props(default)]
    pub children: Option<Element>,
}

#[derive(Clone, Copy)]
pub(super) struct DateRangeInputContext {
    pub(super) start_date: Signal<Option<Date>>,
    pub(super) end_date: Signal<Option<Date>>,
    pub(super) on_format_day_placeholder: Callback<(), String>,
    pub(super) on_format_month_placeholder: Callback<(), String>,
    pub(super) on_format_year_placeholder: Callback<(), String>,
    /// See `DateElementProps::on_format_month` (backlog row 84 finding 2).
    pub(super) on_format_month: Callback<Month, String>,
}

impl DateRangeInputContext {
    /// Sets the start date, but only if it actually differs from the
    /// current value -- the single choke point every producer of
    /// `start_date` (the calendar-driven sync effect in
    /// `DateRangePickerInputValue` and the segment-driven `DateElement` for
    /// the start side) must go through, mirroring the guard
    /// `DatePickerContext::set_date` / `DateRangePickerContext::set_range`
    /// already use one level up. Without it, `start_date` and
    /// `DateElement`'s own `year_value`/`month_value`/`day_value` echo an
    /// unchanged value back and forth forever the instant all three
    /// segments first resolve (`DateElement`'s "sync down from
    /// selected_date" effect writes the segment signals, whose "sync up via
    /// on_date_change" effect writes this signal right back) -- see
    /// backlog row 83 for the full trace (the CPU stays pinned and memory
    /// grows until the tab crashes).
    fn set_start_date(&mut self, date: Option<Date>) {
        if *self.start_date.peek() != date {
            self.start_date.set(date);
        }
    }

    /// See [`Self::set_start_date`]; the same choke point for `end_date`.
    fn set_end_date(&mut self, date: Option<Date>) {
        if *self.end_date.peek() != date {
            self.end_date.set(date);
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub(super) struct DateElementProps {
    /// The start index (used for focus)
    #[props(default = 0)]
    pub start_index: usize,

    /// The selected date
    pub selected_date: ReadSignal<Option<Date>>,

    /// Callback when selected date changes
    #[props(default)]
    pub on_date_change: Callback<Option<Date>>,

    /// Callback when display day placeholder
    #[props(default = Callback::new(|_| "D".to_string()))]
    pub on_format_day_placeholder: Callback<(), String>,

    /// Callback when display month placeholder
    #[props(default = Callback::new(|_| "M".to_string()))]
    pub on_format_month_placeholder: Callback<(), String>,

    /// Callback when display year placeholder
    #[props(default = Callback::new(|_| "Y".to_string()))]
    pub on_format_year_placeholder: Callback<(), String>,

    /// Locale-aware month name for the month segment's `aria-valuetext`
    /// (backlog row 84 finding 2). Defaults to `Month::to_string()` (plain
    /// English), matching `DatePickerCalendarProps::on_format_month`'s own
    /// default for the calendar grid -- pass e.g. `dioxus_i18n`'s `tid!` for
    /// a translated name, the same way the calendar side already can.
    #[props(default = Callback::new(|month: Month| month.to_string()))]
    pub on_format_month: Callback<Month, String>,

    /// The children of the date element.
    #[props(default)]
    pub children: Option<Element>,
}

#[component]
pub(super) fn DateElement(props: DateElementProps) -> Element {
    let ctx = use_context::<BaseDatePickerContext>();
    let selected_date = props.selected_date.peek().cloned();

    let mut day_value = use_signal(move || selected_date.map(|date| date.day()));
    let mut month_value = use_signal(move || selected_date.map(|date| date.month() as u8));
    let mut year_value = use_signal(move || selected_date.map(|date| date.year()));

    use_effect(move || {
        // Sync down from `props.selected_date` into the three editable
        // segment signals -- but only where the decomposed value actually
        // differs from what the segment already holds. `on_date_change`
        // below re-derives `props.selected_date` from these same three
        // signals and calls back up through it every time all three
        // resolve; for a `DateRangePicker` side, that callback writes
        // straight back into the very signal this effect reads
        // (`DateRangeInputContext::set_start_date`/`set_end_date`). An
        // unconditional `.set()` here would close that
        // into an unbounded cycle the instant a date first completes, even
        // though `set_start_date`/`set_end_date` themselves already
        // no-op an unchanged value -- Dioxus's `Signal::set` marks a
        // signal dirty (and reschedules every subscriber) regardless of
        // whether the new value equals the old one, so the *other* half of
        // a two-way sync has to refuse the redundant write too, or the
        // "no-op" write still wakes this effect for nothing. See
        // dev-docs/backlog.md row 83.
        let date = (props.selected_date)();
        let new_year = date.map(|d| d.year());
        if *year_value.peek() != new_year {
            year_value.set(new_year);
        }
        let new_month = date.map(|d| d.month() as u8);
        if *month_value.peek() != new_month {
            month_value.set(new_month);
        }
        let new_day = date.map(|d| d.day());
        if *day_value.peek() != new_day {
            day_value.set(new_day);
        }
    });

    use_effect(move || {
        if let (Some(year), Some(month), Some(day)) = (
            year_value(),
            month_value().and_then(|m| Month::try_from(m).ok()),
            day_value(),
        ) {
            if let Some(date) = Date::from_calendar_date(year, month, day)
                .ok()
                .filter(|date| ctx.enabled_date_range.contains(*date))
                .filter(|date| ctx.available_ranges.read().valid_interval(*date))
            {
                props.on_date_change.call(Some(date));
            }
        }
    });

    use_context_provider(|| DateElementContext {
        start_index: props.start_index,
        year_value,
        month_value,
        day_value,
        on_format_day_placeholder: props.on_format_day_placeholder,
        on_format_month_placeholder: props.on_format_month_placeholder,
        on_format_year_placeholder: props.on_format_year_placeholder,
        on_format_month: props.on_format_month,
    });

    let children = props.children.unwrap_or_else(|| {
        rsx! {
            DatePickerYearSegment {}
            DatePickerSeparator {}
            DatePickerMonthSegment {}
            DatePickerSeparator {}
            DatePickerDaySegment {}
        }
    });

    rsx! {
        {children}
    }
}

/// The editable date value for a single date picker input.
#[component]
pub fn DatePickerInputValue(props: DatePickerInputValueProps) -> Element {
    let mut base_ctx = use_context::<BaseDatePickerContext>();
    let mut ctx = use_context::<DatePickerContext>();

    rsx! {
        DateElement {
            selected_date: ctx.selected_date,
            on_date_change: move |date: Option<Date>| {
                // `DateElement` reports EVERY date its three segments resolve to -- the one just typed
                // AND the one the calendar just selected (the segments re-render from it and commit
                // it back up). Only a typed commit, which changes the value, closes the popover here:
                // the echo of a calendar selection finds the value already set, and closing on it
                // would close the popover on the same frame as the click -- ahead of `close_delay`, and
                // even with `close_on_select: false`. The calendar closes it itself
                // (`BaseDatePickerContext::selection_complete`).
                let changed = *ctx.selected_date.peek() != date;
                ctx.set_date(date);
                if changed {
                    base_ctx.open.set(false);
                }
            },
            on_format_day_placeholder: props.on_format_day_placeholder,
            on_format_month_placeholder: props.on_format_month_placeholder,
            on_format_year_placeholder: props.on_format_year_placeholder,
            on_format_month: props.on_format_month,
            children: props.children,
        }
    }
}

/// The editable date range value for a date range picker input.
#[component]
pub fn DateRangePickerInputValue(props: DateRangePickerInputValueProps) -> Element {
    let base_ctx = use_context::<BaseDatePickerContext>();
    let mut ctx = use_context::<DateRangePickerContext>();
    let selected_range = ctx.date_range.peek().cloned();

    let start_date = use_signal(move || selected_range.map(|range| range.start()));
    let end_date = use_signal(move || selected_range.map(|range| range.end()));
    // Built once, up front, so both this component's own sync-down effect
    // and every `DateElement` child (via the context provider below) write
    // `start_date`/`end_date` through the exact same guarded choke point --
    // see `DateRangeInputContext::set_start_date`'s doc comment for why an
    // unconditional write here forms an unbounded cycle with `DateElement`'s
    // own effects the instant a range first completes.
    let mut range_ctx = DateRangeInputContext {
        start_date,
        end_date,
        on_format_day_placeholder: props.on_format_day_placeholder,
        on_format_month_placeholder: props.on_format_month_placeholder,
        on_format_year_placeholder: props.on_format_year_placeholder,
        on_format_month: props.on_format_month,
    };

    use_effect(move || {
        let date_range = ctx.date_range.cloned();
        range_ctx.set_start_date(date_range.map(|r| r.start()));
        range_ctx.set_end_date(date_range.map(|r| r.end()));
    });

    use_effect(move || {
        if let (Some(start), Some(end)) = (start_date(), end_date()) {
            // force auto validation for input range
            if end < start {
                return;
            }

            // checking non-contiguous ranges
            if base_ctx
                .available_ranges
                .read()
                .available_range(start, base_ctx.enabled_date_range)
                .is_some_and(|r| r.contains(end))
            {
                let range = Some(DateRange::new(start, end));
                ctx.set_range(range);
            }
        };
    });

    use_context_provider(|| range_ctx);

    let children = props.children.unwrap_or_else(|| {
        rsx! {
            DateRangePickerStartValue {}
            DatePickerSeparator {
                symbol: '—',
            }
            DateRangePickerEndValue {}
        }
    });

    rsx! {
        {children}
    }
}

/// The editable start date value in a range picker input.
#[component]
pub fn DateRangePickerStartValue(props: DateRangePickerStartValueProps) -> Element {
    let mut ctx = use_context::<DateRangeInputContext>();

    rsx! {
        DateElement {
            selected_date: ctx.start_date,
            on_date_change: move |date| ctx.set_start_date(date),
            on_format_day_placeholder: ctx.on_format_day_placeholder,
            on_format_month_placeholder: ctx.on_format_month_placeholder,
            on_format_year_placeholder: ctx.on_format_year_placeholder,
            on_format_month: ctx.on_format_month,
            children: props.children,
        }
    }
}

/// The editable end date value in a range picker input.
#[component]
pub fn DateRangePickerEndValue(props: DateRangePickerEndValueProps) -> Element {
    let mut ctx = use_context::<DateRangeInputContext>();

    rsx! {
        DateElement {
            start_index: 3,
            selected_date: ctx.end_date,
            on_date_change: move |date| ctx.set_end_date(date),
            on_format_day_placeholder: ctx.on_format_day_placeholder,
            on_format_month_placeholder: ctx.on_format_month_placeholder,
            on_format_year_placeholder: ctx.on_format_year_placeholder,
            on_format_month: ctx.on_format_month,
            children: props.children,
        }
    }
}

/// The props for the [`DatePickerInput`] component
#[derive(Props, Clone, PartialEq)]
pub struct DatePickerInputProps {
    /// Callback when display day placeholder
    #[props(default = Callback::new(|_| "D".to_string()))]
    pub on_format_day_placeholder: Callback<(), String>,

    /// Callback when display month placeholder
    #[props(default = Callback::new(|_| "M".to_string()))]
    pub on_format_month_placeholder: Callback<(), String>,

    /// Callback when display year placeholder
    #[props(default = Callback::new(|_| "Y".to_string()))]
    pub on_format_year_placeholder: Callback<(), String>,

    /// See `DateElementProps::on_format_month` (backlog row 84 finding 2).
    /// Only applies to the default children (a caller-supplied `children`
    /// renders its own segments directly and must thread this itself, the
    /// same limitation the placeholder callbacks above have).
    #[props(default = Callback::new(|month: Month| month.to_string()))]
    pub on_format_month: Callback<Month, String>,

    /// Additional attributes for the value element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the date picker element
    #[props(default)]
    pub children: Option<Element>,
}

/// # DatePickerInput
///
/// The input element for the [`DatePicker`] component which allow users to enter a date value.
///
/// ## Example
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::{calendar::Calendar, date_picker::*, popover::*, ContentAlign};
/// use time::Date;
/// #[component]
/// fn Demo() -> Element {
///    let mut selected_date = use_signal(|| None::<Date>);
///    rsx! {
///        div {
///            DatePicker {
///                selected_date: selected_date(),
///                on_value_change: move |date| {
///                    tracing::info!("Date changed to: {date:?}");
///                    selected_date.set(date);
///               },
///                DatePickerPopover {
///                    DatePickerInput {
///                        PopoverTrigger {
///                            "Select date"
///                        }
///                        PopoverContent {
///                            align: ContentAlign::End,
///                            DatePickerCalendar {
///                                calendar: Calendar,
///                            }
///                        }
///                    }
///                }
///            }
///        }
///    }
///}
/// ```
#[component]
pub fn DatePickerInput(props: DatePickerInputProps) -> Element {
    let children = props.children.unwrap_or_else(|| {
        rsx! {
            DatePickerInputValue {
                on_format_day_placeholder: props.on_format_day_placeholder,
                on_format_month_placeholder: props.on_format_month_placeholder,
                on_format_year_placeholder: props.on_format_year_placeholder,
                on_format_month: props.on_format_month,
            }
        }
    });

    rsx! {
        div { ..props.attributes,
            {children}
        }
    }
}

/// # DateRangePickerInput
///
/// The input element for the [`DateRangePicker`] component which allow users to enter a date range.
///
/// ## Example
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::{calendar::{DateRange, RangeCalendar}, date_picker::*, popover::*, ContentAlign};
/// #[component]
/// fn Demo() -> Element {
///    let mut selected_range = use_signal(|| None::<DateRange>);
///    rsx! {
///        div {
///            DateRangePicker {
///                selected_range: selected_range(),
///                on_range_change: move |range| {
///                    tracing::info!("Selected range: {:?}", range);
///                    selected_range.set(range);
///               },
///                DatePickerPopover {
///                    DateRangePickerInput {
///                        PopoverTrigger {
///                            "Select date"
///                        }
///                        PopoverContent {
///                            align: ContentAlign::End,
///                            DateRangePickerCalendar {
///                                calendar: RangeCalendar,
///                            }
///                        }
///                    }
///                }
///            }
///        }
///    }
///}
/// ```
#[component]
pub fn DateRangePickerInput(props: DatePickerInputProps) -> Element {
    let children = props.children.unwrap_or_else(|| {
        rsx! {
            DateRangePickerInputValue {
                on_format_day_placeholder: props.on_format_day_placeholder,
                on_format_month_placeholder: props.on_format_month_placeholder,
                on_format_year_placeholder: props.on_format_year_placeholder,
                on_format_month: props.on_format_month,
            }
        }
    });

    rsx! {
        div { ..props.attributes,
            {children}
        }
    }
}
