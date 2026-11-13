//! The safe triangle (backlog row 499): the one rule that keeps a hover-opened panel open while the
//! pointer crosses the gap between its trigger and the panel.
//!
//! Without it a close delay has to be long enough to cover the crossing (this crate used 300 ms for
//! the hover menus and 200 ms for submenus). With it the delay can be Base UI's short one
//! (`@base-ui/react` 1.9.0: `NavigationMenuRoot.mjs:50` `closeDelay = 50`, `MenuSubmenuTrigger.mjs:41`
//! `closeDelay = 0`, `safePolygon` at `NavigationMenuTrigger.mjs:405` and `MenuSubmenuTrigger.mjs:154`).
//!
//! # The rule
//!
//! When the pointer leaves a trigger toward a panel, [`SafeClose::leave`] reads the panel's rect and
//! builds the [`Triangle`] between the leave point and the panel's near edge. From then on the page's
//! `mousemove` (the bridge's document listener, once per frame, installed only while a close is
//! pending) decides:
//!
//! - a move **inside** the triangle keeps the panel open and restarts a stall timer
//!   ([`SAFE_TRIANGLE_STALL`]): a pointer that stops in the gap closes it;
//! - a move **outside** the triangle starts the close, which runs after the host's own delay;
//! - the pointer entering the panel or the trigger again is the host's `cancel`.
//!
//! A close that cannot build a triangle (no panel mounted, the rect unreadable, the pointer left from
//! inside the panel's box) runs after the plain delay.
//!
//! The geometry ([`Point`], [`Rect`], [`Triangle`]) is pure; [`SafeClose`] is the thin wiring that
//! hosts share: `HoverIntent` (NavigationMenu and Navbar) and `SubMenuState` (the submenus of
//! DropdownMenu, ContextMenu and Menubar's hosts).

use std::time::Duration;

use dioxus::prelude::*;
use serde::Deserialize;

use crate::bridge::{self, EventField, EventKind, ListenOptions, Prop, Target};
use crate::delayed_action::{use_delayed_action, DelayedAction};

/// How long the pointer may rest inside the triangle before the panel closes: a pointer that stops
/// in the gap is not on its way to the panel.
pub(crate) const SAFE_TRIANGLE_STALL: Duration = Duration::from_millis(100);

/// How far outside an edge of the triangle a point still counts as inside, in CSS px: the leave
/// point is the last `mouseleave` position, which can sit a pixel off the path that follows.
const SLACK: f64 = 2.0;

/// A point in client (viewport) coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Point {
    pub(crate) x: f64,
    pub(crate) y: f64,
}

impl Point {
    /// Where a mouse event happened, in client coordinates.
    pub(crate) fn of(event: &Event<MouseData>) -> Point {
        let at = event.client_coordinates();
        Point { x: at.x, y: at.y }
    }
}

/// A rectangle in client coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Rect {
    pub(crate) left: f64,
    pub(crate) top: f64,
    pub(crate) right: f64,
    pub(crate) bottom: f64,
}

/// The triangle between a leave point and the near edge of a panel.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Triangle {
    apex: Point,
    a: Point,
    b: Point,
}

impl Triangle {
    /// The triangle from `origin` to the edge of `panel` that faces it: the top edge when the panel is
    /// below the origin, the bottom edge when above, else the left or right edge. `None` when `origin`
    /// is inside the panel's box (there is no gap to cross) or the panel has no area.
    pub(crate) fn toward(origin: Point, panel: Rect) -> Option<Triangle> {
        if panel.right <= panel.left || panel.bottom <= panel.top {
            return None;
        }
        let (a, b) = if origin.y < panel.top {
            (
                Point {
                    x: panel.left,
                    y: panel.top,
                },
                Point {
                    x: panel.right,
                    y: panel.top,
                },
            )
        } else if origin.y > panel.bottom {
            (
                Point {
                    x: panel.left,
                    y: panel.bottom,
                },
                Point {
                    x: panel.right,
                    y: panel.bottom,
                },
            )
        } else if origin.x < panel.left {
            (
                Point {
                    x: panel.left,
                    y: panel.top,
                },
                Point {
                    x: panel.left,
                    y: panel.bottom,
                },
            )
        } else if origin.x > panel.right {
            (
                Point {
                    x: panel.right,
                    y: panel.top,
                },
                Point {
                    x: panel.right,
                    y: panel.bottom,
                },
            )
        } else {
            return None;
        };
        Some(Triangle { apex: origin, a, b })
    }

