use super::*;

/// The props for the [`CalendarGrid`] component.
#[derive(Props, Clone, PartialEq)]
pub struct CalendarGridProps {
    /// Optional ID for the grid
    #[props(default)]
    pub id: Option<String>,

    /// Additional attributes to apply to the grid element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// The props for the [`CalendarGridRoot`] component.
#[derive(Props, Clone, PartialEq)]
pub struct CalendarGridRootProps {
    /// Optional ID for the grid
    #[props(default)]
    pub id: Option<String>,

    /// Additional attributes to apply to the grid element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the grid element
    pub children: Element,
}

/// The props for the [`CalendarGridHead`] component.
#[derive(Props, Clone, PartialEq)]
pub struct CalendarGridHeadProps {
    /// Additional attributes to apply to the grid head element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the grid head element
    pub children: Element,
}

/// The props for the [`CalendarGridHeaderRow`] component.
#[derive(Props, Clone, PartialEq)]
pub struct CalendarGridHeaderRowProps {
    /// Additional attributes to apply to the grid header row element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the grid header row element
    pub children: Element,
}

/// The props for the [`CalendarGridDayHeader`] component.
#[derive(Props, Clone, PartialEq)]
pub struct CalendarGridDayHeaderProps {
    /// The weekday represented by this header
    pub weekday: Weekday,

    /// Additional attributes to apply to the weekday header element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the weekday header element
    #[props(default)]
    pub children: Option<Element>,
}

/// The props for the [`CalendarGridBody`] component.
#[derive(Props, Clone, PartialEq)]
pub struct CalendarGridBodyProps {
    /// Additional attributes to apply to the grid body element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the grid body element
    pub children: Element,
}

/// The props for the [`CalendarGridWeek`] component.
#[derive(Props, Clone, PartialEq)]
pub struct CalendarGridWeekProps {
    /// Additional attributes to apply to the week row element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the week row element
    pub children: Element,
}

/// The props for the [`CalendarGridWeekNumberHeader`] component.
#[derive(Props, Clone, PartialEq)]
pub struct CalendarGridWeekNumberHeaderProps {
    /// Additional attributes to apply to the header cell
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The content of the header cell (empty by default)
    #[props(default)]
    pub children: Element,
}

/// The props for the [`CalendarGridWeekNumber`] component.
#[derive(Props, Clone, PartialEq)]
pub struct CalendarGridWeekNumberProps {
    /// The number of the week this cell labels
    pub week_number: u8,

    /// Additional attributes to apply to the week-number cell
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// Replaces the number as the cell's content
    #[props(default)]
    pub children: Option<Element>,
}

/// The props for the [`CalendarGridCell`] component.
#[derive(Props, Clone, PartialEq)]
pub struct CalendarGridCellProps {
    /// Additional attributes to apply to the day cell element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the day cell element
    pub children: Element,
}

/// Data for a weekday header in a calendar grid.
#[derive(Clone, PartialEq)]
pub struct CalendarGridWeekday {
    weekday: Weekday,
    label: String,
}

impl CalendarGridWeekday {
    /// The weekday represented by this header.
    pub fn weekday(&self) -> Weekday {
        self.weekday
    }

    /// The formatted weekday label.
    pub fn label(&self) -> &str {
        &self.label
    }
}

/// Data returned by [`use_calendar_grid`].
#[derive(Clone, PartialEq)]
pub struct CalendarGridData {
    view_date: Date,
    weekdays: Vec<CalendarGridWeekday>,
    weeks: Vec<Vec<Date>>,
    week_numbers: Vec<u8>,
    show_week_number: bool,
}

impl CalendarGridData {
    /// The first date in the month currently displayed by the grid.
    pub fn view_date(&self) -> Date {
        self.view_date
    }

    /// Weekday headers in display order.
    pub fn weekdays(&self) -> &[CalendarGridWeekday] {
        &self.weekdays
    }

    /// Weeks in the displayed month, each containing seven dates.
    pub fn weeks(&self) -> &[Vec<Date>] {
        &self.weeks
    }

    /// Whether the grid shows a week-number column (the calendar's `show_week_number`).
    pub fn show_week_number(&self) -> bool {
        self.show_week_number
    }

