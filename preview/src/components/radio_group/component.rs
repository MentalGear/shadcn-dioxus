use dioxus::prelude::*;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;
use dioxus_primitives::radio_group::{self, RadioGroupProps, RadioItemProps};

// dev-docs/backlog.md row 32: see checkbox/component.rs's
// header comment for the full delivery-mechanism rationale (asset!() +
// document::Link, embedded in both exported entry points so a
// `shadcn-dioxus add radio_group`-copied component needs no extra wiring).
//
// Its item class is `dx-radio-group-item`, not a bare `dx-radio-item`:
// without a hash suffix, a plain `dx-radio-item` isn't provably this
// component's own -- see `scripts/check-dx-class-prefix.sh`.
#[component]
pub fn RadioGroup(props: RadioGroupProps) -> Element {
    let base = attributes!(div { class: "dx-radio-group", "data-dx-root": "" });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/radio_group/style.css") }
        radio_group::RadioGroup {
            attributes: merged,
            ..props
        }
    }
}

#[component]
pub fn RadioItem(props: RadioItemProps) -> Element {
    // This wrapper forwards `props.id` to the underlying primitive and
    // merges `props.class` instead of clobbering it:
    // `preview/src/components/form/component.rs`'s fixture relies on
    // stable ids (`#plan-lib-pro` etc.) for both its `<label for>`
    // associations and this repo's Playwright oracles to find each radio
    // button. Merges `class` the same way `../popover/component.rs`'s
    // `PopoverContent` does, for the same reason: a caller's own class
    // should extend, not replace, this theme's.
    let class = if let Some(class) = props.class {
        format!("{} {}", "dx-radio-group-item", class)
    } else {
        "dx-radio-group-item".to_string()
    };

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/radio_group/style.css") }
        radio_group::RadioItem {
            class,
            ..props
        }
    }
}
