use super::*;
use dioxus_core::AttributeValue::Text;

fn plain_style(value: &str) -> Attribute {
    Attribute {
        name: "style",
        namespace: None,
        volatile: false,
        value: Text(value.to_string()),
    }
}

fn other(name: &'static str, value: &str) -> Attribute {
    Attribute {
        name,
        namespace: None,
        volatile: false,
        value: Text(value.to_string()),
    }
}

#[test]
fn native_anchor_is_empty_so_no_caller_style_yields_no_style_attribute_at_all() {
    // Off the web arm, `anchor_name_style` is the inert `String::new()`
    // stub -- with no caller style either, there is nothing to say, so
    // this must not manufacture a stray empty `style=""`.
    let result = anchored_trigger_attributes("x", vec![other("role", "button")]);
    assert!(!result.iter().any(|a| a.name == "style"));
    assert!(result.iter().any(|a| a.name == "role"));
}

#[test]
fn native_anchor_is_empty_so_caller_style_passes_through_unchanged() {
    // No anchor binding to fold in on this arm -- the caller's own
    // `style` should come through exactly as given, not prefixed with
    // an empty anchor segment (no stray leading space either).
    let result = anchored_trigger_attributes("x", vec![plain_style("color: red;")]);
    let style = result.iter().find(|a| a.name == "style").expect("style");
    assert!(matches!(&style.value, Text(s) if s == "color: red;"));
}
