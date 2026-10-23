use super::super::component::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::checkbox::Checkbox;
use crate::components::label::Label;
use dioxus::prelude::*;
use dioxus_icons::lucide::GripVertical;
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::direction::Direction;
use dioxus_primitives::drag_and_drop_list::reorder::{
    announce_cancel, announce_drop, announce_lift, announce_move, command_for_key,
    pointer_position, DropPosition, ReorderAxis, ReorderCommand, ReorderState,
};

const COUNT: usize = 200;

const INSTRUCTIONS: &str = "Press Space to start reordering. Use the Up and Down arrow keys to change position. Press Space to confirm or Escape to cancel.";

/// Every third task carries a second line, so row heights differ and the list measures them.
fn has_detail(id: usize) -> bool {
    id.is_multiple_of(3)
}

/// The reorder gesture of the list and what the rows share: the order (the data), the gesture
/// (the library's `ReorderState`, the one `DragAndDropList` and `DataTable` drive), the live-region
/// text and the item whose handle takes focus once the rows have re-rendered.
#[derive(Clone, Copy, PartialEq)]
struct Reorder {
    order: Signal<Vec<usize>>,
    gesture: Signal<ReorderState>,
    announcement: Signal<String>,
    focus_id: Signal<Option<usize>>,
}

impl Reorder {
    /// Applies the lifted row's chosen slot to the order and returns whether anything was dropped.
    fn commit(&mut self) {
        let (next, moved) = (self.gesture)().drop_item();
        if let Some((from, to)) = moved {
            self.gesture.set(next);
            self.order.with_mut(|o| {
                if from < o.len() && to < o.len() {
                    let item = o.remove(from);
                    o.insert(to, item);
                }
            });
        }
    }

    /// The task that is lifted now (read before a drop moves it).
    fn lifted_id(&self) -> Option<usize> {
        (self.gesture)()
            .from()
            .and_then(|from| self.order.peek().get(from).copied())
    }

    /// Ends the gesture; focus goes back to the handle of the task that was lifted.
    fn end(&mut self, id: Option<usize>) {
        self.gesture.set(ReorderState::Idle);
        self.focus_id.set(id);
    }
}

#[derive(Props, Clone, PartialEq)]
struct RowProps {
    reorder: Reorder,
    idx: usize,
    id: usize,
}

