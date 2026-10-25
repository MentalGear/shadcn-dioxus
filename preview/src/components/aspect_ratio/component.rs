use dioxus::prelude::*;
use dioxus_primitives::aspect_ratio::{self, AspectRatioProps};
use dioxus_primitives::{dioxus_attributes::attributes, merge_attributes};

// dev-docs/backlog.md row 32: see checkbox/component.rs's header comment for
// the full asset!()+document::Link rationale.
//
// This wrapper must attach the theme class and link its own stylesheet, not
// forward `props` straight into the raw primitive
// (`dioxus_primitives::aspect_ratio::AspectRatio(props)`): `style.css`
// (`.dx-aspect-ratio-container`/`.dx-aspect-ratio-image`) would otherwise be
// reachable only through the demo-only `variants/main/mod.rs`, which
// `shadcn-dioxus add aspect_ratio` never copies (`component.json`'s
// `exclude` drops `variants`) -- so a real consumer would get the CSS file on
// disk with nothing in the shipped code that ever loads or references it.
// It works the same way `../separator/component.rs` does for a primitive
// whose props carry a bare
// `attributes: Vec<Attribute>` instead of a dedicated `class` field: an
// `attributes!`-built base class merged ahead of the caller's own
// attributes via `merge_attributes`, so a caller-supplied `class` still
// composes rather than overwriting this one.
#[component]
pub fn AspectRatio(props: AspectRatioProps) -> Element {
    let base = attributes!(div {
        class: "dx-aspect-ratio", "data-dx-root": "",
    });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/aspect_ratio/style.css") }
        aspect_ratio::AspectRatio {
            attributes: merged,
            ..props
        }
    }
}
