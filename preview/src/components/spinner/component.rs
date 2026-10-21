use dioxus::prelude::*;
use dioxus_icons::lucide::LoaderCircle;
use dioxus_primitives::activity::use_motion;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;

/// The ring's size, shadcn's `size-*` utilities as a type: 12, 16 (default), 24 or 32 px.
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
#[non_exhaustive]
pub enum SpinnerSize {
    Sm,
    #[default]
    Default,
    Lg,
    Xl,
}

impl SpinnerSize {
    /// The `data-size` value the stylesheet keys on.
    pub fn class(&self) -> &'static str {
        match self {
            SpinnerSize::Sm => "sm",
            SpinnerSize::Default => "default",
            SpinnerSize::Lg => "lg",
            SpinnerSize::Xl => "xl",
        }
    }
}

/// A loading indicator: a continuously rotating ring icon. Renders
/// `role="status"` with an accessible name (default `"Loading"`, overridable
/// via `label`) so assistive tech announces the busy state -- there is no
/// ARIA widget pattern beyond that live-region role, so no primitive
/// underneath.
#[component]
pub fn Spinner(
    /// The accessible name announced for the loading state.
    #[props(default = "Loading".to_string())]
    label: String,
    /// The ring's size. Defaults to 16px.
    #[props(default)]
    size: SpinnerSize,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let base = attributes!(span { class: "dx-spinner", "data-dx-root": "", "aria-label": "{label}" });
    // `role="status"` is required live-region semantics, not a caller
    // default -- owned-wins.
    let owned = attributes!(span { role: "status", "data-size": size.class() });
    // Paused (not stopped) while off-screen or in a hidden tab: `data-dx-motion="paused"` is turned
    // into `animation-play-state: paused` by `dx-components-theme.css`, so the ring keeps its
    // phase and costs nothing while nobody can see it.
    let motion = use_motion();
    let merged = merge_attributes(vec![base, attributes, owned, motion.attributes()]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/spinner/style.css") }
        span { ..merged, LoaderCircle {} }
    }
}
