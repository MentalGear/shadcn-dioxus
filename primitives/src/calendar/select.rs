use super::*;

/// The props for the [`CalendarSelectMonth`] component.
#[derive(Props, Clone, PartialEq)]
pub struct CalendarSelectMonthProps {
    /// Additional attributes to apply to the month select container element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the month select container element.
    #[props(default)]
    pub children: Element,
}

/// The props for the [`CalendarSelectMonthSelect`] component.
#[derive(Props, Clone, PartialEq)]
pub struct CalendarSelectMonthSelectProps {
    /// Additional attributes to apply to the native month select element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// The props for the [`CalendarSelectMonthOption`] component.
#[derive(Props, Clone, PartialEq)]
pub struct CalendarSelectMonthOptionProps {
    /// The month represented by this option.
    pub month: Month,

    /// Additional attributes to apply to the month option element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the month option element.
    #[props(default)]
    pub children: Option<Element>,
}

/// The props for the [`CalendarSelectMonthValue`] component.
#[derive(Props, Clone, PartialEq)]
pub struct CalendarSelectMonthValueProps {
    /// Additional attributes to apply to the displayed month value element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the displayed month value element.
    #[props(default)]
    pub children: Element,

    /// Print the short month name ("Oct", as the month dropdown's options do) instead of the
    /// full one. Used by the value that sits in a dropdown; the plain-text caption keeps the full
    /// name.
    #[props(default)]
    pub short: bool,
}

/// # CalendarSelectMonth
///
/// The [`CalendarSelectMonth`] component provides a container for the month select controls.
///
/// This must be used inside a [`Calendar`] component.
///
/// ## Example
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::calendar::{
///     Calendar, CalendarGrid, CalendarHeader, CalendarNavigation, CalendarNextMonthButton, CalendarPreviousMonthButton,
///     CalendarSelectMonth, CalendarSelectMonthSelect, CalendarSelectMonthValue
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
///                     CalendarSelectMonth {
///                         CalendarSelectMonthSelect {}
///                         CalendarSelectMonthValue {}
///                     }
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
pub fn CalendarSelectMonth(props: CalendarSelectMonthProps) -> Element {
    rsx! {
        span {
            ..props.attributes,
            {props.children}
        }
    }
}

/// The native select element for choosing the visible month.
#[component]
pub fn CalendarSelectMonthSelect(props: CalendarSelectMonthSelectProps) -> Element {
    let base_ctx: BaseCalendarContext = use_context();
    let view_ctx: CalendarViewContext = use_context();

    let months = use_memo(move || {
        // Get the current view date from context
        let view_date = view_ctx.offset_view_date();
        let min_date = base_ctx.enabled_date_range.start();
        let max_date = base_ctx.enabled_date_range.end();
        let mut min_month = Month::January;
        if replace_month(view_date, min_month) < min_date {
            min_month = min_date.month();
        }
        let mut max_month = Month::December;
        if replace_month(view_date, max_month) > max_date {
            max_month = max_date.month();
        }

        let mut month = min_month;
        let mut months = Vec::new();
        loop {
            months.push(month);

            if month == max_month {
                return months;
            }
            month = month.next();
        }
    });

    let defaults = attributes!(select {
        aria_label: "Month"
    });
    let merged = merge_attributes(vec![defaults, props.attributes.clone()]);

    rsx! {
        select {
            onchange: move |e| {
                let mut view_date = view_ctx.offset_view_date();
                let number = e.value().parse().unwrap_or(view_date.month() as u8);
                let cur_month = Month::try_from(number).expect("Month out-of-range");
                view_date = view_date.replace_month(cur_month).unwrap();
                view_ctx.set_offset_view_date(view_date);
            },
            ..merged,
            for month in months() {
                CalendarSelectMonthOption { key: "{month:?}", month }
            }
        }
    }
}

/// An option in the native month select element.
#[component]
pub fn CalendarSelectMonthOption(props: CalendarSelectMonthOptionProps) -> Element {
    let base_ctx: BaseCalendarContext = use_context();
    let view_ctx: CalendarViewContext = use_context();
    let children = props
        .children
        .unwrap_or_else(|| rsx! { {base_ctx.month_short_name(props.month)} });

    // `value`/`selected` are this option's own functional state, owned.
    let owned = attributes!(option {
        value: props.month as u8,
        selected: view_ctx.offset_view_date().month() == props.month,
    });
    let merged = merge_attributes(vec![props.attributes.clone(), owned]);

    rsx! {
        option {
            ..merged,
            {children}
        }
    }
}

/// The displayed month value.
#[component]
pub fn CalendarSelectMonthValue(props: CalendarSelectMonthValueProps) -> Element {
    let base_ctx: BaseCalendarContext = use_context();
    let view_ctx: CalendarViewContext = use_context();
    let month = view_ctx.offset_view_date().month();

    rsx! {
        span {
            ..props.attributes,
            if props.short {
                {base_ctx.month_short_name(month)}
            } else {
                {base_ctx.format_month.call(month)}
            }
            {props.children}
        }
    }
}

/// The props for the [`CalendarSelectYear`] component.
#[derive(Props, Clone, PartialEq)]
pub struct CalendarSelectYearProps {
    /// Additional attributes to apply to the year select container element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the year select container element.
    #[props(default)]
    pub children: Element,
}

/// The props for the [`CalendarSelectYearSelect`] component.
#[derive(Props, Clone, PartialEq)]
pub struct CalendarSelectYearSelectProps {
    /// Additional attributes to apply to the native year select element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// The props for the [`CalendarSelectYearOption`] component.
#[derive(Props, Clone, PartialEq)]
pub struct CalendarSelectYearOptionProps {
    /// The year represented by this option.
    pub year: i32,

    /// Additional attributes to apply to the year option element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the year option element.
    #[props(default)]
    pub children: Option<Element>,
}

/// The props for the [`CalendarSelectYearValue`] component.
#[derive(Props, Clone, PartialEq)]
pub struct CalendarSelectYearValueProps {
    /// Additional attributes to apply to the displayed year value element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the displayed year value element.
    #[props(default)]
    pub children: Element,
}

/// # CalendarSelectYear
///
/// The [`CalendarSelectYear`] component provides a container for the year select controls.
///
/// This must be used inside a [`Calendar`] component.
///
/// ## Example
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::calendar::{
///     Calendar, CalendarGrid, CalendarHeader, CalendarNavigation, CalendarNextMonthButton, CalendarPreviousMonthButton,
///     CalendarSelectYear, CalendarSelectYearSelect, CalendarSelectYearValue
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
///                     CalendarSelectYear {
///                         CalendarSelectYearSelect {}
///                         CalendarSelectYearValue {}
///                     }
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
pub fn CalendarSelectYear(props: CalendarSelectYearProps) -> Element {
    rsx! {
        span {
            ..props.attributes,
            {props.children}
        }
    }
}

/// The native select element for choosing the visible year.
#[component]
pub fn CalendarSelectYearSelect(props: CalendarSelectYearSelectProps) -> Element {
    let base_ctx: BaseCalendarContext = use_context();
    let view_ctx: CalendarViewContext = use_context();

    let years = use_memo(move || {
        // Get the current view date from context
        let view_date = view_ctx.offset_view_date();
        let min_date = base_ctx.enabled_date_range.start();
        let max_date = base_ctx.enabled_date_range.end();
        let month = view_date.month();
        let mut min_year = min_date.year();
        if replace_month(min_date, month) < min_date {
            min_year += 1;
        }
        let mut max_year = max_date.year();
        if replace_month(max_date, month) > max_date {
            max_year -= 1;
        }

        min_year..=max_year
    });

    let defaults = attributes!(select { aria_label: "Year" });
    let merged = merge_attributes(vec![defaults, props.attributes.clone()]);

    rsx! {
        select {
            onchange: move |e| {
                let mut view_date = view_ctx.offset_view_date();
                let year = e.value().parse().unwrap_or(view_date.year());
                view_date = view_date.replace_year(year).unwrap_or(view_date);
                view_ctx.set_offset_view_date(view_date);
            },
            ..merged,
            for year in years() {
                CalendarSelectYearOption { key: "{year}", year }
            }
        }
    }
}

/// An option in the native year select element.
#[component]
pub fn CalendarSelectYearOption(props: CalendarSelectYearOptionProps) -> Element {
    let view_ctx: CalendarViewContext = use_context();
    let children = props.children.unwrap_or_else(|| {
        let year = props.year;
        rsx! { "{year}" }
    });

    // `value`/`selected` are this option's own functional state, owned.
    let owned = attributes!(option {
        value: props.year,
        selected: view_ctx.offset_view_date().year() == props.year,
    });
    let merged = merge_attributes(vec![props.attributes.clone(), owned]);

    rsx! {
        option {
            ..merged,
            {children}
        }
    }
}

/// The displayed year value.
#[component]
pub fn CalendarSelectYearValue(props: CalendarSelectYearValueProps) -> Element {
    let view_ctx: CalendarViewContext = use_context();
    let year = view_ctx.offset_view_date().year();

    rsx! {
        span {
            ..props.attributes,
            "{year}"
            {props.children}
        }
    }
}