    /// Whether `p` is inside the triangle, or within [`SLACK`] px of it.
    pub(crate) fn contains(&self, p: Point) -> bool {
        let edges = [(self.apex, self.a), (self.a, self.b), (self.b, self.apex)];
        // The signed distance to each edge, positive on the inside: the vertex order decides the sign.
        let area = cross(self.apex, self.a, self.b);
        if area == 0.0 {
            return false;
        }
        let orientation = area.signum();
        edges.iter().all(|&(from, to)| {
            let length = ((to.x - from.x).powi(2) + (to.y - from.y).powi(2)).sqrt();
            length == 0.0 || cross(from, to, p) * orientation / length >= -SLACK
        })
    }
}

/// The z component of `(to - from) x (p - from)`.
fn cross(from: Point, to: Point, p: Point) -> f64 {
    (to.x - from.x) * (p.y - from.y) - (to.y - from.y) * (p.x - from.x)
}

/// A close the pointer's path decides.
#[derive(Clone, Copy)]
struct Pending {
    triangle: Triangle,
    delay: Duration,
    close: Callback<()>,
}

/// The pointer position of one `mousemove`.
#[derive(Deserialize)]
struct Moved {
    #[serde(rename = "clientX")]
    x: f64,
    #[serde(rename = "clientY")]
    y: f64,
}

/// The close timer of one hover region, with the safe-triangle rule. `Copy`.
#[derive(Clone, Copy)]
pub(crate) struct SafeClose {
    timer: DelayedAction,
    /// The triangle being watched, while a close is pending.
    pending: Signal<Option<Pending>>,
    /// Bumped by every [`Self::leave`] and [`Self::cancel`], so a rect read that finishes late is dropped.
    epoch: Signal<u64>,
}

/// Create a [`SafeClose`]. A hook: call it unconditionally, once per render. It installs the document
/// `mousemove` listener, which exists only while a close is pending.
pub(crate) fn use_safe_close() -> SafeClose {
    let safe = SafeClose {
        timer: use_delayed_action(),
        pending: use_signal(|| None),
        epoch: use_signal(|| 0),
    };
    bridge::use_event_listener(
        move || safe.pending.read().is_some().then_some(Target::Document),
        EventKind::Mousemove,
        ListenOptions::default(),
        &[EventField::ClientX, EventField::ClientY],
        move |moved: Moved| {
            safe.moved(Point {
                x: moved.x,
                y: moved.y,
            })
        },
    );
    safe
}

impl SafeClose {
    /// Drop a pending close: the pointer is back on the trigger or the panel, or a deliberate action
    /// superseded it.
    pub(crate) fn cancel(&mut self) {
        self.epoch += 1;
        self.timer.cancel();
        self.pending.set(None);
    }

    /// The pointer left the trigger at `origin`: `close` runs after `delay` unless the pointer reaches
    /// the panel (the host's [`Self::cancel`]) or, when `panel` is mounted, stays inside the safe
    /// triangle toward it (see the module doc).
    pub(crate) fn leave(
        &mut self,
        origin: Option<Point>,
        panel_id: Option<String>,
        delay: Duration,
        close: Callback<()>,
    ) {
        self.cancel();
        let (Some(origin), Some(panel_id)) = (origin, panel_id) else {
            self.schedule(delay, close);
            return;
        };
        // Until the rect is read, the stall timer is the close: a pointer that does not move leaves.
        self.schedule(delay.max(SAFE_TRIANGLE_STALL), close);
        let epoch = *self.epoch.peek();
        let (mut pending, current) = (self.pending, self.epoch);
        spawn(async move {
            let Some(rect) = bridge::get(panel_id, [Prop::Rect])
                .await
                .ok()
                .and_then(|props| props.rect())
            else {
                return;
            };
            if *current.peek() != epoch {
                return;
            }
            let rect = Rect {
                left: rect.left,
                top: rect.top,
                right: rect.right,
                bottom: rect.bottom,
            };
            if let Some(triangle) = Triangle::toward(origin, rect) {
                pending.set(Some(Pending {
                    triangle,
                    delay,
                    close,
                }));
            }
        });
    }

