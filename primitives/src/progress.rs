//! Defines the [`Progress`] component and its sub-components.

use crate::state_attrs;
use crate::{merge_attributes, use_id_or, use_unique_id};
use dioxus::prelude::*;
use dioxus_attributes::attributes;

/// What [`ProgressLabel`] and [`ProgressValue`] read from their [`Progress`].
#[derive(Clone, Copy)]
struct ProgressCtx {
    value: ReadSignal<Option<f64>>,
    max: ReadSignal<f64>,
    /// The id of the mounted [`ProgressLabel`], read back by the root's `aria-labelledby`.
    labeled_by: Signal<Option<String>>,
}

/// The props for the [`Progress`] component.
#[derive(Props, Clone, PartialEq)]
pub struct ProgressProps {
    /// The current progress value, between 0 and max.
    pub value: ReadSignal<Option<f64>>,

    /// The maximum value. Defaults to 100.
    #[props(default = ReadSignal::new(Signal::new(100.0)))]
    pub max: ReadSignal<f64>,

    /// Additional attributes to apply to the progress element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the progress component.
    pub children: Element,
}

/// # Progress
///
/// The `Progress` component shows the progress of an operation.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::progress::{Progress, ProgressIndicator};
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         Progress {
///             aria_label: "Progressbar Demo",
///             value: 50.0,
///             ProgressIndicator {}
///         }
///     }
/// }
/// ```
///
/// Put a [`ProgressLabel`] and a [`ProgressValue`] among the children for a visible name and readout; the
/// root is then named by the label (`aria-labelledby`) without any wiring.
///
/// ## Styling
///
/// The [`Progress`] component defines the following data attributes you can use to control styling:
/// - `data-state`: Indicates the state of the progress. Values are `loading` or `indeterminate`.
/// - `data-indeterminate`: Present (empty) while no value is set, absent otherwise.
/// - `data-progressing`: Present (empty) while a value below the maximum is set, absent otherwise.
/// - `data-complete`: Present (empty) once the value reaches the maximum, absent otherwise.
/// - `data-value`: The current progress value between 0 and max.
/// - `data-max`: The maximum progress value.
///
/// The [`Progress`] component defines the following css variables you can use to control styling:
/// - `--progress-value`: A value between 0 and 100 representing the current progress percentage.
#[component]
pub fn Progress(props: ProgressProps) -> Element {
    // Calculate percentage for styling and "data-state"
    let percentage = use_memo(move || {
        props.value.cloned().map(|v| {
            let max = (props.max)();
            (v / max) * 100.0
        })
    });

    let labeled_by = use_signal(|| None::<String>);
    use_context_provider(|| ProgressCtx {
        value: props.value,
        max: props.max,
        labeled_by,
    });

    let state = use_memo(move || match percentage() {
        Some(_) => "loading",
        None => "indeterminate",
    });

    // Only the indeterminate bar loops: it is paused while off-screen or in a hidden tab
    // (`crate::activity`; the theme turns `data-dx-motion="paused"` into
    // `animation-play-state: paused`). A determinate bar subscribes to nothing.
    let motion = crate::activity::use_motion_when(move || state() == "indeterminate");

    // Merged (caller-wins, deduped) rather than set here and then spread
    // over by `..props.attributes` below: doing that emits the SAME
    // attribute name (e.g. a caller `style` override, exercised by the
    // dashboard's own progress bars) twice in the SSR'd HTML, which
    // WHATWG HTML's duplicate-attribute parse error resolves to the
    // *first* value while the CSR/hydrated DOM path (sequential attribute
    // application) resolves to the *last* -- server and client then
    // disagree about which value is in effect
    // (`dev-docs/conformance-harness.md` hydration-parity Rule 4; see
    // `toast.rs`'s `ToastRegionRendered` doc for the same rule).
    let attributes = merge_attributes(vec![
        attributes!(div {
            role: "progressbar",
            "aria-valuemin": 0,
            "aria-valuemax": props.max,
            "aria-valuenow": props.value.cloned(),
            ..state_attrs::progress(props.value.cloned(), (props.max)()),
            "data-value": props.value.cloned().map(|v| v.to_string()),
            "data-max": props.max,
            style: percentage().map(|p| format!("--progress-value: {p}%")),
            // Named by its `ProgressLabel` when it has one; a caller's own `aria-labelledby`/`aria-label` wins
            // (merged after this).
            aria_labelledby: labeled_by,
        }),
        props.attributes,
        motion.attributes(),
    ]);

    rsx! {
        div {
            ..attributes,

            {props.children}
        }
    }
}

/// The props for the [`ProgressIndicator`] component.
#[derive(Props, Clone, PartialEq)]
pub struct ProgressIndicatorProps {
    /// Additional attributes to apply to the indicator element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the indicator component.
    pub children: Element,
}

/// # ProgressIndicator
///
/// The `ProgressIndicator` component represents the visual indicator that shows the progress completion.
///
/// This must be used inside a [`Progress`] component.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::progress::{Progress, ProgressIndicator};
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         Progress {
///             aria_label: "Progressbar Demo",
///             value: 50.0,
///             ProgressIndicator {}
///         }
///     }
/// }
/// ```
#[component]
pub fn ProgressIndicator(props: ProgressIndicatorProps) -> Element {
    rsx! {
        div { ..props.attributes, {props.children} }
    }
}

/// The props for the [`ProgressLabel`] component.
#[derive(Props, Clone, PartialEq)]
pub struct ProgressLabelProps {
    /// Optional id for the label element.
    #[props(default)]
    pub id: ReadSignal<Option<String>>,

    /// Additional attributes to apply to the label element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The label text.
    pub children: Element,
}

/// # ProgressLabel
///
/// The visible name of a [`Progress`] (Base UI's `Progress.Label`). On mount it publishes its id into the
/// enclosing [`Progress`], which points its `aria-labelledby` at it. This must be used inside a [`Progress`].
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::progress::{Progress, ProgressIndicator, ProgressLabel, ProgressValue};
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         Progress {
///             value: 56.0,
///             ProgressLabel { "Upload progress" }
///             ProgressValue {}
///             ProgressIndicator {}
///         }
///     }
/// }
/// ```
#[component]
pub fn ProgressLabel(props: ProgressLabelProps) -> Element {
    let mut ctx: ProgressCtx = use_context();
    let id = use_id_or(use_unique_id(), props.id);

    // Same shape as `command::CommandGroupLabel` / `select::SelectGroupLabel`: a mount-time effect publishes
    // the id, and the root reads it back as `aria-labelledby`.
    use_effect(move || {
        ctx.labeled_by.set(Some(id()));
    });

    // `id` is its own prop (not left in `attributes`), so it is emitted once.
    rsx! {
        span { id, ..props.attributes, {props.children} }
    }
}

/// The props for the [`ProgressValue`] component.
#[derive(Props, Clone, PartialEq)]
pub struct ProgressValueProps {
    /// Additional attributes to apply to the value element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// # ProgressValue
///
/// The readout of a [`Progress`]: its value as a whole-number percent of `max` (`56%`), as Base UI's
/// `Progress.Value` formats it by default. Renders nothing while the progress is indeterminate. It is
/// `aria-hidden` (the root already exposes `aria-valuenow`). This must be used inside a [`Progress`].
#[component]
pub fn ProgressValue(props: ProgressValueProps) -> Element {
    let ctx: ProgressCtx = use_context();
    let text = (ctx.value)().map(|v| {
        let max = (ctx.max)();
        let percent = if max > 0.0 { v / max * 100.0 } else { 0.0 };
        format!("{}%", percent.round())
    });

    let owned = attributes!(span { "aria-hidden": "true" });
    let attributes = merge_attributes(vec![props.attributes, owned]);
    rsx! {
        span { ..attributes,
            if let Some(text) = text {
                "{text}"
            }
        }
    }
}
