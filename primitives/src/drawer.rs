//! Defines the [`Drawer`] component and its sub-components -- a Vaul-style
//! (shadcn/ui "Drawer") sheet with a pointer drag-to-dismiss gesture.
//!
//! ## Composition, not reimplementation
//!
//! `Drawer` is the modal [`crate::dialog::DialogRoot`]/
//! [`crate::dialog::DialogContent`] primitive (see `dialog.rs`'s own module
//! doc for the web-arm native `<dialog>`/`showModal()` engine and the
//! native-arm vendored focus trap) plus a drag gesture layered on top --
//! exactly the composition `preview/src/components/sheet/component.rs`'s
//! themed `Sheet` already does for the *same* primitive, one layer higher.
//! `Drawer` does the same composition one layer lower, at the primitive
//! layer, because the drag gesture is new interaction/a11y logic and this
//! crate's own README asks for that to live here, not in `preview/`.
//!
//! Escape-to-close, focus trap/restore, background inertness, top-layer
//! rendering and scroll lock are therefore *inherited for free* from
//! [`crate::dialog::DialogContent`] -- this module adds nothing on top for
//! any of those; it only adds the pointer-drag gesture and the
//! data-attributes/inline styling that gesture needs.
//!
//! ## Drag construction
//!
//! The whole gesture lives in one real-DOM script (`GESTURE_JS`, installed by
//! `use_drawer_gesture` on [`DrawerContent`]'s own root element), and the Rust
//! side only reacts to three messages: `start` (with the panel's measured size),
//! `move` (the cumulative offset, oriented so positive always means "toward the
//! closing edge") and `end` (the release velocity, and whether the browser
//! cancelled the gesture). Polling `crate::pointer::pointer_position` from a
//! reactive effect and timing its samples with the wall clock of whenever
//! that effect happens to run would go wrong in two ways; the script rules
//! both out by construction:
//!
//! - **The release velocity is not noise.** The displacement between the
//!   last two samples over their effect-run times, floored at 1 ms, reads a
//!   slow drag as a fling on a busy main thread, where several
//!   `pointermove`s collapse into one effect run. The script timestamps every
//!   sample with the event's own `timeStamp` and takes the velocity over the last
//!   `VELOCITY_WINDOW_MS` of movement before the release; a pause before
//!   letting go is therefore no flick, and a flick needs at least
//!   `FLICK_MIN_DISTANCE_PX` of travel so jitter cannot read as one.
//! - **The direction is decided in one place.** Orientation (down for
//!   `Bottom`, up for `Top`, and so on) is applied once, in the script, from the
//!   `vertical`/`sign` pair `EdgeSide::drag_axis` hands it; everything after
//!   -- the live offset, the rubber band, the release decision -- works on that
//!   one oriented number, so a drag toward the OPEN side can never be positive.
//!   `DrawerSide`s are physical screen edges (a `Right` drawer slides in from the
//!   physical right, in RTL too, exactly as `Sheet`'s do), so no `dir` flip
//!   belongs here.
//!
//! A drag that began inside the drawer and ended over its `::backdrop` is not a
//! backdrop click: `crate::use_dialog_backdrop_dismiss` requires the PRESS to have
//! started on the backdrop too, so that rule is dialog-wide, not drawer-specific.
//!
//! Gating *where* a drag may start still needs the real DOM target (Dioxus's
//! synthetic `PointerData` has no `event.target`): a press on an interactive
//! descendant (button/link/form control), or on content scrolled away from the
//! drag edge, must not start a drawer drag. The same script does that, before it
//! starts tracking. A native `pointerdown` bubbles, so one listener on
//! [`DrawerContent`]'s root also catches a gesture begun on [`DrawerHandle`].
//! Mouse, touch and pen all arrive as Pointer Events, so one path serves them;
//! a mouse press other than the primary button never starts a drag, and a
//! `pointercancel` (the browser claiming the gesture for a pan) ends it as a
//! snap-back, never a dismissal.
//!
//! ## Why `transform`, not `translate`, carries the live drag offset
//!
//! The panel's *slide* (the theme's "Edge panel" recipe, `dx-components-theme.css`:
//! `dx-edge-in-*`/`dx-edge-out-*`) is a `@keyframes`/`animation-fill-mode: forwards`
//! pair on the `translate` property -- never `transform`, because a keyframe that
//! sets `transform` replaces whatever `transform` the element already has for as long
//! as it runs (`scripts/check-anchored-keyframes.sh`). It has to be a keyframe, not a
//! `transition`, because a plain CSS `transition` never runs on an element's first
//! style computation and this panel's `data-state` is already `"open"` at the moment
//! it is first inserted (`use_animated_open` only mounts it once `open` is true).
//!
//! Per the CSS Animations cascade, a *running or forwards-held* animation's value for
//! a property overrides that property's normal-origin value for as long as the
//! animation applies to the element -- including a higher-specificity *inline* style
//! on that same property. Driving the live drag offset through an inline `translate`
//! would therefore be silently overridden by the held `forwards` end-state of the
//! slide for the entire time the drawer is open, i.e. the only time a drag can happen
//! -- the panel would not visibly move at all. `transform` is a separate property
//! that *composes* with `translate` instead of competing for the same cascade slot,
//! so the slide's `translate` and this module's inline `transform` both apply,
//! combined, with no precedence conflict and no extra timing coordination needed for
//! the drag-release-into-close handoff (the exit keyframe's `translate` and the
//! frozen-then-reset drag `transform` simply add). The inline `transform` is present
//! only while a drag has moved the panel (or released it into a close): at rest the
//! attribute is absent, so a stylesheet owns `transform` (the nested-drawer offset)
//! and `scale` (the nested-drawer shrink) without contesting it. See
//! `preview/src/components/drawer/style.css` for the rest of the construction.
//!
//! ## One side type, resolved once
//!
//! [`EdgeSide`] is shared by [`Drawer`] and the themed `Sheet`
//! (`SheetSide` and [`DrawerSide`] are aliases of it). Its default, [`EdgeSide::Auto`],
//! is "like shadcn's demo": a bottom panel below the theme's `md` breakpoint
//! (`--dx-breakpoint-md`, 768px) and a right panel at and above it. It is resolved
//! ONCE, in Rust ([`use_resolved_edge_side`]), into a concrete side that everything
//! downstream sees -- the `data-side` attribute the theme keys on, the drag axis
//! `GESTURE_JS` runs on, the handle, the slide -- so no stylesheet or script ever
//! has to know what `auto` means, and an explicit side is exactly that side.
//!
//! ## Snap points
//!
//! `snap_points` (Base UI Drawer's `snapPoints`) lets the panel rest at partial extents: a
//! [`SnapPoint`] is a fraction of the viewport along the drawer's axis or a pixel length. The panel is
//! as tall (wide, for a left/right drawer) as the LARGEST snap point and rests translated toward its
//! closing edge by the difference, so the resting offset has ONE source of truth: the active snap point
//! writes two inline custom properties (`--dx-drawer-snap`, `--dx-drawer-snap-max`), the stylesheet
//! derives `--dx-drawer-rest` from them and the panel's own `transform` reads it (and so does the live
//! drag's inline `transform`, which adds the drag delta to it). Nothing measures the panel to find where
//! it rests; a snap change is a custom-property change, so the same `transform` transition that eases a
//! release settles it. The live drag is the one custom property `--dx-drawer-drag`, added by the same
//! stylesheet formula (always present while snapped, `0px` at rest: the web renderer applies a changed `style`
//! per declaration and never removes one that left the string). The release decision runs on the same numbers in Rust (`snap_rests`, from the
//! viewport a resize listener reports), picked by `settle_on_release`: the panel closes by the one
//! existing rule (`should_close_on_release`, applied to the travel below the LOWEST snap point) and
//! otherwise settles on the snap point nearest the release position projected by the release velocity.
//! `active_snap_point` / `default_active_snap_point` / `on_active_snap_point_change` make the active point
//! controlled or not; a controlled value the parent does not change snaps the panel back. Without
//! `snap_points` the same code runs on the one rest `0` (a plain drawer).
//!
//! ## Non-modal
//!
//! `is_modal: false` composes the dialog primitive's non-modal arm (a `div`, no top layer, no focus
//! trap, no scroll lock, no inert page) and turns the scrim off in ONE place, the root wrapper's
//! `overlay` (`scrim = overlay && is_modal`), so no stylesheet has to know a non-modal drawer has no
//! scrim. The wrapper is full-viewport, so `data-dx-modal="false"` on it is what the theme keys
//! `pointer-events: none` on: only the panel catches the pointer. Dismissal is the dialog arm's own
//! (Escape, a press or focus outside).
//!
//! ## Nested drawers
//!
//! A [`Drawer`] rendered inside another [`DrawerContent`] is nested (Vaul's
//! `NestedRoot`, without a separate component: the context decides). While the child
//! is open the parent carries `data-nested-open="true"`, which the theme turns into a
//! small scale and offset, and closing the child (the moment its close STARTS)
//! restores it. Escape closes only the top-most because the browser's `cancel` goes
//! to the top-most modal `<dialog>` only; a drag closes only the top-most because
//! `GESTURE_JS` ignores a press whose nearest drawer is not its own (a child drawer
//! is a DOM descendant of its parent, so its `pointerdown` bubbles through it). Scroll
//! lock stays single because `crate::scroll_lock`'s counter nests.

