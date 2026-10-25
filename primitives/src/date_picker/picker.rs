use super::*;

/// The context provided by the [`DatePicker`] component to its children.
#[derive(Copy, Clone)]
pub(super) struct DatePickerContext {
    pub(super) on_value_change: Callback<Option<Date>>,
    pub(super) selected_date: ReadSignal<Option<Date>>,
    pub(super) staging: Staging<Date>,
}

impl DatePickerContext {
    pub(super) fn set_date(&mut self, date: Option<Date>) {
        let value = { self.selected_date.peek().cloned() };
        if value != date {
            self.on_value_change.call(date);
        }
    }
}

/// The props for the [`DatePicker`] component.
#[derive(Props, Clone, PartialEq)]
pub struct DatePickerProps {
    /// Callback when value changes
    #[props(default)]
    pub on_value_change: Callback<Option<Date>>,

    /// The selected date
    #[props(default)]
    pub selected_date: ReadSignal<Option<Date>>,

    /// Whether the date picker is disabled
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Sets `contenteditable="false"` on the date segments, nothing more: a read-only picker has no text caret, but the digit and arrow keys still change the value and the calendar popover still opens and picks. Use `disabled` to block interaction.
    #[props(default = ReadSignal::new(Signal::new(false)))]
    pub read_only: ReadSignal<bool>,

    /// Lower limit of the range of available dates
    #[props(default = crate::calendar::DEFAULT_MIN_DATE)]
    pub min_date: Date,

    /// Upper limit of the range of available dates
    #[props(default = crate::calendar::DEFAULT_MAX_DATE)]
    pub max_date: Date,

    /// Unavailable dates
    #[props(default)]
    pub disabled_ranges: ReadSignal<Vec<DateRange>>,

    /// Whether picking a date closes the popover. Defaults to `true`. When `false` the popover
    /// stays open after a selection, so several dates can be tried without reopening it; Escape,
    /// an outside click and the trigger still close it.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub close_on_select: ReadSignal<bool>,

    /// How long the popover stays open after a selection before it closes (and plays its exit
    /// animation), so the selected state is visible for a moment first. Defaults to
    /// [`DEFAULT_CLOSE_DELAY`] (300 ms); `Duration::ZERO` closes immediately. A key press, a month
    /// change or a close/reopen during the pause cancels the pending close, and picking another
    /// date restarts it. Ignored when `close_on_select` is `false`.
    #[props(default = ReadSignal::new(Signal::new(DEFAULT_CLOSE_DELAY)))]
    pub close_delay: ReadSignal<Duration>,

    /// Whether selections are staged behind an Apply / Cancel footer instead of committing at
    /// once. Defaults to `false` here: the primitive only stages if you also render the controls
    /// ([`use_date_picker_confirm`]), and staging without them would never commit anything. The
    /// styled `DatePicker` defaults it to `true` and renders the footer. While `true`, a calendar
    /// selection is held in the panel (the input keeps the committed value) and
    /// `close_on_select` / `close_delay` are ignored; Apply commits and closes, Cancel, Escape
    /// and an outside click restore the previous value and close.
    #[props(default = ReadSignal::new(Signal::new(false)))]
    pub confirm: ReadSignal<bool>,

    /// Whether focus should loop around when reaching the end.
    #[props(default = ReadSignal::new(Signal::new(false)))]
    pub roving_loop: ReadSignal<bool>,

    /// Additional attributes to extend the date picker element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the date picker element
    pub children: Element,
}

