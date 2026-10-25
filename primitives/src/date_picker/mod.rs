//! Defines the [`DatePicker`] and [`DateRangePicker`] components and its subcomponents, which allowing users to enter or select a date value

use crate::{
    calendar::{
        use_today, weekday_abbreviation, AvailableRanges, CalendarProps, DateRange,
        RangeCalendarProps,
    },
    collection::{collection_item, use_collection_provider, use_item, CollectionState},
    dioxus_core::{current_scope_id, Properties, Runtime, Task},
    popover::*,
    use_unique_id,
};

use crate::sleep::sleep;
use dioxus::prelude::*;
use dioxus_attributes::attributes;
use std::{fmt::Display, str::FromStr, time::Duration};
use time::{macros::date, Date, Month, Weekday};

use crate::direction::use_direction;
use crate::merge_attributes;

mod context;
mod date_segments;
mod input;
mod picker;
mod popover;
mod segment;

pub use context::*;
pub use date_segments::*;
pub use input::*;
pub use picker::*;
pub use popover::*;
pub use segment::*;

#[cfg(test)]
mod tests;
