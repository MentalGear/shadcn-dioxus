//! Reorderable rows for `DataTable` (backlog row 363, `dashboard-01` parity): a
//! leading drag-handle column whose gesture is the library's one reorder gesture,
//! `dioxus_primitives::drag_and_drop_list::reorder` (the `ReorderState` machine,
//! its key map and its announcements, the same ones `DragAndDropList` drives), and
//! whose result is the one manual order, `ViewState::move_row` on the row keys.
//! Nothing here orders or drags on its own: it feeds the shared machine from a
//! handle button's pointer and key events and applies the `(from, to)` it resolves.
//!
//! The handle is a real `<button draggable>`: pointer-drag it onto another row, or
//! use the keyboard exactly as in `DragAndDropList` (Space or Enter lifts, Up and
//! Down choose the slot, Space or Enter drops, Escape cancels; with nothing lifted
//! Up, Down, Home and End move focus between the handles). Slots are positions on
//! the visible page; the move is applied next to the neighbouring visible row, so
//! rows a filter hides keep their relative place. A sort wins over a manual order,
//! so while one is active the handles are disabled.
//!
//! Same `#[path]` sibling wiring as `state.rs`.

use super::columns::DataTableState;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::table::TableRow;
use dioxus::prelude::*;
use dioxus_icons::lucide::GripVertical;
use dioxus_primitives::data_view::{KeyOrder, RowAnchor};
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::direction::Direction;
use dioxus_primitives::drag_and_drop_list::reorder::{
    announce_cancel, announce_drop, announce_lift, announce_move, command_for_key, pointer_position, DropPosition,
    ReorderAxis, ReorderCommand, ReorderState,
};
use dioxus_primitives::merge_attributes;
use dioxus_primitives::table_rows::TableScroller;

/// The instructions a handle is described by (the text `DragAndDropList` shows its list).
pub(crate) const INSTRUCTIONS: &str =
    "Press Space to start reordering. Use the Up and Down arrow keys to change position. Press Space to confirm or Escape to cancel.";

/// A resolved row move: `key` lands next to `anchor`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MovePlan {
    pub key: String,
    pub anchor: String,
    pub after: bool,
}

impl MovePlan {
    pub(crate) fn anchor(&self) -> RowAnchor<'_> {
        if self.after {
            RowAnchor::After(&self.anchor)
        } else {
            RowAnchor::Before(&self.anchor)
        }
    }
}

/// What dropping the row at page slot `from` onto slot `to` means for the manual
/// order: the row lands directly before the visible row that follows it, or, when
/// it becomes the last on the page, directly after the one that precedes it, so
/// rows the page does not show stay where they were. `None` for a no-op.
pub(crate) fn plan_move(page_keys: &[String], from: usize, to: usize) -> Option<MovePlan> {
    if from == to {
        return None;
    }
    let key = page_keys.get(from)?.clone();
    let mut moved = KeyOrder::new(page_keys.iter().cloned());
    moved.move_key(&key, to);
    let keys = moved.keys();
    let to = keys.iter().position(|k| *k == key)?;
    match keys.get(to + 1) {
        Some(next) => Some(MovePlan { key, anchor: next.clone(), after: false }),
        None => {
            let previous = keys.get(to.checked_sub(1)?)?.clone();
            Some(MovePlan { key, anchor: previous, after: true })
        }
    }
}

/// The reorder gesture of one table, shared with its rows and handles through context.
#[derive(Clone, Copy)]
pub(crate) struct RowReorder {
    gesture: Signal<ReorderState>,
    announcement: Signal<String>,
    focus_key: Signal<Option<String>>,
    page_keys: Signal<Vec<String>>,
    sorted: Signal<bool>,
    table: DataTableState,
}

/// Creates the table's [`RowReorder`] and provides it to the rows and handles.
pub(crate) fn use_row_reorder(table: DataTableState) -> RowReorder {
    let reorder = RowReorder {
        gesture: use_signal(|| ReorderState::Idle),
        announcement: use_signal(String::new),
        focus_key: use_signal(|| None),
        page_keys: use_signal(Vec::new),
        sorted: use_signal(|| false),
        table,
    };
    use_context_provider(move || reorder);
    // Focus returns to a handle by row key once the rows have re-rendered: a keyed row that moves is
    // re-inserted by the DOM and the browser drops its focus on the way.
    let mut focus_key = reorder.focus_key;
    use_effect(move || {
        let Some(key) = focus_key.read().clone() else {
            return;
        };
        let eval = document::eval(
            r#"const key = await dioxus.recv();
            // focus-ok: returns focus to the handle of the row that was just moved or stepped to
            document.querySelector('[data-dx-reorder-handle="' + CSS.escape(key) + '"]')?.focus();"#,
        );
        let _ = eval.send(key);
        focus_key.set(None);
    });
    reorder
}

