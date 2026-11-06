use dioxus::prelude::*;
use crate::components::icons::ChevronDown;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::{merge_attributes, use_form_value_report};

/// The size of a [`NativeSelect`]: shadcn's `size="sm" | "default"` (`h-7` / `h-8`, the theme's
/// control-height ladder). It shadows the `<select size>` row-count attribute, as shadcn's does.
#[derive(Copy, Clone, PartialEq, Default)]
#[non_exhaustive]
pub enum NativeSelectSize {
    #[default]
    Default,
    Sm,
}

impl NativeSelectSize {
    pub fn class(&self) -> &'static str {
        match self {
            NativeSelectSize::Default => "default",
            NativeSelectSize::Sm => "sm",
        }
    }
}

/// A styled native `<select>` -- for the common case where the full custom
/// listbox behaviour of [`Select`](crate::components::select::Select)
/// (search, portalled popover, custom option rendering) isn't needed and
/// the platform's own picker (native mobile wheel, OS-native styling) is
/// preferable. Wraps a plain `<select>`, so it comes with the browser's
/// own keyboard/typeahead/form-participation behaviour for free -- there is
/// no APG contract to add on top, and no primitive underneath.
#[component]
pub fn NativeSelect(
    /// The control's height: `Default` (the default) or the shorter `Sm`.
    #[props(default)] size: NativeSelectSize,
    /// Called when the selected option changes.
    onchange: Option<EventHandler<FormEvent>>,
    /// Called on every change of the selection (`input`).
    oninput: Option<EventHandler<FormEvent>>,
    /// Called when the select gains focus.
    onfocus: Option<EventHandler<FocusEvent>>,
    /// Called when the select loses focus.
    onblur: Option<EventHandler<FocusEvent>>,
    #[props(extends = GlobalAttributes)]
    #[props(extends = select)]
    attributes: Vec<Attribute>,
    /// The `<option>`s of the select.
    children: Element,
) -> Element {
    let base = attributes!(select { class: "dx-native-select", "data-dx-root": "", "data-size": size.class() });
    // A form reset or a restored selection changes the DOM value without an `input` event; the handlers hear it
    // as an edit, so an owner that mirrors the select into a signal follows (`primitives/src/form_reset.rs`).
    let owned = use_form_value_report(&attributes, oninput, onchange, None);
    let merged = merge_attributes(vec![base, attributes, owned]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/native_select/style.css") }
        div { class: "dx-native-select-wrapper", "data-size": size.class(),
            select {
                onchange: move |e| _ = onchange.map(|callback| callback(e)),
                oninput: move |e| _ = oninput.map(|callback| callback(e)),
                onfocus: move |e| _ = onfocus.map(|callback| callback(e)),
                onblur: move |e| _ = onblur.map(|callback| callback(e)),
                ..merged,
                {children}
            }
            ChevronDown { class: "dx-native-select-icon" }
        }
    }
}
