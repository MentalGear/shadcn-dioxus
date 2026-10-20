//! The one way a form control gets its *default* checkedness, hydration-safe (backlog row 253).
//!
//! A control has two checkednesses: the live `.checked` property, and the default -- the `checked`
//! content attribute / `.defaultChecked` -- which the HTML `<form>` reset algorithm restores from
//! and which a non-scripted form post reads. Dioxus 0.7.9 offers `initial_checked` for the default,
//! and it only works when the browser *creates* the node from a mutation
//! (`dioxus-interpreter-js` `set_attribute.ts`, `case "initial_checked"`). Two places break it:
//!
//! - **SSR** (`dioxus-ssr` `renderer.rs` `write_attribute`) writes any attribute outside its
//!   `BOOL_ATTRS` list as `name=value`, so the server HTML carried a literal `initial_checked=true`
//!   and no `checked` at all.
//! - **Hydration** (`dioxus-web` `hydration/hydrate.rs` `rehydrate_template_node`) only mounts ids
//!   and listeners; it applies no attribute, so the `.defaultChecked` setter never ran either.
//!
//! So a prerendered default-checked input loaded unchecked and `<form reset>` restored the wrong
//! state. The construction: a control renders the plain `checked` attribute (SSR writes it as a
//! content attribute, which the parser turns into `.defaultChecked` and the live state alike; a
//! client-created node gets the live state) and calls [`use_default_checked`], whose effect runs
//! on a client only -- after a fresh render and after hydration alike -- and writes
//! `.defaultChecked` itself. `initial_checked` is never rendered; the unit tests in `checkbox.rs`,
//! `switch.rs` and `radio_group.rs` fail if the server HTML carries any `initial_` attribute.
//!
//! Setting `.defaultChecked` never changes a dirty control's live state, so it cannot fight a
//! controlled `checked` the user has already toggled.

use dioxus::{document::eval, prelude::*};

/// Keeps `.defaultChecked` of the element with the given `id` equal to `default`, on the client.
///
/// Pair it with a plain `checked` attribute on the same input (see the module docs). `id` and
/// `default` are read inside an effect, so a signal passed through either closure re-runs it. The
/// server renders no effect, so this adds nothing to prerendered HTML.
pub fn use_default_checked(id: impl Fn() -> String + 'static, default: impl Fn() -> bool + 'static) {
    use_effect(move || {
        let js = eval(
            r#"
            const id = await dioxus.recv();
            const checked = await dioxus.recv();
            const input = document.getElementById(id);
            if (input) input.defaultChecked = checked;
            "#,
        );
        let _ = js.send(id());
        let _ = js.send(default());
    });
}

/// Shared assertions for the SSR tests of every control that renders a hidden default-checked
/// input (`checkbox.rs`, `switch.rs`, `radio_group.rs`).
#[cfg(test)]
pub(crate) mod test_support {
    use crate::test_support::find_elements;

    /// The hidden mirror `<input>`s of `kind` (`"checkbox"` or `"radio"`) in `html`, as
    /// `(carries the checked content attribute, tag)` in document order. Fails the calling test
    /// when the server HTML carries any `initial_*` attribute: dioxus-ssr writes those as literal
    /// attributes and hydration never applies them (row 253).
    pub(crate) fn hidden_inputs(html: &str, kind: &str) -> Vec<(bool, String)> {
        assert!(
            !html.contains("initial_"),
            "the server HTML carries a literal `initial_*` attribute (row 253): {html}"
        );
        find_elements(html, |a| a.get("type").map(String::as_str) == Some(kind))
            .into_iter()
            .map(|el| (el.attrs.contains_key("checked"), el.tag))
            .collect()
    }
}