use crate::dialog::{
    DialogContent, DialogCtx, DialogDescription, DialogDescriptionProps, DialogRoot, DialogTitle,
    DialogTitleProps,
};
use crate::state_attrs;
use crate::{merge_attributes, use_id_or, use_unique_id, AsChild};
use dioxus::core::use_drop;
use dioxus::prelude::*;
use dioxus_attributes::attributes;

/// Which edge of the viewport an edge panel ([`Drawer`], or the themed `Sheet`) slides in
/// from -- and, for a drawer, is dragged toward to dismiss. `SheetSide` and [`DrawerSide`]
/// are aliases of this type: one panel system has one side type.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum EdgeSide {
    /// Slides in from, and is dragged upward toward, the top edge.
    Top,
    /// Slides in from, and is dragged rightward toward, the right edge.
    Right,
    /// Slides in from, and is dragged downward toward, the bottom edge.
    Bottom,
    /// Slides in from, and is dragged leftward toward, the left edge.
    Left,
    /// The bottom edge below the theme's `md` breakpoint (`--dx-breakpoint-md`, 768px:
    /// a phone, where a bottom sheet is the thumb-reachable one) and the right edge at
    /// and above it (where a floating side panel is). The default, as in shadcn's
    /// drawer demo. The panel follows the viewport live, but only a CLOSED panel is
    /// meant to change side: resizing across the breakpoint with a drawer open moves it
    /// to the other edge. An explicit side is never changed by the viewport.
    #[default]
    Auto,
}

impl EdgeSide {
    /// The lowercase name used for the `data-side` attribute (and by the themed layer's
    /// CSS selectors). Panels render the RESOLVED side ([`EdgeSide::resolve`]), so a
    /// rendered `data-side` is never `"auto"`.
    pub fn as_str(&self) -> &'static str {
        match self {
            EdgeSide::Top => "top",
            EdgeSide::Right => "right",
            EdgeSide::Bottom => "bottom",
            EdgeSide::Left => "left",
            EdgeSide::Auto => "auto",
        }
    }

    /// The concrete side for a viewport that is at or above the `md` breakpoint (`wide`)
    /// or below it. Every explicit side resolves to itself.
    #[must_use]
    pub fn resolve(self, wide: bool) -> EdgeSide {
        match self {
            EdgeSide::Auto if wide => EdgeSide::Right,
            EdgeSide::Auto => EdgeSide::Bottom,
            explicit => explicit,
        }
    }

    /// The screen axis a drag toward this side's closing edge runs along, as
    /// `(vertical, sign)`: `vertical` is `true` for `Top`/`Bottom`, and `sign`
    /// is `1.0` when the closing direction is the positive screen-axis
    /// direction (down for `Bottom`, right for `Right`) and `-1.0` when it is
    /// the negative one (up for `Top`, left for `Left`). Multiplying a raw
    /// pointer delta along that axis by `sign` gives the number the whole
    /// gesture runs on: positive is toward the closing edge, negative is
    /// toward (and past) the open position. The screen sides are physical, so
    /// this does not depend on the text direction. An unresolved `Auto` answers
    /// as its narrow-viewport side (callers hand this the RESOLVED side).
    fn drag_axis(self) -> (bool, f64) {
        match self {
            EdgeSide::Bottom | EdgeSide::Auto => (true, 1.0),
            EdgeSide::Top => (true, -1.0),
            EdgeSide::Right => (false, 1.0),
            EdgeSide::Left => (false, -1.0),
        }
    }
}

/// The side type of [`Drawer`]; an alias of the shared [`EdgeSide`].
pub type DrawerSide = EdgeSide;

/// The breakpoint [`EdgeSide::Auto`] switches at: the theme's `md` token. The script reads
/// the token itself (so a retuned theme moves the switch) and falls back to this value when
/// the theme sheet is not loaded.
const EDGE_AUTO_BREAKPOINT_VAR: &str = "--dx-breakpoint-md";
const EDGE_AUTO_BREAKPOINT_FALLBACK: &str = "48rem";

/// Resolves `side` ([`EdgeSide::Auto`] against the viewport) into the concrete side a panel
/// renders. Call it once per panel root; an explicit side installs nothing.
///
/// The viewport is read from one `matchMedia` listener (`min-width` at
/// `--dx-breakpoint-md`) installed through [`crate::js_listener`], so its removal is derived
/// from the owning scope. Until its first message arrives (a few milliseconds after
/// hydration, long before anyone can open a panel) the viewport counts as narrow.
pub fn use_resolved_edge_side(side: ReadSignal<EdgeSide>) -> Memo<EdgeSide> {
    let mut wide = use_signal(|| false);
    crate::js_listener::use_js_listeners(move || {
        if side() != EdgeSide::Auto {
            return None;
        }
        Some(
            crate::js_listener::JsListeners::install(
                "const token = await dioxus.recv();
                const fallback = await dioxus.recv();
                const breakpoint = getComputedStyle(document.documentElement).getPropertyValue(token).trim() || fallback;
                const query = window.matchMedia('(min-width: ' + breakpoint + ')');
                dioxus.send(query.matches);
                listen(query, 'change', (e) => dioxus.send(e.matches));",
            )
            .arg(EDGE_AUTO_BREAKPOINT_VAR)
            .arg(EDGE_AUTO_BREAKPOINT_FALLBACK)
            .on_message(move |matches: bool| wide.set(matches)),
        )
    });
    use_memo(move || side().resolve(wide()))
}

/// One snap point of a [`Drawer`]: how much of the viewport (along the drawer's axis: the height of a
/// top/bottom drawer, the width of a left/right one) is visible when the panel rests there.
/// `Fraction(1.0)` is fully open. Values past the viewport are clamped to it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SnapPoint {
    /// A fraction of the viewport extent, `0.0..=1.0` (`0.5` is half the viewport).
    Fraction(f64),
    /// A length in CSS pixels.
    Px(f64),
}

impl SnapPoint {
    /// The visible extent in px for a viewport extent of `viewport` px.
    fn visible_px(self, viewport: f64) -> f64 {
        match self {
            SnapPoint::Fraction(f) => f.clamp(0.0, 1.0) * viewport,
            SnapPoint::Px(px) => px.min(viewport),
        }
        .max(0.0)
    }

    /// The same extent as a CSS length (`100dvh`/`100dvw` is the viewport extent, exactly what
    /// [`SnapPoint::visible_px`] is given by the resize listener).
    fn css(self, vertical: bool) -> String {
        let unit = if vertical { "dvh" } else { "dvw" };
        match self {
            SnapPoint::Fraction(f) => format!("calc({} * 100{unit})", f.clamp(0.0, 1.0)),
            SnapPoint::Px(px) => format!("min({}px, 100{unit})", px.max(0.0)),
        }
    }
}

/// The resting offset of every snap point, in px toward the closing edge: the panel is as large as the
/// largest snap point, so a point's rest is `largest - its own extent` (`0` for the largest). With no
/// snap points the drawer has the single rest `0`. `viewport` is the viewport extent along the axis;
/// `None` (the resize listener has not reported yet) reads as all points at rest `0`.
fn snap_rests(points: &[SnapPoint], viewport: Option<f64>) -> Vec<f64> {
    let Some(viewport) = viewport.filter(|_| !points.is_empty()) else {
        return vec![0.0; points.len().max(1)];
    };
    let visible: Vec<f64> = points.iter().map(|p| p.visible_px(viewport)).collect();
    let largest = visible.iter().cloned().fold(0.0_f64, f64::max);
    visible.into_iter().map(|v| largest - v).collect()
}

/// The index of the active snap point: the position of `active` in `points`, or the first when it is
/// unset or not one of them.
fn active_snap_index(points: &[SnapPoint], active: Option<SnapPoint>) -> usize {
    active
        .and_then(|a| points.iter().position(|p| *p == a))
        .unwrap_or(0)
}

/// Where a released drag goes.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Settle {
    /// Rest on snap point `index` (the active one when the drag did not reach another).
    Snap(usize),
    /// Close the drawer.
    Close,
}

/// How far ahead (ms) the release velocity projects the release position when picking the snap point:
/// a flick of 1px/ms reaches 150px past where the pointer let go. Tier-3 "opinion".
const SNAP_PROJECTION_MS: f64 = 150.0;
/// The release velocity used for that projection is capped, so one wild sample cannot skip every point.
const SNAP_PROJECTION_MAX_VELOCITY: f64 = 3.0;

/// The release decision for every drawer, with or without snap points (without, `rests` is `[0.0]`).
/// `rests` are the resting offsets ([`snap_rests`]), `active_rest` the active one's, `raw` the
/// dismiss-oriented drag offset since the press, `panel_size` the panel's extent along the axis and
/// `velocity` the release velocity toward the closing edge.
///
/// Closing is the ONE existing rule ([`should_close_on_release`]) applied to the travel below the lowest
/// snap point (the largest rest) against that point's visible extent, so a drawer without snap points
/// closes by that rule alone and a snap drawer closes only by being dragged below its lowest point. Else
/// the panel settles on the snap point nearest the release position projected by the velocity, kept
/// within the range of the points (a flick may skip points; it never overshoots the ends).
fn settle_on_release(
    rests: &[f64],
    active_rest: f64,
    raw: f64,
    panel_size: Option<f64>,
    velocity: f64,
) -> Settle {
    let lowest = rests.iter().cloned().fold(0.0_f64, f64::max);
    let position = active_rest + raw;
    let lowest_extent = panel_size.map(|size| size - lowest);
    if should_close_on_release(position - lowest, lowest_extent, velocity) {
        return Settle::Close;
    }
    let projected = (position
        + velocity.clamp(-SNAP_PROJECTION_MAX_VELOCITY, SNAP_PROJECTION_MAX_VELOCITY)
            * SNAP_PROJECTION_MS)
        .clamp(0.0, lowest);
    let nearest = rests
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| (**a - projected).abs().total_cmp(&(**b - projected).abs()))
        .map(|(i, _)| i)
        .unwrap_or(0);
    Settle::Snap(nearest)
}

