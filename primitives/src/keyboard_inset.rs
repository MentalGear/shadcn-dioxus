//! `use_keyboard_inset` / [`KeyboardInset`]: lets a modal panel with a text field in it follow the
//! on-screen keyboard.
//!
//! **The gap.** On iOS Safari (and Chromium's default `resizes-visual` mode on Android) opening the
//! keyboard shrinks the VISUAL viewport only. The layout viewport -- the box a `position: fixed`
//! panel is placed in, `100%` of it, `dvh` of it -- keeps its size, so a bottom Drawer's footer, a
//! centred Dialog's lower fields and a Sheet's last input all end up behind the keyboard, and the
//! browser answers by scrolling the page behind the (inert) modal. Anchored popovers already follow
//! the visual viewport (`top_layer.rs`'s `reposition()`); a modal panel is placed by CSS and needs
//! the numbers handed to it.
//!
//! **The construction.** While a panel is mounted, one bridge session streams the visual viewport's size,
//! offset and scale (`resize` and `scroll` of `window.visualViewport`, `resize` of the window, and `focusin`
//! of the document), every message carrying the same four numbers. A pure Rust function ([`keyboard_of`])
//! decides from them whether a keyboard is open and what the stylesheet needs, and the writes go back
//! through the bridge ON THE PANEL (only when the answer changes):
//!
//! * `--dx-keyboard-inset`: the distance, in `px`, from the visual viewport's bottom edge to the
//!   layout viewport's (`innerHeight - (vv.offsetTop + vv.height)`);
//! * `--dx-keyboard-top`: the visual viewport's top offset in the layout viewport (`vv.offsetTop`,
//!   `0` unless the browser has panned it);
//! * `data-dx-keyboard="open"`.
//!
//! All three exist ONLY while the keyboard is open, so a rule written against the attribute leaves
//! the panel byte-for-byte as it is without a keyboard. A sheet consumes them as
//! `.dx-x[data-dx-keyboard] { bottom: var(--dx-keyboard-inset, 0px); max-height: calc(100% - ...) }`
//! (always with a `0px` fallback: the variable is absent without a keyboard).
//!
//! The field the user is typing in is brought into view: when it takes focus, or the keyboard
//! opens or changes height, the nearest vertical scroller (else the panel) gets scroll room for the part of
//! it the keyboard covers (`padding-bottom += overlap`, over a floor of 16px under the content; none
//! when the scroller already ends above the keyboard, which the stylesheet contract above arranges
//! for a panel), and the field is scrolled to the centre of the visible band unless it is already
//! comfortably inside it ([`slack_step`], [`scroll_delta`]: pure functions over the rects the page
//! reports). Everything is undone when the keyboard closes or the panel unmounts.
//!
//! **What counts as a keyboard.** Lifted from Base UI 1.9.0 (MIT),
//! `drawer/virtual-keyboard-provider/DrawerVirtualKeyboardProvider.js`
//! (`KEYBOARD_RESIZE_THRESHOLD = 60`, `KEYBOARD_VISIBILITY_MARGIN = 16`): the keyboard is open when
//! `innerHeight - visualViewport.height` exceeds 60 and `visualViewport.scale === 1`. A smaller change is browser chrome (a collapsing URL bar);
//! a pinch-zoomed page is not a keyboard, and the logic is suspended while zoomed. The layout
//! viewport is read again on a window `resize` (a browser that resizes it after the visual viewport
//! events stop). Adapted: the inset is published as a stylesheet contract on the panel (not on a
//! provider's viewport element), a top offset is published beside it so a centred panel can be
//! placed inside the visible band, the field is scrolled only when it is not already visible, and
//! none of vaul's imperative `bottom` / `height` writes or body `position: fixed` lock is used.
//!
//! **Not ported from Base UI 1.9.0** (backlog row 480), because each is script this file may not
//! grow (`dev-docs/js-interop.md`, `check-js-volume.sh`) or needs a real iPhone first: the touch-tap
//! focus trick (`KEYBOARD_TAP_BLOCKED`, focus synchronously in `touchend` behind a
//! `translateY(-2000px)`); the settle-watching realign passes (`KEYBOARD_REALIGN_INTERVAL = 150`,
//! `KEYBOARD_SETTLE_FRAME_LIMIT = 60`) that re-run the alignment while WebKit's native reveal or a
//! layout transition changes the destination; pinning the page scroll while a modal's keyboard is
//! up; the focus pre-emption for the keyboard's previous/next arrows; and the probe that keeps the
//! keyboard detected in a browser whose layout viewport shrinks with it (there `innerHeight` follows
//! `visualViewport.height`, the contract is absent and the browser places the panel itself).
//!
//! This is not part of `use_dialog_open_driver` / `use_popover_sync`: those own open-time focus and
//! anchored placement. The styled Drawer, Dialog, Sheet and AlertDialog call it.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use dioxus::prelude::*;
use serde::Deserialize;

