//! The one reader of an `id` the caller passed in `attributes`.
//!
//! A wrapper that renders its own element tags it with a generated id (listeners and reads find the
//! element by id) unless the caller supplied one. "Supplied one" is decided here for every wrapper:
//! a text (or number) `id` with no namespace and at least one character. `id=""` names nothing, so it counts as
//! no id and the wrapper's generated id stays in force.

use dioxus::prelude::*;

/// The caller's `id` attribute as text (a number or bool is written out), if the caller passed a
/// usable one.
pub(crate) fn caller_id(attributes: &[Attribute]) -> Option<String> {
    use dioxus_core::AttributeValue as V;
    attributes.iter().find_map(|a| {
        if a.name != "id" || a.namespace.is_some() {
            return None;
        }
        let id = match &a.value {
            V::Text(id) => id.clone(),
            V::Float(id) => id.to_string(),
            V::Int(id) => id.to_string(),
            V::Bool(id) => id.to_string(),
            _ => return None,
        };
        (!id.is_empty()).then_some(id)
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
    fn a_number_is_written_out() {
        let id = Attribute::new("id", 5_i64, None, false);
        assert_eq!(caller_id(&[id]), Some("5".to_string()));
    }

    #[test]
    fn no_id_is_none() {
        assert_eq!(caller_id(&[]), None);
    }
}