/// shadcn/ui Drawer (Vaul)'s distance release heuristic: a drag past this
/// fraction of the panel's own size along its axis counts as "mostly
/// dismissed" and completes the close. Tier-3 "opinion" (see
/// `playwright/drawer.spec.ts`'s header) -- there is no APG/WHATWG rule for
/// a drag-to-dismiss gesture, only Vaul's own convention.
const DISMISS_DRAG_RATIO: f64 = 0.25;

/// shadcn/ui Drawer (Vaul)'s velocity release heuristic: a release *flung*
/// faster than this (in the dismiss direction, CSS pixels per millisecond)
/// closes the drawer even short of [`DISMISS_DRAG_RATIO`]'s distance.
/// Tier-3 "opinion", same source as that constant.
const DISMISS_VELOCITY_PX_PER_MS: f64 = 0.5;

/// A flick must have travelled at least this far toward the closing edge to
/// count: a press with a few pixels of jitter can momentarily exceed
/// [`DISMISS_VELOCITY_PX_PER_MS`] (1 px in 1 ms is already 1 px/ms) and must
/// not dismiss the drawer.
const FLICK_MIN_DISTANCE_PX: f64 = 10.0;

/// The release velocity is measured over the pointer movement of this many
/// milliseconds before letting go (using each event's own `timeStamp`), so a
/// pause before the release is no flick and a long, slow drag followed by a
/// quick wrist flick is read for what the hand did last.
const VELOCITY_WINDOW_MS: f64 = 100.0;

/// How far (px) a press must travel before the drag decides its axis: along the drag axis it
/// becomes a drag, along the other it is abandoned and the content scrolls (see [`GESTURE_JS`]).
const AXIS_LOCK_PX: f64 = 6.0;

/// How much a drag *away* from the dismiss direction -- past the
/// fully-open resting position -- is damped, instead of hard-clamped to
/// exactly `0`: gives a small, springy "rubber band" resistance rather
/// than a dead stop. Vaul has the same overdrag resistance; the exact
/// ratio/cap here are this crate's own tuning, not copied from a spec.
const RUBBER_BAND_RATIO: f64 = 0.25;
/// The rubber band's own maximum travel, in CSS pixels.
const RUBBER_BAND_MAX_PX: f64 = 24.0;

/// The dismiss-oriented `raw` accumulated offset, converted to what should
/// actually be displayed: unchanged (up to `travel`, the room left before the panel is fully out)
/// while dragging toward dismissal, free up to `room` (how far the active snap point rests from fully
/// open, `0` without snap points) while dragging toward open, and rubber-banded -- but never snapped
/// to -- the fully-open position beyond it.
///
/// A pure function of the *cumulative* raw offset (not of the previous
/// display value) on purpose: damping the already-damped display value
/// again each frame would make the panel visibly creep back toward the
/// open position while the pointer kept moving further into the overshoot, which is
/// wrong -- the damping curve must only ever be a function of how far off
/// the resting position the gesture has *actually* travelled.
fn damped_display_offset(raw: f64, travel: Option<f64>, room: f64) -> f64 {
    if raw < -room {
        -room + ((raw + room) * RUBBER_BAND_RATIO).max(-RUBBER_BAND_MAX_PX)
    } else if let Some(travel) = travel {
        raw.min(travel)
    } else {
        raw
    }
}

/// The inline style declaration for a dismiss-oriented `offset`, converted to this side's real
/// screen axis and sign.
///
/// Without snap points it is the panel's `transform` (not `translate` -- see this module's doc), and
/// `None` at rest, so the attribute is absent and a stylesheet owns `transform` (the nested-drawer
/// offset) whenever nothing is being dragged.
///
/// With `snapped` it is ALWAYS the custom property `--dx-drawer-drag` (`0px` at rest) and the
/// stylesheet's one `transform` formula adds it to the resting offset. A declaration is never
/// dropped from the inline style, only its value changes: the web renderer applies a changed `style`
/// per declaration and does not remove one that has left the string, so a `transform` that came and
/// went would stay on the panel after the drag.
fn drag_style(side: EdgeSide, offset: f64, snapped: bool) -> Option<String> {
    let (vertical, sign) = side.drag_axis();
    // Explicit fast path for the resting value, rather than always computing
    // `offset * sign`: for `Top`/`Left` (`sign == -1.0`), `0.0 * -1.0` is IEEE 754
    // negative zero, which `f64`'s `Display` renders as `"-0"`.
    if offset == 0.0 {
        return snapped.then(|| "--dx-drawer-drag: 0px".to_string());
    }
    let signed = offset * sign;
    Some(if snapped {
        format!("--dx-drawer-drag: {signed}px")
    } else {
        let axis = if vertical { "Y" } else { "X" };
        format!("transform: translate{axis}({signed}px)")
    })
}

/// Whether a release should close the drawer: dragged past
/// [`DISMISS_DRAG_RATIO`] of the panel's own size along its axis, or
/// released while moving faster than [`DISMISS_VELOCITY_PX_PER_MS`] toward
/// dismissal after travelling at least [`FLICK_MIN_DISTANCE_PX`] -- shadcn/ui
/// Drawer (Vaul)'s own two release heuristics (tier-3 "opinion", see
/// `playwright/drawer.spec.ts`'s header). `raw_offset` and `velocity` are both
/// dismiss-oriented (positive is toward the closing edge), so a drag toward the
/// OPEN side can never close: a `raw` offset at or behind the resting position
/// (`<= 0`, i.e. the rubber-band overshoot case) never closes, regardless of
/// velocity -- a fling in the "open further" direction is not a dismiss
/// gesture.
fn should_close_on_release(
    raw_offset: f64,
    panel_size: Option<f64>,
    velocity_px_per_ms: f64,
) -> bool {
    if raw_offset <= 0.0 {
        return false;
    }
    let past_distance_threshold = match panel_size {
        Some(size) if size > 0.0 => raw_offset / size > DISMISS_DRAG_RATIO,
        _ => false,
    };
    let flicked =
        velocity_px_per_ms > DISMISS_VELOCITY_PX_PER_MS && raw_offset >= FLICK_MIN_DISTANCE_PX;
    past_distance_threshold || flicked
}

/// Selects an element that must never itself start a drawer drag when a
/// pointerdown lands on (or inside) it -- the drag would otherwise steal
/// the gesture from the control's own click/press behaviour.
const INTERACTIVE_SELECTOR: &str =
    "button, a[href], input, select, textarea, [contenteditable=\"\"], [contenteditable=\"true\"], [role=\"button\"]";

