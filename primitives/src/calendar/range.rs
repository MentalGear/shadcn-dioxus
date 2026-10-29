use super::*;
use crate::state_attrs;

/// The context provided by the [`RangeCalendar`] component to its children.
#[derive(Copy, Clone)]
pub struct RangeCalendarContext {
    // The date that the user clicked on to begin range selection
    pub(super) anchor_date: Signal<Option<Date>>,
    // Currently highlighted date range
    pub(super) highlighted_range: Signal<Option<DateRange>>,
    pub(super) set_selected_range: Callback<Option<DateRange>>,
}

impl RangeCalendarContext {
    /// Set the selected date
    pub fn set_selected_date(&mut self, date: Option<Date>) {
        match (self.anchor_date)() {
            Some(anchor) => {
                if let Some(date) = date {
                    self.anchor_date.set(None);

                    let range = DateRange::new(date, anchor);
                    self.set_selected_range.call(Some(range));
                    self.highlighted_range.set(Some(range));
                }
            }
            None => {
                self.anchor_date.set(date);

                let range = date.map(|d| DateRange::new(d, d));
                self.highlighted_range.set(range);
            }
        }
    }

    /// Set the selected date range by hovered date
    pub fn set_hovered_date(&mut self, date: Date) {
        if let Some(anchor) = (self.anchor_date)() {
            let range = DateRange::new(anchor, date);
            self.highlighted_range.set(Some(range));
        }
    }

    /// Set previous selected range
    pub fn reset_selection(&mut self, range: Option<DateRange>) {
        self.anchor_date.set(None);
        self.highlighted_range.set(range);
    }
}

/// The props for the [`RangeCalendar`] component.
#[derive(Props, Clone, PartialEq)]
pub struct RangeCalendarProps {
    /// The selected range
    #[props(default)]
    pub selected_range: ReadSignal<Option<DateRange>>,

    /// Callback when selected date range changes
    #[props(default)]
    pub on_range_change: Callback<Option<DateRange>>,

    /// Callback when display weekday
    #[props(default = Callback::new(|weekday: Weekday| weekday_abbreviation(weekday).to_string()))]
    pub on_format_weekday: Callback<Weekday, String>,

    /// Callback when display month
    #[props(default = Callback::new(|month: Month| month.to_string()))]
    pub on_format_month: Callback<Month, String>,

    /// Callback when displaying a month inside a month dropdown (shadcn prints "Oct" there while
    /// the caption label keeps "October"). `None` derives it from `on_format_month` by keeping
    /// the first three characters, so a localised calendar gets a localised short name too; pass
    /// a callback for a language where that cut is wrong.
    #[props(default)]
    pub on_format_month_short: Option<Callback<Month, String>>,

    /// The month being viewed. `None` (the default) shows the month of `today`.
    #[props(default)]
    pub view_date: ReadSignal<Option<Date>>,

    /// The current date (used for highlighting today). `None` (the default) is the visitor's
    /// local date, read after mount ([`use_today`]).
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

    /// Show a week-number column at the start of every week row (react-day-picker's
    /// `showWeekNumber`). With a Monday week start the numbers are ISO 8601 weeks (week 1 holds the
    /// year's first Thursday); with any other start day week 1 is the week that holds January 1st,
    /// which is `date-fns`' `getWeek` for the `en-US` locale react-day-picker defaults to.
    #[props(default)]
    pub show_week_number: bool,

    /// Lower limit of the range of available dates
    #[props(default = DEFAULT_MIN_DATE)]
    pub min_date: Date,

    /// Upper limit of the range of available dates
    #[props(default = DEFAULT_MAX_DATE)]
    pub max_date: Date,

    /// Unavailable dates
    #[props(default)]
    pub disabled_ranges: ReadSignal<Vec<DateRange>>,

    /// The text direction -- see [`CalendarProps::dir`]'s identical doc.
    #[props(default)]
    pub dir: Option<Direction>,

    /// Additional attributes to extend the calendar element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the calendar element
    pub children: Element,
}

impl DefaultCalendarProps for RangeCalendarProps {
    fn default_calendar(self) -> Element {
        RangeCalendar(self)
    }
}

