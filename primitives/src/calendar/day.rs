use super::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub(super) enum RelativeMonth {
    Last,
    Current,
    Next,
}

impl RelativeMonth {
    pub(super) fn current_month(&self) -> bool {
        *self == RelativeMonth::Current
    }
}

impl Display for RelativeMonth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RelativeMonth::Last => write!(f, "last"),
            RelativeMonth::Current => write!(f, "current"),
            RelativeMonth::Next => write!(f, "next"),
        }
    }
}

/// Get a human-readable ARIA label for input date
pub(super) fn aria_label(date: &Date) -> String {
    format!(
        "{}, {} {}, {}",
        date.weekday(),
        date.month(),
        date.day(),
        date.year()
    )
}

/// The props for the [`CalendarDay`] component.
#[derive(Props, Clone, PartialEq)]
pub struct CalendarDayProps {
    /// The date for this day cell.
    pub date: Date,
    /// Additional attributes to extend the calendar day element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the calendar day element
    #[props(default)]
    pub children: Option<Element>,
}

/// # CalendarDay
///
/// The [`CalendarDay`] component provides an accessible calendar interface for a date
///
/// This must be used inside a [`CalendarGrid`] component.
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
/// The [`CalendarDay`] component defines the following data attributes you can use to control styling:
/// - `data-disabled`: Indicates if the calendar is disabled. Possible values are `true` or `false`.
/// - `data-unavailable`: Indicates if the date is unavailable. Possible values are `true` or `false`.
/// - `data-today`: Indicates if the cell is today. Possible values are `true` or `false`.
/// - `data-month`: The relative month of the date. Possible values are `last`,
/// - `data-selected`: Indicates if the cell is selected. Possible values are `true` or `false`.
/// - `data-selection-start`: Indicates if cell is the first date in a range selection. Possible values are `true` or `false`.
/// - `data-selection-between`: Indicates if a date interval contains a cell. Possible values are `true` or `false`.
/// - `data-selection-end`: Indicates if cell is the last date in a range selection. Possible values are `true` or `false`.
///
/// Building an entirely custom day cell instead of using [`CalendarDay`]?
/// [`use_calendar_day_state`] and [`calendar_day_attributes`] expose the
/// same state and attributes this component computes for itself.
#[component]
pub fn CalendarDay(props: CalendarDayProps) -> Element {
    let single_context = try_use_context::<CalendarContext>().is_some();
    let CalendarDayProps {
        date,
        attributes,
        children,
    } = props;

    if single_context {
        rsx! {
            SingleCalendarDay { date, attributes: attributes.clone(), children: children.clone() }
        }
    } else {
        rsx! {
            RangeCalendarDay { date, attributes, children }
        }
    }
}

/// The computed interactive/visual state of one calendar day cell -- exactly
/// what the built-in [`CalendarDay`] cell computes for itself, exposed so a
/// custom day-cell UI built outside this crate can render the same states
/// instead of duplicating this logic. Returned by [`use_calendar_day_state`]
/// and turned into markup attributes by [`calendar_day_attributes`] --
/// upstream `DioxusLabs/components#199` ("the APIs to build each component
/// should be public").
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CalendarDayState {
    /// The date this state describes.
    pub date: Date,
    /// Whether this date is selected: the calendar's exact selected date in
    /// a [`Calendar`], or contained in the highlighted range (inclusive of
    /// its start/end) in a [`RangeCalendar`]. Always `false` in a
    /// [`Calendar`] for any date other than the selected one.
    pub selected: bool,
    /// Whether this date is the first date of a range selection. Always
    /// `false` inside a [`Calendar`] (there is no range there).
    pub range_start: bool,
    /// Whether this date is the last date of a range selection. Always
    /// `false` inside a [`Calendar`].
    pub range_end: bool,
    /// Whether this date falls strictly between the start and end of a
    /// range selection, excluding the start/end dates themselves (those are
    /// `range_start`/`range_end` instead). Always `false` inside a
    /// [`Calendar`].
    pub in_range: bool,
    /// Whether this date is the grid's current roving-tabindex/keyboard
    /// focus target.
    pub focused: bool,
    /// Whether this date is today (the calendar's configured `today` prop,
    /// which defaults to the real current date).
    pub today: bool,
    /// Whether this date is disabled: either the whole calendar is
    /// disabled, or this date is `unavailable`.
    pub disabled: bool,
    /// Whether this date falls outside the calendar's configured available
    /// date range (`min_date`/`max_date`/`disabled_ranges`).
    pub unavailable: bool,
    /// Whether this date is outside the month currently shown by the
    /// calendar grid -- a leading or trailing day from an adjacent month.
    pub outside_month: bool,
    pub(super) relative_month: RelativeMonth,
}

