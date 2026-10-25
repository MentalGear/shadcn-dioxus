use dioxus::prelude::*;

use dioxus_icons::lucide::ChevronDown;
use dioxus_primitives::{
    calendar::DateRange,
    date_picker::{
        self, DatePickerDaySegmentProps, DatePickerInputProps, DatePickerMonthSegmentProps,
        DatePickerSeparatorProps, DatePickerYearSegmentProps, DateRangePickerEndValueProps,
        DateRangePickerStartValueProps,
    },
    dioxus_attributes::attributes,
    merge_attributes,
    popover::{PopoverContentProps, PopoverTriggerProps},
    ContentAlign,
};
use std::time::Duration;
use time::{Date, Month, Weekday};

use super::super::calendar::*;
use super::super::popover::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};

// dev-docs/backlog.md row 32: see checkbox/component.rs's
// header comment for the full delivery-mechanism rationale (asset!() +
// document::Link, embedded in the two public entry points -- `DatePicker`
// and `DateRangePicker` -- the same way `select/component.rs` links from
// both of ITS two entry points, since a caller uses exactly one of the two
// and never both).
//
// The segment class is `dx-date-picker-segment`, not a bare `dx-date-segment`:
// without a hash suffix, a plain `dx-date-segment` isn't provably this
// component's own -- see `scripts/check-dx-class-prefix.sh`.
fn fixed_date(year: i32, month: Month, day: u8) -> Date {
    Date::from_calendar_date(year, month, day).expect("valid fixed date")
}

#[derive(Clone, Copy)]
struct StyledDatePickerContext {
    month_count: u8,
    caption_layout: CaptionLayout,
    calendar_format: CalendarFormat,
    apply_label: ReadSignal<String>,
    cancel_label: ReadSignal<String>,
}

fn default_apply_label() -> ReadSignal<String> {
    ReadSignal::new(Signal::new("Apply".to_string()))
}

fn default_cancel_label() -> ReadSignal<String> {
    ReadSignal::new(Signal::new("Cancel".to_string()))
}

fn styled_caption_layout() -> CaptionLayout {
    try_use_context::<StyledDatePickerContext>()
        .map(|ctx| ctx.caption_layout)
        .unwrap_or_default()
}

/// The calendar formatting callbacks the picker root carries to the calendar in its popover.
#[derive(Clone, Copy)]
struct CalendarFormat {
    weekday: Callback<Weekday, String>,
    month: Callback<Month, String>,
    month_short: Option<Callback<Month, String>>,
}

/// The picker root's [`CalendarFormat`], or the calendar's own defaults for an input used
/// outside a styled picker.
fn styled_calendar_format() -> CalendarFormat {
    try_use_context::<StyledDatePickerContext>()
        .map(|ctx| ctx.calendar_format)
        .unwrap_or_else(|| CalendarFormat {
            weekday: Callback::new(|weekday: Weekday| weekday_abbreviation(weekday).to_string()),
            month: Callback::new(|month: Month| month.to_string()),
            month_short: None,
        })
}

