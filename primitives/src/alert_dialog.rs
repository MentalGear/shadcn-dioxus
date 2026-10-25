//! Defines the [`AlertDialogRoot`] component and its sub-components.
//!
//! ## Native `<dialog>` modality (Phase 4.2, dev-docs/plan.md)
//!
//! An alert dialog is always modal (APG alert-dialog pattern), so unlike
//! `dialog.rs` there is no `is_modal` branch here -- [`AlertDialogContent`]
//! is cfg-split directly, the same two arms as `dialog.rs`'s
//! `DialogContentModal` (`dev-docs/archive/research/phase4-spike-findings.md` Construction B):
//! - `#[cfg(not(feature = "web"))]`: the plain
//!   `div` + vendored `FocusTrap` path.
//! - `#[cfg(feature = "web")]`: a real `<dialog role="alertdialog">`
//!   (the explicit `role` stays -- unlike a plain modal `Dialog`, this is a
//!   genuine ARIA-subclass refinement of `<dialog>`'s implicit role,
//!   <https://www.w3.org/TR/html-aria/#el-dialog>), driven by the same
//!   `crate::use_dialog_open_driver`/`crate::use_dialog_close_sync` pair
//!   `dialog.rs` uses. Closing waits for the exit animation (the driver
//!   defers `close()`; see `dialog.rs`), so the `<dialog>` carries its own
//!   `data-state`. No backdrop-click dismiss: unlike `Dialog`,
//!   `AlertDialogContent` does not call `use_outside_dismiss` (APG
//!   discourages light-dismissing an alert dialog), and the web arm has no
//!   equivalent either.

#[cfg(not(feature = "web"))]
use crate::use_global_escape_listener;
use crate::{merge_attributes, use_animated_open, use_controlled, use_id_or, use_unique_id};
#[cfg(not(feature = "web"))]
use dioxus::document;
use dioxus::prelude::*;
use dioxus_attributes::attributes;

#[derive(Clone)]
struct AlertDialogCtx {
    open: Memo<bool>,
    set_open: Callback<bool>,
    overlay: ReadSignal<bool>,
    labelledby: String,
    describedby: String,
}

/// The props for the [`AlertDialogRoot`] component.
#[derive(Props, Clone, PartialEq)]
pub struct AlertDialogRootProps {
    /// The id of the alert dialog root element. If not provided, a unique id will be generated.
    pub id: ReadSignal<Option<String>>,
    /// Whether the alert dialog should be open by default. This is only used if the `open` signal is not provided.
    #[props(default)]
    pub default_open: bool,
    /// The open state of the alert dialog. If this is provided, it will be used to control the open state of the dialog.
    #[props(default)]
    pub open: ReadSignal<Option<bool>>,
    /// Whether the alert dialog dims the page behind it with the shared overlay scrim. Defaults to `true`.
    /// When `false` the `<dialog>` (and the root element) carry `data-dx-overlay="off"`, which the theme
    /// turns into a transparent `::backdrop`; the modality itself is untouched. See
    /// [`crate::dialog::DialogRootProps::overlay`].
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub overlay: ReadSignal<bool>,
    /// Callback to handle changes in the open state of the dialog.
    #[props(default)]
    pub on_open_change: Callback<bool>,
    /// Additional attributes to extend the root element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the alert dialog root element.
    pub children: Element,
}

