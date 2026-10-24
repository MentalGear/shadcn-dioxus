//! The one way a form control gets its *default* checkedness, hydration-safe (backlog row 253).
//!
//! A control has two checkednesses: the live `.checked` property, and the default -- the `checked`
//! content attribute / `.defaultChecked` -- which the HTML `<form>` reset algorithm restores from
//! and which a non-scripted form post reads. Dioxus 0.7.9 offers `initial_checked` for the default,
//! and it only works when the browser *creates* the node from a mutation
//! (`dioxus-interpreter-js` `set_attribute.ts`, `case "initial_checked"`). Two places break it:
//!
//! - **SSR** (`dioxus-ssr` `renderer.rs` `write_attribute`) writes any attribute outside its
//!   `BOOL_ATTRS` list as `name=value`, so the server HTML would carry a literal `initial_checked=true`
//!   and no `checked` at all.
//! - **Hydration** (`dioxus-web` `hydration/hydrate.rs` `rehydrate_template_node`) only mounts ids
//!   and listeners; it applies no attribute, so the `.defaultChecked` setter never ran either.
//!
//! So a prerendered default-checked input would load unchecked and `<form reset>` would restore the wrong
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
pub fn use_default_checked(
    id: impl Fn() -> String + 'static,
    default: impl Fn() -> bool + 'static,
) {
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

#[cfg(test)]
mod tests {
    /// No primitive may render a Dioxus `initial_*` attribute (`initial_checked`, `initial_value`,
    /// `initial_selected`): `dioxus-ssr` writes it as a literal attribute and `dioxus-web`
    /// hydration never applies it, so the default it carries is lost on every prerendered page
    /// (backlog row 253). A default is the plain content attribute plus the client-side
    /// `use_default_checked`. Each allowlist entry is `(file, reason)`; a stale one fails.
    #[test]
    fn no_primitive_renders_an_initial_attribute() {
        const ALLOW: &[(&str, &str)] = &[(
            "select/components/select.rs",
            "the hidden <select>'s options also render `selected`, which SSR writes as a content \
             attribute, so the server HTML is right; the literal `initial_selected` is inert junk \
             (row 253 follow-up: a `use_default_selected` twin of this module removes it)",
        )];
        fn walk(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
            for entry in std::fs::read_dir(dir).unwrap().flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, out);
                } else if path.extension().is_some_and(|e| e == "rs") {
                    out.push(path);
                }
            }
        }
        let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut files = Vec::new();
        walk(&src, &mut files);
        let mut hits = Vec::new();
        let mut used = Vec::new();
        for file in files {
            let rel = file
                .strip_prefix(&src)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            if rel == "default_checked.rs" {
                continue; // this guard names the attributes it forbids
            }
            let text = std::fs::read_to_string(&file).unwrap();
            // An rsx attribute is `initial_xxx:` outside a comment (docs and `//` notes name them).
            let has_attr = text.lines().any(|line| {
                let code = line.trim_start();
                !code.starts_with("//")
                    && ["initial_checked:", "initial_value:", "initial_selected:"]
                        .iter()
                        .any(|attr| code.contains(attr))
            });
            if !has_attr {
                continue;
            }
            match ALLOW.iter().find(|(allowed, _)| *allowed == rel) {
                Some(_) => used.push(rel),
                None => hits.push(rel),
            }
        }
        assert!(
            hits.is_empty(),
            "`initial_*` attribute rendered in {hits:?} (row 253, default_checked.rs)"
        );
        for (allowed, _) in ALLOW {
            assert!(
                used.iter().any(|u| u == allowed),
                "stale allowlist entry {allowed}"
            );
        }
    }
}