fn styled_month_count() -> u8 {
    try_use_context::<StyledDatePickerContext>()
        .map(|ctx| ctx.month_count)
        .unwrap_or(1)
        .max(1)
}

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

    /// Whether the date picker is enable user input
    #[props(default = ReadSignal::new(Signal::new(false)))]
    pub read_only: ReadSignal<bool>,

    /// Lower limit of the range of available dates
    #[props(default = fixed_date(1925, Month::January, 1))]
    pub min_date: Date,

    /// Upper limit of the range of available dates
    #[props(default = fixed_date(2050, Month::December, 31))]
    pub max_date: Date,

    /// Callback when display day placeholder
    #[props(default = Callback::new(|_| "D".to_string()))]
    pub on_format_day_placeholder: Callback<(), String>,

    /// Callback when display month placeholder
    #[props(default = Callback::new(|_| "M".to_string()))]
    pub on_format_month_placeholder: Callback<(), String>,

    /// Callback when display year placeholder
    #[props(default = Callback::new(|_| "Y".to_string()))]
    pub on_format_year_placeholder: Callback<(), String>,

    /// Locale-aware month name, for the month segment's `aria-valuetext` (backlog row 84
    /// finding 2, see `dioxus_primitives::date_picker::DateElementProps::on_format_month`) and
    /// for the caption of the calendar in the popover.
    #[props(default = Callback::new(|month: Month| month.to_string()))]
    pub on_format_month: Callback<Month, String>,

    /// Short month name for the calendar's month dropdown ("Oct"). `None` keeps the first three
    /// characters of `on_format_month`; pass a callback for a language where that cut is wrong
    /// (French "juin" and "juillet" both cut to "jui").
    #[props(default)]
    pub on_format_month_short: Option<Callback<Month, String>>,

    /// Weekday name for the calendar's column headers ("Su").
    #[props(default = Callback::new(|weekday: Weekday| weekday_abbreviation(weekday).to_string()))]
    pub on_format_weekday: Callback<Weekday, String>,

    /// How the calendar's month caption is drawn (shadcn's `captionLayout`): plain text by default,
    /// or month and/or year dropdowns for a far-away date such as a date of birth.
    #[props(default)]
    pub caption_layout: CaptionLayout,

    /// Specify how many months are visible at once
    #[props(default = 1)]
    pub month_count: u8,

    /// Unavailable dates
    #[props(default)]
    pub disabled_ranges: ReadSignal<Vec<DateRange>>,

    /// Whether picking a date closes the popover. Defaults to `true`; `false` keeps it open
    /// after a selection (Escape, an outside click and the trigger still close it).
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub close_on_select: ReadSignal<bool>,

    /// How long the popover stays open after a selection before it closes, so the selected day
    /// is seen first. Defaults to 300 ms; `Duration::ZERO` closes immediately.
    #[props(default = ReadSignal::new(Signal::new(date_picker::DEFAULT_CLOSE_DELAY)))]
    pub close_delay: ReadSignal<Duration>,

    /// Whether picking a date is staged behind an Apply / Cancel footer. Defaults to `true`: a
    /// selection is held in the panel (the input keeps the committed value) until Apply commits it
    /// and closes the popover; Cancel, Escape and an outside click restore the previous value and
    /// close. `false` removes the footer and commits on select (see `close_on_select`). While
    /// `true`, `close_on_select` and `close_delay` are ignored.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub confirm: ReadSignal<bool>,

    /// The Apply button's label (localizable).
    #[props(default = default_apply_label())]
    pub apply_label: ReadSignal<String>,

    /// The Cancel button's label (localizable).
    #[props(default = default_cancel_label())]
    pub cancel_label: ReadSignal<String>,

    /// Whether focus should loop around when reaching the end.
    #[props(default = ReadSignal::new(Signal::new(false)))]
    pub roving_loop: ReadSignal<bool>,

    /// Additional attributes to extend the date picker element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

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

    /// Whether the date picker is enable user input
    #[props(default = ReadSignal::new(Signal::new(false)))]
    pub read_only: ReadSignal<bool>,

    /// Lower limit of the range of available dates
    #[props(default = fixed_date(1925, Month::January, 1))]
    pub min_date: Date,

    /// Upper limit of the range of available dates
    #[props(default = fixed_date(2050, Month::December, 31))]
    pub max_date: Date,

    /// Callback when display day placeholder
    #[props(default = Callback::new(|_| "D".to_string()))]
    pub on_format_day_placeholder: Callback<(), String>,

    /// Callback when display month placeholder
    #[props(default = Callback::new(|_| "M".to_string()))]
    pub on_format_month_placeholder: Callback<(), String>,

    /// Callback when display year placeholder
    #[props(default = Callback::new(|_| "Y".to_string()))]
    pub on_format_year_placeholder: Callback<(), String>,

    /// See `DatePickerProps::on_format_month` (backlog row 84 finding 2).
    #[props(default = Callback::new(|month: Month| month.to_string()))]
    pub on_format_month: Callback<Month, String>,

    /// See `DatePickerProps::on_format_month_short`.
    #[props(default)]
    pub on_format_month_short: Option<Callback<Month, String>>,

    /// See `DatePickerProps::on_format_weekday`.
    #[props(default = Callback::new(|weekday: Weekday| weekday_abbreviation(weekday).to_string()))]
    pub on_format_weekday: Callback<Weekday, String>,

    /// How the calendar's month caption is drawn (shadcn's `captionLayout`): plain text by default,
    /// or month and/or year dropdowns for a far-away date such as a date of birth.
    #[props(default)]
    pub caption_layout: CaptionLayout,

    /// Specify how many months are visible at once
    #[props(default = 1)]
    pub month_count: u8,

    /// Unavailable dates
    #[props(default)]
    pub disabled_ranges: ReadSignal<Vec<DateRange>>,

    /// Whether completing a range (picking its end date) closes the popover. Defaults to `true`;
    /// `false` keeps it open (Escape, an outside click and the trigger still close it).
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub close_on_select: ReadSignal<bool>,

    /// How long the popover stays open after the range is complete before it closes, so the
    /// selected range is seen first. Defaults to 300 ms; `Duration::ZERO` closes immediately.
    #[props(default = ReadSignal::new(Signal::new(date_picker::DEFAULT_CLOSE_DELAY)))]
    pub close_delay: ReadSignal<Duration>,

    /// Whether picking a date is staged behind an Apply / Cancel footer. Defaults to `true`: a
    /// selection is held in the panel (the input keeps the committed value) until Apply commits it
    /// and closes the popover; Cancel, Escape and an outside click restore the previous value and
    /// close. `false` removes the footer and commits on select (see `close_on_select`). While
    /// `true`, `close_on_select` and `close_delay` are ignored.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub confirm: ReadSignal<bool>,

    /// The Apply button's label (localizable).
    #[props(default = default_apply_label())]
    pub apply_label: ReadSignal<String>,

    /// The Cancel button's label (localizable).
    #[props(default = default_cancel_label())]
    pub cancel_label: ReadSignal<String>,

    /// Whether focus should loop around when reaching the end.
    #[props(default = ReadSignal::new(Signal::new(false)))]
    pub roving_loop: ReadSignal<bool>,

    /// Additional attributes to extend the date picker element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

