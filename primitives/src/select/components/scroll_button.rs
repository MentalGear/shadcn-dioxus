//! `SelectScrollUpButton` / `SelectScrollDownButton`: the edge buttons of a scrolling listbox.

use dioxus::html::geometry::PixelsVector2D;
use dioxus::html::ScrollBehavior;
use dioxus::prelude::*;
use dioxus_attributes::attributes;
use std::rc::Rc;
use std::time::Duration;

use crate::{activity::use_motion, interval::use_interval_while, merge_attributes};

/// What a scroll button needs to know about its list: where it is scrolled and how tall the
/// content and the visible window are, all in layout pixels.
#[derive(Clone, Copy, PartialEq)]
struct ScrollMetrics {
    top: f64,
    scroll_height: f64,
    client_height: f64,
}

impl ScrollMetrics {
    fn can_scroll(self, edge: Edge) -> bool {
        match edge {
            Edge::Up => self.top > 1.0,
            Edge::Down => self.top < self.scroll_height - self.client_height - 1.0,
        }
    }
}

/// The rendered listbox's scroll state, provided by `SelectListRendered` to its subtree. Its
/// presence also marks the subtree of a listbox that is actually rendered: while the list is
/// closed `SelectList` renders its children inline, without the listbox, so a scroll button
/// must render nothing there (it has no scroller to watch).
///
/// The list feeds it from its own `onscroll` (exact: [`ScrollData`] carries the layout
/// `scrollTop`/`scrollHeight`/`clientHeight`) and `onresize` (the popup growing or shrinking
/// with its content); the buttons only read it. Nothing is injected into the page and no
/// listener lives outside the element.
#[derive(Clone, Copy)]
pub(super) struct ListScroll {
    list: Signal<Option<Rc<MountedData>>>,
    metrics: Signal<Option<ScrollMetrics>>,
}

impl ListScroll {
    /// `list` is the signal the listbox's `onmounted` fills.
    pub(super) fn new(list: Signal<Option<Rc<MountedData>>>) -> Self {
        Self {
            list,
            metrics: Signal::new(None),
        }
    }

    /// The list scrolled (or was scrolled by a button, a key or the item-aligned placement).
    pub(super) fn on_scroll(mut self, data: &ScrollData) {
        let next = Some(ScrollMetrics {
            top: data.scroll_top(),
            scroll_height: f64::from(data.scroll_height()),
            client_height: f64::from(data.client_height()),
        });
        if *self.metrics.peek() != next {
            self.metrics.set(next);
        }
    }

    /// The list's box changed (it fires once on mount, which is the first measurement).
    ///
    /// A resize carries the border-box height, not `clientHeight`, so the window is a border
    /// width taller than it is: that errs toward "nothing to scroll" for a list that overflows
    /// by only a pixel or two, and the first `onscroll` after replaces it with the exact value.
    pub(super) fn on_resize(mut self, data: &ResizeData) {
        let Ok(size) = data.get_border_box_size() else {
            return;
        };
        let Some(list) = self.list.peek().clone() else {
            return;
        };
        spawn(async move {
            let (Ok(offset), Ok(scroll)) =
                (list.get_scroll_offset().await, list.get_scroll_size().await)
            else {
                return;
            };
            let next = Some(ScrollMetrics {
                top: offset.y,
                scroll_height: scroll.height,
                client_height: size.height,
            });
            if *self.metrics.peek() != next {
                self.metrics.set(next);
            }
        });
    }
}

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
    if try_consume_context::<ListScroll>().is_none() {
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
    let list_scroll = use_context::<ListScroll>();
    let mut hovering = use_signal(|| false);
    let motion = use_motion();

    let can_scroll =
        use_memo(move || (list_scroll.metrics)().is_some_and(|metrics| metrics.can_scroll(edge)));

    // Hover auto-scroll is motion: it runs only while hovered, while there is room to scroll,
    // and while the button is visible (hidden tab / skipped card pause it).
    let active = use_memo(move || hovering() && can_scroll() && (motion.active())());
    use_interval_while(active, SCROLL_PERIOD, move || {
        let Some(list) = list_scroll.list.peek().clone() else {
            return;
        };
        let delta = match edge {
            Edge::Up => -SCROLL_STEP_PX,
            Edge::Down => SCROLL_STEP_PX,
        };
        // One step: read where the list is (not where the last event said, which can be a tick
        // old) and write the absolute position, both axes, so the untouched one stays put.
        spawn(async move {
            let Ok(at) = list.get_scroll_offset().await else {
                return;
            };
            _ = list
                .scroll(
                    PixelsVector2D::new(at.x, at.y + f64::from(delta)),
                    ScrollBehavior::Instant,
                )
                .await;
        });
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