use crate::bridge::{
    use_media_query, Command, ElementExt, EventField, EventKind, ListenOptions, Prop, ScrollMotion,
    Session, Slot, Target, Transaction,
};
use crate::{use_id_or, use_unique_id};

/// Base UI's `KEYBOARD_RESIZE_THRESHOLD`: the least `innerHeight - visualViewport.height`, in px,
/// that is the keyboard and not browser chrome.
pub const KEYBOARD_RESIZE_THRESHOLD_PX: u32 = 60;
/// Base UI's `KEYBOARD_VISIBILITY_MARGIN`: the gap, in px, kept between the focused field and the
/// edges of the visible band, and the least padding under a scroller's content once the keyboard
/// covers part of it.
pub const KEYBOARD_VISIBILITY_MARGIN_PX: u32 = 16;

const THRESHOLD: f64 = KEYBOARD_RESIZE_THRESHOLD_PX as f64;
const MARGIN: f64 = KEYBOARD_VISIBILITY_MARGIN_PX as f64;

/// The visual viewport against the layout viewport: what one message from the page carries.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Viewport {
    /// `window.innerHeight`.
    inner_height: f64,
    /// `visualViewport.height`.
    height: f64,
    /// `visualViewport.offsetTop`: how far the browser has panned the visual viewport.
    offset_top: f64,
    /// `visualViewport.scale`: 1 unless the page is pinch-zoomed.
    scale: f64,
}

/// Whether a keyboard is open, and what a stylesheet needs of it, in whole px.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Keyboard {
    /// No keyboard: the panel carries none of the contract.
    Closed,
    /// A keyboard is open.
    Open {
        /// `--dx-keyboard-inset`: the distance from the visual viewport's bottom edge to the layout
        /// viewport's.
        inset: i64,
        /// `--dx-keyboard-top`: the visual viewport's top offset in the layout viewport.
        top: i64,
    },
}

/// The keyboard is open when the layout viewport is more than [`KEYBOARD_RESIZE_THRESHOLD_PX`] taller than
/// the visual one and the page is not pinch-zoomed (a smaller change is browser chrome, a zoomed page is
/// not a keyboard).
fn keyboard_of(viewport: Viewport) -> Keyboard {
    let open =
        viewport.inner_height - viewport.height > THRESHOLD && (viewport.scale - 1.0).abs() < 0.001;
    if !open {
        return Keyboard::Closed;
    }
    Keyboard::Open {
        inset: (viewport.inner_height - viewport.offset_top - viewport.height)
            .round()
            .max(0.0) as i64,
        top: viewport.offset_top.round().max(0.0) as i64,
    }
}

