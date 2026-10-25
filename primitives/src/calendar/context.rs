use super::*;

/// The base context provided by the [`Calendar`] and the [`RangeCalendar`] component to its children.
#[derive(Copy, Clone)]
pub struct BaseCalendarContext {
    // State
    pub(super) focused_date: Signal<Option<Date>>,
    pub(super) view_date: ReadSignal<Date>,
    pub(super) available_ranges: Memo<AvailableRanges>,
    pub(super) set_view_date: Callback<Date>,
    pub(super) format_weekday: Callback<Weekday, String>,
    pub(super) format_month: Callback<Month, String>,
    pub(super) format_month_short: Option<Callback<Month, String>>,

    // Configuration
    pub(super) disabled: ReadSignal<bool>,
    pub(super) today: ReadSignal<Date>,
    pub(super) first_day_of_week: Weekday,
    pub(super) show_week_number: bool,
    pub(super) enabled_date_range: DateRange,
    pub(super) view_registrations: Signal<Vec<CalendarViewRegistration>>,

    /// Text direction, for the day grid's `ArrowLeft`/`ArrowRight` step --
    /// see [`horizontal_day_step`]'s doc.
    pub(super) direction: ReadSignal<Direction>,
}

impl BaseCalendarContext {
    /// The month's name as a month dropdown prints it: `on_format_month_short`, else the first
    /// three characters of `on_format_month` ("October" -> "Oct").
    pub(super) fn month_short_name(&self, month: Month) -> String {
        match self.format_month_short {
            Some(short) => short.call(month),
            None => self.format_month.call(month).chars().take(3).collect(),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(super) struct CalendarViewRegistration {
    id: ScopeId,
    offset: Option<u8>,
}

impl BaseCalendarContext {
    /// Get the currently focused date
    pub fn focused_date(&self) -> Option<Date> {
        self.focused_date.cloned()
    }

    /// Set the focused date
    pub fn set_focused_date(&mut self, date: Option<Date>) {
        self.focused_date.set(date);
    }

    /// Get the current view date
    pub fn view_date(&self) -> Date {
        self.view_date.cloned()
    }

    /// Set the view date
    pub fn set_view_date(&self, date: Date) {
        (self.set_view_date)(self.enabled_date_range.clamp(date));
    }

    /// Whether the day grid renders a week-number column (`show_week_number`).
    pub fn show_week_number(&self) -> bool {
        self.show_week_number
    }

    /// Check if the calendar is disabled
    pub fn is_disabled(&self) -> bool {
        self.disabled.cloned()
    }

    /// Check if the selected date is unavailable
    pub fn is_unavailable(&self, date: Date) -> bool {
        !self.available_ranges.read().valid_interval(date)
    }

    /// Check if a date is focused
    pub fn is_focused(&self, date: Date) -> bool {
        self.focused_date().is_some_and(|d| d == date)
    }

    /// Return available date range by given date
    pub fn available_range(&self) -> Option<DateRange> {
        try_consume_context::<RangeCalendarContext>().and_then(|ctx| {
            ctx.anchor_date.cloned().and_then(|date| {
                self.available_ranges
                    .read()
                    .available_range(date, self.enabled_date_range)
            })
        })
    }

    pub(super) fn visible_month_count(&self) -> u8 {
        self.view_registrations
            .read()
            .iter()
            .enumerate()
            .map(|(index, view)| {
                view.offset
                    .unwrap_or_else(|| u8::try_from(index).unwrap_or(u8::MAX))
                    .saturating_add(1)
            })
            .max()
            .unwrap_or(1)
    }

    pub(super) fn calendar_view_offset(&self, id: ScopeId, offset: Option<u8>) -> u8 {
        offset.unwrap_or_else(|| {
            self.view_registrations
                .read()
                .iter()
                .position(|view| view.id == id)
                .and_then(|index| u8::try_from(index).ok())
                .unwrap_or_default()
        })
    }

    pub(super) fn register_calendar_view(&self, id: ScopeId, offset: Option<u8>) {
        if self
            .view_registrations
            .read()
            .iter()
            .any(|view| view.id == id && view.offset == offset)
        {
            return;
        }

        let mut view_registrations_signal = self.view_registrations;
        let mut view_registrations = view_registrations_signal.write();
        if let Some(view) = view_registrations.iter_mut().find(|view| view.id == id) {
            view.offset = offset;
        } else {
            view_registrations.push(CalendarViewRegistration { id, offset });
        }
    }

    pub(super) fn unregister_calendar_view(&self, id: ScopeId) {
        if !self
            .view_registrations
            .read()
            .iter()
            .any(|view| view.id == id)
        {
            return;
        }

        let mut view_registrations = self.view_registrations;
        view_registrations.write().retain(|view| view.id != id);
    }
}

/// The context provided by the [`Calendar`] component to its children.
#[derive(Copy, Clone)]
pub struct CalendarContext {
    pub(super) selected_date: ReadSignal<Option<Date>>,
    pub(super) set_selected_date: Callback<Option<Date>>,
}

impl CalendarContext {
    /// Get the currently selected date
    pub fn selected_date(&self) -> Option<Date> {
        self.selected_date.cloned()
    }

    /// Set the selected date
    pub fn set_selected_date(&self, date: Option<Date>) {
        (self.set_selected_date)(date);
    }
}

/// Today's local date, read so that a prerendered page and the hydrating client agree on it.
///
/// The first render (the server's, and the client's hydration render) uses the date the server
/// rendered with, carried to the client through `use_server_cached`, so hydration finds the markup
/// it expects. An effect then reads the visitor's clock once the component is mounted. A page
/// prerendered at build time therefore shows the build day only until hydration, then today
/// (backlog row 380). A component mounted after hydration, a pure client build and plain SSR read
/// the clock directly on their first render.
///
/// This is the one place a component reads the clock; `scripts/check-render-clock.sh` rejects a
/// clock read anywhere else in a component file.
pub fn use_today() -> ReadSignal<Date> {
    #[allow(
        unused_mut,
        unused_assignments,
        reason = "only the fullstack build assigns it, through `fullstack!`"
    )]
    let mut rendered: Option<i32> = None;
    fullstack! {
        // A Julian day number, because `Date` has no serde impl without `time`'s `serde` feature.
        rendered = Some(use_server_cached(|| {
            clock_local_date().to_julian_day()
        }));
    }
    let mut today = use_signal(move || {
        rendered
            .and_then(|day| Date::from_julian_day(day).ok())
            .unwrap_or_else(clock_local_date)
    });
    use_effect(move || {
        let now = clock_local_date();
        if *today.peek() != now {
            today.set(now);
        }
    });
    today.into()
}

/// The visitor's local date, or the UTC date where the local offset is unknown. Called only by
/// [`use_today`], on the first render where no server render came before it and after mount.
fn clock_local_date() -> Date {
    OffsetDateTime::now_local() // render-clock: allowed
        .map(|now| now.date())
        .unwrap_or_else(|_| time::UtcDateTime::now().date()) // render-clock: allowed
}

/// The `today` and `view_date` a calendar root puts in its context: the caller's values, or the
/// clock read after mount ([`use_today`]) and that day's month.
pub(super) fn use_today_and_view(
    today: Option<Date>,
    view_date: ReadSignal<Option<Date>>,
) -> (ReadSignal<Date>, ReadSignal<Date>) {
    let clock = use_today();
    let pick_today = use_reactive((&today,), move |(today,)| {
        today.unwrap_or_else(|| clock.cloned())
    });
    let today = use_memo(pick_today);
    let view_date = use_memo(move || view_date().unwrap_or_else(|| today.cloned()));
    (today.into(), view_date.into())
}