#[component]
pub fn DatePicker(props: DatePickerProps) -> Element {
    let base = attributes!(div {
        class: "dx-date-picker", "data-dx-root": ""
    });
    let merged = merge_attributes(vec![base, props.attributes]);
    let month_count = props.month_count.max(1);
    use_context_provider(|| StyledDatePickerContext {
        month_count,
        caption_layout: props.caption_layout,
        calendar_format: CalendarFormat {
            weekday: props.on_format_weekday,
            month: props.on_format_month,
            month_short: props.on_format_month_short,
        },
        apply_label: props.apply_label,
        cancel_label: props.cancel_label,
    });

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/date_picker/style.css") }
        // Eager, at the picker root: the calendar sits inside the popover, which mounts its content
        // on first open, so the `Link` that `CalendarRoot`/`RangeCalendarRoot` carries would only
        // be inserted then -- a late, non-render-blocking `<link>`, and the calendar paints unstyled
        // for a frame or two on the first open. (The docs site's SSG CSS bundle already carries it;
        // a `shadcn-dioxus add date_picker` project has no bundle.) `document::Link` dedupes on
        // `(href, rel)`, so the later one from the calendar itself is a no-op.
        document::Link { rel: "stylesheet", href: asset!("/src/components/calendar/style.css") }
        div {
            date_picker::DatePicker {
                on_value_change: props.on_value_change,
                selected_date: props.selected_date,
                disabled: props.disabled,
                read_only: props.read_only,
                min_date: props.min_date,
                max_date: props.max_date,
                disabled_ranges: props.disabled_ranges,
                roving_loop: props.roving_loop,
                close_on_select: props.close_on_select,
                close_delay: props.close_delay,
                confirm: props.confirm,
                attributes: merged,
                date_picker::DatePickerPopover {
                    // Matches `ColorPickerPopover`'s own `is_modal: false`
                    // (`../color_picker/component.rs`) -- a calendar
                    // dropdown, like a color swatch popover, is meant to sit
                    // anchored next to its trigger, not centered as a modal
                    // dialog. Requires `primitives/src/date_picker/popover.rs`'s
                    // `DatePickerPopover` to forward `is_modal` to
                    // `PopoverRoot`, and the engine-injected
                    // `@supports (anchor-name: --a)` block every other
                    // non-modal-arm consumer has (`primitives/src/top_layer.rs`),
                    // or this would only trade one broken position for
                    // another.
                    popover_root: PopoverRoot,
                    is_modal: false,
                    DatePickerInput {
                        on_format_day_placeholder: props.on_format_day_placeholder,
                        on_format_month_placeholder: props.on_format_month_placeholder,
                        on_format_year_placeholder: props.on_format_year_placeholder,
                        on_format_month: props.on_format_month,
                    }
                }
            }
        }
    }
}