    /// Run `close` after `delay`, with no triangle (the pointer left the panel, or there is no panel
    /// to cross toward).
    pub(crate) fn schedule(&mut self, delay: Duration, close: Callback<()>) {
        let mut pending = self.pending;
        self.timer.schedule(delay, move || {
            pending.set(None);
            close.call(());
        });
    }

    /// One `mousemove` while a close is pending.
    fn moved(mut self, at: Point) {
        let Some(pending) = *self.pending.peek() else {
            return;
        };
        if pending.triangle.contains(at) {
            self.schedule(SAFE_TRIANGLE_STALL, pending.close);
            // `schedule` clears the watch when the close runs; until then it stays armed.
            self.pending.set(Some(pending));
        } else {
            // Off the path: the host's own delay, and nothing left to watch.
            self.schedule(pending.delay, pending.close);
            self.pending.set(None);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PANEL: Rect = Rect {
        left: 100.0,
        top: 200.0,
        right: 300.0,
        bottom: 400.0,
    };

    fn at(x: f64, y: f64) -> Point {
        Point { x, y }
    }

    #[test]
    fn a_panel_below_gives_a_triangle_on_its_top_edge() {
        let t = Triangle::toward(at(150.0, 180.0), PANEL).expect("triangle");
        assert!(t.contains(at(150.0, 190.0)), "straight down");
        assert!(t.contains(at(200.0, 199.0)), "diagonally toward the panel");
        assert!(t.contains(at(150.0, 180.0)), "the apex");
        assert!(!t.contains(at(40.0, 190.0)), "away to the left");
        assert!(!t.contains(at(150.0, 150.0)), "back up");
        assert!(!t.contains(at(350.0, 195.0)), "past the far corner");
    }

    #[test]
    fn a_panel_above_to_the_left_or_right_picks_its_near_edge() {
        let above = Triangle::toward(at(200.0, 450.0), PANEL).expect("above");
        assert!(above.contains(at(200.0, 420.0)));
        assert!(!above.contains(at(200.0, 480.0)));
        let left_of = Triangle::toward(at(60.0, 300.0), PANEL).expect("left");
        assert!(left_of.contains(at(90.0, 250.0)));
        assert!(!left_of.contains(at(60.0, 100.0)));
        let right_of = Triangle::toward(at(340.0, 300.0), PANEL).expect("right");
        assert!(right_of.contains(at(310.0, 350.0)));
        assert!(!right_of.contains(at(340.0, 450.0)));
    }

    #[test]
    fn an_origin_inside_the_panel_or_an_empty_panel_has_no_triangle() {
        assert_eq!(Triangle::toward(at(200.0, 300.0), PANEL), None);
        let empty = Rect {
            left: 1.0,
            top: 1.0,
            right: 1.0,
            bottom: 9.0,
        };
        assert_eq!(Triangle::toward(at(0.0, 0.0), empty), None);
    }

    #[test]
    fn a_point_a_pixel_off_an_edge_is_still_inside() {
        let t = Triangle::toward(at(150.0, 180.0), PANEL).expect("triangle");
        // The left edge runs from the apex (150, 180) to (100, 200): a point 1 px outside it.
        assert!(t.contains(at(125.0 - 1.0, 190.0)));
        assert!(!t.contains(at(125.0 - 10.0, 190.0)));
    }
}