/// Compute one date's full [`CalendarDayState`] the same way the built-in
/// [`CalendarDay`] cell does, for building a custom day-cell UI outside this
/// crate that still matches the built-in cell's behavior exactly -- see
/// upstream `DioxusLabs/components#199` ("the APIs to build each component
/// should be public"). Pair it with [`calendar_day_attributes`] to also
/// reproduce the built-in cell's `data-*`/`aria-*` attributes.
///
/// Must be called from a descendant of a [`Calendar`] or [`RangeCalendar`]
/// that has also rendered a [`CalendarView`] (directly, or via
/// [`CalendarGrid`], which requires one the same way [`use_calendar_grid`]
/// does). Returns `None`, rather than panicking, when called anywhere else,
/// so a misplaced call is a value to handle, not a crash.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::calendar::{calendar_day_attributes, use_calendar_day_state};
/// use time::Date;
///
/// #[component]
/// fn CustomDay(date: Date) -> Element {
///     let Some(state) = use_calendar_day_state(date) else {
///         return rsx! {};
///     };
///     rsx! {
///         button { ..calendar_day_attributes(&state), "{date.day()}" }
///     }
/// }
/// ```
pub fn use_calendar_day_state(date: Date) -> Option<CalendarDayState> {
    let base_ctx = try_use_context::<BaseCalendarContext>()?;
    let view_ctx = try_use_context::<CalendarViewContext>()?;
    let view_date = view_ctx.offset_view_date();
    let focused = base_ctx
        .focused_date()
        .is_some_and(|d| d == date && d.month() == view_date.month());

    if let Some(ctx) = try_use_context::<CalendarContext>() {
        let selected = (ctx.selected_date)().is_some_and(|d| d == date);
        Some(day_state(
            date,
            &base_ctx,
            view_date,
            focused,
            DaySelection {
                selected,
                range_start: false,
                range_end: false,
                in_range: false,
            },
        ))
    } else if let Some(ctx) = try_use_context::<RangeCalendarContext>() {
        let range = (ctx.highlighted_range)();
        let selected = range.is_some_and(|r| r.contains(date));
        Some(day_state(
            date,
            &base_ctx,
            view_date,
            focused,
            DaySelection {
                selected,
                range_start: is_start(date, range),
                range_end: is_end(date, range),
                in_range: is_between(date, range),
            },
        ))
    } else {
        None
    }
}