#[component]
pub fn DateRangePicker(props: DateRangePickerProps) -> Element {
    let base = attributes!(div {
        class: "dx-date-picker", "data-dx-root": ""
    });
    let merged = merge_attributes(vec![base, props.attributes]);
    let month_count = props.month_count.max(1);
    use_context_provider(|| StyledDatePickerContext {
        month_count,
        caption_layout: props.caption_layout,
        calendar_format: CalendarFormat {
            weekday: props.on_format_weekday,
            month: props.on_format_month,
            month_short: props.on_format_month_short,
        },
        apply_label: props.apply_label,
        cancel_label: props.cancel_label,
    });

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/date_picker/style.css") }
        // Eager, at the picker root: the calendar sits inside the popover, which mounts its content
        // on first open, so the `Link` that `CalendarRoot`/`RangeCalendarRoot` carries would only
        // be inserted then -- a late, non-render-blocking `<link>`, and the calendar paints unstyled
        // for a frame or two on the first open. (The docs site's SSG CSS bundle already carries it;
        // a `shadcn-dioxus add date_picker` project has no bundle.) `document::Link` dedupes on
        // `(href, rel)`, so the later one from the calendar itself is a no-op.
        document::Link { rel: "stylesheet", href: asset!("/src/components/calendar/style.css") }
        div {
            date_picker::DateRangePicker {
                on_range_change: props.on_range_change,
                selected_range: props.selected_range,
                disabled: props.disabled,
                read_only: props.read_only,
                min_date: props.min_date,
                max_date: props.max_date,
                disabled_ranges: props.disabled_ranges,
                roving_loop: props.roving_loop,
                close_on_select: props.close_on_select,
                close_delay: props.close_delay,
                confirm: props.confirm,
                attributes: merged,
                date_picker::DatePickerPopover {
                    // See the comment on `DatePicker`'s own
                    // `date_picker::DatePickerPopover` above -- identical reasoning.
                    popover_root: PopoverRoot,
                    is_modal: false,
                    DateRangePickerInput {
                        on_format_day_placeholder: props.on_format_day_placeholder,
                        on_format_month_placeholder: props.on_format_month_placeholder,
                        on_format_year_placeholder: props.on_format_year_placeholder,
                        on_format_month: props.on_format_month,
                    }
                }
            }
        }
    }
}

