use dioxus::prelude::*;
use dioxus_icons::lucide::{ChevronDown, ChevronLeft, ChevronRight};
use dioxus_primitives::calendar::{
    self, CalendarDayProps, CalendarGridBodyProps, CalendarGridCellProps,
    CalendarGridDayHeaderProps, CalendarGridHeadProps, CalendarGridHeaderRowProps,
    CalendarGridRootProps, CalendarGridWeekProps, CalendarHeaderProps, CalendarNavigationProps,
    CalendarSelectMonthProps, CalendarSelectMonthSelectProps, CalendarSelectMonthValueProps,
    CalendarSelectYearProps, CalendarSelectYearSelectProps, CalendarSelectYearValueProps,
    CalendarViewProps,
};
use dioxus_primitives::{dioxus_attributes::attributes, merge_attributes};
use time::{Date, Month, Weekday};

fn fixed_date(year: i32, month: Month, day: u8) -> Date {
    Date::from_calendar_date(year, month, day).expect("valid fixed date")
}

pub(crate) fn weekday_abbreviation(weekday: Weekday) -> &'static str {
    match weekday {
        Weekday::Monday => "Mo",
        Weekday::Tuesday => "Tu",
        Weekday::Wednesday => "We",
        Weekday::Thursday => "Th",
        Weekday::Friday => "Fr",
        Weekday::Saturday => "Sa",
        Weekday::Sunday => "Su",
    }
}

/// How a month's caption is drawn (react-day-picker's `captionLayout`, shadcn's `Calendar`).
///
/// `Label` (the default) is the plain text "October 2026" between the previous/next arrows. The
/// three dropdown layouts swap the month and/or the year for a native `<select>` (labelled "Month"
/// and "Year"), so a far-away date is one pick instead of many clicks on the arrows; the part that
/// is not a dropdown stays plain text beside it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CaptionLayout {
    /// Plain "Month Year" text.
    #[default]
    Label,
    /// A month select and a year select.
    Dropdown,
    /// A month select; the year is plain text.
    DropdownMonths,
    /// A year select; the month is plain text.
    DropdownYears,
}

impl CaptionLayout {
    fn has_month_dropdown(self) -> bool {
        matches!(self, Self::Dropdown | Self::DropdownMonths)
    }

    fn has_year_dropdown(self) -> bool {
        matches!(self, Self::Dropdown | Self::DropdownYears)
    }
}

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

    /// Callback when displaying a month inside a month dropdown ("Oct"); `None` keeps the first
    /// three characters of `on_format_month`
    #[props(default)]
    pub on_format_month_short: Option<Callback<Month, String>>,

    /// The month being viewed; `None` (the default) shows the month of `today`
    #[props(default)]
    pub view_date: ReadSignal<Option<Date>>,

    /// The current date (used for highlighting today); `None` (the default) is the visitor's local
    /// date, read after mount
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

    /// How the month caption is drawn: plain text (default) or month/year dropdowns
    #[props(default)]
    pub caption_layout: CaptionLayout,

    /// Show a week-number column (ISO weeks with a Monday `first_day_of_week`, otherwise the week
    /// holding January 1st is week 1)
    #[props(default)]
    pub show_week_number: bool,

    /// Lower limit of the range of available dates
    #[props(default = fixed_date(1925, Month::January, 1))]
    pub min_date: Date,

    /// Upper limit of the range of available dates
    #[props(default = fixed_date(2050, Month::December, 31))]
    pub max_date: Date,

    /// Specify how many months are visible at once
    #[props(default = 1)]
    pub month_count: u8,

    /// Unavailable dates
    #[props(default)]
    pub disabled_ranges: ReadSignal<Vec<calendar::DateRange>>,

    /// The text direction for the day grid's `ArrowLeft`/`ArrowRight` step.
    /// Defaults to the nearest [`dioxus_primitives::direction::DirectionProvider`], or
    /// LTR if there is none.
    #[props(default)]
    pub dir: Option<dioxus_primitives::direction::Direction>,

    /// Additional attributes to extend the calendar element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

