use super::super::component::*;
use dioxus::prelude::*;

// Index layout (the submenu is dev-docs/archive/rounds/component-backlog.md row 68):
// Edit=0, Undo=1 (disabled), Duplicate=2, Delete=3, the submenu trigger=4.
// `playwright/context-menu.spec.ts`'s "keyboard navigation" test
// depends on this demo's shape -- Edit first, Undo disabled
// (so a 2nd ArrowDown from Edit skips it and lands on Duplicate) -- so
// those first four items stay in that order; the submenu sits at index 4
// rather than interleaved (unlike `dropdown_menu`'s demo, no context_menu
// spec depends on which item Home/End treat as first/last, so there is no
// ordering constraint on it).
//
// shadcn's own context-menu demo ends with two checkbox items and a radio
// group ("People"), so they follow the same way, at indices 5-8 --
// after everything the specs above pin. They use the primitive's default
// close behaviour (selecting closes the menu, Radix's `onSelect` default), so
// this is the demo that shows it; the `checkboxes` and `radio_group` variants
// show `close_on_select: false`. They are controlled, which a default-closing
// demo has to be: an uncontrolled item's state lives in the item, and the
// content unmounts when the menu closes. Their labels start
// with S/P/C, never D/E, so the typeahead rows in
// `oracle/tier1-apg/keyboard-matrix.spec.ts` ("d" cycling Duplicate/Delete,
// "e" landing on Edit) are unaffected; and a checkable item's role is not
// `menuitem`, so `menu-roles.spec.ts`'s "5 items" count is too.
#[component]
pub fn Demo() -> Element {
    let mut selected_item = use_signal(|| None);
    let mut show_bookmarks = use_signal(|| true);
    let mut show_full_urls = use_signal(|| false);
    let mut person = use_signal(|| "pedro".to_string());

    rsx! {
        ContextMenu {
            ContextMenuTrigger { "right click here" }
            ContextMenuContent {
                ContextMenuItem {
                    value: "edit".to_string(),
                    on_select: move |value| {
                        selected_item.set(Some(value));
                    },
                    "Edit"
                }
                ContextMenuItem {
                    value: "undo".to_string(),
                    disabled: true,
                    on_select: move |value| {
                        selected_item.set(Some(value));
                    },
                    "Undo"
                }
                ContextMenuItem {
                    value: "duplicate".to_string(),
                    on_select: move |value| {
                        selected_item.set(Some(value));
                    },
                    "Duplicate"
                }
                ContextMenuItem {
                    value: "delete".to_string(),
                    on_select: move |value| {
                        selected_item.set(Some(value));
                    },
                    "Delete"
                }
                // Nested submenu (dev-docs/archive/rounds/component-backlog.md row 68): see
                // this file's top-of-file comment for why this sits at
                // index 4 rather than interleaved with the items above.
                ContextMenuSub {
                    ContextMenuSubTrigger { "More tools" }
                    ContextMenuSubContent {
                        ContextMenuSubItem {
                            value: "rename".to_string(),
                            on_select: move |value| {
                                selected_item.set(Some(value));
                            },
                            "Rename"
                        }
                        ContextMenuSubItem {
                            value: "archive".to_string(),
                            on_select: move |value| {
                                selected_item.set(Some(value));
                            },
                            "Archive"
                        }
                    }
                }
                ContextMenuCheckboxItem {
                    checked: Some(show_bookmarks()),
                    on_checked_change: move |checked| show_bookmarks.set(checked),
                    text_value: "Show Bookmarks",
                    "Show Bookmarks"
                }
                ContextMenuCheckboxItem {
                    checked: Some(show_full_urls()),
                    on_checked_change: move |checked| show_full_urls.set(checked),
                    text_value: "Show Full URLs",
                    "Show Full URLs"
                }
                ContextMenuRadioGroup {
                    aria_label: "People",
                    value: Some(person()),
                    on_value_change: move |value| person.set(value),
                    ContextMenuRadioItem {
                        value: "pedro".to_string(),
                        "Pedro Duarte"
                    }
                    ContextMenuRadioItem {
                        value: "colm".to_string(),
                        "Colm Tuite"
                    }
                }
            }
        }

        if let Some(item) = selected_item() {
            span { margin_left: "10px", "Selected: {item}" }
        }
    }
}
