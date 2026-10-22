use crate::components::input::Input;
use crate::components::textarea::{Textarea, TextareaVariant};
use dioxus::prelude::*;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;

/// Which edge of the group an [`InputGroupAddon`] sits against. `Start` / `End` are the inline
/// edges (beside the control); `BlockStart` / `BlockEnd` are full-width rows above / below it
/// (shadcn's `block-start` / `block-end`, e.g. a composer's toolbar under its text).
#[derive(Copy, Clone, PartialEq, Default)]
#[non_exhaustive]
pub enum InputGroupAlign {
    #[default]
    Start,
    End,
    BlockStart,
    BlockEnd,
}

impl InputGroupAlign {
    pub fn class(&self) -> &'static str {
        match self {
            InputGroupAlign::Start => "start",
            InputGroupAlign::End => "end",
            InputGroupAlign::BlockStart => "block-start",
            InputGroupAlign::BlockEnd => "block-end",
        }
    }
}

/// Wraps an [`InputGroupInput`] with leading/trailing
/// [`InputGroupAddon`]s (icons, buttons, static text) inside one bordered
/// control -- e.g. a search field with a leading search icon, or an amount
/// field with a trailing "USD" chip. Plain layout, no ARIA widget role, no
/// primitive underneath.
#[component]
pub fn InputGroup(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(div {
        class: "dx-input-group", "data-dx-root": ""
    });
    let merged = merge_attributes(vec![base, attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/input_group/style.css") }
        div { ..merged, {children} }
    }
}

/// The group's actual `<input>`, composing the themed
/// [`Input`] so the group inherits its
/// event wiring and focus/placeholder styling; its own border/background
/// are flattened by this component's stylesheet so only the group's outer
/// edge shows a border.
#[component]
pub fn InputGroupInput(
    oninput: Option<EventHandler<FormEvent>>,
    onchange: Option<EventHandler<FormEvent>>,
    onfocus: Option<EventHandler<FocusEvent>>,
    onblur: Option<EventHandler<FocusEvent>>,
    #[props(extends = GlobalAttributes)]
    #[props(extends = input)]
    attributes: Vec<Attribute>,
) -> Element {
    let base = attributes!(input {
        class: "dx-input-group-control",
    });
    let merged = merge_attributes(vec![base, attributes]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/input_group/style.css") }
        Input {
            oninput,
            onchange,
            onfocus,
            onblur,
            attributes: merged,
        }
    }
}

/// The group's multi-line control (shadcn's `InputGroupTextarea`), composing the themed
/// [`Textarea`]. Its own fill, edge and resize handle are dropped (`data-bare`, textarea's
/// sheet), so only the group's outer edge draws the chrome; put a `BlockEnd` addon under it for
/// a footer (a character count, a send button). Any other element can be the group's control by
/// wearing the `dx-input-group-control` class: the group's hover, focus, invalid and disabled
/// states follow it.
#[component]
pub fn InputGroupTextarea(
    /// Runs on every edit.
    oninput: Option<EventHandler<FormEvent>>,
    /// Runs when the value is committed.
    onchange: Option<EventHandler<FormEvent>>,
    /// Runs when the textarea gains focus.
    onfocus: Option<EventHandler<FocusEvent>>,
    /// Runs when the textarea loses focus.
    onblur: Option<EventHandler<FocusEvent>>,
    /// Runs on a key press in the textarea.
    onkeydown: Option<EventHandler<KeyboardEvent>>,
    #[props(extends = GlobalAttributes)]
    #[props(extends = textarea)]
    attributes: Vec<Attribute>,
) -> Element {
    let base = attributes!(textarea { class: "dx-input-group-control" });
    // `data-bare` is this wrapper's own marker (it takes the group's chrome), not a caller default.
    let owned = attributes!(textarea { "data-bare": "" });
    let merged = merge_attributes(vec![base, attributes, owned]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/input_group/style.css") }
        Textarea {
            variant: TextareaVariant::Default,
            oninput,
            onchange,
            onfocus,
            onblur,
            onkeydown,
            attributes: merged,
        }
    }
}

/// A leading or trailing slot inside an [`InputGroup`] -- icons, buttons,
/// or static text that sit flush with the input's border.
#[component]
pub fn InputGroupAddon(
    #[props(default)] align: InputGroupAlign,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(div {
        class: "dx-input-group-addon"
    });
    // `data-align` reflects this wrapper's own typed `align` prop, not a
    // caller default -- owned-wins.
    let owned = attributes!(div { "data-align": align.class() });
    let merged = merge_attributes(vec![base, attributes, owned]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/input_group/style.css") }
        div { ..merged, {children} }
    }
}

/// Static, non-interactive text inside an [`InputGroupAddon`] -- e.g. a
/// currency symbol or unit.
#[component]
pub fn InputGroupText(
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let base = attributes!(span {
        class: "dx-input-group-text"
    });
    let merged = merge_attributes(vec![base, attributes]);
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/input_group/style.css") }
        span { ..merged, {children} }
    }
}