#[derive(Props, Clone, PartialEq)]
pub struct RangeCalendarProps {
    /// The selected range
    #[props(default)]
    pub selected_range: ReadSignal<Option<calendar::DateRange>>,

    /// Callback when selected date range changes
    #[props(default)]
    pub on_range_change: Callback<Option<calendar::DateRange>>,

    /// Callback when display weekday
    #[props(default = Callback::new(|weekday: Weekday| weekday_abbreviation(weekday).to_string()))]
    pub on_format_weekday: Callback<Weekday, String>,

    /// Callback when display month
    #[props(default = Callback::new(|month: Month| month.to_string()))]
    pub on_format_month: Callback<Month, String>,

    /// Callback when displaying a month inside a month dropdown ("Oct"); `None` keeps the first
    /// three characters of `on_format_month`
    #[props(default)]
    pub on_format_month_short: Option<Callback<Month, String>>,

    /// The month being viewed; `None` (the default) shows the month of `today`
    #[props(default)]
    pub view_date: ReadSignal<Option<Date>>,

    /// The current date (used for highlighting today); `None` (the default) is the visitor's local
    /// date, read after mount
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

    /// How the month caption is drawn: plain text (default) or month/year dropdowns
    #[props(default)]
    pub caption_layout: CaptionLayout,

    /// Show a week-number column (ISO weeks with a Monday `first_day_of_week`, otherwise the week
    /// holding January 1st is week 1)
    #[props(default)]
    pub show_week_number: bool,

    /// Lower limit of the range of available dates
    #[props(default = fixed_date(1925, Month::January, 1))]
    pub min_date: Date,

    /// Upper limit of the range of available dates
    #[props(default = fixed_date(2050, Month::December, 31))]
    pub max_date: Date,

    /// Specify how many months are visible at once
    #[props(default = 1)]
    pub month_count: u8,

    /// Unavailable dates
    #[props(default)]
    pub disabled_ranges: ReadSignal<Vec<calendar::DateRange>>,

    /// The text direction -- see [`dioxus_primitives::calendar::CalendarProps::dir`]'s identical doc.
    #[props(default)]
    pub dir: Option<dioxus_primitives::direction::Direction>,

