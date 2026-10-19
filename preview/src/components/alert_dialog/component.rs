use dioxus::prelude::*;
use dioxus_primitives::alert_dialog::{
    self, AlertDialogActionsProps, AlertDialogCancelProps, AlertDialogDescriptionProps,
    AlertDialogTitleProps,
};
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::keyboard_inset::{use_keyboard_panel, KeyboardInset};
use dioxus_primitives::merge_attributes;

/// The panel's size, shadcn's `size="default" | "sm"`. `Sm` is a compact, centred panel
/// (`max-width: 20rem` at every viewport width) whose actions share one two-column row.
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
#[non_exhaustive]
pub enum AlertDialogSize {
    #[default]
    Default,
    Sm,
}

impl AlertDialogSize {
    /// The `data-size` value the stylesheet keys on.
    pub fn class(&self) -> &'static str {
        match self {
            AlertDialogSize::Default => "default",
            AlertDialogSize::Sm => "sm",
        }
    }
}

/// The look of an [`AlertDialogAction`]: the confirm button of a destructive prompt (the
/// default, this component's long-standing fill) or the primary button of an ordinary one.
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
#[non_exhaustive]
pub enum AlertDialogActionVariant {
    #[default]
    Destructive,
    Primary,
}

impl AlertDialogActionVariant {
    /// The `data-style` value the stylesheet keys on (the Button vocabulary).
    pub fn class(&self) -> &'static str {
        match self {
            AlertDialogActionVariant::Destructive => "destructive",
            AlertDialogActionVariant::Primary => "primary",
        }
    }
}

/// The props for the [`AlertDialogAction`] component: the primitive's `on_click` plus a typed `variant`.
#[derive(Props, Clone, PartialEq)]
pub struct AlertDialogActionProps {
    /// The click event handler for the action button.
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    /// Whether the action is destructive (the default) or an ordinary primary action.
    #[props(default)]
    pub variant: AlertDialogActionVariant,
    /// Additional attributes to extend the action button element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the action button.
    pub children: Element,
}

/// The props for the [`AlertDialog`] component: [`AlertDialogRootProps`](alert_dialog::AlertDialogRootProps) plus the panel's
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
    /// Whether the scrim is drawn; see [`AlertDialogRootProps::overlay`](alert_dialog::AlertDialogRootProps::overlay).
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

    /// The panel's size (`data-size` on the `<dialog>`). Defaults to [`AlertDialogSize::Default`].
    #[props(default)]
    pub size: AlertDialogSize,

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
                attributes: attributes!(div { "data-size": props.size.class() }),
                initial_focus: props.initial_focus,
                final_focus: props.final_focus,
                {props.children}
                KeyboardInset { panel }
            }
        }
    }
}

/// An icon or image above the title (shadcn's `AlertDialogMedia`): a rounded muted tile that
/// centres whatever it holds, sized by the dialog's `size`.
#[component]
pub fn AlertDialogMedia(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    /// The icon or image shown in the tile.
    children: Element,
) -> Element {
    let base = attributes!(div { class: "dx-alert-dialog-media", "data-slot": "alert-dialog-media" });
    let merged = merge_attributes(vec![base, attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/alert_dialog/style.css") }
        div { ..merged, {children} }
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
        class: "dx-alert-dialog-action",
        "data-style": props.variant.class(),
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