/// # AlertDialogRoot
///
/// The entry point for the alert dialog. It manages the open state of the dialog and provides context to its children. You
/// can use it to create a backdrop for the dialog if needed. The contents will only be rendered when the dialog is open.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::alert_dialog::*;
///
/// #[component]
/// fn Demo() -> Element {
///     let mut open = use_signal(|| false);
///
///     rsx! {
///         button {
///             onclick: move |_| open.set(true),
///             "Show Alert Dialog"
///         }
///         AlertDialogRoot {
///             open: open(),
///             on_open_change: move |v| open.set(v),
///             AlertDialogContent {
///                 AlertDialogTitle { "Delete item" }
///                 AlertDialogDescription { "Are you sure you want to delete this item? This action cannot be undone." }
///                 AlertDialogActions {
///                     AlertDialogCancel { "Cancel" }
///                     AlertDialogAction {
///                         on_click: move |_| tracing::info!("Item deleted"),
///                         "Delete"
///                     }
///                 }
///             }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// The [`AlertDialogRoot`] component defines the following data attributes you can use to control styling:
/// - `data-state`: Indicates if the alert dialog is open or closed. It can be either "open" or "closed".
#[component]
pub fn AlertDialogRoot(props: AlertDialogRootProps) -> Element {
    // See `DialogRoot`'s identical call for why this must be at the root,
    // not only inside `use_scroll_lock` (called by `AlertDialogContent`,
    // which mounts lazily on open).
    use_effect(crate::scroll_lock::ensure_scrollbar_gutter_baseline);

    let labelledby = use_unique_id().to_string();
    let describedby = use_unique_id().to_string();
    let (open, set_open) = use_controlled(props.open, props.default_open, props.on_open_change);
    // See `scroll_lock::use_early_scroll_capture`'s doc / `DialogRoot`'s
    // identical call: captures scroll position before `showModal()` (called
    // from `AlertDialogContent`, mounted as a consequence of the same `open`
    // flip) gets a chance to move it.
    crate::scroll_lock::use_early_scroll_capture(open);
    use_context_provider(|| AlertDialogCtx {
        open,
        set_open,
        overlay: props.overlay,
        labelledby,
        describedby,
    });

    let id = use_unique_id();
    let id = use_id_or(id, props.id);
    let render_element = use_animated_open(id, open);

    // Owned by this component -- open state must win over a caller's own
    // attributes (`dev-docs/backlog.md` row 93's duplicate-attribute hazard).
    let attributes = merge_attributes(vec![
        props.attributes,
        attributes!(div {
            "data-state": if open() { "open" } else { "closed" },
            // On the root too: without a top layer this wrapper is what a stylesheet paints the scrim on.
            "data-dx-overlay": crate::dialog::overlay_off(props.overlay),
        }),
    ]);

    rsx! {
        {crate::focus_trap_script()}
        if render_element() {
            div {
                id,
                ..attributes,
                {props.children}
            }
        }
    }
}

/// The props for the [`AlertDialogContent`] component.
#[derive(Props, Clone, PartialEq)]
pub struct AlertDialogContentProps {
    /// The id of the alert dialog content element. If not provided, a unique id will be generated.
    pub id: ReadSignal<Option<String>>,

    /// The class to apply to the alert dialog content element.
    #[props(default)]
    pub class: Option<String>,

    /// Where focus lands when the panel opens, as a function of how it was opened
    /// ([`OpenModality`](crate::modality::OpenModality)). An input to the one open-focus policy
    /// (`dxOpenFocus`), not a second path: with none, keyboard and mouse opens focus the first
    /// control and touch and pen opens focus the panel (no software keyboard). A selector that
    /// matches nothing falls back to that default.
    #[props(default)]
    pub initial_focus: Option<crate::modality::FocusTargetFn>,

    /// Where focus goes when the panel closes, as a function of the modality of the close.
    /// Default: the element that held focus when it opened (the trigger for a click open, the
    /// previously focused element for a programmatic one).
    #[props(default)]
    pub final_focus: Option<crate::modality::FocusTargetFn>,

    /// Additional attributes to extend the content element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the alert dialog content element.
    pub children: Element,
}