    /// Additional attributes to extend the calendar element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

#[component]
pub fn Calendar(props: CalendarProps) -> Element {
    let month_count = props.month_count.max(1);

    rsx! {
        CalendarRoot {
            dir: props.dir,
            selected_date: props.selected_date,
            on_date_change: props.on_date_change,
            on_format_weekday: props.on_format_weekday,
            on_format_month: props.on_format_month,
            on_format_month_short: props.on_format_month_short,
            view_date: props.view_date,
            today: props.today,
            on_view_change: props.on_view_change,
            disabled: props.disabled,
            first_day_of_week: props.first_day_of_week,
            show_week_number: props.show_week_number,
            min_date: props.min_date,
            max_date: props.max_date,
            disabled_ranges: props.disabled_ranges,
            attributes: props.attributes,
            for offset in 0..month_count {
                CalendarMonthView {
                    key: "{offset}",
                    offset,
                    month_count,
                    caption_layout: props.caption_layout,
                }
            }
        }
    }
}

#[component]
pub fn RangeCalendar(props: RangeCalendarProps) -> Element {
    let month_count = props.month_count.max(1);

    rsx! {
        RangeCalendarRoot {
            dir: props.dir,
            selected_range: props.selected_range,
            on_range_change: props.on_range_change,
            on_format_weekday: props.on_format_weekday,
            on_format_month: props.on_format_month,
            on_format_month_short: props.on_format_month_short,
            view_date: props.view_date,
            today: props.today,
            on_view_change: props.on_view_change,
            disabled: props.disabled,
            first_day_of_week: props.first_day_of_week,
            show_week_number: props.show_week_number,
            min_date: props.min_date,
            max_date: props.max_date,
            disabled_ranges: props.disabled_ranges,
            attributes: props.attributes,
            for offset in 0..month_count {
                CalendarMonthView {
                    key: "{offset}",
                    offset,
                    month_count,
                    caption_layout: props.caption_layout,
                }
            }
        }
    }
}

// dev-docs/backlog.md row 51: the stylesheet `Link` lives HERE, on the
// `pub(crate)` root each of this module's real entry points funnels
// through, not on `Calendar`/`RangeCalendar` above -- `date_picker`
// (`../date_picker/component.rs`) composes `CalendarRoot`/
// `RangeCalendarRoot` directly (`super::super::calendar::*`), bypassing
// `Calendar`/`RangeCalendar` entirely, since `DatePickerCalendar`'s
// `calendar: fn(T) -> Element` prop needs a bare component function to call,
// not a wrapper that also takes a `month_count` prop. A Link only on
// `Calendar`/`RangeCalendar` would never render on that composition path, and
// the calendar's CSS would silently never reach
// `/component/?name=date_picker&`.
// Putting it here instead -- matching `popover/component.rs`'s pattern of a
// `Link` on every one of ITS composable entry points, not just its "main"
// one -- means every path that ends up rendering the primitive (this
// module's own `Calendar`/`RangeCalendar`, and any other themed component
// that composes `CalendarRoot`/`RangeCalendarRoot` the way `date_picker`
// does) gets the stylesheet for free, by construction; `document::Link`'s
// `(href, rel)` dedup would make a second `document::Link` on `Calendar`
// redundant, so `Calendar` carries none.
#[component]
pub(crate) fn CalendarRoot(props: calendar::CalendarProps) -> Element {
    let base = attributes!(div {
        class: "dx-calendar", "data-dx-root": ""
    });
    let merged = merge_attributes(vec![base, props.attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/calendar/style.css") }
        calendar::Calendar {
            dir: props.dir,
            selected_date: props.selected_date,
            on_date_change: props.on_date_change,
            on_format_weekday: props.on_format_weekday,
            on_format_month: props.on_format_month,
            on_format_month_short: props.on_format_month_short,
            view_date: props.view_date,
            today: props.today,
            on_view_change: props.on_view_change,
            disabled: props.disabled,
            first_day_of_week: props.first_day_of_week,
            show_week_number: props.show_week_number,
            min_date: props.min_date,
            max_date: props.max_date,
            disabled_ranges: props.disabled_ranges,
            attributes: merged,
            {props.children}
        }
    }
}

// See the comment on `CalendarRoot` above -- identical reasoning, for the
// range-calendar arm `date_picker::DateRangePickerCalendar` composes.
#[component]
pub(crate) fn RangeCalendarRoot(props: calendar::RangeCalendarProps) -> Element {
    let base = attributes!(div {
        class: "dx-calendar", "data-dx-root": ""
    });
    let merged = merge_attributes(vec![base, props.attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/calendar/style.css") }
        calendar::RangeCalendar {
            dir: props.dir,
            selected_range: props.selected_range,
            on_range_change: props.on_range_change,
            on_format_weekday: props.on_format_weekday,
            on_format_month: props.on_format_month,
            on_format_month_short: props.on_format_month_short,
            view_date: props.view_date,
            today: props.today,
            on_view_change: props.on_view_change,
            disabled: props.disabled,
            first_day_of_week: props.first_day_of_week,
            show_week_number: props.show_week_number,
            min_date: props.min_date,
            max_date: props.max_date,
            disabled_ranges: props.disabled_ranges,
            attributes: merged,
            {props.children}
        }
    }
}

#[component]
pub(crate) fn CalendarMonthView(
    offset: u8,
    month_count: u8,
    #[props(default)] caption_layout: CaptionLayout,
) -> Element {
    let show_previous = offset == 0;
    let show_next = offset.saturating_add(1) == month_count;

    rsx! {
        CalendarView { offset,
            CalendarHeader {
                CalendarNavigation {
                    if show_previous {
                        CalendarPreviousMonthButton {}
                    }
                    CalendarCaption { layout: caption_layout }
                    if show_next {
                        CalendarNextMonthButton {}
                    }
                }
            }
            CalendarGrid {}
        }
    }
}

// react-day-picker's `MonthCaption`: `label` is one live-region text node; the dropdown layouts put
// a native select (or, for the part that is not a dropdown, plain text) per field and keep the full
// "October 2026" as a visually hidden live region, because a select's own name ("Month") does not
// say which month a prev/next click landed on.
#[component]
fn CalendarCaption(layout: CaptionLayout) -> Element {
    if layout == CaptionLayout::Label {
        let attributes = attributes!(div {
            class: "dx-calendar-month-title",
            role: "status",
            aria_live: "polite",
        });
        return rsx! {
            calendar::CalendarMonthTitle { attributes }
        };
    }

    let text = attributes!(span { class: "dx-calendar-caption-text" });
    let hidden = attributes!(div {
        class: "dx-calendar-sr-only",
        role: "status",
        aria_live: "polite",
    });
    rsx! {
        if layout.has_month_dropdown() {
            CalendarSelectMonth {}
        } else {
            calendar::CalendarSelectMonthValue { attributes: text.clone() }
        }
        if layout.has_year_dropdown() {
            CalendarSelectYear {}
        } else {
            calendar::CalendarSelectYearValue { attributes: text }
        }
        calendar::CalendarMonthTitle { attributes: hidden }
    }
}

#[component]
fn CalendarView(props: CalendarViewProps) -> Element {
    let base = attributes!(div { class: "dx-calendar-view" });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        calendar::CalendarView { offset: props.offset, attributes: merged, {props.children} }
    }
}

#[component]
fn CalendarHeader(props: CalendarHeaderProps) -> Element {
    rsx! {
        calendar::CalendarHeader { id: props.id, attributes: props.attributes, {props.children} }
    }
}

#[component]
fn CalendarNavigation(props: CalendarNavigationProps) -> Element {
    let base = attributes!(div { class: "dx-calendar-navigation" });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        calendar::CalendarNavigation { attributes: merged, {props.children} }
    }
}

#[component]
fn CalendarPreviousMonthButton(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let base = attributes!(button { class: "dx-calendar-nav-prev" });
    let merged = merge_attributes(vec![base, attributes]);
    rsx! {
        calendar::CalendarPreviousMonthButton { attributes: merged,
            ChevronLeft { size: "20px" }
        }
    }
}

#[component]
fn CalendarNextMonthButton(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let base = attributes!(button { class: "dx-calendar-nav-next" });
    let merged = merge_attributes(vec![base, attributes]);
    rsx! {
        calendar::CalendarNextMonthButton { attributes: merged,
            ChevronRight { size: "20px" }
        }
    }
}

#[component]
fn CalendarSelectMonth(props: CalendarSelectMonthProps) -> Element {
    let base = attributes!(div { class: "dx-calendar-month-select-container" });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        calendar::CalendarSelectMonth {
            attributes: merged,
            CalendarSelectMonthSelect {}
            CalendarSelectMonthValue { short: true,
                DropDownIcon { }
                {props.children}
            }
        }
    }
}

