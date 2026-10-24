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
    // Leave it out to show the month of today.
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

## Today

`today` (the day marked `data-today`) defaults to the visitor's local date, and an uncontrolled
`view_date` to its month. Both are read after the calendar mounts (`dioxus_primitives::calendar::use_today`),
so a page prerendered at build time shows the build day only until it hydrates, never after. Read the
date the same way in your own code that renders it.

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

In a month dropdown the options and the shown value print the short month name ("Oct"), as shadcn does; the plain-text caption keeps "October". The short name is `on_format_month_short`, or, when that is `None`, the first three characters of `on_format_month`, so a localised calendar gets a localised short name; pass the callback for a language where a three-character cut is wrong.

The default changed from "always both dropdowns" to `Label`, following shadcn: pass `caption_layout: CaptionLayout::Dropdown` to keep the old look. `DatePicker` and `DateRangePicker` take the same `caption_layout` prop (default `Label`) and hand it to the calendar in their popover; the `date_of_birth` variant of the Date Picker page shows the three dropdown layouts for a date of birth.

## Cell size

shadcn's `--cell-size` is `--dx-calendar-cell-size` (default `--dx-space-7`, 28px). The day cells, weekday row, caption height, nav buttons and month width all read it, so setting it on the `Calendar` (for example `style: "--dx-calendar-cell-size: 2.5rem;"`) retunes the whole month. See the `cell_size` variant.

## Week numbers

`show_week_number: true` adds a column of week numbers in front of every week row, and an empty header cell above it (react-day-picker's `showWeekNumber`). With `first_day_of_week: Weekday::Monday` the numbers are ISO 8601 weeks (week 1 holds the year's first Thursday); with any other first day, week 1 is the week that holds January 1st, which is what react-day-picker prints for `en-US`. Each number is a `rowheader` cell labelled "Week N" and is not focusable, so the day grid keeps its single tab stop and its arrow keys. See the `week_numbers` variant (Sunday start) and the `iso_week_numbers` variant (Monday start, opening on January 2027, whose first row is week 53 of 2026).

On the primitive, `use_calendar_grid()` returns `show_week_number()` and `week_numbers()` (one number per row of `weeks()`), and `CalendarGridWeekNumberHeader` / `CalendarGridWeekNumber` render the two cells when you compose your own grid.