/// # DatePicker
///
/// The [`DatePicker`] component provides an accessible date input interface.
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
///
/// # Styling
///
/// The [`DatePicker`] component defines the following data attributes you can use to control styling:
/// - `data-disabled`: Indicates if the DatePicker is disabled. Possible values are `true` or `false`.
#[component]
pub fn DatePicker(props: DatePickerProps) -> Element {
    let open = use_signal(|| false);
    let focus = use_collection_provider(props.roving_loop);
    let available_ranges = use_memo(move || AvailableRanges::new(&props.disabled_ranges.read()));
    let dwell = use_close_dwell(open);
    let (staging, confirm) = use_staging(
        props.confirm,
        open,
        dwell,
        props.selected_date,
        props.on_value_change,
    );

    // Create context provider for child components
    use_context_provider(|| BaseDatePickerContext {
        open,
        read_only: props.read_only,
        disabled: props.disabled,
        focus,
        enabled_date_range: DateRange::new(props.min_date, props.max_date),
        available_ranges,
        close_on_select: props.close_on_select,
        close_delay: props.close_delay,
        dwell,
        confirm,
    });

    use_context_provider(|| DatePickerContext {
        on_value_change: props.on_value_change,
        selected_date: props.selected_date,
        staging,
    });

    let defaults = attributes!(div { aria_label: "Date" });
    let owned = attributes!(div {
        role: "group",
        "data-disabled": (props.disabled)(),
    });
    let merged = merge_attributes(vec![defaults, props.attributes.clone(), owned]);

    // A key pressed anywhere in the picker is the user taking over, so a close that is still
    // waiting out its dwell is cancelled. The selection itself schedules the close only AFTER its
    // own keydown has bubbled through here (Enter's `click` is dispatched once the keydown has
    // finished, Space's on keyup). Not the pointer: a second press on the day that was just
    // selected has to reach `DatePickerCalendar` with the close still pending, see there.
    rsx! {
        div {
            onkeydown: move |_| dwell.cancel(),
            ..merged,
            {props.children}
        }
    }
}

/// The context provided by the [`DateRangePicker`] component to its children.
#[derive(Copy, Clone)]
pub struct DateRangePickerContext {
    // Currently selected date range
    pub(super) date_range: ReadSignal<Option<DateRange>>,
    pub(super) set_selected_range: Callback<Option<DateRange>>,
    pub(super) staging: Staging<DateRange>,
}

impl DateRangePickerContext {
    /// Set the selected range -- but only if it differs from the controlled value, so a caller
    /// that already holds it (an echo of the parent's own value) never reports it back.
    ///
    /// The comparison reads the controlled value through `.peek()`, never tracked syntax, and that
    /// is load-bearing, not style: this is called from `DateRangePickerInputValue`'s sync-up
    /// effect, and a tracked read would subscribe that effect to the parent's value. Every change
    /// the PARENT makes to the range (including the one a calendar click has just reported
    /// upward) would then wake the sync-up effect too -- and, when it runs before the sync-down
    /// effect has copied the new range into the segment signals, it reads the PREVIOUS range
    /// there, finds it differs from the parent's value, and reports the stale range back up. The
    /// parent applies it (one render later), the sync-down effect copies it into the segments,
    /// the sync-up effect reports that, and the two ranges alternate forever -- a wasm loop that
    /// never yields to the browser (it freezes the tab; the order the two effects wake in is
    /// hash-ordered, so it happens on some builds and some opens and not on others).
    /// `DatePickerContext::set_date` reads its value this way too, and the
    /// `check-self-subscribing-effects.sh` gate rejects any method an effect calls that reads
    /// its own fields with tracked syntax.
    pub fn set_range(&mut self, range: Option<DateRange>) {
        if *self.date_range.peek() != range {
            self.set_selected_range.call(range);
        }
    }
}

/// The props for the [`DatePicker`] component.
#[derive(Props, Clone, PartialEq)]
pub struct DateRangePickerProps {
    /// Callback when value changes
    #[props(default)]
    pub on_range_change: Callback<Option<DateRange>>,

    /// The selected date
    #[props(default)]
    pub selected_range: ReadSignal<Option<DateRange>>,

    /// Whether the date picker is disabled
    #[props(default)]
    pub disabled: ReadSignal<bool>,

    /// Sets `contenteditable="false"` on the date segments, nothing more: a read-only picker has no text caret, but the digit and arrow keys still change the value and the calendar popover still opens and picks. Use `disabled` to block interaction.
    #[props(default = ReadSignal::new(Signal::new(false)))]
    pub read_only: ReadSignal<bool>,

    /// Lower limit of the range of available dates
    #[props(default = crate::calendar::DEFAULT_MIN_DATE)]
    pub min_date: Date,

    /// Upper limit of the range of available dates
    #[props(default = crate::calendar::DEFAULT_MAX_DATE)]
    pub max_date: Date,

