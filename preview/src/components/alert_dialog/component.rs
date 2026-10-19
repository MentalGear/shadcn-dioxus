use dioxus::prelude::*;
use dioxus_primitives::alert_dialog::{
    self, AlertDialogActionProps, AlertDialogActionsProps, AlertDialogCancelProps,
    AlertDialogDescriptionProps, AlertDialogRootProps, AlertDialogTitleProps,
};
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::keyboard_inset::{use_keyboard_panel, KeyboardInset};
use dioxus_primitives::merge_attributes;

/// The props for the [`AlertDialog`] component: [`AlertDialogRootProps`] plus the panel's
/// `initial_focus` / `final_focus` (an alert dialog's panel is rendered by this wrapper).
#[derive(Props, Clone, PartialEq)]
pub struct AlertDialogProps {
    /// The id of the alert dialog root element.
    pub id: ReadSignal<Option<String>>,
    /// Whether the alert dialog is open by default (uncontrolled).
    #[props(default)]
    pub default_open: bool,
    /// The controlled open state.
    #[props(default)]
    pub open: ReadSignal<Option<bool>>,
    /// Whether the scrim is drawn; see [`AlertDialogRootProps::overlay`].
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub overlay: ReadSignal<bool>,
    /// Called when the open state changes.
    #[props(default)]
    pub on_open_change: Callback<bool>,
    /// Where focus lands on open, as a function of the open modality
    /// ([`dioxus_primitives::modality::OpenModality`]); see [`dioxus_primitives::alert_dialog::AlertDialogContentProps::initial_focus`].
    #[props(default)]
    pub initial_focus: Option<dioxus_primitives::modality::FocusTargetFn>,

    /// Where focus goes on close; see [`dioxus_primitives::alert_dialog::AlertDialogContentProps::final_focus`].
    #[props(default)]
    pub final_focus: Option<dioxus_primitives::modality::FocusTargetFn>,

    /// Additional attributes for the root element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the alert dialog.
    pub children: Element,
}

#[component]
pub fn AlertDialog(props: AlertDialogProps) -> Element {
    let base = attributes!(div {
        class: "dx-alert-dialog-backdrop"
    });
    let merged = merge_attributes(vec![base, props.attributes]);
    // The panel follows the on-screen keyboard (`primitives/src/keyboard_inset.rs`; `style.css`
    // consumes `--dx-keyboard-inset`).
    let panel = use_keyboard_panel(None);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/alert_dialog/style.css") }
        alert_dialog::AlertDialogRoot {
            id: props.id,
            default_open: props.default_open,
            open: props.open,
            overlay: props.overlay,
            on_open_change: props.on_open_change,
            attributes: merged,
            alert_dialog::AlertDialogContent {
                id: panel.id(),
                class: "dx-alert-dialog".to_string(),
                initial_focus: props.initial_focus,
                final_focus: props.final_focus,
                {props.children}
                KeyboardInset { panel }
            }
        }
    }
}

#[component]
pub fn AlertDialogTitle(props: AlertDialogTitleProps) -> Element {
    let base = attributes!(h2 {
        class: "dx-alert-dialog-title"
    });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/alert_dialog/style.css") }
        alert_dialog::AlertDialogTitle { attributes: merged, {props.children} }
    }
}

#[component]
pub fn AlertDialogDescription(props: AlertDialogDescriptionProps) -> Element {
    let base = attributes!(p {
        class: "dx-alert-dialog-description"
    });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/alert_dialog/style.css") }
        alert_dialog::AlertDialogDescription { attributes: merged, {props.children} }
    }
}

#[component]
pub fn AlertDialogActions(props: AlertDialogActionsProps) -> Element {
    let base = attributes!(div {
        class: "dx-alert-dialog-actions"
    });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/alert_dialog/style.css") }
        alert_dialog::AlertDialogActions { attributes: merged, {props.children} }
    }
}

#[component]
pub fn AlertDialogCancel(props: AlertDialogCancelProps) -> Element {
    let base = attributes!(button {
        class: "dx-alert-dialog-cancel"
    });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/alert_dialog/style.css") }
        alert_dialog::AlertDialogCancel {
            on_click: props.on_click,
            attributes: merged,
            {props.children}
        }
    }
}

#[component]
pub fn AlertDialogAction(props: AlertDialogActionProps) -> Element {
    let base = attributes!(button {
        class: "dx-alert-dialog-action"
    });
    let merged = merge_attributes(vec![base, props.attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/alert_dialog/style.css") }
        alert_dialog::AlertDialogAction {
            on_click: props.on_click,
            attributes: merged,
            {props.children}
        }
    }
}
