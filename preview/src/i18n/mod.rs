//! The locale demo shell of the two `internationalized` demos (Calendar, Date Picker).
//!
//! The site itself is English only, so there is no site-wide locale: each of these demos owns a
//! small i18n root (`use_locale_root`) (`dioxus-i18n`, the `*.ftl` files next to this one) and a locale picker built
//! from the library's own `ToggleGroup`, which keeps the formatting hooks the demos exist for
//! (`on_format_*`, fed by `tid!`) demonstrated. `<html lang>` stays `en`; the demo subtree carries
//! the active locale as its own `lang` (WCAG 3.1.2), so a French calendar is not read with an
//! English voice.

use crate::components::toggle::{ToggleSize, ToggleVariant};
use crate::components::toggle_group::{ToggleGroup, ToggleItem};
use dioxus::prelude::*;
use dioxus_i18n::prelude::{i18n, use_init_i18n, I18nConfig};
use std::collections::HashSet;
use unic_langid::{langid, LanguageIdentifier};

/// The locales a demo can switch between: tag, the name in that language.
const LOCALES: [(LanguageIdentifier, &str); 4] = [
    (langid!("en-US"), "English"),
    (langid!("fr-FR"), "Français"),
    (langid!("es-ES"), "Español"),
    (langid!("de-DE"), "Deutsch"),
];

/// The demo's own i18n root (en-US to start). Call it in the demo's `Demo` component itself, not in
/// [`LocaleDemo`]: a context is found through the scope tree, and the `tid!` callbacks a demo passes
/// are created in `Demo`'s scope, so the provider must be `Demo` or above it.
pub fn use_locale_root() {
    use_init_i18n(|| {
        I18nConfig::new(langid!("en-US"))
            .with_locale((langid!("en-US"), include_str!("en-US.ftl")))
            .with_locale((langid!("fr-FR"), include_str!("fr-FR.ftl")))
            .with_locale((langid!("es-ES"), include_str!("es-ES.ftl")))
            .with_locale((langid!("de-DE"), include_str!("de-DE.ftl")))
    });
}

/// A locale picker above `children`, which must sit under a [`use_locale_root`] caller.
#[component]
pub fn LocaleDemo(children: Element) -> Element {
    let mut i18n = i18n();
    let mut selected = use_signal(|| 0usize);
    let pressed = use_memo(move || Some(HashSet::from([selected()])));
    let tag = LOCALES[selected()].0.to_string();

    rsx! {
        div { class: "dx-locale-demo", style: "display: flex; flex-direction: column; gap: 1rem; align-items: flex-start;",
            ToggleGroup {
                aria_label: "Locale",
                variant: ToggleVariant::Outline,
                size: ToggleSize::Sm,
                pressed,
                on_pressed_change: move |next: HashSet<usize>| {
                    // Single-select: pressing the pressed item again keeps it chosen.
                    if let Some(index) = next.into_iter().next() {
                        selected.set(index);
                        i18n.set_language(LOCALES[index].0.clone());
                    }
                },
                for (index , (id , name)) in LOCALES.iter().enumerate() {
                    ToggleItem { key: "{id}", index, lang: "{id}", "{name}" }
                }
            }
            div { lang: "{tag}", {children} }
        }
    }
}
