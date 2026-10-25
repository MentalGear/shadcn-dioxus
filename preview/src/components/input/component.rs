use dioxus::prelude::*;
use dioxus_primitives::{dioxus_attributes::attributes, merge_attributes};

/// Wraps a plain `<input>` in this crate's themed styling.
///
/// The base `"dx-input"` class is combined with any caller-supplied
/// `class` (e.g. `InputGroupInput`'s `"dx-input-group-control"`) via
/// [`merge_attributes`] rather than a bare `class: "dx-input"` field
/// followed by `..attributes` -- the latter lets Dioxus's normal
/// "later attribute wins" rule for a duplicate name silently *replace*
/// `"dx-input"` instead of appending to it, dropping this component's own
/// UA-style reset (border/appearance/background) for any caller that
/// passes its own `class`. `merge_attributes` special-cases `class` to
/// concatenate (space-joined) instead, which is what every caller here
/// actually wants: their own class alongside `"dx-input"`, never instead
/// of it. See `preview/src/components/input_group/component.rs` for the
/// consumer that depends on it: without the merge, `InputGroupInput`'s
/// wrapped `<input>` would render with *only* `"dx-input-group-control"`
/// (`"dx-input"` silently dropped), so none of `"dx-input"`'s own
/// border/background reset would apply and the browser's native default
/// input chrome would show through inside `InputGroup`'s own border --  a
/// visible double-border/"pill within a pill" seam.
#[component]
pub fn Input(
    /// Called on every edit of the value (`input`).
    oninput: Option<EventHandler<FormEvent>>,
    /// Called when the value is committed.
    onchange: Option<EventHandler<FormEvent>>,
    /// Called when the value fails the input's constraints (`invalid`).
    oninvalid: Option<EventHandler<FormEvent>>,
    /// Called when text is selected inside the input (`select`).
    onselect: Option<EventHandler<SelectionEvent>>,
    /// Called when the selection inside the input changes (`selectionchange`).
    onselectionchange: Option<EventHandler<SelectionEvent>>,
    /// Called when the input gains focus.
    onfocus: Option<EventHandler<FocusEvent>>,
    /// Called when the input loses focus.
    onblur: Option<EventHandler<FocusEvent>>,
    /// Called on `focusin`, which bubbles.
    onfocusin: Option<EventHandler<FocusEvent>>,
    /// Called on `focusout`, which bubbles.
    onfocusout: Option<EventHandler<FocusEvent>>,
    /// Called when a key is pressed down.
    onkeydown: Option<EventHandler<KeyboardEvent>>,
    /// Called when a key that produces a character is pressed (`keypress`).
    onkeypress: Option<EventHandler<KeyboardEvent>>,
    /// Called when a key is released.
    onkeyup: Option<EventHandler<KeyboardEvent>>,
    /// Called when the mouse wheel or a trackpad scrolls over the input.
    onwheel: Option<EventHandler<WheelEvent>>,
    /// Called when an IME composition starts (`compositionstart`).
    oncompositionstart: Option<EventHandler<CompositionEvent>>,
    /// Called as an IME composition changes (`compositionupdate`).
    oncompositionupdate: Option<EventHandler<CompositionEvent>>,
    /// Called when an IME composition is committed (`compositionend`).
    oncompositionend: Option<EventHandler<CompositionEvent>>,
    /// Called when text is copied from the input.
    oncopy: Option<EventHandler<ClipboardEvent>>,
    /// Called when text is cut from the input.
    oncut: Option<EventHandler<ClipboardEvent>>,
    /// Called when text is pasted into the input.
    onpaste: Option<EventHandler<ClipboardEvent>>,
    #[props(extends=GlobalAttributes)]
    #[props(extends=input)]
    attributes: Vec<Attribute>,
    /// Content placed inside the `<input>`. An input is void, so leave this empty.
    children: Element,
) -> Element {
    let base = attributes!(input { class: "dx-input", "data-dx-root": "" });
    // `onwheel` is attached ONLY when the caller passed one. Dioxus registers every bubbling rsx
    // handler once on `#main` with no options (dioxus-interpreter-js `createListener`), so an
    // `onwheel: move |e| _ = onwheel.map(..)` shim on every `<input>`, set
    // even when the prop is `None`, would make the whole page's wheel input non-passive: each wheel
    // tick would have to wait for the main thread before the compositor could scroll
    // (dev-docs/archive/research/scroll-jank-2026-10-04.md, Cause 2b). An rsx event attribute cannot take
    // an `Option<EventHandler>`, so the optional listener is built as an `Attribute` and spread.
    let wheel: Vec<Attribute> = onwheel
        .map(|handler| vec![dioxus::html::events::onwheel(handler)])
        .unwrap_or_default();
    let merged = merge_attributes(vec![base, attributes, wheel]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/input/style.css") }
        input {
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
            ..merged,
            {children}
        }
    }
}
