use dioxus::prelude::*;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;

#[derive(Copy, Clone, PartialEq, Default)]
#[non_exhaustive]
pub enum TextareaVariant {
    #[default]
    Default,
    Fade,
    Outline,
    Ghost,
}

impl TextareaVariant {
    pub fn class(&self) -> &'static str {
        match self {
            TextareaVariant::Default => "default",
            TextareaVariant::Fade => "fade",
            TextareaVariant::Outline => "outline",
            TextareaVariant::Ghost => "ghost",
        }
    }
}

#[component]
pub fn Textarea(
    /// Runs on every edit.
    oninput: Option<EventHandler<FormEvent>>,
    /// Runs when the value is committed.
    onchange: Option<EventHandler<FormEvent>>,
    /// Runs when the textarea fails its validation.
    oninvalid: Option<EventHandler<FormEvent>>,
    /// Runs when text in the textarea is selected.
    onselect: Option<EventHandler<SelectionEvent>>,
    /// Runs when the textarea's selection changes.
    onselectionchange: Option<EventHandler<SelectionEvent>>,
    /// Runs when the textarea gains focus.
    onfocus: Option<EventHandler<FocusEvent>>,
    /// Runs when the textarea loses focus.
    onblur: Option<EventHandler<FocusEvent>>,
    /// Runs when focus enters the textarea.
    onfocusin: Option<EventHandler<FocusEvent>>,
    /// Runs when focus leaves the textarea.
    onfocusout: Option<EventHandler<FocusEvent>>,
    /// Runs when a key is pressed down in the textarea.
    onkeydown: Option<EventHandler<KeyboardEvent>>,
    /// Runs on a key press that types a character in the textarea.
    onkeypress: Option<EventHandler<KeyboardEvent>>,
    /// Runs when a key is released in the textarea.
    onkeyup: Option<EventHandler<KeyboardEvent>>,
    /// Runs when an input-method composition starts in the textarea.
    oncompositionstart: Option<EventHandler<CompositionEvent>>,
    /// Runs as an input-method composition changes in the textarea.
    oncompositionupdate: Option<EventHandler<CompositionEvent>>,
    /// Runs when an input-method composition ends in the textarea.
    oncompositionend: Option<EventHandler<CompositionEvent>>,
    /// Runs when text is copied from the textarea.
    oncopy: Option<EventHandler<ClipboardEvent>>,
    /// Runs when text is cut from the textarea.
    oncut: Option<EventHandler<ClipboardEvent>>,
    /// Runs when text is pasted into the textarea.
    onpaste: Option<EventHandler<ClipboardEvent>>,
    /// Runs once the textarea is mounted, with its element.
    onmounted: Option<EventHandler<MountedEvent>>,
    /// The surface style: `Default`, `Fade`, `Outline` or `Ghost` (transparent). Defaults to
    /// `Default`.
    #[props(default)] variant: TextareaVariant,
    #[props(extends=GlobalAttributes)]
    #[props(extends=textarea)]
    attributes: Vec<Attribute>,
) -> Element {
    let base = attributes!(textarea { class: "dx-textarea", "data-dx-root": "", "data-slot": "textarea" });
    // `data-style` reflects this wrapper's own typed `variant` prop, not a
    // caller default -- owned-wins.
    let owned = attributes!(textarea { "data-style": variant.class() });
    let merged = merge_attributes(vec![base, attributes, owned]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/textarea/style.css") }
        textarea {
            oninput: move |e| _ = oninput.map(|callback| callback(e)),
            onchange: move |e| _ = onchange.map(|callback| callback(e)),
            oninvalid: move |e| _ = oninvalid.map(|callback| callback(e)),
            onselect: move |e| _ = onselect.map(|callback| callback(e)),
            onselectionchange: move |e| _ = onselectionchange.map(|callback| callback(e)),
            onfocus: move |e| _ = onfocus.map(|callback| callback(e)),
            onblur: move |e| _ = onblur.map(|callback| callback(e)),
            onfocusin: move |e| _ = onfocusin.map(|callback| callback(e)),
            onfocusout: move |e| _ = onfocusout.map(|callback| callback(e)),
            onkeydown: move |e| _ = onkeydown.map(|callback| callback(e)),
            onkeypress: move |e| _ = onkeypress.map(|callback| callback(e)),
            onkeyup: move |e| _ = onkeyup.map(|callback| callback(e)),
            oncompositionstart: move |e| _ = oncompositionstart.map(|callback| callback(e)),
            oncompositionupdate: move |e| _ = oncompositionupdate.map(|callback| callback(e)),
            oncompositionend: move |e| _ = oncompositionend.map(|callback| callback(e)),
            oncopy: move |e| _ = oncopy.map(|callback| callback(e)),
            oncut: move |e| _ = oncut.map(|callback| callback(e)),
            onpaste: move |e| _ = onpaste.map(|callback| callback(e)),
            onmounted: move |e| _ = onmounted.map(|callback| callback(e)),
            ..merged,
        }
    }
}