#[component]
fn CalendarSelectMonthSelect(props: CalendarSelectMonthSelectProps) -> Element {
    let base = attributes!(div { class: "dx-calendar-month-select" });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        calendar::CalendarSelectMonthSelect { attributes: merged }
    }
}

#[component]
fn CalendarSelectMonthValue(props: CalendarSelectMonthValueProps) -> Element {
    let base = attributes!(div { class: "dx-calendar-month-select-value" });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        calendar::CalendarSelectMonthValue {
            attributes: merged,
            short: props.short,
            {props.children}
        }
    }
}

#[component]
fn CalendarSelectYear(props: CalendarSelectYearProps) -> Element {
    let base = attributes!(div { class: "dx-calendar-year-select-container" });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        calendar::CalendarSelectYear {
            attributes: merged,
            CalendarSelectYearSelect {}
            CalendarSelectYearValue {
                DropDownIcon { }
                {props.children}
            }
        }
    }
}

#[component]
fn CalendarSelectYearSelect(props: CalendarSelectYearSelectProps) -> Element {
    let base = attributes!(div { class: "dx-calendar-year-select" });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        calendar::CalendarSelectYearSelect { attributes: merged }
    }
}

#[component]
fn CalendarSelectYearValue(props: CalendarSelectYearValueProps) -> Element {
    let base = attributes!(div { class: "dx-calendar-year-select-value" });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        calendar::CalendarSelectYearValue { attributes: merged, {props.children} }
    }
}

