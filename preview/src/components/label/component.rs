use dioxus::prelude::*;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::label::{self, LabelProps};
use dioxus_primitives::merge_attributes;

#[component]
pub fn Label(props: LabelProps) -> Element {
    // `class: "dx-label"` and `attributes: props.attributes.clone()` must be merged,
    // not passed to the primitive separately: `attributes` extends `GlobalAttributes`, so
    // an ad-hoc `class` here and any `class` already inside `props.attributes.clone()`
    // (e.g. Field's `dx-field-label` override) would land as two separate `class`
    // entries in the final Vec -- rendered as two literal `class="..."`
    // attributes on the same `<label>`, a WHATWG duplicate-attribute parse
    // error (hydration-parity Rule 4 on the field demo). Merge them the same way
    // Table/Combobox do.
    let base = attributes!(label { class: "dx-label", "data-dx-root": "" });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/label/style.css") }
        label::Label {
            attributes: merged,
            ..props
        }
    }
}