/// # AlertDialogContent
///
/// The content of the alert dialog. Any interactive content in the dialog should be placed
/// inside this component. It will trap focus within the dialog while it is open
///
/// This must be used inside an [`AlertDialogRoot`] component.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::alert_dialog::*;
///
/// #[component]
/// fn Demo() -> Element {
///     let mut open = use_signal(|| false);
///
///     rsx! {
///         button {
///             onclick: move |_| open.set(true),
///             "Show Alert Dialog"
///         }
///         AlertDialogRoot {
///             open: open(),
///             on_open_change: move |v| open.set(v),
///             AlertDialogContent {
///                 AlertDialogTitle { "Delete item" }
///                 AlertDialogDescription { "Are you sure you want to delete this item? This action cannot be undone." }
///                 AlertDialogActions {
///                     AlertDialogCancel { "Cancel" }
///                     AlertDialogAction {
///                         on_click: move |_| tracing::info!("Item deleted"),
///                         "Delete"
///                     }
///                 }
///             }
///         }
///     }
/// }
/// ```
/// Native (Blitz) target: a plain `div` with the vendored `FocusTrap`
/// (no native `<dialog>`; dev-docs/plan.md Phase 4.2 scopes that to the web).
#[cfg(not(feature = "web"))]
#[component]
pub fn AlertDialogContent(props: AlertDialogContentProps) -> Element {
    let ctx: AlertDialogCtx = use_context();

    let open = ctx.open;
    let set_open = ctx.set_open;

    // Add a escape key listener to the document when the dialog is open. We can't
    // just add this to the dialog itself because it might not be focused if the user
    // is highlighting text or interacting with another element.
    use_global_escape_listener(move || set_open.call(false));

    let gen_id = use_unique_id();
    let id = use_id_or(gen_id, props.id);

    // An alert dialog is always modal, so lock page scroll for as long as
    // it's open. See dev-docs/plan.md Phase 3.2.
    crate::scroll_lock::use_scroll_lock(open);

    use_effect(move || {
        let eval = document::eval(
            r#"let id = await dioxus.recv();
            let is_open = await dioxus.recv();
            let dialog = document.getElementById(id);

            if (is_open) {
                dialog.trap = window.createFocusTrap(dialog);
            }
            if (!is_open && dialog.trap) {
                dialog.trap.remove();
                dialog.trap = null;
            }"#,
        );
        let _ = eval.send(id.to_string());
        let _ = eval.send(open.cloned());
    });

    // Owned by this component -- dialog role/aria wiring must win over a
    // caller's own attributes (`dev-docs/backlog.md` row 93's
    // duplicate-attribute hazard).
    let mut caller_attributes = props.attributes.clone();
    caller_attributes.extend(crate::modality::focus_attributes(
        props.initial_focus,
        props.final_focus,
    ));
    let attributes = merge_attributes(vec![
        caller_attributes,
        attributes!(div {
            role: "alertdialog",
            aria_modal: "true",
            aria_labelledby: ctx.labelledby.clone(),
            aria_describedby: ctx.describedby.clone(),
            "data-dx-overlay": crate::dialog::overlay_off(ctx.overlay),
        }),
    ]);

    rsx! {
        div {
            id,
            class: props.class.clone().unwrap_or_else(|| "dx-alert-dialog".to_string()),
            ..attributes,
            {props.children}
        }
    }
}

