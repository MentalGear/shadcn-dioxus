//! The locale demo shell of the two `internationalized` demos (Calendar, Date Picker).
//!
//! The site itself is English only, so there is no site-wide locale: each of these demos owns a
//! small i18n root (`use_locale_root`, a locale signal in context) and a locale picker built from the
//! library's own `ToggleGroup`, which keeps the formatting hooks the demos exist for (`on_format_*`, fed by
//! [`tid`]) demonstrated. The messages are the `*.ftl` files next to this one, read as flat `key = value`
//! lines (no placeholders, no plurals: the demos need neither); a key missing from the active locale falls
//! back to `en-US`, then to the key itself. `<html lang>` stays `en`; the demo subtree carries
//! the active locale as its own `lang` (WCAG 3.1.2), so a French calendar is not read with an
//! English voice.

use crate::components::toggle::{ToggleSize, ToggleVariant};
use crate::components::toggle_group::{ToggleGroup, ToggleItem};
use dioxus::prelude::*;
use std::collections::HashSet;

/// The locale a lookup falls back to.
const FALLBACK: &str = "en-US";

/// The locales a demo can switch between: tag, the name in that language, the message source.
const LOCALES: [(&str, &str, &str); 4] = [
    ("en-US", "English", include_str!("en-US.ftl")),
    ("fr-FR", "Français", include_str!("fr-FR.ftl")),
    ("es-ES", "Español", include_str!("es-ES.ftl")),
    ("de-DE", "Deutsch", include_str!("de-DE.ftl")),
];

/// The value of `key` in one `key = value` message source (`##` lines and blanks are comments).
fn find<'a>(source: &'a str, key: &str) -> Option<&'a str> {
    source
        .lines()
        .filter(|line| !line.starts_with("##"))
        .filter_map(|line| line.split_once('='))
        .find(|(k, _)| k.trim() == key)
        .map(|(_, v)| v.trim())
}

/// The message `key` in locale `tag`: the locale's own text, else the fallback locale's, else `key`.
fn lookup(tag: &str, key: &str) -> String {
    let source = |t: &str| LOCALES.iter().find(|(id, ..)| *id == t).map(|l| l.2);
    source(tag)
        .and_then(|s| find(s, key))
        .or_else(|| source(FALLBACK).and_then(|s| find(s, key)))
        .unwrap_or(key)
        .to_string()
}

/// The demo's own i18n root (en-US to start). Call it in the demo's `Demo` component itself, not in
/// [`LocaleDemo`]: a context is found through the scope tree, and the [`tid`] callbacks a demo passes
/// are created in `Demo`'s scope, so the provider must be `Demo` or above it.
pub fn use_locale_root() {
    use_context_provider(|| Signal::new(FALLBACK));
}

/// The message `key` in the active locale. Reading it subscribes the calling render to a locale switch.
pub fn tid(key: &str) -> String {
    let tag = consume_context::<Signal<&'static str>>();
    lookup(&tag(), key)
}

/// A locale picker above `children`, which must sit under a [`use_locale_root`] caller.
#[component]
pub fn LocaleDemo(children: Element) -> Element {
    let mut locale = consume_context::<Signal<&'static str>>();
    let mut selected = use_signal(|| 0usize);
    let pressed = use_memo(move || Some(HashSet::from([LOCALES[selected()].0.to_string()])));
    let tag = LOCALES[selected()].0.to_string();

    rsx! {
        div { class: "dx-locale-demo", style: "display: flex; flex-direction: column; gap: 1rem; align-items: flex-start;",
            ToggleGroup {
                aria_label: "Locale",
                variant: ToggleVariant::Outline,
                size: ToggleSize::Sm,
                value: pressed,
                on_value_change: move |next: HashSet<String>| {
                    // Single-select: pressing the pressed item again keeps it chosen.
                    if let Some(index) = next
                        .into_iter()
                        .next()
                        .and_then(|id| LOCALES.iter().position(|(l, ..)| *l == id))
                    {
                        selected.set(index);
                        locale.set(LOCALES[index].0);
                    }
                },
                for (index , (id , name , _)) in LOCALES.iter().enumerate() {
                    ToggleItem { key: "{id}", index, value: id.to_string(), lang: "{id}", "{name}" }
                }
            }
            div { lang: "{tag}", {children} }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn own_locale_wins() {
        assert_eq!(lookup("fr-FR", "January"), "Janvier");
        assert_eq!(lookup("de-DE", "D_Abbr"), "T");
    }

    #[test]
    fn unknown_locale_falls_back_to_default() {
        assert_eq!(lookup("xx-XX", "January"), "January");
    }

    #[test]
    fn key_missing_in_locale_falls_back_then_to_key() {
        assert_eq!(find("A = b\n", "B"), None);
        assert_eq!(lookup("fr-FR", "NoSuchKey"), "NoSuchKey");
    }

    #[test]
    fn comments_and_spacing_are_ignored() {
        assert_eq!(find("## A = x\n\nA  =  y \n", "A"), Some("y"));
    }

    #[test]
    fn every_locale_has_every_default_key() {
        let keys: Vec<&str> = LOCALES[0]
            .2
            .lines()
            .filter(|l| !l.starts_with("##"))
            .filter_map(|l| l.split_once('=').map(|(k, _)| k.trim()))
            .collect();
        for (tag, _, source) in LOCALES {
            for key in &keys {
                assert!(find(source, key).is_some(), "{tag} lacks {key}");
            }
        }
    }
}