#[component]
pub(crate) fn DatePickerInput(props: DatePickerInputProps) -> Element {
    let base = attributes!(div {
        class: "dx-date-picker-group"
    });
    let merged = merge_attributes(vec![base, props.attributes]);
    let extra_children = props.children;
    let month_count = styled_month_count();
    let caption_layout = styled_caption_layout();
    let format = styled_calendar_format();

    rsx! {
        date_picker::DatePickerInput {
            on_format_day_placeholder: props.on_format_day_placeholder,
            on_format_month_placeholder: props.on_format_month_placeholder,
            on_format_year_placeholder: props.on_format_year_placeholder,
            attributes: merged,
            date_picker::DatePickerInputValue {
                on_format_day_placeholder: props.on_format_day_placeholder,
                on_format_month_placeholder: props.on_format_month_placeholder,
                on_format_year_placeholder: props.on_format_year_placeholder,
                on_format_month: props.on_format_month,
                DatePickerYearSegment {}
                DatePickerSeparator {}
                DatePickerMonthSegment {}
                DatePickerSeparator {}
                DatePickerDaySegment {}
            }
            if let Some(extra_children) = extra_children {
                {extra_children}
            }
            DatePickerPopoverTrigger {}
            DatePickerPopoverContent { align: ContentAlign::Center,
                date_picker::DatePickerCalendar {
                    calendar: CalendarRoot,
                    on_format_weekday: format.weekday,
                    on_format_month: format.month,
                    on_format_month_short: format.month_short,
                    for offset in 0..month_count {
                        CalendarMonthView {
                            key: "{offset}",
                            offset,
                            month_count,
                            caption_layout,
                        }
                    }
                }
                DatePickerFooter {}
            }
        }
    }
}

#[component]
pub(crate) fn DateRangePickerInput(props: DatePickerInputProps) -> Element {
    let base = attributes!(div {
        class: "dx-date-picker-group"
    });
    let merged = merge_attributes(vec![base, props.attributes]);
    let extra_children = props.children;
    let month_count = styled_month_count();
    let caption_layout = styled_caption_layout();
    let format = styled_calendar_format();

    rsx! {
        date_picker::DateRangePickerInput {
            on_format_day_placeholder: props.on_format_day_placeholder,
            on_format_month_placeholder: props.on_format_month_placeholder,
            on_format_year_placeholder: props.on_format_year_placeholder,
            attributes: merged,
            date_picker::DateRangePickerInputValue {
                on_format_day_placeholder: props.on_format_day_placeholder,
                on_format_month_placeholder: props.on_format_month_placeholder,
                on_format_year_placeholder: props.on_format_year_placeholder,
                on_format_month: props.on_format_month,
                DateRangePickerStartValue {
                    DatePickerYearSegment {}
                    DatePickerSeparator {}
                    DatePickerMonthSegment {}
                    DatePickerSeparator {}
                    DatePickerDaySegment {}
                }
                DatePickerSeparator { symbol: '—' }
                DateRangePickerEndValue {
                    DatePickerYearSegment {}
                    DatePickerSeparator {}
                    DatePickerMonthSegment {}
                    DatePickerSeparator {}
                    DatePickerDaySegment {}
                }
            }
            if let Some(extra_children) = extra_children {
                {extra_children}
            }
            DatePickerPopoverTrigger {}
            DatePickerPopoverContent {
                align: ContentAlign::Center,
                date_picker::DateRangePickerCalendar {
                    calendar: RangeCalendarRoot,
                    on_format_weekday: format.weekday,
                    on_format_month: format.month,
                    on_format_month_short: format.month_short,
                    for offset in 0..month_count {
                        CalendarMonthView {
                            key: "{offset}",
                            offset,
                            month_count,
                            caption_layout,
                        }
                    }
                }
                DatePickerFooter {}
            }
        }
    }
}

/// The Cancel / Apply footer of a `confirm` picker (the default); renders nothing when `confirm`
/// is `false`. Both buttons are the real `Button`. Apply is disabled until a staged selection
/// differs from the committed value -- for a range that is until BOTH ends are picked.
#[component]
pub(crate) fn DatePickerFooter() -> Element {
    let confirm = date_picker::use_date_picker_confirm();
    let labels = try_use_context::<StyledDatePickerContext>();
    if !confirm.enabled() {
        return rsx! {};
    }
    let apply = labels
        .map(|ctx| (ctx.apply_label)())
        .unwrap_or_else(|| "Apply".to_string());
    let cancel = labels
        .map(|ctx| (ctx.cancel_label)())
        .unwrap_or_else(|| "Cancel".to_string());
    rsx! {
        div { class: "dx-date-picker-footer",
            Button {
                variant: ButtonVariant::Outline,
                size: ButtonSize::Sm,
                r#type: "button",
                onclick: move |_| confirm.cancel(),
                "{cancel}"
            }
            Button {
                size: ButtonSize::Sm,
                r#type: "button",
                disabled: !confirm.can_apply(),
                onclick: move |_| confirm.apply(),
                "{apply}"
            }
        }
    }
}

