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

fn shorthand_style(prop: &'static str, value: &str) -> Attribute {
    Attribute {
        name: prop,
        namespace: Some("style"),
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

/// Every style attribute in the returned list, as `(name, value)`
/// pairs -- lets the assertions below check there is exactly one no
/// matter which merge order `merge_attributes` happens to produce.
fn style_attrs(attrs: &[Attribute]) -> Vec<(&'static str, String)> {
    attrs
        .iter()
        .filter(|a| a.name == "style")
        .map(|a| match &a.value {
            Text(s) => (a.name, s.clone()),
            _ => (a.name, String::new()),
        })
        .collect()
}

#[test]
fn no_caller_style_yields_just_the_anchor_binding() {
    let result =
        anchored_content_attributes(AnchorKind::Tooltip, "x", vec![other("role", "tooltip")]);
    let styles = style_attrs(&result);
    assert_eq!(
        styles.len(),
        1,
        "expected exactly one style attribute: {result:?}"
    );
    assert_eq!(styles[0].1, "position-anchor: --dxa-x;");
    // Non-style attributes pass through untouched.
    assert!(result.iter().any(|a| a.name == "role"));
}

/// The values of every `data-dx-anchored` attribute in the list.
fn anchored_attrs(attrs: &[Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter(|a| a.name == "data-dx-anchored")
        .filter_map(|a| match &a.value {
            Text(s) => Some(s.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn the_kind_is_written_as_the_one_engine_attribute() {
    let result = anchored_content_attributes(AnchorKind::HoverCard, "x", Vec::new());
    assert_eq!(anchored_attrs(&result), ["hover-card"]);
}

#[test]
fn a_caller_cannot_unscope_or_retarget_the_engine_attribute() {
    let result = anchored_content_attributes(
        AnchorKind::Popover,
        "x",
        vec![other("data-dx-anchored", "tooltip")],
    );
    assert_eq!(anchored_attrs(&result), ["popover"]);
}

#[test]
fn a_self_placed_select_list_carries_no_engine_attribute() {
    let result = anchored_content_attributes(AnchorKind::SelectItemAligned, "x", Vec::new());
    assert!(anchored_attrs(&result).is_empty());
    // It still binds its anchor, only the engine's `anchor()` rules are withheld.
    assert_eq!(style_attrs(&result)[0].1, "position-anchor: --dxa-x;");
}

#[test]
fn plain_caller_style_is_folded_after_the_anchor_binding() {
    let result = anchored_content_attributes(
        AnchorKind::Tooltip,
        "x",
        vec![plain_style("min-height: 100px;")],
    );
    let styles = style_attrs(&result);
    assert_eq!(
        styles.len(),
        1,
        "expected exactly one style attribute: {result:?}"
    );
    let style = &styles[0].1;
    assert!(style.contains("position-anchor: --dxa-x;"));
    assert!(style.contains("min-height: 100px;"));
    // Anchor binding first, so the caller's own declaration -- listed
    // after -- can override any property it names without ever being
    // able to drop the anchor binding itself.
    assert!(style.find("position-anchor").unwrap() < style.find("min-height").unwrap());
}

#[test]
fn shorthand_caller_style_is_folded_after_the_anchor_binding() {
    let result = anchored_content_attributes(
        AnchorKind::Tooltip,
        "x",
        vec![shorthand_style("padding", "1rem")],
    );
    let styles = style_attrs(&result);
    assert_eq!(
        styles.len(),
        1,
        "expected exactly one style attribute: {result:?}"
    );
    let style = &styles[0].1;
    assert!(style.contains("position-anchor: --dxa-x;"));
    assert!(style.contains("padding:1rem;"));
    assert!(style.find("position-anchor").unwrap() < style.find("padding").unwrap());
}

#[test]
fn shorthand_and_plain_caller_style_both_fold_into_the_one_attribute() {
    // The shape that breaks without folding: a caller mixing a plain
    // `style` literal with shorthand style props (or a themed wrapper
    // contributing one of each), on top of this component's own
    // `position-anchor` literal -- three style-contributing values that
    // must all end up in the one served `style="..."`.
    let result = anchored_content_attributes(
        AnchorKind::Tooltip,
        "x",
        vec![
            plain_style("min-height: 100px;"),
            shorthand_style("padding", "1rem"),
            other("role", "tooltip"),
        ],
    );
    let styles = style_attrs(&result);
    assert_eq!(
        styles.len(),
        1,
        "expected exactly one style attribute: {result:?}"
    );
    let style = &styles[0].1;
    assert!(style.contains("position-anchor: --dxa-x;"));
    assert!(style.contains("min-height: 100px;"));
    assert!(style.contains("padding:1rem;"));
    assert!(style.find("position-anchor").unwrap() < style.find("min-height").unwrap());
    assert!(result.iter().any(|a| a.name == "role"));
}

// -----------------------------------------------------------------
// `anchored_trigger_attributes` -- the trigger-side mirror of the
// `anchored_content_attributes` tests above. Same four shapes (no
// caller style / plain / shorthand / both), asserted the same way,
// just against `anchor-name` instead of `position-anchor`. The web
// arm always yields a non-empty `anchor_name_style`, so these only
// exercise the `(false, ..)` arms of its `match`; the empty-anchor
// (native) arms are covered by `trigger_attributes_native_tests`
// below, which is not `feature = "web"`-gated.
// -----------------------------------------------------------------

#[test]
fn trigger_no_caller_style_yields_just_the_anchor_binding() {
    let result = anchored_trigger_attributes("x", vec![other("role", "button")]);
    let styles = style_attrs(&result);
    assert_eq!(
        styles.len(),
        1,
        "expected exactly one style attribute: {result:?}"
    );
    assert_eq!(styles[0].1, "anchor-name: --dxa-x;");
    assert!(result.iter().any(|a| a.name == "role"));
}

#[test]
fn trigger_plain_caller_style_is_folded_after_the_anchor_binding() {
    let result = anchored_trigger_attributes("x", vec![plain_style("color: red;")]);
    let styles = style_attrs(&result);
    assert_eq!(
        styles.len(),
        1,
        "expected exactly one style attribute: {result:?}"
    );
    let style = &styles[0].1;
    assert!(style.contains("anchor-name: --dxa-x;"));
    assert!(style.contains("color: red;"));
    // Anchor binding first, so the caller's own declaration -- listed
    // after -- can override any property it names without ever being
    // able to drop the anchor binding itself (the silent-loss failure
    // mode of an overwriting merge -- see this function's own
    // doc and `style_attribute_is_folded_not_overwritten` in
    // `primitives/src/lib.rs`).
    assert!(style.find("anchor-name").unwrap() < style.find("color").unwrap());
}

#[test]
fn trigger_shorthand_caller_style_is_folded_after_the_anchor_binding() {
    let result = anchored_trigger_attributes("x", vec![shorthand_style("cursor", "pointer")]);
    let styles = style_attrs(&result);
    assert_eq!(
        styles.len(),
        1,
        "expected exactly one style attribute: {result:?}"
    );
    let style = &styles[0].1;
    assert!(style.contains("anchor-name: --dxa-x;"));
    assert!(style.contains("cursor:pointer;"));
    assert!(style.find("anchor-name").unwrap() < style.find("cursor").unwrap());
}

#[test]
fn trigger_shorthand_and_plain_caller_style_both_fold_into_the_one_attribute() {
    let result = anchored_trigger_attributes(
        "x",
        vec![
            plain_style("color: red;"),
            shorthand_style("cursor", "pointer"),
            other("role", "button"),
        ],
    );
    let styles = style_attrs(&result);
    assert_eq!(
        styles.len(),
        1,
        "expected exactly one style attribute: {result:?}"
    );
    let style = &styles[0].1;
    assert!(style.contains("anchor-name: --dxa-x;"));
    assert!(style.contains("color: red;"));
    assert!(style.contains("cursor:pointer;"));
    assert!(style.find("anchor-name").unwrap() < style.find("color").unwrap());
    assert!(result.iter().any(|a| a.name == "role"));
}

/// A caller-supplied `id` (or any other non-style attribute) passes
/// through untouched, deduped against nothing since this function only
/// ever contributes `style` -- guards against a future edit accidentally
/// routing more than style through the `dioxus_attributes::attributes!`
/// base list this function builds.
#[test]
fn trigger_non_style_attributes_pass_through_untouched() {
    let result =
        anchored_trigger_attributes("x", vec![other("id", "caller-id"), other("role", "button")]);
    assert!(result
        .iter()
        .any(|a| a.name == "id" && matches!(&a.value, Text(s) if s == "caller-id")));
    assert!(result.iter().any(|a| a.name == "role"));
}
