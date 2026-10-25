use super::*;

/// The props for the [`CalendarView`] component.
#[derive(Props, Clone, PartialEq)]
pub struct CalendarViewProps {
    /// An offset from the beginning of the view date that this should display
    #[props(default)]
    pub offset: Option<u8>,

    /// Additional attributes to apply to the view element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the calendar element
    pub children: Element,
}

#[derive(Copy, Clone, PartialEq)]
pub(super) struct CalendarViewContext {
    pub(super) offset: u8,
}

impl CalendarViewContext {
    pub(super) fn offset_view_date(&self) -> Date {
        let base_ctx: BaseCalendarContext = consume_context();
        let view_date = (base_ctx.view_date)();

        nth_month_next(view_date, self.offset).unwrap_or(view_date)
    }

    pub(super) fn set_offset_view_date(&self, date: Date) {
        let base_ctx: BaseCalendarContext = consume_context();
        let view_date = base_ctx.view_date();
        // The date is currently relative to the offset, so we need to adjust it back
        let date = nth_month_previous(date, self.offset).unwrap_or(view_date);
        base_ctx.set_view_date(date);
    }
}

/// A calendar view for one visible month.
///
/// Render one [`CalendarView`] for each month you want visible. The calendar derives the
/// visible month count from the registered views and uses each view's render order as
/// its month offset unless `offset` is provided.
#[component]
pub fn CalendarView(props: CalendarViewProps) -> Element {
    let base_ctx: BaseCalendarContext = use_context();
    let view_id = current_scope_id();

    use_hook(move || {
        base_ctx.register_calendar_view(view_id, props.offset);
    });

    use_effect(move || {
        base_ctx.register_calendar_view(view_id, props.offset);
    });

    use_effect_cleanup(move || {
        base_ctx.unregister_calendar_view(view_id);
    });

    let offset = base_ctx.calendar_view_offset(view_id, props.offset);

    use_context_provider(|| CalendarViewContext { offset });

    rsx! {
        div { ..props.attributes,
            {props.children}
        }
    }
}

/// The props for the [`CalendarHeader`] component.
#[derive(Props, Clone, PartialEq)]
pub struct CalendarHeaderProps {
    /// Optional ID for the header
    #[props(default)]
    pub id: Option<String>,

    /// Additional attributes to extend the header element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the header element
    pub children: Element,
}

/// # CalendarHeader
///
/// The [`CalendarHeader`] component displays the header for the calendar. It typically contains the [`CalendarNavigation`] component
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
#[component]
pub fn CalendarHeader(props: CalendarHeaderProps) -> Element {
    // `role`/`aria-level` are owned (semantics this element must keep);
    // merge with the caller's attributes instead of leaving both a literal
    // and a caller override on the same element (backlog row 93).
    let owned = attributes!(div {
        role: "heading",
        "aria-level": "2",
    });
    let merged = merge_attributes(vec![props.attributes.clone(), owned]);

    rsx! {
        div {
            id: props.id,
            ..merged,

            {props.children}
        }
    }
}

/// The props for the [`CalendarNavigation`] component.
#[derive(Props, Clone, PartialEq)]
pub struct CalendarNavigationProps {
    /// Optional ID for the navigation
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the navigation element
    #[props(default)]
    pub children: Element,
}

/// # CalendarNavigation
///
/// The [`CalendarNavigation`] component provides a container for navigation buttons in the calendar header.
/// It typically contains the [`CalendarPreviousMonthButton`], [`CalendarNextMonthButton`], and [`CalendarMonthTitle`] components.
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
#[component]
pub fn CalendarNavigation(props: CalendarNavigationProps) -> Element {
    rsx! {
        div { ..props.attributes,
            {props.children}
        }
    }
}

/// The props for the [`CalendarPreviousMonthButton`] component.
#[derive(Props, Clone, PartialEq)]
pub struct CalendarPreviousMonthButtonProps {
    /// Additional attributes to apply to the button
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the button element
    pub children: Element,
}