/// Only the keyboard opening or changing height re-reveals the field; a pan (`top`) alone does not.
fn needs_reveal(before: Keyboard, after: Keyboard) -> bool {
    match (before, after) {
        (_, Keyboard::Closed) => false,
        (Keyboard::Open { inset: a, .. }, Keyboard::Open { inset: b, .. }) => a != b,
        (Keyboard::Closed, Keyboard::Open { .. }) => true,
    }
}

/// The scroll room one scroller holds for the keyboard: the inline values to give back and the padding the
/// room is added to.
#[derive(Clone, Debug, PartialEq)]
struct Slack {
    /// The scroller's own inline `padding-bottom`, given back with the room (empty when unset).
    padding: String,
    /// The scroller's own inline `overflow-anchor`, given back with the room (empty when unset).
    anchor: String,
    /// The padding the room is added to: the scroller's own, over a floor of [`KEYBOARD_VISIBILITY_MARGIN_PX`].
    base: f64,
}

/// What to do about the scroll room of the field's scroller.
#[derive(Clone, Debug, PartialEq)]
enum SlackStep {
    /// Make the scroller's `padding-bottom` `padding` px, over `base`.
    Add {
        /// The padding the room is added to.
        base: f64,
        /// The padding to write, in px.
        padding: f64,
    },
    /// Nothing is covered: give any room back.
    Release,
}

/// Scroll room is exactly what the keyboard covers (`overlap`, in px) over a floor of the margin: no fixed
/// extra. A scroller that already holds room keeps the base it was first given, so the room never compounds.
fn slack_step(overlap: f64, held: Option<&Slack>, computed_padding: f64) -> SlackStep {
    if overlap <= 0.0 {
        return SlackStep::Release;
    }
    let base = held.map_or_else(|| computed_padding.max(MARGIN), |slack| slack.base);
    SlackStep::Add {
        base,
        padding: base + overlap,
    }
}

/// How far to scroll the scroller (its rect `scroller`, as `(top, bottom)`) to bring `field` to the centre of
/// the visible band (`visible`, the visual viewport's top and bottom) inside it, or `None` when the field is
/// already inside the band less a margin each side.
fn scroll_delta(scroller: (f64, f64), field: (f64, f64), visible: (f64, f64)) -> Option<f64> {
    let band_top = scroller.0.max(visible.0) + MARGIN;
    let band_bottom = scroller.1.min(visible.1) - MARGIN;
    if field.0 >= band_top && field.1 <= band_bottom {
        return None;
    }
    Some((field.0 + field.1) / 2.0 - (band_top + band_bottom) / 2.0)
}

/// What the page's `matches` answers for a text field the keyboard serves: a textarea, an input that takes
/// text, or editable content.
const TEXT_FIELD: &str = "textarea, \
    input:not([type=checkbox i]):not([type=radio i]):not([type=button i]):not([type=submit i]):not([type=reset i]):not([type=range i]):not([type=file i]):not([type=color i]):not([type=image i]):not([type=hidden i]), \
    :is([contenteditable=\"\"], [contenteditable=true i], [contenteditable=plaintext-only i]), \
    :is([contenteditable=\"\"], [contenteditable=true i], [contenteditable=plaintext-only i]) *:not([contenteditable=false i])";

/// The one message of the session: the viewport numbers, and the event they came with.
#[derive(Deserialize)]
struct Fact {
    #[serde(rename = "type")]
    event: String,
    #[serde(rename = "viewport.innerHeight")]
    inner_height: Option<f64>,
    #[serde(rename = "viewport.height")]
    height: Option<f64>,
    #[serde(rename = "viewport.offsetTop")]
    offset_top: Option<f64>,
    #[serde(rename = "viewport.scale")]
    scale: Option<f64>,
}

impl Fact {
    fn viewport(&self) -> Option<Viewport> {
        Some(Viewport {
            inner_height: self.inner_height?,
            height: self.height?,
            offset_top: self.offset_top?,
            scale: self.scale?,
        })
    }
}