/// [`use_drawer_gesture`]'s script: the whole drag gesture, in the real DOM.
/// A [`JsListeners`](crate::js_listener::JsListeners) body: its listeners
/// go through `listen`, so the teardown is derived from the hook's scope.
///
/// Parameters, in order: the content element's id, `vertical`, `sign` (from
/// [`EdgeSide::drag_axis`]), [`INTERACTIVE_SELECTOR`], [`VELOCITY_WINDOW_MS`],
/// [`AXIS_LOCK_PX`], and `enabled` (the drawer's `dismissible`).
///
/// A `pointerdown` starts a drag unless
/// 1. it is a non-primary mouse button, or
/// 2. its nearest `[data-dx-drawer]` is not this panel -- the press landed inside a
///    NESTED drawer (a DOM descendant, so its `pointerdown` bubbles through this
///    one), whose own listener owns it: only the top-most drawer drags, or
/// 3. `event.target.closest(INTERACTIVE_SELECTOR)` matches -- the press landed on
///    (or inside) an interactive descendant, or
/// 4. (not at a partial snap point, where content does not scroll) the nearest scrollable
///    ancestor *along the drag axis* is scrolled away from
///    `0` (the edge nearest the drag) -- that press should scroll that region
///    first. [`DrawerHandle`] itself is never nested inside such a region, so a
///    drag started there walks no scrollable ancestor at all.
///
/// Such a press is only *pending*: nothing is sent, and the panel never moves, until the pointer
/// has travelled [`AXIS_LOCK_PX`] and that first decisive movement is predominantly along the
/// drag axis (and the scrollable ancestor from rule 4 cannot scroll that way). If the cross axis
/// wins (a scroll with sideways wobble), or the content can scroll, the press is abandoned for
/// that pointer and the browser scrolls undisturbed. Only then is `start` sent. A mouse press has
/// no scroll gesture to protect, so it commits at once (and `data-dragging` keeps text from
/// being selected under the drag).
///
/// Then, once committed and until that pointer is released or cancelled, every `pointermove` updates
/// the oriented offset (`raw delta * sign`, positive toward the closing edge) and
/// one timestamped sample; the offset is forwarded at most once a frame, and the
/// release forwards the last offset, then the velocity over the final
/// `windowMs` (0 when the pointer paused or was cancelled). Messages are
/// `[kind, a, b]`: `start` (size of the panel along the axis), `move` (offset),
/// `end` (velocity, cancelled as 0/1).
const GESTURE_JS: &str = r#"const id = await dioxus.recv();
const vertical = await dioxus.recv();
const sign = await dioxus.recv();
const interactiveSelector = await dioxus.recv();
const windowMs = await dioxus.recv();
const lockPx = await dioxus.recv();
const enabled = await dioxus.recv();
const el = document.getElementById(id);
if (el && enabled) {
    const finite = (n) => (Number.isFinite(n) ? n : 0);
    let drag = null;
    let raf = 0;
    const flush = () => {
        raf = 0;
        if (drag) dioxus.send(['move', finite(drag.offset), 0]);
    };
    const velocityAt = (t) => {
        const recent = drag.samples.filter(([at]) => at >= t - windowMs);
        if (recent.length < 2) return 0;
        const [t0, o0] = recent[0];
        const [t1, o1] = recent[recent.length - 1];
        return t1 > t0 ? (o1 - o0) / (t1 - t0) : 0;
    };
    // The axis lock (vaul / Base UI): a press is only PENDING until the pointer has moved
    // `lockPx`. If that first decisive movement is predominantly along the dismiss axis (and the
    // scrollable ancestor found at press time cannot itself scroll that way) the drag commits and
    // `start` is sent; if the cross axis wins, or the content can scroll, the press is abandoned
    // for this pointer, so a scroll's sideways wobble can never translate the panel.
    const canScroll = (node, delta) => {
        // The finger moving by `delta` moves the content's scroll position by `-delta`.
        const max = vertical ? node.scrollHeight - node.clientHeight : node.scrollWidth - node.clientWidth;
        const pos = vertical ? node.scrollTop : node.scrollLeft;
        const rtl = !vertical && getComputedStyle(node).direction === 'rtl';
        const lo = rtl ? -max : 0;
        const hi = rtl ? 0 : max;
        return delta > 0 ? pos > lo + 1 : pos < hi - 1;
    };
    const onMove = (e) => {
        if (!drag || e.pointerId !== drag.pointerId) return;
        const dx = e.clientX - drag.startX;
        const dy = e.clientY - drag.startY;
        const along = vertical ? dy : dx;
        const cross = vertical ? dx : dy;
        if (!drag.committed) {
            if (Math.max(Math.abs(along), Math.abs(cross)) < lockPx) return;
            if (Math.abs(along) <= Math.abs(cross)
                || (drag.scroller && canScroll(drag.scroller, along))) {
                drag = null;
                stop();
                return;
            }
            drag.committed = true;
            dioxus.send(['start', vertical ? drag.rect.height : drag.rect.width, 0]);
        }
        drag.offset = along * sign;
        drag.samples.push([e.timeStamp, drag.offset]);
        if (!raf) raf = requestAnimationFrame(flush);
    };
    // The window listeners of the drag in flight: installed at its `pointerdown`, removed when it
    // ends (`listen`'s early `off`), and by the teardown if the panel goes first.
    let dragOff = [];
    const stop = () => {
        dragOff.forEach((off) => off());
        dragOff = [];
        if (raf) { cancelAnimationFrame(raf); raf = 0; }
    };
    const end = (e, cancelled) => {
        if (!drag || e.pointerId !== drag.pointerId) return;
        if (!drag.committed) {
            drag = null;
            stop();
            return;
        }
        const velocity = cancelled ? 0 : finite(velocityAt(e.timeStamp));
        const offset = finite(drag.offset);
        drag = null;
        stop();
        dioxus.send(['move', offset, 0]);
        dioxus.send(['end', velocity, cancelled ? 1 : 0]);
    };
    const onUp = (e) => end(e, false);
    const onCancel = (e) => end(e, true);
    const onDown = (e) => {
        if (drag) return;
        if (e.pointerType === 'mouse' && e.button !== 0) return;
        if (e.target.closest('[data-dx-drawer]') !== el) return;
        if (e.target.closest(interactiveSelector)) return;
        // At a partial snap point the panel's content does not scroll (the stylesheet clips it): a
        // press anywhere drags it toward the next snap point, whatever it was scrolled to before.
        let node = el.dataset.snap === 'partial' ? null : e.target;
        let scroller = null;
        while (node && node !== el.parentElement) {
            const style = getComputedStyle(node);
            const overflow = vertical ? style.overflowY : style.overflowX;
            const scrollable = (overflow === 'auto' || overflow === 'scroll')
                && (vertical ? node.scrollHeight > node.clientHeight : node.scrollWidth > node.clientWidth);
            if (scrollable) {
                const offset = vertical ? node.scrollTop : node.scrollLeft;
                if (offset !== 0) return;
                scroller = node;
                break;
            }
            if (node === el) break;
            node = node.parentElement;
        }
        drag = {
            pointerId: e.pointerId, startX: e.clientX, startY: e.clientY, offset: 0,
            samples: [[e.timeStamp, 0]], committed: false, scroller, rect: el.getBoundingClientRect(),
        };
        // A mouse press has no scroll gesture to protect, and `data-dragging` is what suppresses
        // text selection during the drag, so it commits at once; only touch and pen wait for the lock.
        if (e.pointerType === 'mouse') {
            drag.committed = true;
            dioxus.send(['start', vertical ? drag.rect.height : drag.rect.width, 0]);
        }
        dragOff = [
            listen(window, 'pointermove', onMove),
            listen(window, 'pointerup', onUp),
            listen(window, 'pointercancel', onCancel),
        ];
    };
    listen(el, 'pointerdown', onDown);
    cleanup(() => {
        drag = null;
        stop();
    });
}"#;

/// One message from [`GESTURE_JS`], already oriented toward the closing edge.
#[derive(Debug, Clone, Copy, PartialEq)]
enum DragEvent {
    /// A drag was approved; `size` is the panel's extent along the drag axis.
    Start { size: f64 },
    /// The cumulative offset toward the closing edge (negative past the open
    /// position).
    Move { offset: f64 },
    /// The pointer was released (or cancelled) at `velocity` px/ms toward the
    /// closing edge.
    End { velocity: f64, cancelled: bool },
}

impl DragEvent {
    /// Decodes a `[kind, a, b]` wire message; an unknown kind is ignored.
    fn from_wire(kind: &str, a: f64, b: f64) -> Option<Self> {
        match kind {
            "start" => Some(Self::Start { size: a }),
            "move" => Some(Self::Move { offset: a }),
            "end" => Some(Self::End {
                velocity: a,
                cancelled: b != 0.0,
            }),
            _ => None,
        }
    }
}

/// Installs the [`GESTURE_JS`] listener on the element identified by
/// `content_id` for the lifetime of the calling component, calling `on_event`
/// for every message. Reinstalled (not left stale) when `side` or `dismissible`
/// changes.
fn use_drawer_gesture(
    content_id: Memo<String>,
    side: ReadSignal<DrawerSide>,
    dismissible: ReadSignal<bool>,
    on_event: impl FnMut(DragEvent) + Clone + 'static,
) {
    crate::js_listener::use_js_listeners(move || {
        // Reactive reads: `side` and `dismissible` changing (unusual, but not
        // disallowed -- both are plain `ReadSignal` props) tears down and
        // reinstalls this listener with the new axis / enablement rather than
        // leaving the script silently stale.
        let (vertical, sign) = side.cloned().drag_axis();
        let enabled = dismissible.cloned();
        let mut on_event = on_event.clone();
        Some(
            crate::js_listener::JsListeners::install(GESTURE_JS)
                .arg(content_id.cloned())
                .arg(vertical)
                .arg(sign)
                .arg(INTERACTIVE_SELECTOR)
                .arg(VELOCITY_WINDOW_MS)
                .arg(AXIS_LOCK_PX)
                .arg(enabled)
                .on_message(move |(kind, a, b): (String, f64, f64)| {
                    if let Some(event) = DragEvent::from_wire(&kind, a, b) {
                        on_event(event);
                    }
                }),
        )
    });
}

/// Context shared from [`Drawer`] down to [`DrawerContent`]: the
/// `side`/`dismissible` configuration the drag gesture needs. Unlike
/// [`crate::dialog::DialogCtx`]'s `open`/`set_open` (which
/// [`DrawerContent`] fetches directly, since it is exactly what
/// `DialogCtx` already provides), neither field exists on `DialogCtx`, so
/// this module needs its own small context for them -- the same reason
/// `context_menu.rs`'s `ContextMenuCtx`, `command.rs`'s `CommandContext`,
/// etc. each define their own rather than trying to fit new fields into an
/// unrelated existing one.
#[derive(Clone, Copy)]
struct DrawerCtx {
    /// The RESOLVED side ([`use_resolved_edge_side`]): never [`EdgeSide::Auto`].
    side: ReadSignal<DrawerSide>,
    dismissible: ReadSignal<bool>,
    /// The snap points (empty: no snapping, the panel rests fully open).
    snap_points: ReadSignal<Vec<SnapPoint>>,
    /// The active snap point (controlled, else the last one a drag settled on, else the default).
    active_snap_point: Memo<Option<SnapPoint>>,
    /// Makes `point` the active snap point and tells the owner.
    set_active_snap_point: Callback<SnapPoint>,
    /// Puts an uncontrolled active snap point back to its default (the drawer was unmounted).
    reset_active_snap_point: Callback<()>,
    /// The viewport `(width, height)`, reported by a resize listener that exists only while there
    /// are snap points; `None` until its first message.
    viewport: ReadSignal<Option<(f64, f64)>>,
}

