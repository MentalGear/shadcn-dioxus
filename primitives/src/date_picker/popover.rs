use super::*;

/// The props for the [`DatePickerPopover`] component.
#[allow(unpredictable_function_pointer_comparisons)]
#[derive(Props, Clone, PartialEq)]
pub struct DatePickerPopoverProps {
    /// Whether the popover is a modal and should capture focus. Defaults to `false`, like
    /// [`PopoverRootProps::is_modal`] (which this is forwarded to) and shadcn's date picker: the
    /// calendar is a non-modal popover, so the page behind it stays interactive and scrollable.
    #[props(default = ReadSignal::new(Signal::new(false)))]
    pub is_modal: ReadSignal<bool>,

    /// Whether a **modal** popover dims the page behind it with the shared overlay scrim, like
    /// [`PopoverRootProps::overlay`] (which this is forwarded to). Defaults to `false`: no scrim.
    /// Has no effect while `is_modal` is `false`, which the styled pickers always are.
    #[props(default = ReadSignal::new(Signal::new(false)))]
    pub overlay: ReadSignal<bool>,

    /// The controlled open state of the popover.
    pub open: ReadSignal<Option<bool>>,

    /// The default open state when uncontrolled.
    #[props(default)]
    pub default_open: bool,

    /// Callback fired when the open state changes.
    #[props(default)]
    pub on_open_change: Callback<bool>,

    /// Additional attributes to apply to the popover root element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the popover root component.
    pub children: Element,

    /// The popover root component to use.
    #[props(default = PopoverRoot)]
    pub popover_root: fn(PopoverRootProps) -> Element,
}