/// The state one panel's session keeps between messages.
#[derive(Default)]
struct State {
    /// What the panel carries now.
    keyboard: Cell<Option<Keyboard>>,
    /// The latest viewport the page reported.
    viewport: Cell<Option<Viewport>>,
    /// The scroll room a scroller holds, if one does.
    slack: RefCell<Option<Slack>>,
    /// A reveal is running; `again` asks it for another pass.
    revealing: Cell<bool>,
    again: Cell<bool>,
}

impl State {
    fn is_open(&self) -> bool {
        matches!(self.keyboard.get(), Some(Keyboard::Open { .. }))
    }
}

/// The scroller that holds scroll room, wherever it is inside the panel (the panel itself included).
fn slacked(panel_id: &str) -> Target {
    Target::selector(format!(
        "[id=\"{panel_id}\"][{attr}], [id=\"{panel_id}\"] [{attr}]",
        attr = Slot::KeyboardSlack.wire()
    ))
}

/// Gives the scroll room back: the inline values the scroller had, and no marker.
fn give_back(tx: Transaction, scroller: Target, slack: &Slack) -> Transaction {
    tx.write(
        scroller.clone(),
        Slot::PaddingBottom,
        Some(slack.padding.clone()),
    )
    .write(
        scroller.clone(),
        Slot::OverflowAnchor,
        Some(slack.anchor.clone()),
    )
    .write(scroller, Slot::KeyboardSlack, None)
}

/// Removes everything the panel carries for the keyboard.
fn clear(panel_id: &str, state: &State) {
    let panel = Target::id(panel_id);
    let mut tx = Transaction::new()
        .set_var(panel.clone(), "--dx-keyboard-inset", None)
        .set_var(panel.clone(), "--dx-keyboard-top", None)
        .write(panel, Slot::KeyboardOpen, None);
    if let Some(slack) = state.slack.borrow_mut().take() {
        tx = give_back(tx, slacked(panel_id), &slack);
    }
    tx.fire();
}

/// Applies the keyboard the viewport now shows to the panel, and reveals the field when it should.
fn update(panel_id: &str, state: &Rc<State>, viewport: Viewport, reduced: ReadSignal<bool>) {
    state.viewport.set(Some(viewport));
    let next = keyboard_of(viewport);
    let before = state
        .keyboard
        .replace(Some(next))
        .unwrap_or(Keyboard::Closed);
    match next {
        Keyboard::Closed => {
            if before != Keyboard::Closed {
                clear(panel_id, state);
            }
        }
        Keyboard::Open { inset, top } => {
            if before != next {
                let panel = Target::id(panel_id);
                Transaction::new()
                    .set_var(
                        panel.clone(),
                        "--dx-keyboard-inset",
                        Some(format!("{inset}px")),
                    )
                    .set_var(panel.clone(), "--dx-keyboard-top", Some(format!("{top}px")))
                    .write(panel, Slot::KeyboardOpen, Some("open".into()))
                    .fire();
            }
        }
    }
    if needs_reveal(before, next) {
        reveal(panel_id, state, reduced);
    }
}

/// Asks for the field to be brought into view: one pass runs at a time, and a request that arrives during
/// one makes it run again.
fn reveal(panel_id: &str, state: &Rc<State>, reduced: ReadSignal<bool>) {
    if state.revealing.replace(true) {
        state.again.set(true);
        return;
    }
    let (panel_id, state) = (panel_id.to_string(), state.clone());
    spawn(async move {
        loop {
            state.again.set(false);
            bring_into_view(&panel_id, &state, reduced).await;
            if !state.again.get() {
                break;
            }
        }
        state.revealing.set(false);
    });
}
/// The element id of a modal panel, for [`KeyboardInset`]: the caller's own `id` when it set one,
/// else a generated one. Build it with [`use_keyboard_panel`].
#[derive(Clone, Copy, PartialEq)]
pub struct KeyboardPanel {
    id: Memo<String>,
    id_prop: Memo<Option<String>>,
}