/// Provided by every [`DrawerContent`] to its subtree, so a [`Drawer`] rendered inside it
/// knows it is nested and can tell its parent it is open.
#[derive(Clone, Copy)]
struct DrawerNestCtx {
    /// How many nested drawers are open right now (open, not merely mounted: a child that
    /// has started closing has already left the count).
    open_children: Signal<usize>,
}

/// The props for the [`Drawer`] component.
#[derive(Props, Clone, PartialEq)]
pub struct DrawerRootProps {
    /// The ID of the drawer root element.
    pub id: ReadSignal<Option<String>>,

    /// Whether the drawer is modal (the default). A modal drawer traps focus, makes the page
    /// inert, locks page scroll and dims the page. `false` (Base UI's `modal: false`) is the
    /// non-modal arm of [`crate::dialog::DialogRoot`]: no scrim (`overlay` is ignored), no top
    /// layer, no focus trap, no scroll lock, the page stays interactive and scrollable. Escape, a
    /// press outside the panel and focus moving outside it still dismiss it (the dialog
    /// primitive's light dismissal); the root carries `data-dx-modal="false"`.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub is_modal: ReadSignal<bool>,

    /// Base UI's `disablePointerDismissal`; forwarded unchanged to
    /// [`crate::dialog::DialogRoot`]. When `true` a press or focus outside the panel does not
    /// dismiss the drawer (Escape and the close controls still do).
    #[props(default = ReadSignal::new(Signal::new(false)))]
    pub disable_pointer_dismissal: ReadSignal<bool>,

    /// Whether the drawer dims the page behind it with the shared overlay scrim.
    /// Defaults to `true`. Forwarded unchanged to [`crate::dialog::DialogRoot`]:
    /// `false` puts `data-dx-overlay="off"` on the `<dialog>`, which the theme
    /// turns into a transparent `::backdrop`; focus trap, inertness and
    /// click-outside dismissal are untouched.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub overlay: ReadSignal<bool>,

    /// The controlled `open` state of the drawer.
    pub open: ReadSignal<Option<bool>>,

    /// The default `open` state of the drawer if it is not controlled.
    #[props(default)]
    pub default_open: bool,

    /// A callback that is called when the open state changes.
    #[props(default)]
    pub on_open_change: Callback<bool>,

    /// Which edge of the viewport the drawer slides in from, and is dragged toward to
    /// dismiss. Defaults to [`EdgeSide::Auto`]: the bottom below the theme's `md`
    /// breakpoint, the right at and above it; an explicit side is exactly that side.
    #[props(default = ReadSignal::new(Signal::new(EdgeSide::Auto)))]
    pub side: ReadSignal<DrawerSide>,

    /// Whether the drawer can be dismissed by dragging
    /// [`DrawerHandle`]/[`DrawerContent`] toward `side`. Escape and
    /// [`DrawerClose`] still close the drawer when this is `false` -- only
    /// the drag gesture is disabled, mirroring Vaul's own `dismissible`
    /// prop.
    #[props(default = ReadSignal::new(Signal::new(true)))]
    pub dismissible: ReadSignal<bool>,

    /// The partial extents the panel can rest at (Base UI's `snapPoints`), smallest first by
    /// convention. The panel is as large as the largest and rests translated by the difference;
    /// a drag settles on the nearest point (the release velocity projects the release position),
    /// and dragging below the LOWEST point closes the drawer. Empty (the default): no snapping,
    /// the panel is as large as its content (up to 80% of the viewport) and a drag only closes it.
    #[props(default = ReadSignal::new(Signal::new(Vec::new())))]
    pub snap_points: ReadSignal<Vec<SnapPoint>>,

    /// The controlled active snap point (Base UI's `snapPoint`). `None` leaves it uncontrolled.
    /// A value that is not one of `snap_points` reads as the first.
    #[props(default)]
    pub active_snap_point: ReadSignal<Option<SnapPoint>>,

    /// The initial active snap point when uncontrolled; the first of `snap_points` by default. A
    /// closed drawer is back at it when it opens again.
    #[props(default)]
    pub default_active_snap_point: Option<SnapPoint>,

    /// Called with the new active snap point when a drag settles on another one.
    #[props(default)]
    pub on_active_snap_point_change: Callback<SnapPoint>,

    /// Additional attributes to apply to the drawer root element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the drawer root component.
    pub children: Element,
}

/// # Drawer
///
/// The entry point for the drawer -- a modal [`crate::dialog::DialogRoot`]
/// (see that component's own doc for the full modality/focus-trap/
/// scroll-lock behaviour, all inherited) plus the `side`/
/// `dismissible` configuration [`DrawerContent`]'s drag gesture consumes.
/// Manages the open state and provides context to its children; the
/// contents only render while open.
///
/// ## Example
///
/// ```rust
/// use dioxus::prelude::*;
/// use dioxus_primitives::drawer::{Drawer, DrawerContent, DrawerDescription, DrawerTitle};
///
/// #[component]
/// fn Demo() -> Element {
///     let mut open = use_signal(|| false);
///
///     rsx! {
///         button { onclick: move |_| open.set(true), "Show Drawer" }
///         Drawer {
///             open: open(),
///             on_open_change: move |v| open.set(v),
///             DrawerContent {
///                 DrawerTitle { "Item information" }
///                 DrawerDescription { "Here is some additional information about the item." }
///             }
///         }
///     }
/// }
/// ```
///
/// ## Styling
///
/// [`DrawerContent`] defines the following data attributes you can use to
/// control styling:
/// - `data-state`: `"open"` or `"closed"`.
/// - `data-open` / `data-closed`: Present (empty) while open / closed, absent otherwise.
/// - `data-side`: `"top"`, `"right"`, `"bottom"`, or `"left"` -- the RESOLVED side
///   (never `"auto"`; see [`EdgeSide::Auto`]).
/// - `data-dragging`: `"true"` while an active pointer drag is in
///   progress, `"false"` otherwise.
/// - `data-nested-open`: `"true"` while a drawer nested inside this one is open (the
///   theme scales and offsets this one), `"false"` otherwise.
/// - `data-snap`: with `snap_points`, `"partial"` while the active snap point is not the largest
///   (the content does not scroll and a press anywhere drags) and `"full"` at the largest; absent
///   without snap points.
/// - `data-dx-drawer`: always `"content"`; the marker `GESTURE_JS` uses to tell this
///   panel's presses from a nested drawer's.
#[component]
pub fn Drawer(props: DrawerRootProps) -> Element {
    let resolved = use_resolved_edge_side(props.side);

    // Snap state. Uncontrolled it lives here (outliving the content, which unmounts on close);
    // controlled it is the prop. The panel's resting offset is derived from it in
    // `DrawerContent`, never stored beside it.
    let default_active = props.default_active_snap_point;
    let mut internal_active = use_signal(move || default_active);
    let controlled = props.active_snap_point;
    let active_snap_point = use_memo(move || controlled().or_else(&*internal_active));
    let on_active_change = props.on_active_snap_point_change;
    let set_active_snap_point = use_callback(move |point: SnapPoint| {
        internal_active.set(Some(point));
        on_active_change.call(point);
    });
    let reset_active_snap_point = use_callback(move |_: ()| internal_active.set(default_active));

    // The viewport, for the release decision's px numbers. Only while there are snap points.
    let mut viewport = use_signal(|| None::<(f64, f64)>);
    let snap_points = props.snap_points;
    crate::js_listener::use_js_listeners(move || {
        if snap_points().is_empty() {
            return None;
        }
        Some(
            crate::js_listener::JsListeners::install(
                "const report = () => dioxus.send([window.innerWidth, window.innerHeight]);
                report();
                listen(window, 'resize', report);",
            )
            .on_message(move |size: (f64, f64)| viewport.set(Some(size))),
        )
    });

    use_context_provider(|| DrawerCtx {
        side: ReadSignal::from(resolved),
        dismissible: props.dismissible,
        snap_points,
        active_snap_point,
        set_active_snap_point,
        reset_active_snap_point,
        viewport: ReadSignal::from(viewport),
    });

    // A non-modal drawer has no scrim, whatever `overlay` says: with no top layer the root
    // wrapper is what paints it (and blocks the page), so it is switched off here, once.
    let is_modal = props.is_modal;
    let overlay = props.overlay;
    let scrim = use_memo(move || overlay() && is_modal());
    let attributes = merge_attributes(vec![
        props.attributes,
        attributes!(div {
            "data-dx-modal": (!is_modal()).then_some("false"),
        }),
    ]);

    rsx! {
        DialogRoot {
            id: props.id,
            is_modal: props.is_modal,
            overlay: ReadSignal::from(scrim),
            disable_pointer_dismissal: props.disable_pointer_dismissal,
            open: props.open,
            default_open: props.default_open,
            on_open_change: props.on_open_change,
            attributes,
            {props.children}
        }
    }
}

/// The props for the [`DrawerContent`] component.
#[derive(Props, Clone, PartialEq)]
pub struct DrawerContentProps {
    /// The ID of the drawer content element.
    pub id: ReadSignal<Option<String>>,

