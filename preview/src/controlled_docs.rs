//! The `/docs` page's "Controlled" section: the one explanation of controlled and uncontrolled props.
//!
//! Every component page whose props hold a controlled pair gets a generated "Controlled state" block that
//! lists its own pairs and links here (`controlled_scan.rs`), so this is the only place the rules are written.

use crate::DocsPre;
use dioxus::prelude::*;

/// The Controlled section's body (the `h2` is rendered by `Docs()`, so it is a `DOCS_SECTIONS` entry with
/// an anchor and a sidebar sub-item like the others).
#[component]
pub(crate) fn ControlledDocs() -> Element {
    rsx! {
        p {
            "A prop that holds state, such as "
            code { "open" }
            ", "
            code { "checked" }
            " or "
            code { "value" }
            ", comes with a callback and a start value: "
            code { "X" }
            ", "
            code { "on_X_change" }
            " and "
            code { "default_X" }
            ". A component's \"Controlled state\" block lists the pairs it actually has."
        }
        ul {
            li {
                strong { "Controlled" }
                ": you pass "
                code { "X" }
                " and an "
                code { "on_X_change" }
                " handler that stores what it reports. The component never keeps its own copy: it shows what you pass, and asks you to change it by calling the handler."
            }
            li {
                strong { "Uncontrolled" }
                ": you pass nothing, or only "
                code { "default_X" }
                ". The component owns the state, starting from that value, and "
                code { "on_X_change" }
                " just tells you what it became."
            }
            li {
                strong { "Forms" }
                ": every change the browser makes by itself (a native form reset, a restored value after navigating back) is reported through the same "
                code { "on_X_change" }
                ", so a controlled component sees it and your signal stays in step. If you handle some other event instead, a reset leaves your state behind the screen."
            }
        }
        p {
            "The usual mistake is a literal where a signal belongs. A literal passed to a controlled prop is the value every render, so the component cannot move:"
        }
        DocsPre {
            code {
                "// Frozen: open is always false, so the dialog never opens.\nDialog {{ open: false, on_open_change: move |_| {{}}, /* ... */ }}\n\n// Controlled: the app's signal is the state.\nlet mut open = use_signal(|| false);\nDialog {{ open: open(), on_open_change: move |next| open.set(next), /* ... */ }}\n\n// Uncontrolled: the component owns it.\nDialog {{ default_open: false, /* ... */ }}"
            }
        }
        p {
            "Pass either "
            code { "X" }
            " or "
            code { "default_X" }
            ", not both: a controlled component ignores its "
            code { "default_X" }
            "."
        }
    }
}