/// # DatePickerPopover
///
/// The `DatePickerPopover` component wraps all the popover components and manages the state.
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
pub fn DatePickerPopover(props: DatePickerPopoverProps) -> Element {
    let ctx = use_context::<BaseDatePickerContext>();
    let mut open = ctx.open;

    let PopoverRoot = props.popover_root;

    rsx! {
        PopoverRoot {
            // `is_modal` (documented "whether the popover is a modal and
            // should capture focus") must be forwarded here, or it silently
            // has no effect and every `DatePickerPopover` renders with
            // `PopoverRoot`'s own default no matter what a caller passed.
            // See `preview/src/components/date_picker/component.rs` for why
            // this matters in practice: `PopoverModalContent`'s DOM-
            // relative "centering trick" (`position: absolute; left: 50%;
            // transform: translateX(-50%)`, `../popover/style.css`) centers
            // the calendar under its *positioned ancestor* (the narrow
            // `.dx-date-picker` input group), not its trigger, and has no
            // collision/edge-avoidance -- so a calendar wider than that
            // ancestor renders partly off-screen whenever the ancestor sits
            // near a viewport edge. Forwarding `is_modal` is what lets that
            // preview component's `is_modal: false` switch the calendar
            // onto the non-modal, trigger-anchored arm instead.
            is_modal: props.is_modal,
            overlay: props.overlay,
            open: open(),
            on_open_change: move |v| open.set(v),
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[doc(hidden)]
/// A trait for types that can provide default calendar rendering.
pub trait DefaultCalendarProps {
    /// Provide a default calendar rendering function.
    fn default_calendar(self) -> Element;
}

/// The props for the Calendar component.
#[allow(unpredictable_function_pointer_comparisons)]
#[derive(Props, Clone, PartialEq)]
pub struct DatePickerCalendarProps<T: DefaultCalendarProps + Properties + PartialEq> {
    /// Callback when display weekday
    #[props(default = Callback::new(|weekday: Weekday| weekday_abbreviation(weekday).to_string()))]
    pub on_format_weekday: Callback<Weekday, String>,

    /// Callback when display month
    #[props(default = Callback::new(|month: Month| month.to_string()))]
    pub on_format_month: Callback<Month, String>,

    /// Callback when displaying a month inside a month dropdown; forwarded to the calendar's
    /// `on_format_month_short` (`None` derives it from `on_format_month`).
    #[props(default)]
    pub on_format_month_short: Option<Callback<Month, String>>,

    /// The month being viewed. `None` (the default) shows the month of `today`.
    #[props(default)]
    pub view_date: ReadSignal<Option<Date>>,

    /// The current date (used for highlighting today). `None` (the default) is the visitor's
    /// local date, read after mount ([`crate::calendar::use_today`]).
    #[props(default)]
    pub today: Option<Date>,

    /// Callback when view date changes
    #[props(default)]
    pub on_view_change: Callback<Date>,

    /// Whether the calendar is disabled
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// First day of the week
    #[props(default = Weekday::Sunday)]
    pub first_day_of_week: Weekday,

    /// Lower limit of the range of available dates
    #[props(default = date!(1925-01-01))]
    pub min_date: Date,

    /// Upper limit of the range of available dates
    #[props(default = date!(2050-12-31))]
    pub max_date: Date,

    /// Unavailable dates
    #[props(default)]
    pub disabled_ranges: ReadSignal<Vec<DateRange>>,

    /// Additional attributes to extend the calendar element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the calendar element
    pub children: Element,

    /// The calendar to render with
    #[props(default = T::default_calendar)]
    pub calendar: fn(T) -> Element,
}

/// # DatePickerCalendar
///
/// The [`DatePickerCalendar`] component provides an accessible calendar interface with arrow key navigation, month switching, and date selection.
/// Used as date picker popover component
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
///                            DatePickerCalendar {}
///                        }
///                    }
///                }
///            }
///        }
///    }
///}
/// ```
#[component]
pub fn DatePickerCalendar(props: DatePickerCalendarProps<CalendarProps>) -> Element {
    let base_ctx = use_context::<BaseDatePickerContext>();
    let mut ctx = use_context::<DatePickerContext>();

    #[allow(non_snake_case)]
    let Calendar = props.calendar;
    // `None` until something picks a month: the calendar then shows the month of `today`.
    let mut view_date = use_signal(|| (props.view_date)());
    // The calendar follows what it displays: the staged selection while one is pending, the
    // committed date otherwise.
    let displayed = ctx.staging.displayed;
    use_effect(move || {
        if let Some(date) = displayed() {
            view_date.set(Some(date));
        }
    });

    let min_date = base_ctx.enabled_date_range.start();
    let max_date = base_ctx.enabled_date_range.end();

    rsx! {
        Calendar {
            selected_date: displayed,
            on_date_change: move |date: Option<Date>| {
                // `confirm`: the selection is staged, nothing is committed and nothing closes --
                // Apply (or Cancel / Escape / an outside click) decides.
                if ctx.staging.stage(base_ctx.confirm, date) {
                    return;
                }
                // While a close is pending, "no date" is a click on the day that was JUST
                // selected (a double-click, or someone confirming what they are watching land),
                // which the calendar reads as a toggle-off. That is not what they mean, and the
                // dwell is what opened the window for it, so it keeps the selection and restarts
                // the wait instead of clearing the value. Without a pending close (`close_on_select:
                // false`, or the popover reopened) a click on the selected day still clears it.
                if date.is_none() && base_ctx.dwell.is_pending() {
                    base_ctx.selection_complete();
                    return;
                }
                ctx.set_date(date);
                base_ctx.selection_complete();
            },
            disabled_ranges: base_ctx.available_ranges.read().to_disabled_ranges(),
            on_format_weekday: props.on_format_weekday,
            on_format_month: props.on_format_month,
            on_format_month_short: props.on_format_month_short,
            view_date: view_date(),
            on_view_change: move |date| {
                base_ctx.dwell.cancel();
                view_date.set(Some(date));
            },
            today: props.today,
            disabled: props.disabled,
            first_day_of_week: props.first_day_of_week,
            min_date,
            max_date,
            attributes: props.attributes,
            {props.children}
        }
    }
}

/// # DateRangePickerCalendar
///
/// The [`DateRangePickerCalendar`] component provides an accessible calendar interface with arrow key navigation, month switching, and date range selection.
/// Used as date picker popover component
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
///                            DateRangePickerCalendar {}
///                        }
///                    }
///                }
///            }
///        }
///    }
///}
/// ```
#[component]
pub fn DateRangePickerCalendar(props: DatePickerCalendarProps<RangeCalendarProps>) -> Element {
    let base_ctx = use_context::<BaseDatePickerContext>();
    let mut ctx = use_context::<DateRangePickerContext>();

    #[allow(non_snake_case)]
    let RangeCalendar = props.calendar;
    let mut view_date = use_signal(|| (props.view_date)());
    let displayed = ctx.staging.displayed;
    use_effect(move || {
        if let Some(r) = displayed() {
            view_date.set(Some(r.start()));
        }
    });

    let min_date = base_ctx.enabled_date_range.start();
    let max_date = base_ctx.enabled_date_range.end();

    rsx! {
        RangeCalendar {
            selected_range: displayed,
            // `RangeCalendar` only reports a range once its END date is picked (the first click
            // just anchors it), so this is the "range is complete" moment -- never the start.
            on_range_change: move |range| {
                // `confirm`: stage the completed range; Apply commits it.
                if ctx.staging.stage(base_ctx.confirm, range) {
                    return;
                }
                ctx.set_range(range);
                base_ctx.selection_complete();
            },
            disabled_ranges: base_ctx.available_ranges.read().to_disabled_ranges(),
            on_format_weekday: props.on_format_weekday,
            on_format_month: props.on_format_month,
            on_format_month_short: props.on_format_month_short,
            view_date: view_date(),
            on_view_change: move |date| {
                base_ctx.dwell.cancel();
                view_date.set(Some(date));
            },
            today: props.today,
            disabled: props.disabled,
            first_day_of_week: props.first_day_of_week,
            min_date,
            max_date,
            attributes: props.attributes,
            {props.children}
        }
    }
}