/// Build the `data-*`/`aria-*` attributes the built-in [`CalendarDay`] cell
/// itself renders, from a [`CalendarDayState`] (e.g. from
/// [`use_calendar_day_state`]). Spread the result onto a custom day cell's
/// own element (`..calendar_day_attributes(&state)`) to match the built-in
/// cell's styling hooks and accessible name exactly.
///
/// Does not include `tabindex`: unlike every attribute here, its value
/// depends on every date in the grid (only the single "roving" date gets
/// `tabindex="0"`), not just this one, so it can't be derived from a single
/// date's own [`CalendarDayState`] -- nor any event handler.
pub fn calendar_day_attributes(state: &CalendarDayState) -> Vec<Attribute> {
    fn attr(name: &'static str, value: dioxus_core::AttributeValue) -> Attribute {
        Attribute {
            name,
            namespace: None,
            volatile: false,
            value,
        }
    }

    let mut attrs = vec![attr("aria-label", Text(aria_label(&state.date)))];
    if state.today {
        attrs.push(attr("data-today", Bool(true)));
    }
    attrs.push(attr("data-selected", Bool(state.selected)));
    if state.unavailable {
        attrs.push(attr("data-unavailable", Bool(true)));
    }
    attrs.push(attr("data-disabled", Bool(state.disabled)));
    // backlog row 84: an unavailable day (or a day inside a
    // disabled calendar -- `state.disabled` is already `(calendar disabled)
    // || unavailable`, see `day_state`) must tell assistive tech it cannot
    // be chosen: the cell has no native `disabled` attribute (a
    // genuinely disabled `<button>` would drop out of the tab order, which
    // `SingleCalendarDay`/`RangeCalendarDay` deliberately avoid so the grid
    // keeps a single roving tabstop -- see their own `handle_day_select`
    // early-return instead), so nothing else stands in for it.
    // `aria-disabled` (unlike a native `disabled` attribute) does not
    // remove the element from the tab order, so this is the correct
    // "focusable but not operable" signal -- matches this crate's own
    // established convention for exactly that shape of control (e.g.
    // `combobox`/`select`/`command`'s own options: `aria_disabled:
    // (option.disabled)()` alongside a still-focusable role). Mirrors
    // `data-disabled` immediately above: same source value, same
    // unconditional presence (a `data-disabled=false` day contributes
    // `aria-disabled=false`, which is the correct, explicit "not disabled"
    // ARIA state for a role that has no native disabled semantics of its
    // own, not merely "no opinion").
    attrs.push(attr("aria-disabled", Bool(state.disabled)));
    if state.range_start {
        attrs.push(attr("data-selection-start", Bool(true)));
    }
    if state.in_range {
        attrs.push(attr("data-selection-between", Bool(true)));
    }
    if state.range_end {
        attrs.push(attr("data-selection-end", Bool(true)));
    }
    attrs.push(attr("data-month", Text(state.relative_month.to_string())));
    attrs
}

/// The selection-related fields of [`CalendarDayState`], grouped into their
/// own type so [`day_state`] takes one struct instead of four positional
/// bools (clippy's `too_many_arguments`) -- each caller's own selection
/// model differs (`Option<Date>` equality for `Calendar`, `DateRange`
/// containment for `RangeCalendar`), so these are computed by the caller,
/// not by `day_state` itself.
#[derive(Default)]
pub(super) struct DaySelection {
    pub(super) selected: bool,
    pub(super) range_start: bool,
    pub(super) range_end: bool,
    pub(super) in_range: bool,
}

/// Shared state computation behind [`use_calendar_day_state`] and the
/// built-in [`SingleCalendarDay`]/[`RangeCalendarDay`] cells -- the one
/// place `today`/`disabled`/`unavailable`/`outside_month` get computed, so
/// the built-in cell and a caller's own custom cell can't drift apart.
/// `focused`/`selection` are taken as already-computed rather than
/// recomputed here because the built-in cells need `focused` as a *live*,
/// signal-reading closure (not this function's one-shot snapshot) to drive
/// `use_day_mounted_ref`'s reactive auto-focus effect -- see those
/// components' own call sites for exactly what each one passes.
pub(super) fn day_state(
    date: Date,
    base_ctx: &BaseCalendarContext,
    view_date: Date,
    focused: bool,
    selection: DaySelection,
) -> CalendarDayState {
    let relative_month = relative_calendar_month(date, base_ctx, view_date.month());
    let unavailable = base_ctx.is_unavailable(date);
    let disabled = (base_ctx.disabled)() || unavailable;

    CalendarDayState {
        date,
        selected: selection.selected,
        range_start: selection.range_start,
        range_end: selection.range_end,
        in_range: selection.in_range,
        focused,
        today: date == (base_ctx.today)(),
        disabled,
        unavailable,
        outside_month: !relative_month.current_month(),
        relative_month,
    }
}