/// One task: a drag handle, a checkbox, a label (and for some a second line). The row is also a
/// drop target of the gesture and draws the drop line.
#[component]
fn Row(props: RowProps) -> Element {
    let mut reorder = props.reorder;
    let (idx, id) = (props.idx, props.id);
    let mut mounted: Signal<Option<std::rc::Rc<MountedData>>> = use_signal(|| None);
    let gesture = (reorder.gesture)();
    let source = gesture.from() == Some(idx);
    let drop = match (gesture.to() == Some(idx), gesture.position()) {
        (true, DropPosition::Before) => Some("before"),
        (true, DropPosition::After) => Some("after"),
        _ => None,
    };
    let lifted = gesture.is_lifted() && source;
    let len = COUNT;
    let onkeydown = move |event: KeyboardEvent| {
        let Some(command) = command_for_key(&event.key(), ReorderAxis::Vertical, Direction::Ltr)
        else {
            return;
        };
        let lifted = (reorder.gesture)().is_lifted();
        match command {
            ReorderCommand::Toggle if lifted => {
                let from = (reorder.gesture)().from().unwrap_or(idx);
                let to = (reorder.gesture)().to().unwrap_or(idx);
                let id = reorder.lifted_id();
                reorder.commit();
                reorder.end(id);
                reorder.announcement.set(announce_drop(from, to));
            }
            ReorderCommand::Toggle => {
                reorder.gesture.set(ReorderState::lift(idx));
                let next = (reorder.gesture)().hover(idx, DropPosition::Undefined);
                reorder.gesture.set(next);
                reorder.announcement.set(announce_lift(idx, len));
            }
            ReorderCommand::Step(delta) if lifted => {
                let next = (reorder.gesture)().step(idx, delta, len);
                reorder.gesture.set(next);
                reorder
                    .announcement
                    .set(announce_move(next.to().unwrap_or(idx), len));
            }
            ReorderCommand::Step(delta) => {
                let slot = (idx as isize + delta).clamp(0, len as isize - 1) as usize;
                let target = reorder.order.peek().get(slot).copied();
                reorder.focus_id.set(target);
            }
            ReorderCommand::Cancel if lifted => {
                let from = (reorder.gesture)().from().unwrap_or(0);
                let id = reorder.lifted_id();
                reorder.end(id);
                reorder.announcement.set(announce_cancel(from));
            }
            _ => return,
        }
        event.prevent_default();
    };
    let handle = attributes!(button {
        class: "dx-vl-reorder-handle",
        "data-item-button": "{id}",
        draggable: "true",
        aria_roledescription: "drag handle",
        aria_label: "Reorder task {id}",
        aria_describedby: "vl-reorder-hint",
        aria_pressed: lifted.to_string(),
        ondragstart: move |event: DragEvent| {
            reorder.gesture.set(ReorderState::lift(idx));
            event.data_transfer().set_effect_allowed("move");
            event.data_transfer().set_drop_effect("move");
            // Firefox starts no drag without data on the transfer.
            let _ = event.data_transfer().set_data("text/html", "");
        },
        ondragend: move |_| {
            let id = reorder.lifted_id();
            reorder.end(id);
        },
    });
    rsx! {
        div {
            class: "dx-vl-reorder-row",
            "data-item": "{id}",
            "data-reorder": source.then_some("source"),
            "data-drop": drop,
            onmounted: move |event: MountedEvent| mounted.set(Some(event.data())),
            ondragover: move |event: DragEvent| {
                event.prevent_default();
                event.data_transfer().set_drop_effect("move");
                async move {
                    let Some(row) = mounted() else {
                        return;
                    };
                    if let Ok(rect) = row.get_client_rect().await {
                        let position = pointer_position(
                            ReorderAxis::Vertical,
                            Direction::Ltr,
                            event.client_coordinates().y,
                            rect.origin.y,
                            rect.size.height,
                        );
                        let next = (reorder.gesture)().hover(idx, position);
                        reorder.gesture.set(next);
                    }
                }
            },
            ondrop: move |event: DragEvent| {
                event.prevent_default();
                let id = reorder.lifted_id();
                reorder.commit();
                reorder.end(id);
            },
            Button {
                variant: ButtonVariant::Ghost,
                size: ButtonSize::IconSm,
                attributes: handle,
                onkeydown,
                GripVertical { class: "dx-vl-reorder-grip", "aria-hidden": "true", size: "16px" }
            }
            Checkbox { id: "vl-task-{id}", "data-item-check": "{id}" }
            div { class: "dx-vl-reorder-text",
                Label { html_for: "vl-task-{id}", "Task {id}" }
                if has_detail(id) {
                    span { class: "dx-vl-reorder-detail", "Due in {id % 7 + 1} days, assigned to the platform team." }
                }
            }
        }
    }
}

/// The list and the order it shows. `item_key` says who is at an index, so focus, a row's own state
/// (the checkbox) and the measured heights follow the task when the order changes.
#[component]
fn Rows(reorder: Reorder) -> Element {
    let order = reorder.order;
    let render_item = move |idx: usize| {
        let id = order.read()[idx];
        rsx! {
            Row { reorder, idx, id }
        }
    };
    rsx! {
        VirtualList {
            count: COUNT,
            buffer: 6usize,
            virtualize: VirtualizeMode::Windowed,
            estimate_size: |_idx| 45,
            item_key: move |idx: usize| format!("task-{}", order.read()[idx]),
            "aria-label": "Tasks",
            render_item,
        }
    }
}

