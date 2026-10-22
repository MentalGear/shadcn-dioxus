//! The Columns menu of a `DataTable` (backlog row 363): shadcn's data-table
//! "Columns" dropdown, a `DropdownMenu` of checkbox items that show and hide
//! columns, plus the keyboard half shadcn has no answer for: **Alt+Up / Alt+Down**
//! on a focused item moves that column one place earlier or later, in the menu
//! and in the table at once (the same `KeyOrder` move rows use; there is no APG
//! pattern for column reorder, so the accessible path is this list, not the
//! header). Same `#[path]` sibling wiring as `state.rs`.

use crate::components::button::{Button, ButtonVariant};
use crate::components::dropdown_menu::{DropdownMenu, DropdownMenuCheckboxItem, DropdownMenuContent, DropdownMenuLabel, DropdownMenuTrigger};
use dioxus::prelude::*;
use dioxus_icons::lucide::ChevronDown;
use dioxus_primitives::dioxus_attributes::attributes;

/// One row of the Columns menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnsMenuItem {
    /// The column's id.
    pub id: String,
    /// The label the menu shows (the column's header text).
    pub label: String,
    /// Whether the column is shown.
    pub visible: bool,
}

/// Props for [`DataTableColumnsMenu`].
#[derive(Props, Clone, PartialEq)]
pub struct DataTableColumnsMenuProps {
    /// The columns the menu offers, in the order they have in the table.
    pub items: Vec<ColumnsMenuItem>,

    /// Fired with the column's id and whether it should now be shown.
    pub on_visible_change: Callback<(String, bool)>,

    /// Fired with the column's id and `-1` (Alt+Up, one place earlier) or `1`
    /// (Alt+Down, one place later). The owner applies the move; the menu keeps
    /// focus on the same column.
    pub on_move: Callback<(String, isize)>,

    /// The trigger's text. Defaults to "Columns".
    #[props(default = "Columns".to_string())]
    pub label: String,
}

/// A "Columns" button that opens a menu of checkbox items, one per column: toggling one shows or
/// hides the column; Alt+Up and Alt+Down on a focused item move it (the owner announces the move).
#[component]
pub fn DataTableColumnsMenu(props: DataTableColumnsMenuProps) -> Element {
    // After a move the items re-render in place (they are keyed by position, so the focused DOM node
    // stays put and now shows another column); focus follows the moved column to its new slot.
    let mut focus_slot = use_signal(|| None::<usize>);
    use_effect(move || {
        let Some(slot) = focus_slot.read().as_ref().copied() else {
            return;
        };
        let _ = document::eval(&format!(
            // focus-ok: the focused menu item stays on the column the user just moved
            "document.querySelectorAll('[data-dx-column-item]')[{slot}]?.focus();"
        ));
        focus_slot.set(None);
    });
    let on_visible_change = props.on_visible_change;
    let on_move = props.on_move;
    let trigger_label = props.label.clone();
    let count = props.items.len();

    rsx! {
        DropdownMenu {
            DropdownMenuTrigger {
                class: "dx-data-table-columns-trigger",
                "data-slot": "data-table-columns-trigger",
                r#as: move |attributes: Vec<Attribute>| rsx! {
                    Button { variant: ButtonVariant::Outline, attributes,
                        "{trigger_label}"
                        ChevronDown { size: "16px", "aria-hidden": "true" }
                    }
                },
            }
            DropdownMenuContent { class: "dx-data-table-columns-menu",
                DropdownMenuLabel { "Toggle columns" }
                for (i , item) in props.items.iter().enumerate() {
                    DropdownMenuCheckboxItem {
                        index: i,
                        checked: Some(item.visible),
                        on_checked_change: {
                            let id = item.id.clone();
                            move |checked: bool| on_visible_change.call((id.clone(), checked))
                        },
                        close_on_select: false,
                        text_value: item.label.clone(),
                        attributes: {
                            let id = item.id.clone();
                            attributes!(div {
                                "data-dx-column-item": id.clone(),
                                aria_keyshortcuts: "Alt+ArrowUp Alt+ArrowDown",
                                onkeydown: move |event: KeyboardEvent| {
                                    if !event.modifiers().alt() {
                                        return;
                                    }
                                    let delta = match event.key() {
                                        Key::ArrowUp => -1,
                                        Key::ArrowDown => 1,
                                        _ => return,
                                    };
                                    // The menu's own arrow handling must not also move focus.
                                    event.prevent_default();
                                    event.stop_propagation();
                                    let target = i as isize + delta;
                                    if target >= 0 && (target as usize) < count {
                                        focus_slot.set(Some(target as usize));
                                    }
                                    on_move.call((id.clone(), delta));
                                },
                            })
                        },
                        "{item.label}"
                    }
                }
            }
        }
    }
}
