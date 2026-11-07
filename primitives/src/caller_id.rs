//! The one reader of an `id` the caller passed in `attributes`.
//!
//! A wrapper that renders its own element tags it with a generated id (listeners and reads find the
//! element by id) unless the caller supplied one. "Supplied one" is decided here for every wrapper:
//! a text `id` with no namespace and at least one character. `id=""` names nothing, so it counts as
//! no id and the wrapper's generated id stays in force.

use dioxus::prelude::*;

/// The text of the caller's `id` attribute, if the caller passed a usable one.
pub(crate) fn caller_id(attributes: &[Attribute]) -> Option<String> {
    attributes.iter().find_map(|a| match (&a.value, a.name) {
        (dioxus_core::AttributeValue::Text(id), "id")
            if a.namespace.is_none() && !id.is_empty() =>
        {
            Some(id.clone())
        }
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attr(name: &'static str, namespace: Option<&'static str>, value: &str) -> Attribute {
        Attribute::new(name, value.to_string(), namespace, false)
    }

    #[test]
    fn a_text_id_is_the_callers() {
        assert_eq!(
            caller_id(&[attr("class", None, "x"), attr("id", None, "menu")]),
            Some("menu".to_string())
        );
    }

    #[test]
    fn an_empty_id_is_none() {
        assert_eq!(caller_id(&[attr("id", None, "")]), None);
    }

    #[test]
    fn a_namespaced_id_is_none() {
        assert_eq!(caller_id(&[attr("id", Some("svg"), "menu")]), None);
    }

    #[test]
    fn no_id_is_none() {
        assert_eq!(caller_id(&[]), None);
    }
}