    /// The class to apply to the drawer content element.
    #[props(default)]
    pub class: Option<String>,

    /// Additional attributes to apply to the drawer content element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// The children of the drawer content.
    pub children: Element,
}

/// # DrawerContent
///
/// The content of the drawer -- must be used inside a [`Drawer`]. Renders
/// [`crate::dialog::DialogContent`] (so it inherits that primitive's focus
/// trap/restore, inertness, top layer and Escape handling) with
/// `data-side`/`data-state`/`data-dragging` data attributes and, while a
/// drag has moved the panel, an inline `transform` (see this module's doc
/// for why `transform` and not `translate`) tracking the pointer.
///
/// Owns the drag gesture for both itself and any [`DrawerHandle`] rendered
/// inside it -- see `GESTURE_JS`'s doc for why one listener on this
/// component's own root element covers both surfaces.
///
/// ## Example
///
/// See [`Drawer`]'s own example.
#[component]
pub fn DrawerContent(props: DrawerContentProps) -> Element {
    let drawer_ctx: DrawerCtx = use_context();
    let dialog_ctx: DialogCtx = use_context();
    let side = drawer_ctx.side;
    let dismissible = drawer_ctx.dismissible;

    let gen_id = use_unique_id();
    let id = use_id_or(gen_id, props.id);
    let id_opt = use_memo(move || Some(id()));

    // Nesting. The PARENT's context is looked up BEFORE this panel provides its own (a
    // lookup starts at the calling scope). While this drawer is open it is one of its
    // parent's `open_children`; the count leaves it the moment it starts closing (the
    // effect re-runs when `open` flips) and when it unmounts, so the parent is restored as
    // soon as the child begins to go, not after its exit has played.
    let parent_nest: Option<DrawerNestCtx> = try_use_context();
    let open_children = use_signal(|| 0_usize);
    use_context_provider(|| DrawerNestCtx { open_children });
    crate::use_effect_with_cleanup(move || {
        let counted = parent_nest.filter(|_| dialog_ctx.is_open());
        if let Some(parent) = counted {
            let mut count = parent.open_children;
            *count.write() += 1;
        }
        move || {
            if let Some(parent) = counted {
                let mut count = parent.open_children;
                if let Ok(mut open) = count.try_write() {
                    *open = open.saturating_sub(1);
                };
            }
        }
    });

    // Snap geometry. The resting offset itself is never computed here for the panel (the stylesheet
    // derives it from the active snap point's custom properties, below); these numbers exist only
    // for the live drag's clamps and the release decision, and come from the same snap list.
    let snap_points = drawer_ctx.snap_points;
    let active_snap_point = drawer_ctx.active_snap_point;
    let viewport = drawer_ctx.viewport;
    let set_active_snap_point = drawer_ctx.set_active_snap_point;
    let reset_active_snap_point = drawer_ctx.reset_active_snap_point;
    let rests = use_memo(move || {
        let (vertical, _) = side.cloned().drag_axis();
        let extent = viewport().map(|(w, h)| if vertical { h } else { w });
        snap_rests(&snap_points(), extent)
    });
    let active_index = use_memo(move || active_snap_index(&snap_points(), active_snap_point()));
    let active_rest = move || rests().get(active_index()).copied().unwrap_or(0.0);
    // An uncontrolled snap point is back at its default when the drawer closes and unmounts.
    use_drop(move || reset_active_snap_point.call(()));

    let mut dragging = use_signal(|| false);
    // Cumulative offset toward the closing edge, as `GESTURE_JS` reports it
    // (negative past the open position); the panel's extent along the drag
    // axis, measured by the script when the drag starts; the active snap point's rest when the
    // drag started (the room to drag toward open); and, once a drag has
    // CLOSED the drawer, the offset the panel was released at.
    let mut raw_offset = use_signal(|| 0.0_f64);
    let mut panel_size = use_signal(|| None::<f64>);
    let mut drag_room = use_signal(|| 0.0_f64);
    let mut released_offset = use_signal(|| None::<f64>);

    // Gesture: see this module's doc for why it is a real DOM script rather
    // than a declarative `onpointerdown`.
    use_drawer_gesture(id, side, dismissible, move |event| match event {
        DragEvent::Start { size } => {
            if !dismissible() || *dragging.peek() {
                return;
            }
            raw_offset.set(0.0);
            panel_size.set(Some(size));
            drag_room.set(active_rest());
            released_offset.set(None);
            dragging.set(true);
        }
        DragEvent::Move { offset } => {
            if *dragging.peek() {
                raw_offset.set(offset);
            }
        }
        DragEvent::End {
            velocity,
            cancelled,
        } => {
            if !*dragging.peek() {
                return;
            }
            let raw = *raw_offset.peek();
            let size = *panel_size.peek();
            let room = *drag_room.peek();
            dragging.set(false);
            let settle = if cancelled {
                Settle::Snap(active_index())
            } else {
                settle_on_release(&rests(), room, raw, size, velocity)
            };
            match settle {
                Settle::Close => {
                    // Hold the panel where the pointer left it: the exit keyframe
                    // (`translate`) then carries it the rest of the way from HERE.
                    // Resetting the offset to 0 instead would make the inline `transform` ease
                    // back to rest while the exit slid the panel out -- a visible hitch against
                    // the release point.
                    released_offset.set(Some(damped_display_offset(
                        raw,
                        size.map(|s| s - room),
                        room,
                    )));
                    dialog_ctx.set_open(false);
                }
                Settle::Snap(index) => {
                    // The new rest is the new active point's custom properties; the panel eases
                    // from the drag offset to it through the `transform` transition.
                    // The drag offset is cleared FIRST and the point is copied out of the list
                    // before the callback runs: the owner's handler re-renders the props this list
                    // is read from, so no borrow of it may be alive across the call.
                    raw_offset.set(0.0);
                    let point = snap_points.peek().get(index).copied();
                    if let Some(point) = point.filter(|_| index != active_index()) {
                        set_active_snap_point.call(point);
                    }
                }
            }
        }
    });

    // A drawer reopened while it was still closing starts from rest, not from
    // wherever the last drag released it. Reads `open` only; the writes are
    // guarded by a `peek`, so this never subscribes to what it writes.
    use_effect(move || {
        if dialog_ctx.is_open() && released_offset.peek().is_some() {
            released_offset.set(None);
            raw_offset.set(0.0);
        }
    });

    let is_open = dialog_ctx.is_open();
    let room = if dragging() {
        drag_room()
    } else {
        active_rest()
    };
    // The panel is displaced only WHILE a drag is live, or held where a drag-close released it: at
    // every other moment the drag offset is zero by construction, whatever `raw_offset` last held.
    let offset = match released_offset() {
        Some(released) => released,
        None if dragging() => {
            damped_display_offset(raw_offset(), panel_size().map(|s| s - room), room)
        }
        None => 0.0,
    };
    let points = snap_points();
    let snapped = !points.is_empty();
    let mut style = Vec::new();
    if snapped {
        let (vertical, _) = side.cloned().drag_axis();
        let active = points[active_index().min(points.len() - 1)];
        let largest = points
            .iter()
            .map(|p| p.css(vertical))
            .collect::<Vec<_>>()
            .join(", ");
        style.push(format!(
            "--dx-drawer-snap: {}; --dx-drawer-snap-max: max({largest})",
            active.css(vertical)
        ));
    }
    style.extend(drag_style(side.cloned(), offset, snapped));
    let style = (!style.is_empty()).then(|| style.join("; "));
    // `partial` while the active point is not the largest: the content does not scroll there and a
    // press anywhere drags (Base UI), `full` once it is fully extended.
    let snap_state = snapped.then(|| {
        if active_rest() > 0.5 {
            "partial"
        } else {
            "full"
        }
    });

    let content_base = attributes!(div {
        "data-dx-drawer": "content",
        "data-side": side.cloned().as_str(),
        ..state_attrs::open(is_open),
        ..state_attrs::valued(state_attrs::Flag::Dragging, dragging()),
        "data-nested-open": if open_children() > 0 { "true" } else { "false" },
        "data-snap": snap_state,
        style: style,
    });
    let content_attributes = merge_attributes(vec![content_base, props.attributes]);

    rsx! {
        DialogContent {
            id: ReadSignal::from(id_opt),
            class: props.class,
            attributes: content_attributes,
            {props.children}
        }
    }
}

