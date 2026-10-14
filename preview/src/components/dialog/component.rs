use crate::components::button::{Button, ButtonSize, ButtonVariant};
use dioxus::prelude::*;
use dioxus_icons::lucide::X;
use dioxus_primitives::dialog::{self, DialogCtx, DialogDescriptionProps, DialogTitleProps};
use dioxus_primitives::{dioxus_attributes::attributes, merge_attributes};

/// The props for the [`Dialog`] component: [`dialog::DialogRootProps`] plus `show_close_button`.
#[derive(Props, Clone, PartialEq)]
pub struct DialogProps {
    /// The ID of the dialog root element.
    pub id: ReadSignal<Option<String>>,

    /// Whether the dialog is modal. If true, it traps focus within the dialog when open.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub is_modal: ReadSignal<bool>,

    /// Whether a modal dialog dims the page behind it with the shared overlay scrim.
    /// Defaults to `true`; see [`dialog::DialogRootProps::overlay`].
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub overlay: ReadSignal<bool>,

    /// The controlled `open` state of the dialog.
    pub open: ReadSignal<Option<bool>>,

    /// The default `open` state of the dialog if it is not controlled.
    #[props(default)]
    pub default_open: bool,

    /// A callback that is called when the open state changes.
    #[props(default)]
    pub on_open_change: Callback<bool>,

    /// Whether the panel renders its own close button, an `X` at the top end. Defaults to `true`,
    /// as shadcn's `DialogContent showCloseButton`. The button is the last child of the panel, so
    /// a keyboard open focuses the first control in `children` (the first field), not the X.
    #[props(default = true)]
    pub show_close_button: bool,

    /// Additional attributes to apply to the dialog panel element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the dialog.
    pub children: Element,
}

#[component]
pub fn Dialog(props: DialogProps) -> Element {
    let base = attributes!(div {
        class: "dx-dialog",
        "data-slot": "dialog-content",
    });
    let merged = merge_attributes(vec![base, props.attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/dialog/style.css") }
        dialog::DialogRoot {
            class: "dx-dialog-backdrop",
            id: props.id,
            is_modal: props.is_modal,
            overlay: props.overlay,
            open: props.open,
            default_open: props.default_open,
            on_open_change: props.on_open_change,
            dialog::DialogContent {
                class: None,
                attributes: merged,
                {props.children}
                if props.show_close_button {
                    DialogCornerClose {}
                }
            }
        }
    }
}

/// The panel's own close button (`show_close_button`). Its own function so nothing the panel
/// merges onto `.dx-dialog` is in scope here: the Button receives only the close behaviour.
///
/// shadcn's `DialogContent` renders the X after `children`: it is last in DOM order, so
/// `showModal()`'s initial focus (the first focusable descendant) lands on the first control of
/// the body. The wrapper owns the placement; the button is the real Button, untouched by this
/// sheet.
#[component]
fn DialogCornerClose() -> Element {
    rsx! {
        div { class: "dx-dialog-close", "data-slot": "dialog-close-slot",
            DialogClose {
                "data-slot": "dialog-close",
                aria_label: "Close",
                r#as: |close_attributes| rsx! {
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::IconSm,
                        attributes: close_attributes,
                        X {}
                    }
                },
            }
        }
    }
}

/// Closes the enclosing [`Dialog`] (shadcn's `DialogClose`). Renders a `<button type="button">`;
/// `r#as` renders another element (typically a `Button`) with the close behaviour in its
/// `attributes`, as shadcn's `render={<Button />}`.
// No `document::Link` (as `SheetClose`): it reads `DialogCtx`, so it only renders inside a
// `Dialog`, which already links the stylesheet.
#[component]
pub fn DialogClose(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    r#as: Option<Callback<Vec<Attribute>, Element>>,
    children: Element,
) -> Element {
    let ctx: DialogCtx = use_context();

    let base = attributes!(button {
        r#type: "button",
        onclick: move |_| {
            ctx.set_open(false);
        }
    });
    let merged = merge_attributes(vec![base, attributes]);

    if let Some(dynamic) = r#as {
        dynamic.call(merged)
    } else {
        rsx! {
            button { ..merged, {children} }
        }
    }
}

/// Groups the title and description (shadcn's `DialogHeader`: a column with `gap-2`).
#[component]
pub fn DialogHeader(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(div { class: "dx-dialog-header", "data-slot": "dialog-header" });
    let merged = merge_attributes(vec![base, attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/dialog/style.css") }
        div { ..merged, {children} }
    }
}

/// The action row (shadcn's `DialogFooter`): a muted band across the panel's foot, a column
/// reversed on a narrow screen and a row aligned to the end from `sm` up. `show_close_button`
/// appends an outline "Close" button, as shadcn's.
#[component]
pub fn DialogFooter(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    #[props(default = false)] show_close_button: bool,
    children: Element,
) -> Element {
    let base = attributes!(div { class: "dx-dialog-footer", "data-slot": "dialog-footer" });
    let merged = merge_attributes(vec![base, attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/dialog/style.css") }
        div { ..merged,
            {children}
            if show_close_button {
                DialogClose {
                    r#as: |close_attributes| rsx! {
                        Button { variant: ButtonVariant::Outline, attributes: close_attributes, "Close" }
                    },
                }
            }
        }
    }
}

#[component]
pub fn DialogTitle(props: DialogTitleProps) -> Element {
    let base = attributes!(h2 {
        class: "dx-dialog-title",
    });
    let merged = merge_attributes(vec![base, props.attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/dialog/style.css") }
        dialog::DialogTitle {
            id: props.id,
            attributes: merged,
            {props.children}
        }
    }
}

#[component]
pub fn DialogDescription(props: DialogDescriptionProps) -> Element {
    let base = attributes!(p {
        class: "dx-dialog-description",
    });
    let merged = merge_attributes(vec![base, props.attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/dialog/style.css") }
        dialog::DialogDescription {
            id: props.id,
            attributes: merged,
            {props.children}
        }
    }
}
