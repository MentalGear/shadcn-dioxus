use dioxus::prelude::*;
use dioxus_icons::lucide::ChevronDown;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;

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
    #[props(default)] size: NativeSelectSize,
    onchange: Option<EventHandler<FormEvent>>,
    oninput: Option<EventHandler<FormEvent>>,
    onfocus: Option<EventHandler<FocusEvent>>,
    onblur: Option<EventHandler<FocusEvent>>,
    #[props(extends = GlobalAttributes)]
    #[props(extends = select)]
    attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(select { class: "dx-native-select", "data-dx-root": "", "data-size": size.class() });
    let merged = merge_attributes(vec![base, attributes]);
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
