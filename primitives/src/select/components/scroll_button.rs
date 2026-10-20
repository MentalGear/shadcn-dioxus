//! `SelectScrollUpButton` / `SelectScrollDownButton`: the edge buttons of a scrolling listbox.

use dioxus::prelude::*;
use dioxus_attributes::attributes;
use std::time::Duration;

use super::super::context::SelectContext;
use crate::{
    activity::use_motion,
    interval::use_interval_while,
    js_listener::{use_js_listeners, JsListeners},
    merge_attributes,
};

/// Marks the subtree of a listbox that is actually rendered (`SelectListRendered`). While the
/// list is closed `SelectList` renders its children inline, without the listbox, so a scroll
/// button must render nothing there: it has no scroller to watch.
#[derive(Clone, Copy)]
pub(super) struct InRenderedList;

/// Pixels the list scrolls per tick of a hovered button (about 360 px/s at the 16 ms period).
const SCROLL_STEP_PX: i32 = 6;
const SCROLL_PERIOD: Duration = Duration::from_millis(16);

#[derive(Clone, Copy, PartialEq)]
enum Edge {
    Up,
    Down,
}

impl Edge {
    fn as_str(self) -> &'static str {
        match self {
            Edge::Up => "up",
            Edge::Down => "down",
        }
    }
}

/// Props for [`SelectScrollUpButton`] and [`SelectScrollDownButton`].
#[derive(Props, Clone, PartialEq)]
pub struct SelectScrollButtonProps {
    /// Additional attributes for the button element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The button's content, usually a chevron icon.
    pub children: Element,
}

/// # SelectScrollUpButton
///
/// Put it first inside [`SelectList`](super::list::SelectList). It is shown (`data-visible="true"`)
/// only while the list can scroll up, and scrolls the list up while the pointer rests on it.
/// Hidden from assistive technology: the listbox stays keyboard-scrollable by its options.
///
/// ## Styling
///
/// - `data-visible`: `true` while the list can scroll in this direction.
/// - `data-direction`: `up`.
#[component]
pub fn SelectScrollUpButton(props: SelectScrollButtonProps) -> Element {
    scroll_button(Edge::Up, props)
}

/// # SelectScrollDownButton
///
/// Put it last inside [`SelectList`](super::list::SelectList). See [`SelectScrollUpButton`];
/// this one shows while the list can scroll down.
#[component]
pub fn SelectScrollDownButton(props: SelectScrollButtonProps) -> Element {
    scroll_button(Edge::Down, props)
}

fn scroll_button(edge: Edge, props: SelectScrollButtonProps) -> Element {
    // The hooks live in the inner component so their order never depends on the context.
    if try_consume_context::<InRenderedList>().is_none() {
        return VNode::empty();
    }
    rsx! {
        ScrollButtonInner { edge, attributes: props.attributes, children: props.children }
    }
}

#[derive(Props, Clone, PartialEq)]
struct ScrollButtonInnerProps {
    /// Which edge the button sits on.
    edge: Edge,
    /// Additional attributes for the button element.
    attributes: Vec<Attribute>,
    /// The button's content.
    children: Element,
}

#[component]
fn ScrollButtonInner(props: ScrollButtonInnerProps) -> Element {
    let edge = props.edge;
    let ctx = use_context::<SelectContext>();
    let list_id = ctx.selectable.list_id;
    let mut can_scroll = use_signal(|| false);
    let mut hovering = use_signal(|| false);
    let motion = use_motion();

    // The scroll position is read in the page and only changes are sent: one passive `scroll`
    // listener (removed with the component by `JsListeners`), no per-event round trip.
    use_js_listeners(move || {
        let id = list_id.cloned()?;
        Some(
            JsListeners::install(
                "const id = await dioxus.recv();
                 const up = (await dioxus.recv()) === 'up';
                 const list = document.getElementById(id);
                 if (!list) return;
                 let last = null;
                 const check = () => {
                     const max = list.scrollHeight - list.clientHeight;
                     const can = up ? list.scrollTop > 1 : list.scrollTop < max - 1;
                     if (can !== last) { last = can; dioxus.send(can); }
                 };
                 listen(list, 'scroll', check, { passive: true });
                 listen(window, 'resize', check, { passive: true });
                 requestAnimationFrame(check);",
            )
            .arg(id)
            .arg(edge.as_str())
            .on_message(move |can: bool| can_scroll.set(can)),
        )
    });

    // Hover auto-scroll is motion: it runs only while hovered, while there is room to scroll,
    // and while the button is visible (hidden tab / skipped card pause it).
    let active = use_memo(move || hovering() && can_scroll() && (motion.active())());
    use_interval_while(active, SCROLL_PERIOD, move || {
        let Some(id) = list_id.cloned() else { return };
        let delta = match edge {
            Edge::Up => -SCROLL_STEP_PX,
            Edge::Down => SCROLL_STEP_PX,
        };
        // Not a listener: a one-shot scroll step.
        document::eval(&format!(
            "const l = document.getElementById({id:?}); if (l) l.scrollTop += {delta};"
        ));
    });
    // Nothing left to scroll: the pointer is still over a hidden button, so stop tracking it.
    use_effect(move || {
        if !can_scroll() && *hovering.peek() {
            hovering.set(false);
        }
    });

    let owned = attributes!(div {
        aria_hidden: "true",
        "data-slot": "select-scroll-button",
        "data-direction": edge.as_str(),
        "data-visible": can_scroll().to_string(),
    });
    let merged = merge_attributes(vec![props.attributes, owned, motion.attributes()]);

    rsx! {
        div {
            onpointerenter: move |event: PointerEvent| {
                if event.pointer_type() == "mouse" {
                    hovering.set(true);
                }
            },
            onpointerleave: move |_| hovering.set(false),
            ..merged,
            {props.children}
        }
    }
}
