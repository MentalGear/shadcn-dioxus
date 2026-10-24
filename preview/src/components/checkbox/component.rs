use dioxus::prelude::*;
use dioxus_icons::lucide::{Check, Minus};
use dioxus_primitives::checkbox::{self, CheckboxProps};
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;

// dev-docs/backlog.md row 32: this stylesheet is
// plain, unhashed `dx-`-prefixed CSS (collision safety comes from
// scripts/check-dx-class-prefix.sh instead of a hash), delivered the same
// way `main.rs`'s `GlobalHead` delivers its sheets: an
// `asset!()`-bundled path linked via `document::Link`. Unlike that
// call site this one lives *inside* the themed wrapper itself so a
// `shadcn-dioxus add checkbox`-copied component keeps working with zero
// extra wiring in the consumer's own app -- `asset!()` resolves this path
// from the crate root (manganis-macro's `resolve_path`), and `shadcn-dioxus add`
// preserves this file's `src/components/checkbox/` location verbatim in
// the consumer project (dioxus-cli's `components_root` defaults to
// `<crate>/src/components`), so the path keeps resolving post-copy.
// `document::Link` dedupes by `(href, rel)` (dioxus-document's own doc
// comment on `LinkProps::href`), so re-rendering `Checkbox` many times on
// one page inserts the stylesheet exactly once.
#[component]
pub fn Checkbox(props: CheckboxProps) -> Element {
    let base = attributes!(input { class: "dx-checkbox", "data-dx-root": "" });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/checkbox/style.css") }
        checkbox::Checkbox {
            checked: props.checked,
            default_checked: props.default_checked,
            required: props.required,
            disabled: props.disabled,
            name: props.name,
            form: props.form,
            value: props.value,
            on_checked_change: props.on_checked_change,
            attributes: merged,
            checkbox::CheckboxIndicator { class: "dx-checkbox-indicator",
                // The primitive mounts these for checked AND indeterminate; CSS shows the
                // glyph that matches the indicator's `data-state` (never both).
                Check { class: "dx-checkbox-glyph-check", size: "1rem" }
                Minus { class: "dx-checkbox-glyph-minus", size: "1rem" }
            }
        }
    }
}