/// The props for the [`DrawerHandle`] component.
#[derive(Props, Clone, PartialEq)]
pub struct DrawerHandleProps {
    /// Additional attributes to apply to the handle element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}

/// # DrawerHandle
///
/// A decorative grab bar, meant to be rendered inside [`DrawerContent`].
/// Purely a pointer-drag affordance (`aria-hidden="true"`): keyboard users
/// dismiss the drawer with Escape or [`DrawerClose`] instead.
///
/// This component owns no pointer-event logic of its own: starting a drag
/// here is handled by the *same* listener [`DrawerContent`] installs on
/// its own root element -- a native `pointerdown` bubbles up to it from
/// any descendant, this handle included. It only needs to render as a
/// recognizable surface (styled by the themed layer) somewhere inside a
/// [`DrawerContent`].
///
/// ## Example
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use dioxus_primitives::drawer::DrawerHandle;
/// # fn demo() -> Element {
/// rsx! {
///     DrawerHandle {}
/// }
/// # }
/// ```
#[component]
pub fn DrawerHandle(props: DrawerHandleProps) -> Element {
    let base = attributes!(div {
        "aria-hidden": "true",
    });
    let merged = merge_attributes(vec![base, props.attributes]);

    rsx! {
        div { ..merged }
    }
}

/// # DrawerTitle
///
/// The title of the drawer; it labels the drawer for accessibility. Thin
/// wrapper over [`crate::dialog::DialogTitle`] -- must be used inside a
/// [`DrawerContent`]. An accessible name is required:
/// [`crate::dialog::DialogRoot`] always wires the dialog's
/// `aria-labelledby` to a `DialogTitle` id whether or not one is rendered,
/// so omitting this leaves a broken reference (axe's `aria-dialog-name`
/// rule) rather than simply an unnamed drawer -- see
/// [`crate::command::CommandDialog`]'s doc for the same rule.
///
/// ## Example
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use dioxus_primitives::drawer::DrawerTitle;
/// # fn demo() -> Element {
/// rsx! {
///     DrawerTitle { "Move Goal" }
/// }
/// # }
/// ```
#[component]
pub fn DrawerTitle(props: DialogTitleProps) -> Element {
    rsx! {
        DialogTitle { id: props.id, attributes: props.attributes, {props.children} }
    }
}

/// # DrawerDescription
///
/// The description of the drawer; it describes the drawer for accessibility.
/// Thin wrapper over [`crate::dialog::DialogDescription`] -- must be used
/// inside a [`DrawerContent`].
///
/// ## Example
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use dioxus_primitives::drawer::DrawerDescription;
/// # fn demo() -> Element {
/// rsx! {
///     DrawerDescription { "Set your daily activity goal." }
/// }
/// # }
/// ```
#[component]
pub fn DrawerDescription(props: DialogDescriptionProps) -> Element {
    rsx! {
        DialogDescription { id: props.id, attributes: props.attributes, {props.children} }
    }
}

/// The props for the [`DrawerClose`] component.
#[derive(Props, Clone, PartialEq)]
pub struct DrawerCloseProps {
    /// Additional attributes to apply to the close button element.
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,

    /// Render a different element (e.g. an `a`) instead of the default
    /// `button`, while keeping the close behaviour -- shadcn/ui's
    /// `asChild` pattern. Mirrors
    /// `preview/src/components/sheet/component.rs`'s `SheetClose`.
    /// The callback gets the merged attributes (listeners included) and the children, see
    /// [`AsChild`].
    #[props(default)]
    pub r#as: Option<Callback<AsChild, Element>>,