#[component]
fn CalendarGrid(
    #[props(default)] id: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let grid = calendar::use_calendar_grid();
    let show_week_number = grid.show_week_number();
    // The root's owned `data-*` hook for the week-number column's wider grid (style.css).
    let state = attributes!(div { "data-week-numbers": show_week_number });
    let attributes = merge_attributes(vec![state, attributes]);

    rsx! {
        CalendarGridRoot { id, attributes,
            CalendarGridHead {
                CalendarGridHeaderRow {
                    if show_week_number {
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
                        if show_week_number {
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

#[component]
fn CalendarGridWeekNumberHeader() -> Element {
    let base = attributes!(th {
        class: "dx-calendar-grid-weeknum"
    });
    rsx! {
        calendar::CalendarGridWeekNumberHeader { attributes: base }
    }
}

#[component]
fn CalendarGridWeekNumber(week_number: u8) -> Element {
    let base = attributes!(td {
        class: "dx-calendar-grid-weeknum"
    });
    rsx! {
        calendar::CalendarGridWeekNumber { week_number, attributes: base }
    }
}

#[component]
fn CalendarGridRoot(props: CalendarGridRootProps) -> Element {
    let base = attributes!(div { class: "dx-calendar-grid" });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        calendar::CalendarGridRoot { id: props.id, attributes: merged, {props.children} }
    }
}

#[component]
fn CalendarGridHead(props: CalendarGridHeadProps) -> Element {
    rsx! {
        calendar::CalendarGridHead {
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[component]
fn CalendarGridHeaderRow(props: CalendarGridHeaderRowProps) -> Element {
    let base = attributes!(div { class: "dx-calendar-grid-header" });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        calendar::CalendarGridHeaderRow { attributes: merged, {props.children} }
    }
}

#[component]
fn CalendarGridDayHeader(props: CalendarGridDayHeaderProps) -> Element {
    let base = attributes!(div { class: "dx-calendar-grid-day-header" });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        calendar::CalendarGridDayHeader { weekday: props.weekday, attributes: merged, {props.children} }
    }
}

#[component]
fn CalendarGridBody(props: CalendarGridBodyProps) -> Element {
    let base = attributes!(div { class: "dx-calendar-grid-body" });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        calendar::CalendarGridBody { attributes: merged, {props.children} }
    }
}

#[component]
fn CalendarGridWeek(props: CalendarGridWeekProps) -> Element {
    let base = attributes!(div { class: "dx-calendar-grid-week" });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        calendar::CalendarGridWeek { attributes: merged, {props.children} }
    }
}

#[component]
fn CalendarGridCell(props: CalendarGridCellProps) -> Element {
    rsx! {
        calendar::CalendarGridCell {
            attributes: props.attributes,
            {props.children}
        }
    }
}

#[component]
fn CalendarDay(props: CalendarDayProps) -> Element {
    let base = attributes!(div { class: "dx-calendar-grid-cell" });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        calendar::CalendarDay {
            date: props.date,
            attributes: merged,
            children: props.children,
        }
    }
}

#[component]
fn DropDownIcon() -> Element {
    rsx! {
        ChevronDown {
            size: "20px",
            stroke: "var(--dx-foreground)",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[component]
    fn CalendarWithDefaultView() -> Element {
        rsx! {
            Calendar {
                view_date: fixed_date(2026, Month::May, 15),
            }
        }
    }

    #[component]
    fn CalendarWithDefaultMonthCount() -> Element {
        rsx! {
            Calendar {
                view_date: fixed_date(2026, Month::May, 15),
                month_count: 3,
            }
        }
    }

    #[test]
    fn calendar_renders_default_view_when_children_are_omitted() {
        let mut dom = VirtualDom::new(CalendarWithDefaultView);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);

        assert!(html.contains("Calendar"));
        assert_eq!(html.matches("role=\"grid\"").count(), 1);
    }

    #[test]
    fn calendar_month_count_renders_multiple_default_views() {
        let mut dom = VirtualDom::new(CalendarWithDefaultMonthCount);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);

        assert_eq!(html.matches("role=\"grid\"").count(), 3);
    }
}