/// # CalendarPreviousMonthButton
///
/// The [`CalendarPreviousMonthButton`] component provides a button to navigate to the previous month in the calendar.
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
#[component]
pub fn CalendarPreviousMonthButton(props: CalendarPreviousMonthButtonProps) -> Element {
    let ctx: BaseCalendarContext = use_context();
    let view_ctx: CalendarViewContext = use_context();

    // disable previous button when we reach the limit
    let button_disabled = use_memo(move || {
        // Get the current view date from context
        let view_date = view_ctx.offset_view_date();
        match previous_month(view_date) {
            Some(date) => {
                ctx.enabled_date_range
                    .start
                    .replace_day(1)
                    .expect("every month has a day 1")
                    > date
            }
            None => true,
        }
    });
    // disable previous button when the current selection range does not include the previous month
    let navigate_disabled = use_memo(move || {
        // Get the current view date from context
        let view_date = view_ctx.offset_view_date();
        ctx.available_range()
            .is_some_and(|range| range.start.month() == view_date.month())
    });

    // Handle navigation to previous month
    let handle_prev_month = move |e: Event<MouseData>| {
        e.prevent_default();
        let current_view = (ctx.view_date)();
        if let Some(date) = previous_month(current_view) {
            ctx.set_view_date.call(date)
        }
    };

    // `aria_label`/`type` are overridable defaults; `disabled` reflects this
    // component's own limit/range logic and is owned. Merge with the
    // caller's attributes instead of leaving both a literal and a caller
    // override on the same element (backlog row 93).
    let defaults = attributes!(button {
        aria_label: "Previous month",
        r#type: "button",
    });
    let owned = attributes!(button {
        disabled: (ctx.disabled)() || button_disabled() || navigate_disabled(),
    });
    let merged = merge_attributes(vec![defaults, props.attributes.clone(), owned]);

    rsx! {
        button {
            onclick: handle_prev_month,
            ..merged,

            {props.children}
        }
    }
}

/// The props for the [`CalendarNextMonthButton`] component.
#[derive(Props, Clone, PartialEq)]
pub struct CalendarNextMonthButtonProps {
    /// Additional attributes to apply to the button
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the button element
    pub children: Element,
}

/// # CalendarNextMonthButton
///
/// The [`CalendarNextMonthButton`] component provides a button to navigate to the next month in the calendar.
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
#[component]
pub fn CalendarNextMonthButton(props: CalendarNextMonthButtonProps) -> Element {
    let ctx: BaseCalendarContext = use_context();
    let view_ctx: CalendarViewContext = use_context();

    // disable next button when we reach the limit
    let button_disabled = use_memo(move || {
        // Get the current view date from context
        let view_date = view_ctx.offset_view_date();
        match next_month(view_date) {
            Some(date) => {
                let max = ctx.enabled_date_range.end();
                let last_day = max.month().length(max.year());
                max.replace_day(last_day)
                    .expect("`last_day` is the length of `max`'s month")
                    < date
            }
            None => true,
        }
    });
    // disable next button when the current selection range does not include the next month
    let navigate_disabled = use_memo(move || {
        // Get the current view date from context
        let view_date = view_ctx.offset_view_date();
        ctx.available_range()
            .is_some_and(|range| range.end.month() == view_date.month())
    });

    // Handle navigation to next month
    let handle_next_month = move |e: Event<MouseData>| {
        e.prevent_default();
        let current_view = (ctx.view_date)();
        if let Some(date) = next_month(current_view) {
            ctx.set_view_date.call(date)
        }
    };

    // Same reasoning as `CalendarPreviousMonthButton` above.
    let defaults = attributes!(button {
        aria_label: "Next month",
        r#type: "button",
    });
    let owned = attributes!(button {
        disabled: (ctx.disabled)() || button_disabled() || navigate_disabled(),
    });
    let merged = merge_attributes(vec![defaults, props.attributes.clone(), owned]);

    rsx! {
        button {
            onclick: handle_next_month,
            ..merged,

            {props.children}
        }
    }
}

/// The props for the [`CalendarMonthTitle`] component.
#[derive(Props, Clone, PartialEq)]
pub struct CalendarMonthTitleProps {
    /// Additional attributes to apply to the title element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// # CalendarMonthTitle
///
/// The [`CalendarMonthTitle`] component displays the title of the current month in the calendar. It will contain
/// the month and year information as text in the children.
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
#[component]
pub fn CalendarMonthTitle(props: CalendarMonthTitleProps) -> Element {
    let base_ctx: BaseCalendarContext = use_context();
    let view_ctx: CalendarViewContext = use_context();
    // Format the current month and year (the month through `on_format_month`, so a localised
    // calendar prints a localised title)
    let month_year = use_memo(move || {
        let view_date = view_ctx.offset_view_date();
        format!(
            "{} {}",
            base_ctx.format_month.call(view_date.month()),
            view_date.year()
        )
    });

    rsx! {
        div {
            ..props.attributes,

            {month_year}
        }
    }
}
