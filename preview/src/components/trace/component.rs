use dioxus::prelude::*;
use dioxus_icons::lucide::ChevronRight;
use dioxus_primitives::activity::use_motion_when;
use dioxus_primitives::collapsible::{
    self, CollapsibleContentProps, CollapsibleProps, CollapsibleTriggerProps,
};
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;

/// Where a [`TraceLine`]'s step is.
#[derive(Copy, Clone, PartialEq)]
#[non_exhaustive]
pub enum TraceStatus {
    /// In progress: the title shimmers (paused while off-screen, off under reduced motion).
    Running,
    /// Failed: the line reads in the destructive text colour. Colour is not the only cue:
    /// say it in the title ("Failed to read file").
    Error,
    /// Not started yet: the icon is dimmed (the text keeps its contrast).
    Pending,
}

impl TraceStatus {
    pub fn class(&self) -> &'static str {
        match self {
            TraceStatus::Running => "running",
            TraceStatus::Error => "error",
            TraceStatus::Pending => "pending",
        }
    }
}

/// Whether the enclosing [`TraceLine`] is running, so its [`TraceTitle`] can shimmer.
#[derive(Clone, Copy)]
struct TraceLineRunning(Memo<bool>);

/// A stack of trace lines: the quiet record of what an agent did in one turn (thinking, a
/// tool call, a file read). Plain layout with no role, so there is no primitive underneath.
/// Put `role: "status"` on it while a step is running if the update should be announced.
#[component]
pub fn Trace(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    /// The lines and collapsibles.
    children: Element,
) -> Element {
    let base = attributes!(div {
        class: "dx-trace",
        "data-slot": "trace",
    });
    let merged = merge_attributes(vec![base, attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/trace/style.css") }
        div { ..merged, {children} }
    }
}

/// One step: a [`TraceIcon`], a [`TraceTitle`] and an optional [`TraceDescription`]. Static
/// text; put it inside a [`TraceCollapsibleTrigger`] to make it expandable.
#[component]
pub fn TraceLine(
    /// Where the step is. `None` is a finished step.
    status: Option<TraceStatus>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    /// The icon, title and description.
    children: Element,
) -> Element {
    let running = use_memo(use_reactive!(|status| status == Some(TraceStatus::Running)));
    use_context_provider(|| TraceLineRunning(running));
    let base = attributes!(span {
        class: "dx-trace-line",
        "data-slot": "trace-line",
    });
    // `data-status` reflects this component's own typed prop -- owned-wins.
    let owned = attributes!(span { "data-status": status.map(|s| s.class()) });
    let merged = merge_attributes(vec![base, attributes, owned]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/trace/style.css") }
        span { ..merged, {children} }
    }
}

/// A decorative icon slot. Hidden from assistive tech: the [`TraceTitle`] carries the meaning.
#[component]
pub fn TraceIcon(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    /// The icon.
    children: Element,
) -> Element {
    let base = attributes!(span {
        class: "dx-trace-icon",
        "data-slot": "trace-icon",
    });
    // Decorative by construction, not a caller default -- owned-wins.
    let owned = attributes!(span { "aria-hidden": "true" });
    let merged = merge_attributes(vec![base, attributes, owned]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/trace/style.css") }
        span { ..merged, {children} }
    }
}

/// The step's name ("Read 4 files", "Searched the web"). Truncates, and shimmers while its
/// [`TraceLine`] is `Running`.
#[component]
pub fn TraceTitle(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    /// The title.
    children: Element,
) -> Element {
    let running = try_use_context::<TraceLineRunning>()
        .map(|ctx| ctx.0)
        .unwrap_or_else(|| use_memo(|| false));
    let motion = use_motion_when(move || running());
    let base = attributes!(span {
        class: "dx-trace-title",
        "data-slot": "trace-title",
    });
    let shimmer = attributes!(span { class: if running() { "dx-shimmer" } else { "" } });
    let merged = merge_attributes(vec![base, attributes, shimmer, motion.attributes()]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/trace/style.css") }
        span { ..merged, {children} }
    }
}

/// A short trailing detail (a duration, a count). Never truncates.
#[component]
pub fn TraceDescription(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    /// The detail.
    children: Element,
) -> Element {
    let base = attributes!(span {
        class: "dx-trace-description",
        "data-slot": "trace-description",
    });
    let merged = merge_attributes(vec![base, attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/trace/style.css") }
        span { ..merged, {children} }
    }
}

/// A line that expands to show its detail. Uncontrolled by default; pass `open` and
/// `on_open_change` to drive it (for example open while a reasoning step streams).
#[component]
pub fn TraceCollapsible(props: CollapsibleProps) -> Element {
    let base = attributes!(div {
        class: "dx-trace-collapsible",
        "data-slot": "trace-collapsible",
    });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/trace/style.css") }
        collapsible::Collapsible {
            keep_mounted: props.keep_mounted,
            hidden_until_found: props.hidden_until_found,
            default_open: props.default_open,
            disabled: props.disabled,
            open: props.open,
            on_open_change: props.on_open_change,
            as: props.r#as,
            attributes: merged,
            {props.children}
        }
    }
}

/// The button that toggles the detail: put a [`TraceLine`] inside; the chevron is added after
/// it. A real `<button>` with `aria-expanded` and `aria-controls`, so Enter and Space work.
#[component]
pub fn TraceCollapsibleTrigger(props: CollapsibleTriggerProps) -> Element {
    let base = attributes!(button {
        class: "dx-trace-collapsible-trigger",
        "data-slot": "trace-collapsible-trigger",
    });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/trace/style.css") }
        collapsible::CollapsibleTrigger { as: props.r#as, attributes: merged,
            {props.children}
            TraceIcon { class: "dx-trace-chevron",
                ChevronRight { size: "1rem" }
            }
        }
    }
}

/// The detail, shown while open (unmounted while closed unless `keep_mounted`).
#[component]
pub fn TraceCollapsibleContent(props: CollapsibleContentProps) -> Element {
    let base = attributes!(div {
        class: "dx-trace-collapsible-content",
        "data-slot": "trace-collapsible-content",
    });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/trace/style.css") }
        collapsible::CollapsibleContent { id: props.id, attributes: merged,
            div { class: "dx-trace-collapsible-content-inner", "data-slot": "trace-collapsible-content-inner",
                {props.children}
            }
        }
    }
}
