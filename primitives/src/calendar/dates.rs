use super::*;

// A collection of [`Weekday`]s stored as a single byte
// Implemented as a bitmask where bits 1-7 correspond to Monday-Sunday
#[derive(Clone, Copy)]
pub(super) struct WeekdaySet(pub(super) u8); // the 8-th bit is always 0

impl WeekdaySet {
    // Get the first day in the collection, starting from Monday
    // Returns `None` if the collection is empty
    pub(super) const fn first(self) -> Option<Weekday> {
        if self.is_empty() {
            return None;
        }

        // Find the first non-zero bit
        Some(Weekday::Monday.nth_next(self.0.trailing_zeros() as u8))
    }

    // Create a `WeekdaySet` from a single [`Weekday`]
    pub(super) const fn single(weekday: Weekday) -> Self {
        Self(1 << weekday.number_days_from_monday())
    }

    // Iterate over the [`Weekday`]s in the collection starting from a given day
    // Wraps around from Sunday to Monday if necessary
    pub(super) const fn iter(self, start: Weekday) -> WeekdaySetIter {
        WeekdaySetIter { days: self, start }
    }

    // Returns `true` if the collection is empty
    pub(super) const fn is_empty(self) -> bool {
        self.0 == 0
    }

    // Split the collection in two at the given day. Returns a tuple `(before, after)`
    // `before` contains all days starting from Monday up to but NOT including `weekday`
    // `after` contains all days starting from `weekday` up to and including Sunday
    const fn split_at(self, weekday: Weekday) -> (Self, Self) {
        let days_after = 0b1000_0000 - Self::single(weekday).0;
        let days_before = days_after ^ 0b0111_1111;
        (Self(self.0 & days_before), Self(self.0 & days_after))
    }

    // Returns `true` if the collection contains the given day
    pub(super) const fn contains(self, day: Weekday) -> bool {
        self.0 & Self::single(day).0 != 0
    }

    // Removes a day from the collection
    // Returns `true` if the collection did contain the day
    pub(super) fn remove(&mut self, day: Weekday) -> bool {
        if self.contains(day) {
            self.0 &= !Self::single(day).0;
            return true;
        }

        false
    }
}

// An iterator over a collection of weekdays, starting from a given day
pub(super) struct WeekdaySetIter {
    days: WeekdaySet,
    start: Weekday,
}

impl Iterator for WeekdaySetIter {
    type Item = Weekday;

    fn next(&mut self) -> Option<Self::Item> {
        if self.days.is_empty() {
            return None;
        }

        let (before, after) = self.days.split_at(self.start);
        let days = if after.is_empty() { before } else { after };

        let next = days.first().expect("the collection is not empty");
        self.days.remove(next);
        Some(next)
    }
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

// The number of days since the first weekday of current date
pub(super) fn days_since(date: Date, weekday: Weekday) -> i64 {
    let lhs = date.replace_day(1).unwrap().weekday() as i64;
    let rhs = weekday as i64;
    if lhs < rhs {
        7 + lhs - rhs
    } else {
        lhs - rhs
    }
}

/// The week number of the week that starts on `week_start_date` (a date on `first_day_of_week`),
/// numbered the way react-day-picker's `showWeekNumber` does (`date-fns` `getWeek`).
///
/// A Monday week start is ISO 8601: week 1 is the week that holds the year's first Thursday (the
/// first week with at least four days in the year). Any other start day numbers from the week that
/// holds January 1st (`en-US`: Sunday start, one day is enough), so a week that straddles the year
/// end is week 1 of the new year when it holds January 1st, and the last week of the old year
/// otherwise.
pub(crate) fn week_number(week_start_date: Date, first_day_of_week: Weekday) -> u8 {
    // The day of January that anchors week 1: the 4th (ISO) or the 1st.
    let anchor_day = if first_day_of_week == Weekday::Monday {
        4
    } else {
        1
    };
    let week_start = |date: Date| date.saturating_sub(days_before(date, first_day_of_week).days());
    let first_week_start = |year: i32| {
        Date::from_calendar_date(year, Month::January, anchor_day)
            .map(week_start)
            .ok()
    };

    let date = week_start(week_start_date);
    let year = date.year();
    // The week belongs to the year whose first week it is on or after, so December's last
    // days can be week 1 of next year.
    let week_year = match (first_week_start(year + 1), first_week_start(year)) {
        (Some(next), _) if date >= next => year + 1,
        (_, Some(this)) if date >= this => year,
        _ => year - 1,
    };
    let Some(first) = first_week_start(week_year) else {
        return 1;
    };
    let weeks = (date - first).whole_days().div_euclid(7) + 1;
    u8::try_from(weeks).unwrap_or(1)
}

/// The number of days from the most recent `weekday` (inclusive) to `date`.
fn days_before(date: Date, weekday: Weekday) -> i64 {
    (i64::from(date.weekday().number_days_from_monday())
        - i64::from(weekday.number_days_from_monday()))
    .rem_euclid(7)
}

pub(super) fn next_month(date: Date) -> Option<Date> {
    let next_month = date.month().next();
    let last_day = next_month.length(date.year());
    // Clamp the day to the length of the next month
    let current_day = date.day();
    let new_day = current_day.min(last_day);
    Date::from_calendar_date(
        date.year() + if next_month == Month::January { 1 } else { 0 },
        next_month,
        new_day,
    )
    .ok()
}

pub(super) fn previous_month(date: Date) -> Option<Date> {
    let previous_month = date.month().previous();
    let last_day = previous_month.length(date.year());
    // Clamp the day to the length of the previous month
    let current_day = date.day();
    let new_day = current_day.min(last_day);
    Date::from_calendar_date(
        date.year()
            + if previous_month == Month::December {
                -1
            } else {
                0
            },
        previous_month,
        new_day,
    )
    .ok()
}

pub(super) fn replace_month(date: Date, month: Month) -> Date {
    let year = date.year();
    let num_days = month.length(year);
    Date::from_calendar_date(year, month, std::cmp::min(date.day(), num_days))
        .expect("invalid or out-of-range date")
}

/// Move forward n months from the given date, handling year transitions
pub(super) fn nth_month_next(date: Date, n: u8) -> Option<Date> {
    match n {
        0 => Some(date),
        n => {
            let month = date.month();
            let nth_month = month.nth_next(n);
            let year = date.year() + if month > nth_month { 1 } else { 0 };
            let max_day = nth_month.length(year);
            Date::from_calendar_date(year, nth_month, date.day().min(max_day)).ok()
        }
    }
}

/// Move backward n months from the given date, handling year transitions
pub(super) fn nth_month_previous(date: Date, n: u8) -> Option<Date> {
    match n {
        0 => Some(date),
        n => {
            let month = date.month();
            let nth_month = month.nth_prev(n);
            let year = date.year() - if month < nth_month { 1 } else { 0 };
            let max_day = nth_month.length(year);
            Date::from_calendar_date(year, nth_month, date.day().min(max_day)).ok()
        }
    }
}

/// Resolve `ArrowLeft`/`ArrowRight` on the day grid to a `+1`/`-1` day step,
/// honoring [`Direction`] the same way every other roving-focus consumer in
/// this crate does (`Direction::resolve_horizontal`): the day "to the
/// right" is always `+1` day in LTR and `-1` day in RTL, since a calendar's
/// week row is itself a horizontal, direction-mirrored layout (the same
/// spatial-arrow-key convention every Radix source uses for roving focus,
/// Slider, and the menu family; no Radix/shadcn Calendar exists
/// to cite directly, since shadcn's own Calendar wraps `react-day-picker`,
/// not a Radix primitive). `ArrowUp`/`ArrowDown` (`±7 days`, a full week
/// row) are never direction-dependent and stay outside this function,
/// exactly like every other consumer's vertical arrows.
pub(super) fn horizontal_day_step(key: &Key, direction: Direction) -> Option<i64> {
    match direction.resolve_horizontal(key)? {
        HorizontalNav::Next => Some(1),
        HorizontalNav::Prev => Some(-1),
    }
}

/// Calendar date range
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct DateRange {
    /// The start date of the range
    pub(super) start: Date,
    /// The end date of the range
    pub(super) end: Date,
}

impl DateRange {
    /// Create a new date range
    pub fn new(start: Date, end: Date) -> Self {
        if start <= end {
            Self { start, end }
        } else {
            Self {
                start: end,
                end: start,
            }
        }
    }