impl KeyboardPanel {
    /// The id to give the panel element (`DialogContent { id: panel.id(), .. }`).
    pub fn id(&self) -> ReadSignal<Option<String>> {
        ReadSignal::from(self.id_prop)
    }
}

/// Settles the id of a modal panel so [`KeyboardInset`] can find it: `user_id` when given and set,
/// else a generated, hydration-safe one. Call it in the component that renders the panel's content
/// component and pass [`KeyboardPanel::id`] as that component's `id`.
pub fn use_keyboard_panel(user_id: Option<ReadSignal<Option<String>>>) -> KeyboardPanel {
    let generated = use_unique_id();
    let none = use_hook(|| ReadSignal::new(Signal::new(None)));
    let user_id = user_id.unwrap_or(none);
    let id = use_id_or(generated, user_id);
    let id_prop = use_memo(move || Some(id()));
    KeyboardPanel { id, id_prop }
}

/// One pass of bringing the focused field into view: reads, in one page turn, the focused element, the
/// field's scroller and their rects; adds or gives back scroll room and reads the rects again in the next
/// turn (the room changes the layout); then scrolls. Every number is the page's, every decision is
/// [`slack_step`] and [`scroll_delta`].
async fn bring_into_view(panel_id: &str, state: &State, reduced: ReadSignal<bool>) {
    let (Some(viewport), true) = (state.viewport.get(), state.is_open()) else {
        return;
    };
    let scroller = Target::scroller(panel_id);
    let Ok(read) = Transaction::new()
        .get(Target::id(panel_id), [Prop::ActiveElement])
        .get(Target::Active, [Prop::Matches(TEXT_FIELD.into())])
        .get(
            scroller.clone(),
            [
                Prop::Rect,
                Prop::Style("padding-bottom".into()),
                Prop::Inline("padding-bottom".into()),
                Prop::Inline("overflow-anchor".into()),
                Prop::Attr(Slot::KeyboardSlack.wire().into()),
            ],
        )
        .run()
        .await
    else {
        return;
    };
    let [panel, active, held] = read.as_slice() else {
        return;
    };
    let (Some(panel), Some(active), Some(held)) = (panel.props(), active.props(), held.props())
    else {
        return;
    };
    let within = panel
        .value(&Prop::ActiveElement)
        .and_then(|element| element.get("within"))
        .and_then(serde_json::Value::as_bool);
    if within != Some(true) || active.bool(&Prop::Matches(TEXT_FIELD.into())) != Some(true) {
        return;
    }
    let Some(scroller_rect) = held.rect() else {
        return;
    };
    let visible = (viewport.offset_top, viewport.offset_top + viewport.height);
    let overlap = (scroller_rect.bottom - visible.1).max(0.0);
    let marked = held
        .string(&Prop::Attr(Slot::KeyboardSlack.wire().into()))
        .is_some();
    let step = {
        let held_here = state.slack.borrow().clone().filter(|_| marked);
        slack_step(
            overlap,
            held_here.as_ref(),
            held.px(&Prop::Style("padding-bottom".into()))
                .unwrap_or(0.0),
        )
    };

    let mut tx = Transaction::new();
    // room held by another scroller goes back first
    if !marked {
        if let Some(other) = state.slack.borrow_mut().take() {
            tx = give_back(tx, slacked(panel_id), &other);
        }
    }
    match step {
        SlackStep::Add { base, padding } => {
            if !marked {
                *state.slack.borrow_mut() = Some(Slack {
                    padding: held
                        .string(&Prop::Inline("padding-bottom".into()))
                        .unwrap_or_default()
                        .to_string(),
                    anchor: held
                        .string(&Prop::Inline("overflow-anchor".into()))
                        .unwrap_or_default()
                        .to_string(),
                    base,
                });
                tx = tx.write(scroller.clone(), Slot::KeyboardSlack, Some(String::new()));
            }
            tx = tx
                .write(scroller.clone(), Slot::OverflowAnchor, Some("none".into()))
                .write(
                    scroller.clone(),
                    Slot::PaddingBottom,
                    Some(format!("{padding}px")),
                );
        }
        SlackStep::Release => {
            if let Some(slack) = state.slack.borrow_mut().take().filter(|_| marked) {
                tx = give_back(tx, scroller.clone(), &slack);
            }
        }
    }
    let Ok(laid_out) = tx
        .get(scroller.clone(), [Prop::Rect])
        .get(Target::Active, [Prop::Rect])
        .run()
        .await
    else {
        return;
    };
    let [.., scroller_after, field_after] = laid_out.as_slice() else {
        return;
    };
    let (Some(scroller_rect), Some(field_rect)) = (
        scroller_after.props().and_then(|p| p.rect()),
        field_after.props().and_then(|p| p.rect()),
    ) else {
        return;
    };
    if !state.is_open() {
        return;
    }
    let visible = state
        .viewport
        .get()
        .map_or(visible, |v| (v.offset_top, v.offset_top + v.height));
    if let Some(top) = scroll_delta(
        (scroller_rect.top, scroller_rect.bottom),
        (field_rect.top, field_rect.bottom),
        visible,
    ) {
        scroller.scroll_by_top(
            top,
            if reduced() {
                ScrollMotion::Auto
            } else {
                ScrollMotion::Smooth
            },
        );
    }
}

