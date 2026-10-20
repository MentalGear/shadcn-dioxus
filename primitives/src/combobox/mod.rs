//! Autocomplete input with a filterable popup list.
//!
//! `ComboboxInput` is the text input and trigger. `ComboboxList` contains
//! `ComboboxOption` children.

mod components;
mod context;
mod multi;

pub use components::{
    Combobox, ComboboxEmpty, ComboboxEmptyProps, ComboboxInput, ComboboxInputProps,
    ComboboxItemIndicator, ComboboxItemIndicatorProps, ComboboxList, ComboboxListProps,
    ComboboxOption, ComboboxOptionProps, ComboboxProps,
};

pub use context::default_combobox_filter;

pub use multi::{
    ComboboxChips, ComboboxChipsProps, ComboboxClear, ComboboxClearProps, ComboboxMulti,
    ComboboxMultiProps,
};
