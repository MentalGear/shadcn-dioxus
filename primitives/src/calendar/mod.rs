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
use time::{ext::NumericalDuration, macros::date, Date, Month, OffsetDateTime, Weekday};

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