/// Publishes `--dx-keyboard-inset`, `--dx-keyboard-top` and `data-dx-keyboard` on the panel with
/// element id `panel_id` while the on-screen keyboard is open, and scrolls the focused field into
/// view. See the module docs. Active for as long as the calling component is mounted, so call it
/// from a component that lives exactly as long as the panel (see [`KeyboardInset`]).
pub fn use_keyboard_inset(panel_id: Memo<String>) {
    let reduced = use_media_query("(prefers-reduced-motion: reduce)");
    crate::use_effect_with_cleanup(move || {
        let panel_id = panel_id.cloned();
        let state = Rc::new(State::default());
        let fields = [
            EventField::Type,
            EventField::ViewportInnerHeight,
            EventField::ViewportHeight,
            EventField::ViewportOffsetTop,
            EventField::ViewportScale,
        ];
        let passive = ListenOptions::default();
        let subscription = Session::new()
            .command(Command::Listen {
                target: Target::VisualViewport,
                event: EventKind::Resize,
                options: passive,
                fields: fields.to_vec(),
                initial: true,
            })
            .listen(Target::VisualViewport, EventKind::Scroll, passive, &fields)
            .listen(Target::Window, EventKind::Resize, passive, &fields)
            .listen(
                Target::Document,
                EventKind::Focusin,
                ListenOptions::CAPTURE,
                &fields,
            )
            .open({
                let (panel_id, state) = (panel_id.clone(), state.clone());
                move |fact: Fact| {
                    let Some(viewport) = fact.viewport() else {
                        return;
                    };
                    update(&panel_id, &state, viewport, reduced);
                    // a field that takes focus is brought into view even when the keyboard did not change
                    if fact.event == "focusin" {
                        reveal(&panel_id, &state, reduced);
                    }
                }
            });
        move || {
            drop(subscription);
            clear(&panel_id, &state);
        }
    });
}

/// Renders nothing and runs [`use_keyboard_inset`] for `panel`. Put it inside the panel's content
/// (`DialogContent { id: panel.id(), {children} KeyboardInset { panel } }`): the content is mounted
/// only while the modal is open or animating out, so the listeners exist exactly then.
#[component]
pub fn KeyboardInset(
    /// The panel, from [`use_keyboard_panel`].
    panel: KeyboardPanel,
) -> Element {
    use_keyboard_inset(panel.id);
    rsx! {}
}

#[cfg(test)]
mod tests {
    use super::*;