/// A sortable task list of 200 rows: drag a grip (or Space, the arrows, Space) to reorder, or use
/// the buttons to reverse or shuffle in one click. The order is data and `item_key` says who is at
/// an index, so focus, a row's own state (the checkbox) and the measured heights follow the task
/// when the order changes (what `item_key` changes is described in the Virtual List docs).
#[component]
pub fn Demo() -> Element {
    let order = use_signal(|| (0..COUNT).collect::<Vec<usize>>());
    let gesture = use_signal(|| ReorderState::Idle);
    let announcement = use_signal(String::new);
    let focus_id = use_signal(|| None::<usize>);
    let reorder = Reorder {
        order,
        gesture,
        announcement,
        focus_id,
    };
    let mut order_mut = order;
    // A deterministic shuffle, so the demo (and its spec) behave the same every time.
    let shuffle = move |_| {
        order_mut.with_mut(|o| {
            let mut x: usize = 0x9E3779B9;
            for i in (1..o.len()).rev() {
                x = x.wrapping_mul(1103515245).wrapping_add(12345);
                o.swap(i, (x >> 8) % (i + 1));
            }
        })
    };
    // Focus returns to a handle by task id once the rows have re-rendered: a keyed row that moves is
    // re-inserted by the DOM and the browser drops its focus on the way.
    use_effect(move || {
        let Some(id) = focus_id.read().to_owned() else {
            return;
        };
        let eval = document::eval(
            r#"const id = await dioxus.recv();
            // focus-ok: returns focus to the handle of the task that was just moved or stepped to
            document.querySelector('[data-item-button="' + id + '"]')?.focus();"#,
        );
        let _ = eval.send(id);
    });
    rsx! {
        div { class: "dx-virtual-list-demo", "data-vl-reorder-demo": "",
            style { r#".dx-virtual-list-container {{
  position: relative;
  max-height: 24rem;
  contain: layout paint;
  overflow-y: auto;
}}

.dx-virtual-list-demo {{
  display: flex;
  flex-direction: column;
  margin: 0 auto;
  gap: 0.75rem;
}}

/* The preview frame centres its child on the cross axis, so a wrapper with no width of its own
   shrinks to its content (the list was as wide as its longest label). Keyed by the data attribute:
   every variant of this page declares `.dx-virtual-list-demo`, and the last stylesheet wins. */
.dx-virtual-list-demo[data-vl-reorder-demo] {{
  width: 100%;
}}

.dx-vl-reorder-frame {{
  border: 1px solid var(--dx-border);
  border-radius: var(--dx-radius-lg);
  overflow: hidden;
}}

.dx-vl-reorder-row {{
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.375rem 0.75rem 0.375rem 0.5rem;
  border-bottom: 1px solid var(--dx-border);
  background: var(--dx-background);
}}

.dx-vl-reorder-row[data-reorder="source"] {{
  opacity: var(--dx-dnd-ghost-opacity, 0.5);
}}

.dx-vl-reorder-row[data-drop="before"] {{
  box-shadow: inset 0 2px 0 var(--dx-drop-indicator);
}}

.dx-vl-reorder-row[data-drop="after"] {{
  box-shadow: inset 0 -2px 0 var(--dx-drop-indicator);
}}

.dx-vl-reorder-grip {{
  color: var(--dx-muted-foreground);
  cursor: grab;
}}

.dx-vl-reorder-handle:active .dx-vl-reorder-grip {{
  cursor: grabbing;
}}

.dx-vl-reorder-handle[aria-pressed="true"] .dx-vl-reorder-grip {{
  color: var(--dx-foreground);
}}

.dx-vl-reorder-text {{
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: 0.125rem;
}}

.dx-vl-reorder-detail {{
  color: var(--dx-muted-foreground);
  font-size: var(--dx-text-sm);
}}

.dx-vl-reorder-sr-only {{
  position: absolute;
  overflow: hidden;
  width: 1px;
  height: 1px;
  clip-path: inset(50%);
  white-space: nowrap;
}}"# }
            div { class: "flex flex-wrap items-center gap-2",
                Button {
                    variant: ButtonVariant::Outline,
                    "data-reorder": "reverse",
                    onclick: move |_| order_mut.with_mut(|o| o.reverse()),
                    "Reverse"
                }
                Button {
                    variant: ButtonVariant::Outline,
                    "data-reorder": "shuffle",
                    onclick: shuffle,
                    "Shuffle"
                }
            }
            div { class: "dx-vl-reorder-frame",
                Rows { reorder }
            }
            span { id: "vl-reorder-hint", class: "dx-vl-reorder-sr-only", {INSTRUCTIONS} }
            div {
                class: "dx-vl-reorder-sr-only",
                role: "status",
                aria_live: "assertive",
                aria_atomic: "true",
                "{announcement()}"
            }
        }
    }
}
