The Calendar component is used to display a calendar interface, allowing users to select dates. It provides a grid layout of days for a specific month and year.

## Component Structure

```rust
Calendar {
    // The currently selected date in the calendar (if any).
    selected_date,
    on_date_change: |date: Option<CalendarDate>| {
        // This callback is triggered when a date is selected in the calendar.
        // The date parameter contains the selected date.
    },
    // The current view date of the calendar, which determines the month and year displayed.
    view_date,
    on_view_change: |date: CalendarDate| {
        // This callback is triggered when the view date changes.
        // The date parameter contains the new view date.
    },
    // Optional number of pre-composed month views to render.
    month_count: 1,
    // How the month caption is drawn: Label (default), Dropdown, DropdownMonths, DropdownYears.
    caption_layout: CaptionLayout::Label,
    // Show a week-number column.
    show_week_number: false,
}
```

The styled `Calendar` and `RangeCalendar` render a complete month view by default, including the header, navigation, caption (see `caption_layout`), and date grid.

## Direction / RTL

`Calendar`/`RangeCalendar` accept a `dir: Option<Direction>` prop. Under RTL, the day grid's `ArrowLeft`/`ArrowRight` swap (`ArrowLeft` moves to the next day, `ArrowRight` to the previous one); `ArrowUp`/`ArrowDown` (+/-7 days) are unaffected. No Radix Calendar exists to cite (shadcn's own wraps `react-day-picker`) -- this follows the same arrow-key convention as every other RTL-aware component here, and matches standard calendar-widget RTL practice generally. See the `rtl` variant.

## Multiple months

`month_count` renders that many months in one calendar. They sit side by side in a flex row with a
gap (the previous-month button on the first month, the next-month button on the last) and wrap onto
their own rows when the row is wider than the space available -- on a phone, or in a narrow card, two
months stack vertically instead of overflowing. Inside a `DatePicker`/`DateRangePicker` popover, the
panel grows to fit the months. See the `multi_month` variant.

## Caption layout

`caption_layout` chooses how each month's caption is drawn (react-day-picker's `captionLayout`, which shadcn's `Calendar` passes through):

| `CaptionLayout` | Caption |
| --- | --- |
| `Label` (default) | plain text, "October 2026", between the previous and next arrows |
| `Dropdown` | a month select and a year select |
| `DropdownMonths` | a month select; the year is plain text |
| `DropdownYears` | a year select; the month is plain text |

The selects are native `<select>` elements named "Month" and "Year", so they keep the platform keyboard and touch behaviour (the options are limited by `min_date` / `max_date`). The caption text is a polite live region; in the dropdown layouts the full "October 2026" is kept as a visually hidden live region too, so a screen reader hears where an arrow click landed. The label (and the dropdown option text) goes through `on_format_month`. See the `caption` variant for all four side by side.

The default changed from "always both dropdowns" to `Label`, following shadcn: pass `caption_layout: CaptionLayout::Dropdown` to keep the old look. `DatePicker` and `DateRangePicker` use the default.

## Week numbers

`show_week_number: true` adds a column of week numbers in front of every week row, and an empty header cell above it (react-day-picker's `showWeekNumber`). With `first_day_of_week: Weekday::Monday` the numbers are ISO 8601 weeks (week 1 holds the year's first Thursday); with any other first day, week 1 is the week that holds January 1st, which is what react-day-picker prints for `en-US`. Each number is a `rowheader` cell labelled "Week N" and is not focusable, so the day grid keeps its single tab stop and its arrow keys. See the `week_numbers` variant.

On the primitive, `use_calendar_grid()` returns `show_week_number()` and `week_numbers()` (one number per row of `weeks()`), and `CalendarGridWeekNumberHeader` / `CalendarGridWeekNumber` render the two cells when you compose your own grid.