    fn viewport(keyboard: f64, offset_top: f64, scale: f64) -> Viewport {
        Viewport {
            inner_height: 844.0,
            height: (844.0 - keyboard) / scale,
            offset_top,
            scale,
        }
    }

    /// The keyboard is `innerHeight - visualViewport.height` over 60px: exactly 60 is browser chrome.
    #[test]
    fn a_keyboard_is_more_than_the_threshold() {
        assert_eq!(keyboard_of(viewport(0.0, 0.0, 1.0)), Keyboard::Closed);
        assert_eq!(keyboard_of(viewport(44.0, 0.0, 1.0)), Keyboard::Closed);
        assert_eq!(keyboard_of(viewport(60.0, 0.0, 1.0)), Keyboard::Closed);
        assert_eq!(
            keyboard_of(viewport(61.0, 0.0, 1.0)),
            Keyboard::Open { inset: 61, top: 0 }
        );
        assert_eq!(
            keyboard_of(viewport(336.0, 0.0, 1.0)),
            Keyboard::Open { inset: 336, top: 0 }
        );
    }

    /// Pinch zoom suspends the logic: a zoomed page is not a keyboard.
    #[test]
    fn a_zoomed_viewport_is_not_a_keyboard() {
        assert_eq!(keyboard_of(viewport(336.0, 0.0, 1.5)), Keyboard::Closed);
        assert_eq!(keyboard_of(viewport(336.0, 0.0, 0.5)), Keyboard::Closed);
        assert!(matches!(
            keyboard_of(viewport(336.0, 0.0, 1.0005)),
            Keyboard::Open { .. }
        ));
    }

    /// The inset is measured from the visual viewport's bottom edge, the top is the pan, both whole px and
    /// never negative.
    #[test]
    fn the_inset_and_the_top_follow_the_pan_in_whole_pixels() {
        assert_eq!(
            keyboard_of(Viewport {
                inner_height: 844.0,
                height: 508.4,
                offset_top: 100.6,
                scale: 1.0,
            }),
            Keyboard::Open {
                inset: 235,
                top: 101
            }
        );
        assert_eq!(
            keyboard_of(Viewport {
                inner_height: 844.0,
                height: 500.0,
                offset_top: 400.0,
                scale: 1.0,
            }),
            Keyboard::Open { inset: 0, top: 400 }
        );
    }

    /// Only the keyboard opening or changing height re-reveals the field; a pan or a close does not.
    #[test]
    fn a_reveal_is_for_an_opening_or_a_height_change() {
        let open = |inset, top| Keyboard::Open { inset, top };
        assert!(needs_reveal(Keyboard::Closed, open(300, 0)));
        assert!(needs_reveal(open(300, 0), open(420, 0)));
        assert!(!needs_reveal(open(300, 0), open(300, 40)));
        assert!(!needs_reveal(open(300, 0), Keyboard::Closed));
        assert!(!needs_reveal(Keyboard::Closed, Keyboard::Closed));
    }

    /// Scroll room is exactly what the keyboard covers over a floor of the margin: no fixed extra.
    #[test]
    fn scroll_room_is_the_covered_height_over_a_floor_of_the_margin() {
        assert_eq!(slack_step(0.0, None, 4.0), SlackStep::Release);
        assert_eq!(slack_step(-5.0, None, 4.0), SlackStep::Release);
        // a scroller's own 4px padding is under the floor: 16 + 120
        assert_eq!(
            slack_step(120.0, None, 4.0),
            SlackStep::Add {
                base: 16.0,
                padding: 136.0
            }
        );
        // a roomier padding is kept: 24 + 120
        assert_eq!(
            slack_step(120.0, None, 24.0),
            SlackStep::Add {
                base: 24.0,
                padding: 144.0
            }
        );
    }