    /// The children of the close button.
    pub children: Element,
}

/// # DrawerClose
///
/// A button that closes the drawer when activated. Thin wrapper over
/// [`crate::dialog::DialogCtx::set_open`] -- must be used inside a
/// [`Drawer`]. Mirrors
/// `preview/src/components/sheet/component.rs`'s `SheetClose`, which does
/// the same thing without a primitive-layer `DialogClose` to wrap (this
/// crate's `dialog.rs` does not define one); `DrawerClose` provides that
/// missing piece at the primitive layer for `Drawer` specifically.
///
/// ## Example
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use dioxus_primitives::drawer::DrawerClose;
/// # fn demo() -> Element {
/// rsx! {
///     DrawerClose { "Close" }
/// }
/// # }
/// ```
#[component]
pub fn DrawerClose(props: DrawerCloseProps) -> Element {
    let ctx: DialogCtx = use_context();

    let base = attributes!(button {
        onclick: move |_| {
            ctx.set_open(false);
        }
    });
    let merged = merge_attributes(vec![base, props.attributes]);

    if let Some(dynamic) = props.r#as {
        dynamic.call(AsChild::new(merged, props.children))
    } else {
        rsx! {
            button { ..merged, {props.children} }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_side_defaults_to_auto() {
        assert_eq!(EdgeSide::default(), EdgeSide::Auto);
        // The alias is the same type, not a lookalike.
        let _: DrawerSide = EdgeSide::Auto;
    }

    /// `Auto` is bottom below the `md` breakpoint and right at and above it; an explicit side is
    /// exactly that side at every width.
    #[test]
    fn auto_resolves_by_viewport_and_an_explicit_side_never_changes() {
        assert_eq!(EdgeSide::Auto.resolve(false), EdgeSide::Bottom);
        assert_eq!(EdgeSide::Auto.resolve(true), EdgeSide::Right);
        for side in [
            EdgeSide::Top,
            EdgeSide::Right,
            EdgeSide::Bottom,
            EdgeSide::Left,
        ] {
            assert_eq!(side.resolve(false), side);
            assert_eq!(side.resolve(true), side);
        }
        // A resolved side is never `auto`, so `data-side` never is.
        assert_ne!(EdgeSide::Auto.resolve(false).as_str(), "auto");
        assert_ne!(EdgeSide::Auto.resolve(true).as_str(), "auto");
    }

    /// The orientation every drag runs on: `(vertical, sign)` such that a raw delta
    /// times `sign` is positive toward the side's own closing edge. `GESTURE_JS`
    /// applies exactly this, so this table is the specification of which way each
    /// side closes -- a bottom drawer closes DOWN (never up), a top drawer UP, a
    /// right drawer RIGHT, a left drawer LEFT.
    #[test]
    fn drag_axis_points_toward_each_sides_own_closing_edge() {
        assert_eq!(EdgeSide::Bottom.drag_axis(), (true, 1.0));
        assert_eq!(EdgeSide::Top.drag_axis(), (true, -1.0));
        assert_eq!(EdgeSide::Right.drag_axis(), (false, 1.0));
        assert_eq!(EdgeSide::Left.drag_axis(), (false, -1.0));
    }

    /// Applies the script's orientation to a pointer delta, for the cases below.
    fn oriented(side: DrawerSide, delta_x: f64, delta_y: f64) -> f64 {
        let (vertical, sign) = side.drag_axis();
        (if vertical { delta_y } else { delta_x }) * sign
    }

    #[test]
    fn movement_toward_the_closing_edge_is_positive_and_away_from_it_negative() {
        // Screen y grows downward: a bottom drawer closes with +y, a top one with -y.
        assert!(oriented(EdgeSide::Bottom, 0.0, 10.0) > 0.0);
        assert!(oriented(EdgeSide::Bottom, 0.0, -10.0) < 0.0);
        assert!(oriented(EdgeSide::Top, 0.0, -10.0) > 0.0);
        assert!(oriented(EdgeSide::Top, 0.0, 10.0) < 0.0);
        assert!(oriented(EdgeSide::Right, 10.0, 0.0) > 0.0);
        assert!(oriented(EdgeSide::Right, -10.0, 0.0) < 0.0);
        assert!(oriented(EdgeSide::Left, -10.0, 0.0) > 0.0);
        assert!(oriented(EdgeSide::Left, 10.0, 0.0) < 0.0);
        // The cross axis never counts.
        assert_eq!(oriented(EdgeSide::Bottom, 500.0, 0.0), 0.0);
        assert_eq!(oriented(EdgeSide::Right, 0.0, 500.0), 0.0);
    }

    #[test]
    fn drag_event_decodes_the_wire_messages_and_ignores_unknown_ones() {
        assert_eq!(
            DragEvent::from_wire("start", 346.0, 0.0),
            Some(DragEvent::Start { size: 346.0 })
        );
        assert_eq!(
            DragEvent::from_wire("move", -12.5, 0.0),
            Some(DragEvent::Move { offset: -12.5 })
        );
        assert_eq!(
            DragEvent::from_wire("end", 0.8, 1.0),
            Some(DragEvent::End {
                velocity: 0.8,
                cancelled: true
            })
        );
        assert_eq!(DragEvent::from_wire("nonsense", 1.0, 1.0), None);
    }

    #[test]
    fn damped_display_offset_passes_through_toward_dismissal_up_to_panel_size() {
        assert_eq!(damped_display_offset(50.0, Some(400.0), 0.0), 50.0);
        // Clamped at the panel's own measured size, not left to overshoot.
        assert_eq!(damped_display_offset(500.0, Some(400.0), 0.0), 400.0);
        // No measurement yet -- pass through unclamped rather than stall
        // the drag.
        assert_eq!(damped_display_offset(500.0, None, 0.0), 500.0);
    }

    #[test]
    fn damped_display_offset_rubber_bands_overshoot_instead_of_hard_clamping_to_zero() {
        let damped = damped_display_offset(-40.0, Some(400.0), 0.0);
        // Neither the raw overshoot (a hard "no resistance") nor exactly
        // 0.0 (a dead stop) -- a damped value strictly between them.
        assert!(damped < 0.0 && damped > -40.0);
        // Capped at the rubber band's own maximum travel.
        assert_eq!(
            damped_display_offset(-1000.0, Some(400.0), 0.0),
            -RUBBER_BAND_MAX_PX
        );
    }

    #[test]
    fn damped_display_offset_is_a_pure_function_of_cumulative_raw_offset() {
        // The rule this module's own doc states: re-damping an
        // already-damped *display* value each frame would
        // make two equal cumulative raw offsets, reached via different
        // paths, disagree -- they must not.
        let via_one_big_step = damped_display_offset(-40.0, Some(400.0), 0.0);
        let raw_after_two_steps = -20.0 + -20.0;
        let via_two_small_steps = damped_display_offset(raw_after_two_steps, Some(400.0), 0.0);
        assert_eq!(via_one_big_step, via_two_small_steps);
    }

    #[test]
    fn drag_style_is_absent_at_rest_for_every_side() {
        // `0.0 * sign` is negative zero for `Top`/`Left` (`sign == -1.0`), which
        // `f64::Display` would otherwise render as the literal text `"-0"`. At rest there is no
        // inline style at all, so a stylesheet owns `transform` (the nested-drawer offset).
        for side in [
            EdgeSide::Top,
            EdgeSide::Right,
            EdgeSide::Bottom,
            EdgeSide::Left,
        ] {
            assert_eq!(drag_style(side, 0.0, false), None);
        }
    }

    #[test]
    fn drag_style_orients_toward_each_sides_own_edge() {
        let style = |side, offset| drag_style(side, offset, false).unwrap();
        assert_eq!(style(EdgeSide::Bottom, 40.0), "transform: translateY(40px)");
        assert_eq!(style(EdgeSide::Top, 40.0), "transform: translateY(-40px)");
        assert_eq!(style(EdgeSide::Right, 40.0), "transform: translateX(40px)");
        assert_eq!(style(EdgeSide::Left, 40.0), "transform: translateX(-40px)");
    }

    #[test]
    fn should_close_on_release_past_distance_threshold() {
        // 30% of a 400px panel exceeds the 25% ratio threshold.
        assert!(should_close_on_release(120.0, Some(400.0), 0.0));
        // 20% does not, and the velocity is below threshold too.
        assert!(!should_close_on_release(80.0, Some(400.0), 0.0));
    }

    #[test]
    fn should_close_on_release_past_velocity_threshold_even_short_of_distance() {
        // Only 5% of the panel's distance, but flung faster than 0.5px/ms.
        assert!(should_close_on_release(20.0, Some(400.0), 0.8));
    }

    #[test]
    fn should_close_on_release_never_closes_on_overshoot_regardless_of_velocity() {
        // A fast release while still past the resting position (dragging
        // the *wrong* way) must never be read as a dismiss.
        assert!(!should_close_on_release(-10.0, Some(400.0), 5.0));
        assert!(!should_close_on_release(0.0, Some(400.0), 5.0));
    }

    #[test]
    fn should_close_on_release_with_no_measured_panel_size_falls_back_to_velocity_only() {
        assert!(!should_close_on_release(1000.0, None, 0.0));
        assert!(should_close_on_release(40.0, None, 0.6));
    }

    #[test]
    fn should_close_on_release_ignores_a_fast_twitch_that_travelled_almost_nowhere() {
        // 3px in 2ms is 1.5px/ms, but it is jitter, not a flick.
        assert!(!should_close_on_release(3.0, Some(400.0), 1.5));
        // The same speed after a real throw is.
        assert!(should_close_on_release(30.0, Some(400.0), 1.5));
    }

    #[test]
    fn should_close_on_release_never_closes_a_drag_toward_the_open_side() {
        // Dragging a bottom drawer UP (negative, oriented) --
        // however far, however fast -- is never a dismissal.
        assert!(!should_close_on_release(-300.0, Some(400.0), 2.0));
        assert!(!should_close_on_release(-300.0, Some(400.0), -2.0));
    }

    // ---- snap points ----

    const HALF_AND_FULL: [SnapPoint; 2] = [SnapPoint::Fraction(0.5), SnapPoint::Fraction(1.0)];

    #[test]
    fn rests_are_the_distance_below_the_largest_point_and_zero_without_points() {
        assert_eq!(snap_rests(&[], Some(800.0)), vec![0.0]);
        assert_eq!(snap_rests(&HALF_AND_FULL, Some(800.0)), vec![400.0, 0.0]);
        // Px points, a point past the viewport clamps to it.
        let points = [
            SnapPoint::Px(148.0),
            SnapPoint::Px(355.0),
            SnapPoint::Px(5000.0),
        ];
        assert_eq!(snap_rests(&points, Some(800.0)), vec![652.0, 445.0, 0.0]);
        // Unknown viewport: every point at rest zero rather than a guess.
        assert_eq!(snap_rests(&HALF_AND_FULL, None), vec![0.0, 0.0]);
    }

    #[test]
    fn the_active_snap_point_is_found_in_the_list_else_the_first() {
        assert_eq!(
            active_snap_index(&HALF_AND_FULL, Some(SnapPoint::Fraction(1.0))),
            1
        );
        assert_eq!(active_snap_index(&HALF_AND_FULL, None), 0);
        assert_eq!(
            active_snap_index(&HALF_AND_FULL, Some(SnapPoint::Px(10.0))),
            0
        );
    }

    #[test]
    fn snap_points_render_as_css_lengths_on_the_drag_axis() {
        assert_eq!(SnapPoint::Fraction(0.5).css(true), "calc(0.5 * 100dvh)");
        assert_eq!(SnapPoint::Fraction(0.5).css(false), "calc(0.5 * 100dvw)");
        assert_eq!(SnapPoint::Px(355.0).css(true), "min(355px, 100dvh)");
    }

    /// Rests for a 800px viewport with 0.25 / 0.5 / 1.0: 600, 400, 0 (panel 800 tall).
    const REST: [f64; 3] = [600.0, 400.0, 0.0];

    #[test]
    fn a_slow_release_settles_on_the_nearest_snap_point_by_distance() {
        // From the middle point dragged 60px toward open: nearer 400 than 0.
        assert_eq!(
            settle_on_release(&REST, 400.0, -60.0, Some(800.0), 0.0),
            Settle::Snap(1)
        );
        // 250px toward open: nearer 0 (position 150).
        assert_eq!(
            settle_on_release(&REST, 400.0, -250.0, Some(800.0), 0.0),
            Settle::Snap(2)
        );
        // 120px toward closing from the middle (position 520): nearer 600.
        assert_eq!(
            settle_on_release(&REST, 400.0, 120.0, Some(800.0), 0.0),
            Settle::Snap(0)
        );
    }

    #[test]
    fn a_flick_projects_the_release_position_to_another_point() {
        // 30px toward open, flung up at 1.2px/ms: projects 180px further, to ~190 -> full.
        assert_eq!(
            settle_on_release(&REST, 400.0, -30.0, Some(800.0), -1.2),
            Settle::Snap(2)
        );
        // The same 30px with no velocity stays.
        assert_eq!(
            settle_on_release(&REST, 400.0, -30.0, Some(800.0), 0.0),
            Settle::Snap(1)
        );
        // A flick down from the top lands on the middle, not past the lowest.
        assert_eq!(
            settle_on_release(&REST, 0.0, 40.0, Some(800.0), 1.4),
            Settle::Snap(1)
        );
    }

    #[test]
    fn only_dragging_below_the_lowest_point_closes() {
        // From the lowest (rest 600, 200px visible): 40px down is 20%: stays; 80px is 40%: closes.
        assert_eq!(
            settle_on_release(&REST, 600.0, 40.0, Some(800.0), 0.0),
            Settle::Snap(0)
        );
        assert_eq!(
            settle_on_release(&REST, 600.0, 80.0, Some(800.0), 0.0),
            Settle::Close
        );
        // A flick from the lowest closes (the one existing flick rule).
        assert_eq!(
            settle_on_release(&REST, 600.0, 20.0, Some(800.0), 0.9),
            Settle::Close
        );
        // A long drag from the TOP that stays above the lowest point never closes.
        assert_eq!(
            settle_on_release(&REST, 0.0, 500.0, Some(800.0), 0.0),
            Settle::Snap(0)
        );
    }

    #[test]
    fn without_snap_points_the_release_is_a_plain_close_or_stay() {
        assert_eq!(
            settle_on_release(&[0.0], 0.0, 120.0, Some(400.0), 0.0),
            Settle::Close
        );
        assert_eq!(
            settle_on_release(&[0.0], 0.0, 80.0, Some(400.0), 0.0),
            Settle::Snap(0)
        );
        assert_eq!(
            settle_on_release(&[0.0], 0.0, -80.0, Some(400.0), -3.0),
            Settle::Snap(0)
        );
    }

    #[test]
    fn dragging_toward_open_is_free_up_to_the_snap_rest_then_rubber_banded() {
        // 400px of room: a 300px drag toward open moves 1:1, the overshoot is damped.
        assert_eq!(damped_display_offset(-300.0, Some(400.0), 400.0), -300.0);
        let over = damped_display_offset(-440.0, Some(400.0), 400.0);
        assert!(over < -400.0 && over > -440.0);
        assert_eq!(
            damped_display_offset(-5000.0, Some(400.0), 400.0),
            -400.0 - RUBBER_BAND_MAX_PX
        );
    }

    #[test]
    fn a_snapped_drag_writes_the_drag_variable_and_never_drops_it() {
        let style = |side, offset| drag_style(side, offset, true).unwrap();
        assert_eq!(style(EdgeSide::Bottom, 40.0), "--dx-drawer-drag: 40px");
        assert_eq!(style(EdgeSide::Left, 40.0), "--dx-drawer-drag: -40px");
        assert_eq!(style(EdgeSide::Top, -12.5), "--dx-drawer-drag: 12.5px");
        // At rest the declaration stays (value 0), for every side, so it is never removed.
        for side in [
            EdgeSide::Top,
            EdgeSide::Right,
            EdgeSide::Bottom,
            EdgeSide::Left,
        ] {
            assert_eq!(style(side, 0.0), "--dx-drawer-drag: 0px");
        }
    }
}
