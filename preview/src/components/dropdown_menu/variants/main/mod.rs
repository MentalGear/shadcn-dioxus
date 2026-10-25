use super::super::component::{
    DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSub, DropdownMenuSubContent,
    DropdownMenuSubItem, DropdownMenuSubTrigger, DropdownMenuTrigger,
};
use dioxus::prelude::*;

// dev-docs/backlog.md row 32: no `#[css_module]` of its own here, and no
// `document::Link` needed either -- this `Demo` always renders the
// `DropdownMenu`/`DropdownMenuTrigger`/`DropdownMenuContent`/
// `DropdownMenuItem` themed wrappers below, each of which
// carries its own `document::Link` for `style.css`.
// `document::Link` dedupes on `(href, rel)`, so those links cover this page.
//
// Item order (the submenu is dev-docs/archive/rounds/component-backlog.md row 68):
// Edit, Undo (disabled), Duplicate, the submenu trigger, Delete. Several
// Playwright specs depend on this demo's *shape*, not just its presence:
// `playwright/dropdown-menu.spec.ts` and
// `oracle/tier1-apg/keyboard-matrix.spec.ts` both require Edit first, Undo
// disabled (so a 2nd ArrowDown from Edit skips it and lands on Duplicate),
// and -- keyboard-matrix.spec.ts specifically -- Delete reachable via End
// as the *last* item and as ArrowDown's wrap-around target from it. Roving
// focus follows document order (row 396), so the order above is the order
// the items are written in.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Operation {
    Edit,
    Undo,
    Duplicate,
}

impl Operation {
    const ALL: [Operation; 3] = [Operation::Edit, Operation::Undo, Operation::Duplicate];
}

impl std::fmt::Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

// The nested-submenu-only items (dev-docs/archive/rounds/component-backlog.md row 68), kept
// as their own small enum so the submenu's own indices -- scoped to
// `DropdownMenuSubContent`'s own roving-focus collection, never the root
// menu's -- read as obviously independent of `Operation` above.
#[derive(Debug, Clone, Copy, PartialEq)]
enum ToolOperation {
    Rename,
    Archive,
}

impl ToolOperation {
    const ALL: [ToolOperation; 2] = [ToolOperation::Rename, ToolOperation::Archive];
}

impl std::fmt::Display for ToolOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[component]
pub fn Demo() -> Element {
    let mut selected_operation = use_signal(|| None);

    // `text_value` is set explicitly on every item below (dev-docs/backlog.md
    // row 11, typeahead): `Operation`/`ToolOperation` are plain enums, not
    // `String`/`&str`, so the item components' own value-based typeahead
    // fallback (`crate::selection::option_text_value`) can't derive a label
    // from `value` alone here the way it can for a `String`-valued item --
    // an explicit `text_value` is required for these rows to be
    // typeahead-searchable at all, not just a nicety.
    let operations = Operation::ALL.into_iter().map(|o| {
        rsx! {
            DropdownMenuItem::<Operation> {
                class: "dx-dropdown-menu-item",
                value: o,
                disabled: matches!(o, Operation::Undo),
                text_value: o.to_string(),
                on_select: move |value: Operation| {
                    selected_operation.set(Some(value.to_string()));
                },
                {o.to_string()}
            }
        }
    });

    let tool_operations = ToolOperation::ALL.into_iter().map(|o| {
        rsx! {
            DropdownMenuSubItem::<ToolOperation> {
                value: o,
                text_value: o.to_string(),
                on_select: move |value: ToolOperation| {
                    selected_operation.set(Some(value.to_string()));
                },
                {o.to_string()}
            }
        }
    });

    rsx! {
        DropdownMenu { class: "dx-dropdown-menu", default_open: false,
            DropdownMenuTrigger { class: "dx-dropdown-menu-trigger", "Open Menu" }
            DropdownMenuContent { class: "dx-dropdown-menu-content",
                {operations}
                // Nested submenu (dev-docs/archive/rounds/component-backlog.md row 68),
                // between Duplicate and Delete: see this file's top-of-file comment.
                DropdownMenuSub {
                    DropdownMenuSubTrigger {
                        text_value: "More tools",
                        "More tools"
                    }
                    DropdownMenuSubContent { {tool_operations} }
                }
                DropdownMenuItem::<String> {
                    class: "dx-dropdown-menu-item",
                    value: "Delete".to_string(),
                    text_value: "Delete",
                    on_select: move |value: String| {
                        selected_operation.set(Some(value));
                    },
                    "Delete"
                }
            }
        }
        if let Some(op) = selected_operation() {
            "Selected: {op}"
        }
    }
}