    /// Unavailable dates
    #[props(default)]
    pub disabled_ranges: ReadSignal<Vec<DateRange>>,

    /// Whether completing a range (picking its end date) closes the popover. Defaults to `true`.
    /// When `false` the popover stays open after a range is picked, so several can be tried
    /// without reopening it; Escape, an outside click and the trigger still close it. Picking the
    /// range's start never closes it either way.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub close_on_select: ReadSignal<bool>,

    /// How long the popover stays open after the range's end date is picked before it closes (and
    /// plays its exit animation), so the selected range is visible for a moment first. Defaults to
    /// [`DEFAULT_CLOSE_DELAY`] (300 ms); `Duration::ZERO` closes immediately. Any press or key
    /// inside the picker (so the first click of a new range), a month change or a close/reopen
    /// during the pause cancels the pending close. Ignored when `close_on_select` is `false`.
    #[props(default = ReadSignal::new(Signal::new(DEFAULT_CLOSE_DELAY)))]
    pub close_delay: ReadSignal<Duration>,

    /// Whether selections are staged behind an Apply / Cancel footer instead of committing at
    /// once. Defaults to `false` here: the primitive only stages if you also render the controls
    /// ([`use_date_picker_confirm`]), and staging without them would never commit anything. The
    /// styled `DatePicker` defaults it to `true` and renders the footer. While `true`, a calendar
    /// selection is held in the panel (the input keeps the committed value) and
    /// `close_on_select` / `close_delay` are ignored; Apply commits and closes, Cancel, Escape
    /// and an outside click restore the previous value and close.
    #[props(default = ReadSignal::new(Signal::new(false)))]
    pub confirm: ReadSignal<bool>,

    /// Whether focus should loop around when reaching the end.
    #[props(default = ReadSignal::new(Signal::new(false)))]
    pub roving_loop: ReadSignal<bool>,

    /// Additional attributes to extend the date picker element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the date picker element
    pub children: Element,
}

/// # DateRangePicker
///
/// The [`DateRangePicker`] component provides an accessible date range input interface.
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
///                    DatePickerInput {
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
///
/// # Styling
///
/// The [`DateRangePicker`] component defines the following data attributes you can use to control styling:
/// - `data-disabled`: Indicates if the DateRangePicker is disabled. Possible values are `true` or `false`.
#[component]
pub fn DateRangePicker(props: DateRangePickerProps) -> Element {
    let open = use_signal(|| false);
    let focus = use_collection_provider(props.roving_loop);

    let available_ranges = use_memo(move || AvailableRanges::new(&props.disabled_ranges.read()));
    let dwell = use_close_dwell(open);
    let (staging, confirm) = use_staging(
        props.confirm,
        open,
        dwell,
        props.selected_range,
        props.on_range_change,
    );

    // Create context provider for child components
    use_context_provider(|| BaseDatePickerContext {
        open,
        read_only: props.read_only,
        disabled: props.disabled,
        focus,
        enabled_date_range: DateRange::new(props.min_date, props.max_date),
        available_ranges,
        close_on_select: props.close_on_select,
        close_delay: props.close_delay,
        dwell,
        confirm,
    });

    use_context_provider(|| DateRangePickerContext {
        date_range: props.selected_range,
        set_selected_range: props.on_range_change,
        staging,
    });

    let defaults = attributes!(div {
        aria_label: "Date Range"
    });
    let owned = attributes!(div {
        role: "group",
        "data-disabled": (props.disabled)(),
    });
    let merged = merge_attributes(vec![defaults, props.attributes.clone(), owned]);

    // A press or a key anywhere in the picker is the user taking over, so a close that is still
    // waiting out its dwell is cancelled -- in particular, the first click of a NEW range during
    // the dwell of the previous one (that click only anchors the range and reports nothing, so
    // this is the only place that can see it). The selection itself schedules the close only AFTER
    // its own pointerdown/keydown has bubbled through here (Enter's `click` is dispatched once the
    // keydown has finished, Space's on keyup).
    rsx! {
        div {
            onpointerdown: move |_| dwell.cancel(),
            onkeydown: move |_| dwell.cancel(),
            ..merged,
            {props.children}
        }
    }
}