impl RowReorder {
    /// What the table shows now: the page's keys and whether a sort is active.
    pub(crate) fn sync(&self, page_keys: &[String], sorted: bool) {
        let (mut keys, mut flag) = (self.page_keys, self.sorted);
        if *keys.peek() != page_keys {
            keys.set(page_keys.to_vec());
        }
        if *flag.peek() != sorted {
            flag.set(sorted);
        }
    }

    pub(crate) fn announcement(&self) -> String {
        (self.announcement)()
    }

    fn is_sorted(&self) -> bool {
        (self.sorted)()
    }

    fn count(&self) -> usize {
        self.page_keys.peek().len()
    }

    fn lifted(&self) -> bool {
        (self.gesture)().is_lifted()
    }

    /// `data-reorder` / `data-drop` for the row at `index`: is it the lifted one, and which side of it
    /// the drop line is on.
    fn marks(&self, index: usize) -> (bool, Option<&'static str>) {
        let gesture = (self.gesture)();
        let source = gesture.from() == Some(index);
        let drop = match (gesture.to() == Some(index), gesture.position()) {
            (true, DropPosition::Before) => Some("before"),
            (true, DropPosition::After) => Some("after"),
            _ => None,
        };
        (source, drop)
    }

    fn start_drag(&mut self, index: usize) {
        self.gesture.set(ReorderState::lift(index));
    }

    fn hover(&mut self, index: usize, position: DropPosition) {
        let next = (self.gesture)().hover(index, position);
        self.gesture.set(next);
    }

    fn step(&mut self, index: usize, delta: isize) {
        let len = self.count();
        let next = (self.gesture)().step(index, delta, len);
        self.gesture.set(next);
        self.announcement.set(announce_move(next.to().unwrap_or(index), len));
    }

    /// Applies the lifted row's chosen slot to the manual order (a second call of one gesture, or one
    /// with no slot chosen, applies nothing) and returns whether the gesture had a drop to apply.
    fn commit(&mut self) -> Option<(usize, usize)> {
        let (next, moved) = (self.gesture)().drop_item();
        let (from, to) = moved?;
        self.gesture.set(next);
        let keys = self.page_keys.peek().clone();
        if let Some(plan) = plan_move(&keys, from, to) {
            self.table.move_row(&plan.key, plan.anchor());
        }
        Some((from, to))
    }

    /// The pointer dropped on the table body: apply the chosen slot and end the gesture (the handle's own
    /// `dragend` may never reach a node the move has re-inserted).
    pub(crate) fn drop_pointer(&mut self) {
        self.commit();
        self.end();
    }

    /// Announces `message` in the table's live region.
    pub(crate) fn announce(&mut self, message: String) {
        self.announcement.set(message);
    }

    fn end(&mut self) {
        let lifted_key = (self.gesture)().from().and_then(|from| self.page_keys.peek().get(from).cloned());
        self.gesture.set(ReorderState::Idle);
        self.focus_key.set(lifted_key);
    }

    /// Space or Enter: lift the row, or drop the lifted one (keyboard).
    fn toggle(&mut self, index: usize) {
        if self.lifted() {
            let from = (self.gesture)().from().unwrap_or(index);
            let to = (self.gesture)().to().unwrap_or(index);
            self.commit();
            self.end();
            self.announcement.set(announce_drop(from, to));
        } else {
            self.start_drag(index);
            self.hover(index, DropPosition::Undefined);
            self.announcement.set(announce_lift(index, self.count()));
        }
    }

    fn cancel(&mut self) {
        let from = (self.gesture)().from().unwrap_or(0);
        self.end();
        self.announcement.set(announce_cancel(from));
    }

    /// Moves focus to the handle of the row at `slot` (clamped), nothing lifted.
    fn focus_slot(&mut self, slot: isize) {
        let keys = self.page_keys.peek();
        let Some(last) = keys.len().checked_sub(1) else {
            return;
        };
        let key = keys.get(slot.clamp(0, last as isize) as usize).cloned();
        drop(keys);
        self.focus_key.set(key);
    }
}

/// Props for [`ReorderableRow`].
#[derive(Props, Clone, PartialEq)]
pub(crate) struct ReorderableRowProps {
    /// The row's slot on the page.
    pub index: usize,
    /// Whether the row is selected.
    pub selected: bool,
    /// The row's cells, the handle cell first.
    pub children: Element,
}