    /// The week number of each row of [`weeks`](Self::weeks), in the same order.
    pub fn week_numbers(&self) -> &[u8] {
        &self.week_numbers
    }
}

/// Return the weekday headers and week rows for the current calendar grid view.
pub fn use_calendar_grid() -> CalendarGridData {
    let ctx: BaseCalendarContext = use_context();
    let view_ctx: CalendarViewContext = use_context();

    use_memo(move || {
        let view_date = view_ctx.offset_view_date();
        let weeks = calendar_grid_weeks(view_date, ctx.first_day_of_week);
        let week_numbers = weeks
            .iter()
            .map(|week| week_number(week[0], ctx.first_day_of_week))
            .collect();
        CalendarGridData {
            view_date,
            weekdays: calendar_grid_weekdays(ctx.first_day_of_week, ctx.format_weekday),
            weeks,
            week_numbers,
            show_week_number: ctx.show_week_number,
        }
    })()
}

fn calendar_grid_weekdays(
    first_day_of_week: Weekday,
    format_weekday: Callback<Weekday, String>,
) -> Vec<CalendarGridWeekday> {
    WeekdaySet(0b111_1111)
        .iter(first_day_of_week)
        .map(|weekday| CalendarGridWeekday {
            weekday,
            label: format_weekday.call(weekday),
        })
        .collect()
}

pub(super) fn calendar_grid_weeks(view_date: Date, first_day_of_week: Weekday) -> Vec<Vec<Date>> {
    let mut grid = Vec::new();

    let previous_month = view_date
        .replace_day(1)
        .expect("invalid or out-of-range date");
    let num_days = days_since(view_date, first_day_of_week);
    let mut date = previous_month.saturating_sub(num_days.days());
    for _ in 1..=num_days {
        grid.push(date);
        date = date.next_day().expect("invalid or out-of-range date");
    }

    let mut date = view_date;
    let num_days_in_month = view_date.month().length(view_date.year());
    for day in 1..=num_days_in_month {
        date = view_date
            .replace_day(day)
            .expect("invalid or out-of-range date");
        grid.push(date);
    }

    let remainder = grid.len() % 7;
    if remainder > 0 {
        for _ in 1..=(7 - remainder) {
            date = date.next_day().expect("invalid or out-of-range date");
            grid.push(date);
        }
    }

    grid.chunks(7).map(|chunk| chunk.to_vec()).collect()
}

/// # CalendarGrid
///
/// The [`CalendarGrid`] component displays the grid of days for the current month in the calendar.
///
/// This must be used inside a [`Calendar`] component.
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
/// ## Styling
///
/// The [`CalendarGrid`] component renders days in a grid that can be styled using CSS. They define the following data attributes:
/// - `data-today`: If the date is today. Possible values are `true` or `false`
/// - `data-selected`: If the date is selected. Possible values are `true` or `false`
/// - `data-month`: The relative month of the date. Possible values are `last`, `current`, or `next`
#[component]
pub fn CalendarGrid(props: CalendarGridProps) -> Element {
    let grid = use_calendar_grid();

    rsx! {
        CalendarGridRoot {
            id: props.id,
            attributes: props.attributes,
            CalendarGridHead {
                CalendarGridHeaderRow {
                    if grid.show_week_number() {
                        CalendarGridWeekNumberHeader {}
                    }
                    for weekday in grid.weekdays().iter().cloned() {
                        CalendarGridDayHeader {
                            key: "{weekday.weekday():?}",
                            weekday: weekday.weekday(),
                            {weekday.label().to_string()}
                        }
                    }
                }
            }
            CalendarGridBody {
                for (week , week_number) in grid.weeks().iter().zip(grid.week_numbers().iter().copied()) {
                    CalendarGridWeek {
                        if grid.show_week_number() {
                            CalendarGridWeekNumber { week_number }
                        }
                        for date in week.iter().copied() {
                            CalendarGridCell {
                                key: "{date}",
                                CalendarDay { date }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// The root table element for a calendar grid.
///
/// ## Styling
///
/// - `data-direction`: The resolved text direction. Values are `ltr` or `rtl`.
#[component]
pub fn CalendarGridRoot(props: CalendarGridRootProps) -> Element {
    let base_ctx: BaseCalendarContext = use_context();

    // `role`/`dir`/`data-direction` are all owned semantics/state; merge with
    // the caller's attributes instead of leaving both a literal and a caller
    // override on the same element (backlog row 93).
    let owned = attributes!(table {
        role: "grid",
        dir: (base_ctx.direction)().as_str(),
        "data-direction": (base_ctx.direction)().as_str(),
    });
    let merged = merge_attributes(vec![props.attributes.clone(), owned]);

    rsx! {
        table {
            id: props.id,
            ..merged,
            {props.children}
        }
    }
}

/// The header section of a calendar grid.
#[component]
pub fn CalendarGridHead(props: CalendarGridHeadProps) -> Element {
    let owned = attributes!(thead {
        aria_hidden: "true"
    });
    let merged = merge_attributes(vec![props.attributes.clone(), owned]);

    rsx! {
        thead {
            ..merged,
            {props.children}
        }
    }
}

/// The row that contains weekday header cells.
#[component]
pub fn CalendarGridHeaderRow(props: CalendarGridHeaderRowProps) -> Element {
    rsx! {
        tr {
            ..props.attributes,
            {props.children}
        }
    }
}

/// A weekday header cell in a calendar grid.
#[component]
pub fn CalendarGridDayHeader(props: CalendarGridDayHeaderProps) -> Element {
    let ctx: BaseCalendarContext = use_context();
    let children = props.children.unwrap_or_else(|| {
        let label = ctx.format_weekday.call(props.weekday);
        rsx! { {label} }
    });

    rsx! {
        th {
            ..props.attributes,
            {children}
        }
    }
}

/// The body section of a calendar grid.
#[component]
pub fn CalendarGridBody(props: CalendarGridBodyProps) -> Element {
    rsx! {
        tbody {
            ..props.attributes,
            {props.children}
        }
    }
}

/// A week row in a calendar grid.
#[component]
pub fn CalendarGridWeek(props: CalendarGridWeekProps) -> Element {
    let owned = attributes!(tr { role: "row" });
    let merged = merge_attributes(vec![props.attributes.clone(), owned]);

    rsx! {
        tr {
            ..merged,
            {props.children}
        }
    }
}

/// A day cell in a calendar grid.
#[component]
pub fn CalendarGridCell(props: CalendarGridCellProps) -> Element {
    rsx! {
        td {
            ..props.attributes,
            {props.children}
        }
    }
}

/// The empty header cell above the week-number column (react-day-picker's `week_number_header`).
///
/// Render it first in the [`CalendarGridHeaderRow`] when [`CalendarGridData::show_week_number`]
/// is set. It sits in the `aria-hidden` header, so it is purely a layout cell.
#[component]
pub fn CalendarGridWeekNumberHeader(props: CalendarGridWeekNumberHeaderProps) -> Element {
    let defaults = attributes!(th {
        aria_label: "Week number"
    });
    let merged = merge_attributes(vec![defaults, props.attributes.clone()]);

    rsx! {
        th {
            ..merged,
            {props.children}
        }
    }
}

/// The week-number cell that starts a week row (react-day-picker's `week_number`).
///
/// Render it first in the [`CalendarGridWeek`] when [`CalendarGridData::show_week_number`] is set.
/// It is a `rowheader` labelled "Week N", so a screen reader announces the week of the day cell
/// it is on; it is not focusable, so the grid keeps its single roving tab stop.
#[component]
pub fn CalendarGridWeekNumber(props: CalendarGridWeekNumberProps) -> Element {
    let week_number = props.week_number;
    let label = format!("Week {week_number}");
    // `role` is the cell's owned semantics; the label is an overridable default.
    let defaults = attributes!(td { aria_label: label });
    let owned = attributes!(td { role: "rowheader" });
    let merged = merge_attributes(vec![defaults, props.attributes.clone(), owned]);
    let children = props.children.unwrap_or_else(|| rsx! { "{week_number}" });

    rsx! {
        td {
            ..merged,
            {children}
        }
    }
}