#[component]
pub(crate) fn DatePickerYearSegment(props: DatePickerYearSegmentProps) -> Element {
    let base = attributes!(span {
        class: "dx-date-picker-segment"
    });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        date_picker::DatePickerYearSegment { attributes: merged }
    }
}

#[component]
pub(crate) fn DatePickerMonthSegment(props: DatePickerMonthSegmentProps) -> Element {
    let base = attributes!(span {
        class: "dx-date-picker-segment"
    });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        date_picker::DatePickerMonthSegment { attributes: merged }
    }
}

#[component]
pub(crate) fn DatePickerDaySegment(props: DatePickerDaySegmentProps) -> Element {
    let base = attributes!(span {
        class: "dx-date-picker-segment"
    });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        date_picker::DatePickerDaySegment { attributes: merged }
    }
}

#[component]
pub(crate) fn DatePickerSeparator(props: DatePickerSeparatorProps) -> Element {
    let base = attributes!(span {
        class: "dx-date-picker-segment"
    });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        date_picker::DatePickerSeparator { symbol: props.symbol, attributes: merged }
    }
}

#[component]
pub(crate) fn DateRangePickerStartValue(props: DateRangePickerStartValueProps) -> Element {
    rsx! {
        date_picker::DateRangePickerStartValue {
            {props.children}
        }
    }
}

#[component]
pub(crate) fn DateRangePickerEndValue(props: DateRangePickerEndValueProps) -> Element {
    rsx! {
        date_picker::DateRangePickerEndValue {
            {props.children}
        }
    }
}

#[component]
pub(crate) fn DatePickerPopoverTrigger(props: PopoverTriggerProps) -> Element {
    let base = attributes!(button {
        class: "dx-date-picker-popover-trigger",
        aria_label: "Show Calendar",
    });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        PopoverTrigger {
            attributes: merged,
            ChevronDown {
                class: "dx-date-picker-trigger",
                size: "20px",
                stroke: "var(--dx-muted-foreground)",
            }
        }
    }
}

#[component]
pub(crate) fn DatePickerPopoverContent(props: PopoverContentProps) -> Element {
    rsx! {
        PopoverContent {
            class: "dx-date-picker-popover-content".to_string(),
            id: props.id,
            side: props.side,
            align: props.align,
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus_primitives::calendar::DateRange;

    #[component]
    fn DatePickerWithDefaultInput() -> Element {
        rsx! {
            DatePicker {
                selected_date: Some(fixed_date(2026, Month::May, 7)),
            }
        }
    }

    #[component]
    fn DateRangePickerWithDefaultInput() -> Element {
        rsx! {
            DateRangePicker {
                selected_range: Some(DateRange::new(
                    fixed_date(2026, Month::May, 7),
                    fixed_date(2026, Month::May, 11),
                )),
            }
        }
    }

    #[test]
    fn date_picker_renders_default_input_when_children_are_omitted() {
        let mut dom = VirtualDom::new(DatePickerWithDefaultInput);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);

        assert!(html.contains("2026"));
        assert!(html.contains("05"));
        assert!(html.contains("07"));
        assert!(html.contains("Show Calendar"));
    }

    #[test]
    fn date_range_picker_renders_default_input_when_children_are_omitted() {
        let mut dom = VirtualDom::new(DateRangePickerWithDefaultInput);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);

        assert!(html.contains("2026"));
        assert!(html.contains("05"));
        assert!(html.contains("07"));
        assert!(html.contains("11"));
        assert!(html.contains("Show Calendar"));
    }
}