    /// A scroller that already holds room keeps its first base, so the room follows the keyboard without
    /// compounding the padding the room itself made.
    #[test]
    fn held_room_keeps_its_first_base() {
        let held = Slack {
            padding: "4px".into(),
            anchor: String::new(),
            base: 16.0,
        };
        assert_eq!(
            slack_step(200.0, Some(&held), 136.0),
            SlackStep::Add {
                base: 16.0,
                padding: 216.0
            }
        );
        assert_eq!(slack_step(0.0, Some(&held), 136.0), SlackStep::Release);
    }

    /// A field comfortably inside the band is left alone; one outside it is brought to the band's centre.
    #[test]
    fn a_field_is_centred_in_the_visible_band_unless_it_is_comfortably_inside() {
        // scroller 0..800 (taller than the band), visible band 0..508: band 16..492
        let scroller = (0.0, 800.0);
        let visible = (0.0, 508.0);
        assert_eq!(scroll_delta(scroller, (100.0, 140.0), visible), None);
        assert_eq!(scroll_delta(scroller, (16.0, 492.0), visible), None);
        // below the band: centre 620 against the band's centre 254
        assert_eq!(scroll_delta(scroller, (600.0, 640.0), visible), Some(366.0));
        // above it (the scroller is scrolled): negative
        assert_eq!(
            scroll_delta((-300.0, 500.0), (-200.0, -160.0), visible),
            Some(-430.0)
        );
        // partly under the keyboard
        assert!(scroll_delta(scroller, (480.0, 520.0), visible).is_some());
        // a panned viewport moves the band
        assert_eq!(scroll_delta(scroller, (300.0, 340.0), (200.0, 708.0)), None);
    }

    /// The band is also bounded by the scroller itself.
    #[test]
    fn the_band_is_inside_the_scroller_too() {
        // scroller 100..300, visible 0..500: band 116..284, centre 200
        assert_eq!(
            scroll_delta((100.0, 300.0), (290.0, 330.0), (0.0, 500.0)),
            Some(110.0)
        );
    }

    /// The text-field selector excludes what takes no text, as the keyboard does not serve it.
    #[test]
    fn the_text_field_selector_names_what_takes_text() {
        for excluded in [
            "checkbox", "radio", "button", "submit", "reset", "range", "file", "color", "image",
            "hidden",
        ] {
            assert!(
                TEXT_FIELD.contains(&format!(":not([type={excluded} i])")),
                "{excluded}"
            );
        }
        assert!(TEXT_FIELD.starts_with("textarea, input:not("));
        assert!(TEXT_FIELD.contains("contenteditable"));
    }

    /// The contract a stylesheet reads: the names are the public API of this module.
    #[test]
    fn the_contract_names_are_the_documented_ones() {
        assert_eq!(Slot::KeyboardOpen.wire(), "data-dx-keyboard");
        assert_eq!(KEYBOARD_RESIZE_THRESHOLD_PX, 60);
        assert_eq!(KEYBOARD_VISIBILITY_MARGIN_PX, 16);
    }

    /// A message from the page is the four viewport numbers and the event; one that lacks a number (no visual
    /// viewport) is not a viewport.
    #[test]
    fn a_fact_is_a_viewport_when_it_has_all_four_numbers() {
        let fact: Fact = serde_json::from_value(serde_json::json!({
            "type": "resize", "viewport.innerHeight": 844, "viewport.height": 508,
            "viewport.offsetTop": 0, "viewport.scale": 1,
        }))
        .expect("fact");
        assert_eq!(
            fact.viewport(),
            Some(Viewport {
                inner_height: 844.0,
                height: 508.0,
                offset_top: 0.0,
                scale: 1.0
            })
        );
        let partial: Fact = serde_json::from_value(serde_json::json!({
            "type": "resize", "viewport.innerHeight": 844, "viewport.height": null,
            "viewport.offsetTop": null, "viewport.scale": null,
        }))
        .expect("fact");
        assert_eq!(partial.viewport(), None);
    }
}
