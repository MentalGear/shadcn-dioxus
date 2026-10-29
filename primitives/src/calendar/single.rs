use super::*;
use crate::state_attrs;

/// The props for the [`Calendar`] component.
#[derive(Props, Clone, PartialEq)]
pub struct CalendarProps {
    /// The selected date
    #[props(default)]
    pub selected_date: ReadSignal<Option<Date>>,

    /// Callback when selected date changes
    #[props(default)]
    pub on_date_change: Callback<Option<Date>>,

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

    /// The text direction for the day grid's `ArrowLeft`/`ArrowRight` step.
    /// Defaults to the nearest [`crate::direction::DirectionProvider`], or
    /// LTR if there is none.
    #[props(default)]
    pub dir: Option<Direction>,

    /// Additional attributes to extend the calendar element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the calendar element
    pub children: Element,
}

impl DefaultCalendarProps for CalendarProps {
    fn default_calendar(self) -> Element {
        Calendar(self)
    }
}

/// # Calendar
///
/// The [`Calendar`] component provides an accessible calendar interface with arrow key navigation, month switching, and date selection.
///
/// ## Example
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::calendar::{
///     Calendar, CalendarGrid, CalendarHeader, CalendarMonthTitle, CalendarNavigation, CalendarNextMonthButton, CalendarPreviousMonthButton
/// };
/// use time::{Date, Month, UtcDateTime};
/// #[component]
/// fn Demo() -> Element {
///     let mut selected_date = use_signal(|| None::<Date>);
///     let mut view_date = use_signal(|| UtcDateTime::now().date());
///     rsx! {
///         Calendar {
///             selected_date: selected_date(),
///             on_date_change: move |date| {
///                 tracing::info!("Selected date: {:?}", date);
///                 selected_date.set(date);
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
/// The [`Calendar`] component defines the following data attributes you can use to control styling:
/// - `data-disabled`: Indicates if the calendar is disabled. Possible values are `true` or `false`.
#[component]
pub fn Calendar(props: CalendarProps) -> Element {
    let available_ranges = use_memo(move || AvailableRanges::new(&props.disabled_ranges.read()));
    let view_registrations = use_signal(Vec::new);
    let direction = use_direction(props.dir);
    let direction_signal = use_direction_signal(direction);
    let (today, view_date) = use_today_and_view(props.today, props.view_date);

    // Create base context provider for child components
    let mut base_ctx = use_context_provider(|| BaseCalendarContext {
        focused_date: Signal::new(None),
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
    // Create Calendar context provider for child components
    use_context_provider(|| CalendarContext {
        selected_date: props.selected_date,
        set_selected_date: props.on_date_change,
    });

    // `role`/the `data-*` state are owned by the component (roving-focus and
    // disabled-state wiring below reads them back); `aria_label` is an
    // overridable default. Merge with the caller's `attributes` so a caller
    // override survives hydration instead of colliding with these literals
    // on the same element (backlog row 93).
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
                let Some(focused_date) = (base_ctx.focused_date)() else {
                    return;
                };
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
                    _ => {}
                }
            },
            ..merged,
            {props.children}
        }
    }
}