/// Web target (Phase 4.2, dev-docs/plan.md) -- a real `<dialog>` opened with
/// `showModal()`. `open` is never bound as an attribute here; the browser
/// supplies the focus trap, focus restore, background inertness, and
/// top-layer rendering. `role="alertdialog"` stays explicit -- see this
/// module's doc comment.
#[cfg(feature = "web")]
#[component]
pub fn AlertDialogContent(props: AlertDialogContentProps) -> Element {
    let ctx: AlertDialogCtx = use_context();

    let open = ctx.open;
    let set_open = ctx.set_open;

    let gen_id = use_unique_id();
    let id = use_id_or(gen_id, props.id);

    // An alert dialog is always modal, so lock page scroll for as long as
    // it's open. See dev-docs/plan.md Phase 3.2.
    crate::scroll_lock::use_scroll_lock(open);

    // Same eval-channel close-sync + `.open`-guarded open-driver pair as
    // `dialog.rs`'s web modal arm -- see `crate::use_dialog_close_sync`/
    // `crate::use_dialog_open_driver` (lib.rs), which keep a native close
    // from stranding the `open` signal at `true`
    // (dev-docs/reference/platform-rationale.md Caveat 1). Deliberately no
    // `use_global_escape_listener` and no focus-trap eval: the browser's own
    // `showModal()`/`cancel`/`close` events already cover Escape, focus
    // trap, and focus restore.
    crate::use_dialog_close_sync(id, set_open);
    crate::use_dialog_open_driver(id, open);

    // Owned by this component -- see the native arm's identical
    // construction above (`dev-docs/backlog.md` row 93).
    let mut caller_attributes = props.attributes.clone();
    caller_attributes.extend(crate::modality::focus_attributes(
        props.initial_focus,
        props.final_focus,
    ));
    let attributes = merge_attributes(vec![
        caller_attributes,
        attributes!(dialog {
            role: "alertdialog",
            aria_modal: "true",
            aria_labelledby: ctx.labelledby.clone(),
            aria_describedby: ctx.describedby.clone(),
            // The dialog's own state: closing keeps it open (modal) while it animates out, so a
            // stylesheet keys the exit on `[data-state="closed"]`. See `crate::use_dialog_open_driver`.
            "data-state": if open() { "open" } else { "closed" },
            "data-dx-overlay": crate::dialog::overlay_off(ctx.overlay),
        }),
    ]);

    rsx! {
        dialog {
            id,
            class: props.class.clone().unwrap_or_else(|| "dx-alert-dialog".to_string()),
            ..attributes,
            {props.children}
        }
    }
}

/// The props for the [`AlertDialogTitle`] component.
#[derive(Props, Clone, PartialEq)]
pub struct AlertDialogTitleProps {
    /// Additional attributes to extend the title element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the title element.
    pub children: Element,
}

/// # AlertDialogTitle
///
/// The title of the alert dialog. This will be used to label the dialog for accessibility purposes.
///
/// This must be used inside an [`AlertDialogRoot`] component and should be placed inside an [`AlertDialogContent`] component.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::alert_dialog::*;
///
/// #[component]
/// fn Demo() -> Element {
///     let mut open = use_signal(|| false);
///
///     rsx! {
///         button {
///             onclick: move |_| open.set(true),
///             "Show Alert Dialog"
///         }
///         AlertDialogRoot {
///             open: open(),
///             on_open_change: move |v| open.set(v),
///             AlertDialogContent {
///                 AlertDialogTitle { "Delete item" }
///                 AlertDialogDescription { "Are you sure you want to delete this item? This action cannot be undone." }
///                 AlertDialogActions {
///                     AlertDialogCancel { "Cancel" }
///                     AlertDialogAction {
///                         on_click: move |_| tracing::info!("Item deleted"),
///                         "Delete"
///                     }
///                 }
///             }
///         }
///     }
/// }
/// ```
#[component]
pub fn AlertDialogTitle(props: AlertDialogTitleProps) -> Element {
    let ctx: AlertDialogCtx = use_context();
    // Owned by this component -- `id` is the labelledby target read back by
    // `AlertDialogContent`'s `aria-labelledby`, so it must win over a
    // caller's own `id` rather than risk the SSR duplicate-attribute hazard
    // (`dev-docs/backlog.md` row 93 names this exact site).
    let attributes = merge_attributes(vec![
        props.attributes,
        attributes!(h2 {
            id: ctx.labelledby.clone()
        }),
    ]);
    rsx! {
        h2 { ..attributes, {props.children} }
    }
}

/// The props for the [`AlertDialogDescription`] component.
#[derive(Props, Clone, PartialEq)]
pub struct AlertDialogDescriptionProps {
    /// Additional attributes to extend the description element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the description element.
    pub children: Element,
}