pub(super) fn relative_calendar_month(
    date: Date,
    base_ctx: &BaseCalendarContext,
    current_month: Month,
) -> RelativeMonth {
    if date < base_ctx.enabled_date_range.start {
        RelativeMonth::Last
    } else if date > base_ctx.enabled_date_range.end {
        RelativeMonth::Next
    } else {
        match date.month().cmp(&current_month) {
            std::cmp::Ordering::Less => RelativeMonth::Last,
            std::cmp::Ordering::Equal => RelativeMonth::Current,
            std::cmp::Ordering::Greater => RelativeMonth::Next,
        }
    }
}

pub(super) fn is_between(date: Date, range: Option<DateRange>) -> bool {
    range.is_some_and(|r| r.contained_in_interval(date))
}

pub(super) fn is_start(date: Date, range: Option<DateRange>) -> bool {
    range.is_some_and(|r| r.start == date && date != r.end)
}

pub(super) fn is_end(date: Date, range: Option<DateRange>) -> bool {
    range.is_some_and(|r| r.end == date && date != r.start)
}

fn use_day_mounted_ref(
    mut is_focused: impl FnMut() -> bool + 'static,
) -> impl FnMut(MountedEvent) + 'static {
    let mut day_ref: Signal<Option<Rc<MountedData>>> = use_signal(|| None);
    use_effect(move || {
        if let Some(day) = day_ref() {
            if is_focused() {
                spawn(async move {
                    _ = day.set_focus(true).await;
                });
            }
        }
    });
    move |e| day_ref.set(Some(e.data()))
}

