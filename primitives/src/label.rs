//! Defines the [`Label`] component

use crate::js_listener::{use_js_listeners, JsListeners};
use crate::{merge_attributes, use_unique_id};
use dioxus::prelude::*;
use dioxus_attributes::attributes;

/// The props for the [`Label`] component
#[derive(Props, Clone, PartialEq)]
pub struct LabelProps {
    /// The id of the element that this label is associated with
    pub html_for: ReadSignal<String>,

    /// Additional attributes to apply to the label element
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the label element
    pub children: Element,
}

/// # Label
///
/// The `Label` component is used to create a label for form elements. It must be associated with an element using the [`LabelProps::html_for`] attribute.
///
/// Like Radix's Label, a double click (or any later click of a rapid series) on the label does not
/// select its text, while a drag still does: the label's `mousedown` is cancelled when the click
/// count (`event.detail`) is above 1, unless it landed on a button, input, select or textarea
/// inside the label or a handler of yours already called `prevent_default`. Dioxus's mouse events
/// do not carry the click count, so this is a small scoped script that is installed the first time
/// the pointer reaches the label (the click that starts a double click is always preceded by it)
/// and removed when the label unmounts. Nothing runs on the server or before a pointer arrives.
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::label::Label;
///
/// #[component]
/// fn Demo() -> Element {
///     rsx! {
///         Label {
///             html_for: "name",
///             "Name"
///         }
///
///         input {
///             id: "name",
///             placeholder: "Enter your name",
///         }
///     }
/// }
/// ```
#[component]
pub fn Label(props: LabelProps) -> Element {
    // The script finds the label by this marker, not by `id`, which belongs to the caller.
    let marker = use_unique_id();
    let armed = use_signal(|| false);
    use_js_listeners(move || {
        if !armed() {
            return None;
        }
        Some(
            JsListeners::install(
                "const marker = await dioxus.recv();
                const label = document.querySelector('[data-dx-label=\"' + marker + '\"]');
                listen(label, 'mousedown', (event) => {
                    if (event.detail > 1 && !event.defaultPrevented
                        && !(event.target.closest && event.target.closest('button, input, select, textarea'))) {
                        event.preventDefault();
                    }
                });",
            )
            .arg(marker.cloned()),
        )
    });

    let owned = attributes!(label {
        r#for: props.html_for,
        "data-dx-label": marker.cloned(),
    });
    let merged = merge_attributes(vec![props.attributes.clone(), owned]);

    rsx! {
        label {
            onmouseenter: move |_| arm(armed),
            onmousedown: move |_| arm(armed),
            ..merged,

            {props.children}
        }
    }
}

/// Turns the double-click guard on; a no-op once it is, so a hover does not notify the effect again.
fn arm(mut armed: Signal<bool>) {
    if !*armed.peek() {
        armed.set(true);
    }
}