/// # AlertDialogDescription
///
/// The description of the alert dialog. This will be used to describe the dialog for accessibility purposes.
///
/// This must be used inside an [`AlertDialogRoot`] component and should be placed inside an [`AlertDialogContent`] component.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::alert_dialog::*;
///
/// #[component]
/// fn Demo() -> Element {
///     let mut open = use_signal(|| false);
///
///     rsx! {
///         button {
///             onclick: move |_| open.set(true),
///             "Show Alert Dialog"
///         }
///         AlertDialogRoot {
///             open: open(),
///             on_open_change: move |v| open.set(v),
///             AlertDialogContent {
///                 AlertDialogTitle { "Delete item" }
///                 AlertDialogDescription { "Are you sure you want to delete this item? This action cannot be undone." }
///                 AlertDialogActions {
///                     AlertDialogCancel { "Cancel" }
///                     AlertDialogAction {
///                         on_click: move |_| tracing::info!("Item deleted"),
///                         "Delete"
///                     }
///                 }
///             }
///         }
///     }
/// }
/// ```
#[component]
pub fn AlertDialogDescription(props: AlertDialogDescriptionProps) -> Element {
    let ctx: AlertDialogCtx = use_context();
    // Owned by this component -- see `AlertDialogTitle`'s identical
    // construction above (`dev-docs/backlog.md` row 93 names this exact site).
    let attributes = merge_attributes(vec![
        props.attributes,
        attributes!(p {
            id: ctx.describedby.clone()
        }),
    ]);
    rsx! {
        p { ..attributes, {props.children} }
    }
}

/// The props for the [`AlertDialogActions`] component.
#[derive(Props, Clone, PartialEq)]
pub struct AlertDialogActionsProps {
    /// Additional attributes to extend the actions element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the actions element.
    pub children: Element,
}

/// # AlertDialogActions
///
/// The actions of the alert dialog. This will be used to group the actions.
///
/// This must be used inside an [`AlertDialogRoot`] component and should be placed inside an [`AlertDialogContent`] component.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::alert_dialog::*;
///
/// #[component]
/// fn Demo() -> Element {
///     let mut open = use_signal(|| false);
///
///     rsx! {
///         button {
///             onclick: move |_| open.set(true),
///             "Show Alert Dialog"
///         }
///         AlertDialogRoot {
///             open: open(),
///             on_open_change: move |v| open.set(v),
///             AlertDialogContent {
///                 AlertDialogTitle { "Delete item" }
///                 AlertDialogDescription { "Are you sure you want to delete this item? This action cannot be undone." }
///                 AlertDialogActions {
///                     AlertDialogCancel { "Cancel" }
///                     AlertDialogAction {
///                         on_click: move |_| tracing::info!("Item deleted"),
///                         "Delete"
///                     }
///                 }
///             }
///         }
///     }
/// }
/// ```
#[component]
pub fn AlertDialogActions(props: AlertDialogActionsProps) -> Element {
    rsx! {
        div { ..props.attributes, {props.children} }
    }
}

/// The props for the [`AlertDialogAction`] component.
#[derive(Props, Clone, PartialEq)]
pub struct AlertDialogActionProps {
    /// The click event handler for the action button.
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    /// Additional attributes to extend the action button element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the action button.
    pub children: Element,
}

/// # AlertDialogAction
///
/// An action button for the alert dialog. In addition to running the `on_click` callback, it will also close the dialog when clicked.
///
/// This must be used inside an [`AlertDialogRoot`] component and should be placed inside an [`AlertDialogContent`] component.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::alert_dialog::*;
///
/// #[component]
/// fn Demo() -> Element {
///     let mut open = use_signal(|| false);
///
///     rsx! {
///         button {
///             onclick: move |_| open.set(true),
///             "Show Alert Dialog"
///         }
///         AlertDialogRoot {
///             open: open(),
///             on_open_change: move |v| open.set(v),
///             AlertDialogContent {
///                 AlertDialogTitle { "Delete item" }
///                 AlertDialogDescription { "Are you sure you want to delete this item? This action cannot be undone." }
///                 AlertDialogActions {
///                     AlertDialogCancel { "Cancel" }
///                     AlertDialogAction {
///                         on_click: move |_| tracing::info!("Item deleted"),
///                         "Delete"
///                     }
///                 }
///             }
///         }
///     }
/// }
/// ```
#[component]
pub fn AlertDialogAction(props: AlertDialogActionProps) -> Element {
    let ctx: AlertDialogCtx = use_context();
    let open = ctx.open;
    let set_open = ctx.set_open;
    let user_on_click = props.on_click;
    let on_click = use_callback(move |evt: MouseEvent| {
        set_open.call(false);
        if let Some(cb) = &user_on_click {
            cb.call(evt.clone());
        }
    });
    // Owned by this component -- roving-focus tabindex/type must win over
    // a caller's own attributes (`dev-docs/backlog.md` row 93's
    // duplicate-attribute hazard).
    let attributes = merge_attributes(vec![
        props.attributes,
        attributes!(button {
            tabindex: if open() { "0" } else { "-1" },
            type: "button",
        }),
    ]);
    rsx! {
        button {
            onclick: on_click,
            ..attributes,
            {props.children}
        }
    }
}

