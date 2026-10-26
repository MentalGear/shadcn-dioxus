use dioxus::prelude::*;
use dioxus_primitives::AsChild;
use dioxus_icons::lucide::X;
use dioxus_primitives::dialog::{
    self, DialogCtx, DialogDescriptionProps, DialogTitleProps,
};
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::drawer::{use_resolved_edge_side, EdgeSide};
use dioxus_primitives::keyboard_inset::{use_keyboard_panel, KeyboardInset};
use dioxus_primitives::merge_attributes;

/// The side a [`Sheet`] slides in from. One type for both edge panels: this is
/// [`EdgeSide`], the same enum `Drawer` takes (`DrawerSide` is its other alias). The
/// default of the enum is `Auto` (bottom below the `md` breakpoint, right at and above
/// it); `Sheet` itself defaults its `side` prop to `Right`, as shadcn's does.
pub type SheetSide = EdgeSide;

/// The props for the [`Sheet`] component: [`dialog::DialogRootProps`] plus a typed `side`.
#[derive(Props, Clone, PartialEq)]
pub struct SheetProps {
    /// The ID of the sheet root element.
    pub id: ReadSignal<Option<String>>,

    /// Whether the sheet is modal. If true, it traps focus within the sheet when open.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub is_modal: ReadSignal<bool>,

    /// Whether a modal sheet dims the page behind it with the shared overlay scrim.
    /// Defaults to `true`; see [`dialog::DialogRootProps::overlay`].
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub overlay: ReadSignal<bool>,

    /// The controlled `open` state of the sheet.
    pub open: ReadSignal<Option<bool>>,

    /// The default `open` state of the sheet if it is not controlled.
    #[props(default)]
    pub default_open: bool,

    /// A callback that is called when the open state changes.
    #[props(default)]
    pub on_open_change: Callback<bool>,

    /// Which edge the sheet slides in from. Defaults to [`SheetSide::Right`]. A raw
    /// `"data-side"` attribute (the untyped spelling, which `Sidebar` uses)
    /// overrides it.
    #[props(default = ReadSignal::new(Signal::new(SheetSide::Right)))]
    pub side: ReadSignal<SheetSide>,

    /// Base UI's `disablePointerDismissal`: when `true`, a press outside the panel and focus
    /// moving outside it do not close the dialog (a modal's backdrop click included), so the
    /// first press on the page of a non-modal dialog only acts on the page. Escape and the
    /// dialog's own close controls still close it. Defaults to `false`.
    #[props(default = ReadSignal::new(Signal::new(false)))]
    pub disable_pointer_dismissal: ReadSignal<bool>,

    /// Where focus lands when the panel opens, as a function of how it was opened
    /// ([`OpenModality`](dioxus_primitives::modality::OpenModality)). An input to the one open-focus policy
    /// (`dxOpenFocus`), not a second path: with none, keyboard and mouse opens focus the first
    /// control and touch and pen opens focus the panel (no software keyboard). A selector that
    /// matches nothing falls back to that default.
    #[props(default)]
    pub initial_focus: Option<dioxus_primitives::modality::FocusTargetFn>,

    /// Where focus goes when the panel closes, as a function of the modality of the close.
    /// Default: the element that held focus when it opened (the trigger for a click open, the
    /// previously focused element for a programmatic one).
    #[props(default)]
    pub final_focus: Option<dioxus_primitives::modality::FocusTargetFn>,

    /// Additional attributes to apply to the sheet panel element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the sheet.
    pub children: Element,
}

#[component]
pub fn Sheet(props: SheetProps) -> Element {
    let side = use_resolved_edge_side(props.side);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/sheet/style.css") }
        dialog::DialogRoot {
            disable_pointer_dismissal: props.disable_pointer_dismissal,
            class: "dx-sheet-root",
            "data-slot": "sheet-root",
            id: props.id,
            is_modal: props.is_modal,
            overlay: props.overlay,
            open: props.open,
            default_open: props.default_open,
            on_open_change: props.on_open_change,
            SheetPanel {
                side: side(),
                initial_focus: props.initial_focus,
                final_focus: props.final_focus,
                attributes: props.attributes,
                {props.children}
            }
        }
    }
}