#[component]
fn SingleCalendarDay(props: CalendarDayProps) -> Element {
    let CalendarDayProps {
        date,
        attributes,
        children,
    } = props;
    let mut base_ctx: BaseCalendarContext = use_context();
    let view_ctx: CalendarViewContext = use_context();
    let day = date.day();
    let content = children.unwrap_or_else(|| rsx! { {day.to_string()} });
    let view_date = view_ctx.offset_view_date();
    let is_focused = move || {
        base_ctx
            .focused_date()
            .is_some_and(|d| d == date && d.month() == view_date.month())
    };
    let is_unavailable = base_ctx.is_unavailable(date);
    let onmounted = use_day_mounted_ref(is_focused);

    let ctx: CalendarContext = use_context();
    let is_selected = move || (ctx.selected_date)().is_some_and(|d| d == date);

    // Single source of truth for this cell's state (styling attributes
    // below) and the built-in `CalendarDayState`/`use_calendar_day_state`
    // public API -- see `day_state`'s own doc for why `focused`/`selected`
    // are passed in already-computed rather than recomputed inside it.
    let state = day_state(
        date,
        &base_ctx,
        view_date,
        is_focused(),
        DaySelection {
            selected: is_selected(),
            range_start: false,
            range_end: false,
            in_range: false,
        },
    );
    let in_current_month = !state.outside_month;

    // Handle day selection
    let mut handle_day_select = move |day: u8| {
        if (base_ctx.disabled)() || is_unavailable {
            return;
        }
        let view_date = view_ctx.offset_view_date();
        let date = view_date
            .replace_day(day)
            .expect("the grid renders only the days of the view month");
        ctx.set_selected_date.call((!is_selected()).then_some(date));
        base_ctx.focused_date.set(Some(date));
    };

    let focusable_date = (base_ctx.focused_date)()
        .filter(|d| d.month() == view_date.month())
        .or_else(|| {
            ctx.selected_date
                .cloned()
                .filter(|d| d.month() == view_date.month())
        })
        .unwrap_or(view_date);

    // `type` is an overridable default; the roving-focus `tabindex` and
    // `calendar_day_attributes`'s aria/data state are owned, so they merge
    // in last (owned wins; backlog row 93), with no duplicate attribute
    // for a caller override to collide with after hydration.
    let defaults = attributes!(button { r#type: "button" });
    let owned = merge_attributes(vec![
        calendar_day_attributes(&state),
        attributes!(button {
            tabindex: if date == focusable_date { "0" } else { "-1" },
        }),
    ]);
    let merged = merge_attributes(vec![defaults, attributes, owned]);

    rsx! {
        button {
            onclick: move |e| {
                e.prevent_default();
                if in_current_month {
                    handle_day_select(day);
                }
            },
            onfocus: move |_| {
                if in_current_month {
                    base_ctx.focused_date.set(Some(date));
                }
            },
            onmounted,
            ..merged,
            {content}
        }
    }
}

#[component]
fn RangeCalendarDay(props: CalendarDayProps) -> Element {
    let CalendarDayProps {
        date,
        attributes,
        children,
    } = props;
    let mut base_ctx: BaseCalendarContext = use_context();
    let day = date.day();
    let content = children.unwrap_or_else(|| rsx! { {day.to_string()} });
    let view_ctx: CalendarViewContext = use_context();
    let view_date = view_ctx.offset_view_date();
    let is_focused = move || {
        base_ctx
            .focused_date()
            .is_some_and(|d| d == date && d.month() == view_date.month())
    };
    let is_unavailable = base_ctx.is_unavailable(date);
    let onmounted = use_day_mounted_ref(is_focused);

    let mut ctx: RangeCalendarContext = use_context();
    let range = ctx.highlighted_range.cloned();
    let selected = range.is_some_and(|r| r.contains(date));

    // Single source of truth for this cell's state (styling attributes
    // below) and the built-in `CalendarDayState`/`use_calendar_day_state`
    // public API -- see `day_state`'s own doc for why `focused`/`selected`/
    // `range_*` are passed in already-computed rather than recomputed
    // inside it. `selected` uses `DateRange::contains` (inclusive of the
    // range's start/end) while `range_start`/`range_end`/`in_range` use
    // `is_start`/`is_end`/`is_between` (which exclude them) -- these are
    // deliberately different predicates for deliberately different
    // attributes; see `DateRange::contains` vs `contained_in_interval`.
    let state = day_state(
        date,
        &base_ctx,
        view_date,
        is_focused(),
        DaySelection {
            selected,
            range_start: is_start(date, range),
            range_end: is_end(date, range),
            in_range: is_between(date, range),
        },
    );
    let in_current_month = !state.outside_month;

    let clamp_date_to_available_range = move |date| {
        let available_range = base_ctx.available_range();
        available_range.map_or(date, |range| range.clamp(date))
    };

    // Handle day selection
    let mut handle_day_select = move |day: u8| {
        if (base_ctx.disabled)() || is_unavailable {
            return;
        }

        let view_date = view_ctx.offset_view_date();
        let date = view_date
            .replace_day(day)
            .ok()
            .map(clamp_date_to_available_range);
        ctx.set_selected_date(date);
        base_ctx.focused_date.set(date);
    };

    let focusable_date = (base_ctx.focused_date)()
        .filter(|d| d.month() == view_date.month())
        .or_else(|| {
            ctx.anchor_date
                .cloned()
                .filter(|d| d.month() == view_date.month())
        })
        .unwrap_or(view_date);

    // Same reasoning as `SingleCalendarDay` above.
    let defaults = attributes!(button { r#type: "button" });
    let owned = merge_attributes(vec![
        calendar_day_attributes(&state),
        attributes!(button {
            tabindex: if date == focusable_date { "0" } else { "-1" },
        }),
    ]);
    let merged = merge_attributes(vec![defaults, attributes, owned]);

    rsx! {
        button {
            onclick: move |e| {
                e.prevent_default();
                if in_current_month {
                    handle_day_select(day);
                }
            },
            onfocus: move |_| {
                if in_current_month {
                    base_ctx.focused_date.set(Some(date));
                }
            },
            onmouseover: move |_| {
                if in_current_month {
                    ctx.set_hovered_date(clamp_date_to_available_range(date));
                }
            },
            onmounted,
            ..merged,
            {content}
        }
    }
}