/// The props for the [`AlertDialogCancel`] component.
#[derive(Props, Clone, PartialEq)]
pub struct AlertDialogCancelProps {
    /// The click event handler for the cancel button.
    #[props(default)]
    pub on_click: Option<EventHandler<MouseEvent>>,
    /// Additional attributes to extend the cancel button element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
    /// The children of the cancel button.
    pub children: Element,
}

/// # AlertDialogCancel
///
/// An cancel button for the alert dialog. In addition to running the `on_click` callback, it will also close the dialog when clicked.
///
/// This must be used inside an [`AlertDialogRoot`] component and should be placed inside an [`AlertDialogContent`] component.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::alert_dialog::*;
///
/// #[component]
/// fn Demo() -> Element {
///     let mut open = use_signal(|| false);
///
///     rsx! {
///         button {
///             onclick: move |_| open.set(true),
///             "Show Alert Dialog"
///         }
///         AlertDialogRoot {
///             open: open(),
///             on_open_change: move |v| open.set(v),
///             AlertDialogContent {
///                 AlertDialogTitle { "Delete item" }
///                 AlertDialogDescription { "Are you sure you want to delete this item? This action cannot be undone." }
///                 AlertDialogActions {
///                     AlertDialogCancel { "Cancel" }
///                     AlertDialogAction {
///                         on_click: move |_| tracing::info!("Item deleted"),
///                         "Delete"
///                     }
///                 }
///             }
///         }
///     }
/// }
/// ```
#[component]
pub fn AlertDialogCancel(props: AlertDialogCancelProps) -> Element {
    let ctx: AlertDialogCtx = use_context();
    let open = ctx.open;
    let set_open = ctx.set_open;
    let user_on_click = props.on_click;
    let on_click = use_callback(move |evt: MouseEvent| {
        set_open.call(false);
        if let Some(cb) = &user_on_click {
            cb.call(evt.clone());
        }
    });

    // Owned by this component -- see `AlertDialogAction`'s identical
    // construction above (`dev-docs/backlog.md` row 93).
    let attributes = merge_attributes(vec![
        props.attributes,
        attributes!(button {
            tabindex: if open() { "0" } else { "-1" },
            type: "button",
        }),
    ]);

    rsx! {
        button {
            onclick: on_click,
            ..attributes,
            {props.children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::find_elements;

    fn root_state(app: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        let roots = find_elements(&html, |a| a.contains_key("data-dx-overlay"));
        roots[0].attrs["data-state"].clone()
    }

    #[component]
    fn DefaultOpen() -> Element {
        rsx! {
            AlertDialogRoot { default_open: true }
        }
    }

    #[component]
    fn Closed() -> Element {
        rsx! {
            AlertDialogRoot {}
        }
    }

    #[component]
    fn ControlledClosed() -> Element {
        rsx! {
            AlertDialogRoot { default_open: true, open: Some(false) }
        }
    }

    #[test]
    fn default_open_opens_an_uncontrolled_root() {
        assert_eq!(root_state(DefaultOpen), "open");
        assert_eq!(root_state(Closed), "closed");
    }

    #[test]
    fn a_controlled_open_wins_over_default_open() {
        assert_eq!(root_state(ControlledClosed), "closed");
    }
}
