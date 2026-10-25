//! Defines the [`Calendar`] component and its sub-components, which provide a calendar interface with date selection and navigation.

use dioxus::{
    core::{current_scope_id, ScopeId},
    prelude::*,
};
use std::{
    collections::HashSet,
    fmt::{self, Display},
    rc::Rc,
};

use dioxus_attributes::attributes;
use dioxus_core::AttributeValue::{Bool, Text};
use time::{ext::NumericalDuration, Date, Month, OffsetDateTime, Weekday};

/// A calendar date written as a `const`, so the default bounds below need no `time` macro (`time`'s `macros` feature
/// pulls in `time-macros`; backlog row 371).
const fn const_date(year: i32, month: Month, day: u8) -> Date {
    match Date::from_calendar_date(year, month, day) {
        Ok(date) => date,
        Err(_) => panic!("const_date: not a valid calendar date"),
    }
}

/// Default lower bound of the selectable dates of `Calendar`, `RangeCalendar` and `DatePicker`.
pub(crate) const DEFAULT_MIN_DATE: Date = const_date(1925, Month::January, 1);
/// Default upper bound of the selectable dates of `Calendar`, `RangeCalendar` and `DatePicker`.
pub(crate) const DEFAULT_MAX_DATE: Date = const_date(2050, Month::December, 31);

use crate::{
    date_picker::DefaultCalendarProps,
    direction::{use_direction, use_direction_signal, Direction, HorizontalNav},
    merge_attributes, use_effect_cleanup,
};

mod context;
mod dates;
mod day;
mod grid;
mod range;
mod select;
mod single;
#[cfg(test)]
mod tests;
mod view;

pub use context::*;
pub use dates::*;
pub use day::*;
pub use grid::*;
pub use range::*;
pub use select::*;
pub use single::*;
pub use view::*;