/// # RangeCalendar
///
/// The [`RangeCalendar`] component provides an accessible calendar interface with arrow key navigation, month switching, and date selection.
///
/// ## Example
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::calendar::*;
/// use time::{Date, Month, UtcDateTime};
/// #[component]
/// fn Demo() -> Element {
///     let mut selected_range = use_signal(|| None::<DateRange>);
///     let mut view_date = use_signal(|| UtcDateTime::now().date());
///     rsx! {
///         RangeCalendar {
///             selected_range: selected_range(),
///             on_range_change: move |range| {
///                 tracing::info!("Selected range: {:?}", range);
///                 selected_range.set(range);
///             },
///             view_date: view_date(),
///             on_view_change: move |new_view: Date| {
///                 tracing::info!("View changed to: {}-{}", new_view.year(), new_view.month());
///                 view_date.set(new_view);
///             },
///             CalendarHeader {
///                 CalendarNavigation {
///                     CalendarPreviousMonthButton {
///                         "<"
///                     }
///                     CalendarMonthTitle {}
///                     CalendarNextMonthButton {
///                         ">"
///                     }
///                 }
///             }
///             CalendarGrid {}
///         }
///     }
/// }
/// ```
///
/// # Styling
///
/// The [`RangeCalendar`] component defines the following data attributes you can use to control styling:
/// - `data-disabled`: Indicates if the calendar is disabled. Possible values are `true` or `false`.
#[component]
pub fn RangeCalendar(props: RangeCalendarProps) -> Element {
    let focused_date = use_signal(|| {
        let range = (props.selected_range)();
        range.map(|r| r.end)
    });
    let anchor_date = use_signal(|| None::<Date>);
    let highlighted_range = use_signal(|| (props.selected_range)());
    let available_ranges = use_memo(move || AvailableRanges::new(&props.disabled_ranges.read()));
    let view_registrations = use_signal(Vec::new);
    let direction = use_direction(props.dir);
    let direction_signal = use_direction_signal(direction);
    let (today, view_date) = use_today_and_view(props.today, props.view_date);

    // Create base context provider for child components
    let mut base_ctx = use_context_provider(|| BaseCalendarContext {
        focused_date,
        view_date,
        set_view_date: props.on_view_change,
        available_ranges,
        format_weekday: props.on_format_weekday,
        format_month: props.on_format_month,
        format_month_short: props.on_format_month_short,
        disabled: props.disabled,
        today,
        first_day_of_week: props.first_day_of_week,
        show_week_number: props.show_week_number,
        enabled_date_range: DateRange::new(props.min_date, props.max_date),
        view_registrations,
        direction: direction_signal,
    });

    // Create RangeCalendar context provider for child components
    let mut ctx = use_context_provider(|| RangeCalendarContext {
        anchor_date,
        highlighted_range,
        set_selected_range: props.on_range_change,
    });

    // Same reasoning as `Calendar` above: owned `role`/`data-*` state merged
    // with the caller's attributes so an override survives hydration.
    let defaults = attributes!(div {
        aria_label: "Calendar"
    });
    let owned = attributes!(div {
        role: "application",
        ..state_attrs::disabled((props.disabled)()),
        "data-direction": direction.as_str(),
    });
    let merged = merge_attributes(vec![defaults, props.attributes.clone(), owned]);

    rsx! {
        div {
            dir: direction.as_str(),
            onkeydown: move |e| {
                let Some(mut focused_date) = (base_ctx.focused_date)() else {
                    return;
                };
                if let (Some(range), Some(date)) = (
                    (ctx.highlighted_range)(),
                    (ctx.anchor_date)(),
                ) {
                    if date != range.start {
                        focused_date = range.start
                    } else {
                        focused_date = range.end
                    }
                }
                let mut set_focused_date = |new_date: Option<Date>| {
                    if let Some(date) = new_date {
                        let min_date = (base_ctx.view_date)().replace_day(1).expect("every month has a day 1");
                        if date < min_date {
                            let view_date = previous_month(min_date).unwrap_or(min_date);
                            (base_ctx.set_view_date)(view_date);
                        } else {
                            let max_date = nth_month_next(min_date, base_ctx.visible_month_count())
                                .unwrap_or(min_date);
                            if date >= max_date {
                                let view_date = next_month(min_date).unwrap_or(min_date);
                                (base_ctx.set_view_date)(view_date);
                            }
                        }
                    }
                    match new_date {
                        Some(date) => {
                            if base_ctx.enabled_date_range.contains(date) {
                                base_ctx.focused_date.set(new_date);
                                let date = match base_ctx.available_range() {
                                    Some(range) => range.clamp(date),
                                    None => date,
                                };
                                ctx.set_hovered_date(date);
                            }
                        }
                        None => base_ctx.focused_date.set(None),
                    }
                };
                match e.key() {
                    Key::ArrowLeft | Key::ArrowRight => {
                        e.prevent_default();
                        if let Some(step) = horizontal_day_step(&e.key(), direction) {
                            let target = if step > 0 {
                                focused_date.next_day()
                            } else {
                                focused_date.previous_day()
                            };
                            set_focused_date(target);
                        }
                    }
                    Key::ArrowUp => {
                        e.prevent_default();
                        if e.modifiers().shift() {
                            if let Some(date) = previous_month(focused_date) {
                                set_focused_date(Some(date));
                            }
                        } else {
                            set_focused_date(Some(focused_date.saturating_sub(7.days())));
                        }
                    }
                    Key::ArrowDown => {
                        e.prevent_default();
                        if e.modifiers().shift() {
                            if let Some(date) = next_month(focused_date) {
                                set_focused_date(Some(date));
                            }
                        } else {
                            set_focused_date(Some(focused_date.saturating_add(7.days())));
                        }
                    }
                    Key::Escape => {
                        ctx.reset_selection((props.selected_range)());
                    }
                    _ => {}
                }
            },
            ..merged,
            {props.children}
        }
    }
}