/// The panel: an edge panel of the theme's recipe (`data-dx-surface="edge"` plus the resolved
/// `data-side`). It is its own component to read the dialog's open state: the panel carries its
/// own `data-state`, because the recipe's slide is keyed on it and the non-web/non-modal arms
/// of the dialog primitive put none on their content element.
#[component]
fn SheetPanel(
    /// The edge the panel slides in from, already resolved from the sheet's `side`.
    side: SheetSide,
    /// The sheet's `initial_focus`, for the dialog content.
    initial_focus: Option<dioxus_primitives::modality::FocusTargetFn>,
    /// The sheet's `final_focus`, for the dialog content.
    final_focus: Option<dioxus_primitives::modality::FocusTargetFn>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    /// The sheet's content.
    children: Element,
) -> Element {
    let ctx: DialogCtx = use_context();
    // The panel follows the on-screen keyboard (`primitives/src/keyboard_inset.rs`; `style.css`
    // consumes `--dx-keyboard-inset`).
    let panel = use_keyboard_panel(None);
    let content_base = attributes!(div {
        class: "dx-sheet",
        "data-slot": "sheet-content",
        "data-dx-surface": "edge",
        "data-side": side.as_str(),
        "data-state": if ctx.is_open() { "open" } else { "closed" },
    });
    let content_attributes = merge_attributes(vec![content_base, attributes]);

    rsx! {
        dialog::DialogContent {
            id: panel.id(),
            class: None,
            initial_focus,
            final_focus,
            attributes: content_attributes,
            {children}
            KeyboardInset { panel }
        }
    }
}

#[component]
pub fn SheetContentClose(#[props(extends = GlobalAttributes)] attributes: Vec<Attribute>) -> Element {
    // axe `button-name` (dev-docs/backlog.md row 34): this button's
    // only content is the `X` icon, with no text and no accessible name --
    // `DialogClose`/`AlertDialogClose` get theirs per call site
    // (`dialog/variants/main/mod.rs`'s `aria_label: "Close"`); it is baked in
    // here because both call sites
    // (`sheet/variants/main/mod.rs`, `sidebar/component.rs`) render this
    // shared wrapper with no children of their own to derive a name from.
    let base = attributes!(button {
        class: "dx-sheet-close",
        aria_label: "Close",
    });
    let attributes = merge_attributes(vec![base, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/sheet/style.css") }
        SheetClose { attributes,
            // shadcn's close icon is `size-4` (16px) -- style.css's own
            // `.dx-sheet-close` comment has the full rationale for keeping
            // the 24px button box around it as a WCAG 2.5.8 hit target.
            X { size: "16px" }
        }
    }
}

#[component]
pub fn SheetHeader(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    /// The header's content: usually a `SheetTitle` and a `SheetDescription`.
    children: Element,
) -> Element {
    let base = attributes!(div { class: "dx-sheet-header", "data-slot": "sheet-header" });
    let merged = merge_attributes(vec![base, attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/sheet/style.css") }
        div { ..merged, {children} }
    }
}

#[component]
pub fn SheetFooter(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    /// The footer's content, usually buttons.
    children: Element,
) -> Element {
    let base = attributes!(div { class: "dx-sheet-footer", "data-slot": "sheet-footer" });
    let merged = merge_attributes(vec![base, attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/sheet/style.css") }
        div { ..merged, {children} }
    }
}

#[component]
pub fn SheetTitle(props: DialogTitleProps) -> Element {
    let base = attributes!(div { class: "dx-sheet-title", "data-slot": "sheet-title" });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/sheet/style.css") }
        dialog::DialogTitle {
            attributes: merged,
            ..props
        }
    }
}

#[component]
pub fn SheetDescription(props: DialogDescriptionProps) -> Element {
    let base = attributes!(div { class: "dx-sheet-description", "data-slot": "sheet-description" });
    let merged = merge_attributes(vec![base, props.attributes.clone()]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/sheet/style.css") }
        dialog::DialogDescription {
            attributes: merged,
            ..props
        }
    }
}

// No `document::Link` here (unlike this file's other exported components):
// `SheetClose` reads `DialogCtx` via `use_context()` below, so it can only
// ever render as a descendant of a `dialog::DialogRoot` -- and in this file
// that context is provided by `Sheet` alone, which already links the sheet
// stylesheet. A context lookup failure would panic before this component
// could render unstyled, so there's no code path where `SheetClose` reaches
// the DOM without `Sheet`'s own `Link` already in the document head. It also
// has no single `rsx!` block both branches share (the `r#as` branch returns
// straight from the caller's own `Callback`), so adding a `Link` here would
// mean wrapping each branch individually -- unlike this file's other parts,
// that's more than a mechanical one-line insertion for a guarantee the
// context dependency already gives for free.
#[component]
pub fn SheetClose(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    /// Renders the close control as the element the callback returns, instead of a `<button>`. The
    /// callback gets the close handler and attributes, and the children.
    r#as: Option<Callback<AsChild, Element>>,
    /// The close control's content.
    children: Element,
) -> Element {
    let ctx: DialogCtx = use_context();

    let base = attributes! {
        button {
            onclick: move |_| {
                ctx.set_open(false);
            }
        }
    };
    let merged = merge_attributes(vec![base, attributes]);

    if let Some(dynamic) = r#as {
        dynamic.call(AsChild::new(merged, children))
    } else {
        rsx! {
            button { ..merged, {children} }
        }
    }
}