/// A body row that is also a drop target of the table's reorder gesture: it reports where the
/// pointer is over it (before or after its midpoint) and marks itself as the lifted row or as
/// the one the drop line is drawn on.
#[component]
pub(crate) fn ReorderableRow(props: ReorderableRowProps) -> Element {
    let mut reorder: RowReorder = use_context();
    let index = props.index;
    let mut mounted: Signal<Option<std::rc::Rc<MountedData>>> = use_signal(|| None);
    let (source, drop) = reorder.marks(index);
    let owned = attributes!(tr {
        "data-reorder": source.then_some("source"),
        "data-drop": drop,
    });
    let events = attributes!(tr {
        onmounted: move |event: MountedEvent| mounted.set(Some(event.data())),
        ondragover: move |event: DragEvent| {
            event.prevent_default();
            event.data_transfer().set_drop_effect("move");
            async move {
                let Some(row) = mounted() else {
                    return;
                };
                if let Ok(rect) = row.get_client_rect().await {
                    let position =
                        pointer_position(ReorderAxis::Vertical, Direction::Ltr, event.client_coordinates().y, rect.origin.y, rect.size.height);
                    reorder.hover(index, position);
                }
            }
        },
    });
    let merged = merge_attributes(vec![events, owned]);
    rsx! {
        TableRow { selected: props.selected, attributes: merged, {props.children} }
    }
}

/// Props for [`RowHandle`].
#[derive(Props, Clone, PartialEq)]
pub(crate) struct RowHandleProps {
    /// The row's slot on the page.
    pub index: usize,
    /// The row's key (what focus returns to by identity).
    pub row_key: String,
}

/// The drag handle of one row: a ghost icon `Button` that is the gesture's pointer source and its
/// keyboard control. `aria-pressed` is whether the row is lifted; the instructions are its description.
#[component]
pub(crate) fn RowHandle(props: RowHandleProps) -> Element {
    let mut reorder: RowReorder = use_context();
    let scroller: TableScroller = use_context();
    let index = props.index;
    let hint_id = format!("{}-reorder-hint-{index}", scroller.id.read());
    let disabled = reorder.is_sorted();
    let lifted = reorder.lifted() && (reorder.gesture)().from() == Some(index);
    let onkeydown = move |event: KeyboardEvent| {
        let Some(command) = command_for_key(&event.key(), ReorderAxis::Vertical, Direction::Ltr) else {
            return;
        };
        let lifted = reorder.lifted();
        match command {
            ReorderCommand::Toggle => reorder.toggle(index),
            ReorderCommand::Step(delta) if lifted => reorder.step(index, delta),
            ReorderCommand::Step(delta) => reorder.focus_slot(index as isize + delta),
            ReorderCommand::Cancel if lifted => reorder.cancel(),
            ReorderCommand::First if !lifted => reorder.focus_slot(0),
            ReorderCommand::Last if !lifted => reorder.focus_slot(isize::MAX),
            _ => return,
        }
        event.prevent_default();
    };
    let handle = attributes!(button {
        class: "dx-data-table-handle",
        "data-dx-reorder-handle": props.row_key.clone(),
        draggable: if disabled { "false" } else { "true" },
        aria_roledescription: "drag handle",
        aria_label: "Reorder row {props.row_key}",
        aria_describedby: hint_id.clone(),
        aria_pressed: lifted.to_string(),
        title: disabled.then_some("Clear the sort to reorder rows"),
        ondragstart: move |event: DragEvent| {
            reorder.start_drag(index);
            event.data_transfer().set_effect_allowed("move");
            event.data_transfer().set_drop_effect("move");
            // Firefox starts no drag without data on the transfer.
            let _ = event.data_transfer().set_data("text/html", "");
        },
        ondragend: move |_| reorder.end(),
    });
    rsx! {
        Button {
            variant: ButtonVariant::Ghost,
            size: ButtonSize::IconSm,
            disabled,
            attributes: handle,
            onkeydown,
            GripVertical { class: "dx-data-table-handle-icon", "aria-hidden": "true", size: "16px" }
        }
        span { id: "{hint_id}", class: "dx-data-table-sr-only", {INSTRUCTIONS} }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(ids: &[&str]) -> Vec<String> {
        ids.iter().map(|k| k.to_string()).collect()
    }

    #[test]
    fn a_move_lands_before_the_row_that_now_follows_it() {
        let plan = plan_move(&keys(&["a", "b", "c", "d"]), 3, 1).unwrap();
        assert_eq!(plan, MovePlan { key: "d".into(), anchor: "b".into(), after: false });
        let plan = plan_move(&keys(&["a", "b", "c", "d"]), 0, 1).unwrap();
        assert_eq!(plan, MovePlan { key: "a".into(), anchor: "c".into(), after: false });
    }

    #[test]
    fn a_move_to_the_end_of_the_page_lands_after_the_row_before_it() {
        let plan = plan_move(&keys(&["a", "b", "c"]), 0, 2).unwrap();
        assert_eq!(plan, MovePlan { key: "a".into(), anchor: "c".into(), after: true });
    }

    #[test]
    fn dropping_where_it_was_moves_nothing() {
        assert_eq!(plan_move(&keys(&["a", "b"]), 1, 1), None);
        assert_eq!(plan_move(&keys(&["a", "b"]), 5, 0), None);
        assert_eq!(plan_move(&keys(&["a"]), 0, 0), None);
    }
}