    /// Returns true if date is contained in the range.
    pub fn contains(&self, date: Date) -> bool {
        self.start <= date && date <= self.end
    }

    pub(super) fn contained_in_interval(&self, date: Date) -> bool {
        self.start < date && date < self.end
    }

    pub(super) fn clamp(&self, date: Date) -> Date {
        date.clamp(self.start, self.end)
    }

    /// Get the start of the range
    pub fn start(&self) -> Date {
        self.start
    }

    /// Get the end of the range
    pub fn end(&self) -> Date {
        self.end
    }
}

impl Display for DateRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {}", self.start, self.end)
    }
}

/// Calendar available dates
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AvailableRanges {
    /// A sorted list of dates. Values after an odd number of elements are disabled.
    changes: Vec<Date>,
}

impl AvailableRanges {
    /// Create a new available dates
    pub fn new(disabled_ranges: &[DateRange]) -> Self {
        let mut sorted_range: Vec<_> = disabled_ranges
            .iter()
            .enumerate()
            .flat_map(|(index, date)| [(index, date.start), (index, date.end)])
            .collect();

        sorted_range.sort_by_key(|(_, date)| *date);

        // Merge any overlapping ranges
        let mut open_ranges = HashSet::new();
        let mut deduped_ranges = Vec::with_capacity(sorted_range.len());

        for (index, date) in sorted_range {
            let end_of_range = open_ranges.remove(&index);
            if open_ranges.is_empty() {
                deduped_ranges.push(date);
            }
            if !end_of_range {
                open_ranges.insert(index);
            }
        }

        Self {
            changes: deduped_ranges,
        }
    }

    /// Check the availability of given date
    pub fn valid_interval(&self, date: Date) -> bool {
        match self.changes.binary_search(&date) {
            Ok(_) => false,
            Err(index) => index % 2 == 0,
        }
    }

    /// Get the available range of given date
    pub fn available_range(&self, date: Date, date_range: DateRange) -> Option<DateRange> {
        let date_index = self.changes.binary_search(&date).err()?;
        let min_date = date_range.start();
        let max_date = date_range.end();

        let valid = date_index % 2 == 0;
        if !valid {
            return None;
        }

        let start = date_index
            .checked_sub(1)
            .and_then(|index| self.changes.get(index).copied())
            .map(|start| start.next_day().unwrap_or(start))
            .unwrap_or(min_date);
        let end = self
            .changes
            .get(date_index)
            .copied()
            .map(|end| end.previous_day().unwrap_or(end))
            .unwrap_or(max_date);

        Some(DateRange::new(start, end))
    }

    /// Get disabled ranges
    pub fn to_disabled_ranges(&self) -> Vec<DateRange> {
        self.changes
            .chunks(2)
            .map(|d| DateRange::new(d[0], d[1]))
            .collect()
    }
}
