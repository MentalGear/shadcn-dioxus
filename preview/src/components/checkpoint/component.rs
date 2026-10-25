use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::marker::{Marker, MarkerContent, MarkerVariant};
use dioxus::prelude::*;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;

/// A point in the conversation you can return to: a labeled [`Marker`] separator after a turn
/// with a Restore button. Restore keeps that turn and drops every message after it; this
/// component only reports the click (`on_restore`), because what restoring means (truncate a
/// chat history, rewind a workspace) is the app's.
///
/// This is shadcn/ui's documented recipe (a separator Marker with a button in its content) as
/// one component, so the part that is easy to get wrong cannot be: `label` is required, and
/// the button's accessible name is built from it (`"Restore checkpoint: <label>"`), so a thread
/// with many checkpoints never has several buttons that are all just "Restore". The visible
/// word stays the first word of that name (WCAG 2.5.3, label in name). The row is never
/// `role="separator"`, which would hide the label from assistive tech.
#[component]
pub fn Checkpoint(
    /// What this point is, as a short phrase ("Before refactor", "Step 3, 12:04"). Shown in the
    /// separator and used in the button's accessible name.
    #[props(into)]
    label: String,
    /// The visible text of the button. Its accessible name still reads `"<this> checkpoint: <label>"`.
    #[props(into, default = "Restore".to_string())]
    action: String,
    /// Called when the Restore button is pressed.
    #[props(default)]
    on_restore: EventHandler<()>,
    /// Disable the button, for the point the thread is already at or while a restore runs.
    #[props(default)]
    disabled: bool,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let base = attributes!(div {
        "data-slot": "checkpoint",
    });
    let merged = merge_attributes(vec![base, attributes]);
    let name = format!("{action} checkpoint: {label}");
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/checkpoint/style.css") }
        Marker { variant: MarkerVariant::Separator, attributes: merged,
            MarkerContent {
                span { class: "dx-checkpoint-row",
                    "{label}"
                    Button {
                        variant: ButtonVariant::Outline,
                        size: ButtonSize::Xs,
                        disabled,
                        aria_label: name,
                        "data-slot": "checkpoint-restore",
                        onclick: move |_| on_restore.call(()),
                        "{action}"
                    }
                }
            }
        }
    }
}
