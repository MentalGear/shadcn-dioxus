//! `use_motion` / `use_motion_active_for` / `use_document_visible`: is anybody able to see this
//! element's motion right now?
//!
//! A looping animation or a self-driven timer costs main-thread work whether or not anyone is
//! looking at it. A page full of live demos would spend hundreds of animation frames per scroll on
//! spinners and skeletons in cards that are not on screen
//! (backlog row 174, the scroll profile: the infinite animations alone were 15% of
//! the scroll busy time). Pausing them one by one at each call site is the same
//! mistake as the uncleared `setInterval` (`interval.rs`): the next animation added forgets to.
//! Here "is this visible" is answered once, in one place, and components subscribe to the answer.
//!
//! Two signals share the registry (and one observer per kind).
//!
//! * **Continuous loops and timers** -- [`use_motion`], [`use_motion_when`],
//!   [`use_motion_active_for`]: spinner, skeleton, progress, avatar, shimmer hosts, carousel
//!   autoplay, the home-page tickers. These have a pre-roll margin so they are already moving
//!   when they scroll in.
//! * **One-shot entrances** -- [`use_entered_view_when`]: chart load animations. NO pre-roll: an
//!   entrance that starts before the element is on screen is an entrance nobody saw.
//! * **Timers that carry meaning** -- [`use_document_visible`]: toast auto-dismiss.
//!
//! A loop's motion is **active** unless any of these holds:
//!
//! * the element is outside the viewport. ONE `IntersectionObserver` is shared by the whole page
//!   (every subscribed element is `observe`d on it), with a `rootMargin` of [`ROOT_MARGIN`], so the
//!   motion is already running a moment before it scrolls into view (no visible restart) and stops
//!   once it is clearly off-screen above, below or to the side. This is the primary signal.
//! * the document is hidden (`visibilitychange`: another tab, a minimized window);
//! * the element sits inside a `content-visibility: auto` subtree the browser is skipping
//!   (`contentvisibilityautostatechange`, captured once at the document). A skipped subtree has
//!   no box, so the observer already reports it as outside the viewport; the event is the
//!   explicit second signal and also covers a subtree the observer's margin would consider close.
//!
//! The reduced-motion preference is deliberately NOT part of it: it is a different question
//! ("should this move at all") that the stylesheet answers with `@media (prefers-reduced-motion)`.
//!
//! # Pausing a CSS animation
//!
//! ```rust,ignore
//! let motion = use_motion();
//! rsx! { span { ..merge_attributes(vec![base, motion.attributes()]) } }
//! ```
//!
//! `motion.attributes()` carries a lookup key and, while inactive, `data-dx-motion="paused"`.
//! ONE rule in `dx-components-theme.css` turns that into
//! `animation-play-state: paused` for the element and everything under it, so the component
//! never mentions an animation by name and a future animation is covered without being listed.
//! `paused` (not `animation: none`) keeps the phase: the spinner resumes where it stopped, at no
//! cost while stopped. An element that animates ONLY sometimes (an indeterminate bar, an avatar
//! that is still loading) uses [`use_motion_when`], which subscribes nothing while it is false.
//!
//! # Stopping a timer
//!
//! ```rust,ignore
//! let motion = use_motion();
//! use_interval_while(motion.active(), Duration::from_secs(1), move || seconds += 1);
//! ```
//!
//! [`crate::interval::use_interval_while`] drops its task while the signal is false and starts a
//! fresh countdown when it turns true, so a hidden component makes no timer wakeups at all.
//! [`use_motion_active_for`] does the same for a component that does not own its root element
//! (the carousel's autoplay observes the carousel's scroller by id).
//!
//! # Timers that carry meaning
//!
//! A timer the user is waiting on (a toast's auto-dismiss) must NOT stop because its element
//! scrolled out of view, only because nobody can read the page at all: use
//! [`use_document_visible`], which follows the tab and ignores position.
//!
//! # Cost and lifetime
//!
//! Each subscription owns one bridge `IntersectionObserver` ([`crate::bridge`]); its removal is
//! derived from the owning scope. The page-wide facts are shared: the first mounted subscriber
//! holds the two document listeners (`visibilitychange` and `contentvisibilityautostatechange`),
//! and when it unmounts the next one takes them over, so the listener inventory returns to its
//! baseline. The decisions (`decide`) are pure Rust over the facts the page reports. Before the
//! observer reports (and on the server, and when the element cannot be found) a motion is
//! active, which is also what the server renders, so there is no hydration mismatch.

use dioxus::prelude::*;

use crate::bridge::{self, EventKind, Intersection, ListenOptions, ObserveOptions, Prop, Target};
use crate::dioxus_attributes::attributes;

/// The observer's `rootMargin`: how far outside the viewport a motion keeps running.
///
/// 200 px is a quarter of a laptop viewport. It is large enough that a spinner or shimmer is
/// already moving when its card enters (a wheel tick is 100 px, a trackpad flick covers it in
/// a frame or two, and an animation that resumes one frame late is invisible because `paused`
/// keeps the phase), and small enough that only the handful of elements within one screen of the
/// viewport animate, instead of the whole page. A bigger margin buys nothing visible and keeps
/// off-screen motion alive; a smaller one lets the resume land inside the viewport on a fast
/// scroll.
pub const ROOT_MARGIN: &str = "200px";

/// How much of an element must be on screen before an entrance ([`use_entered_view_when`])
/// counts as seen: 40% of its own area, or 40% of the viewport's height for an element taller
/// than the viewport (a ratio alone would never be reached by a tall card). Low enough that a
/// chart whose top third has scrolled in starts animating while the user is still looking at
/// it, high enough that a card brushing the viewport edge while flinging past does not spend
/// its one entrance unseen. No pre-roll margin: an entrance that starts before it is on screen
/// is simply missed.
pub const ENTER_RATIO: f64 = 0.4;

/// The activity decisions: pure functions over the typed facts the page reports (tab visibility,
/// the observer's answer, whether a `content-visibility: auto` subtree skips the element), the same
/// on every renderer (row 451).
mod decide {
    use super::ENTER_RATIO;

    /// The `IntersectionObserver` thresholds of an entrance: every tenth, so the observer reports
    /// each step of the ratio and the height rule below is checked as the element scrolls in.
    pub(super) const ENTER_THRESHOLDS: [f64; 11] =
        [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];

    /// What an `IntersectionObserver` entry reports, as far as the decisions read it.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub(super) struct Sighting {
        /// The entry intersects the root (with its margin).
        pub(super) is_intersecting: bool,
        /// `intersectionRatio`.
        pub(super) ratio: f64,
        /// `intersectionRect.height`.
        pub(super) intersection_height: f64,
        /// The root's height (`rootBounds.height`, the window's inner height without a root).
        pub(super) root_height: f64,
    }

    /// Whether a loop's element counts as on screen: any intersection with the margin-grown root.
    pub(super) fn loop_seen(sighting: Sighting) -> bool {
        sighting.is_intersecting
    }

    /// Whether an entrance's element counts as seen: [`ENTER_RATIO`] of its own area, or of the
    /// root's height when the element is taller than the root.
    pub(super) fn enter_seen(sighting: Sighting) -> bool {
        sighting.is_intersecting
            && (sighting.ratio >= ENTER_RATIO
                || sighting.intersection_height >= ENTER_RATIO * sighting.root_height)
    }

    /// The motion of an element loop: `None` until the observer has answered while the tab is
    /// visible (nothing is reported yet), `Some(false)` in a hidden tab, in a skipped
    /// `content-visibility` subtree or off screen.
    pub(super) fn loop_active(tab_hidden: bool, seen: Option<bool>, skipped: bool) -> Option<bool> {
        if tab_hidden {
            return Some(false);
        }
        seen.map(|seen| seen && !skipped)
    }

    /// The motion of a document-level subscription: the tab alone.
    pub(super) fn document_active(tab_hidden: bool) -> bool {
        !tab_hidden
    }

    /// Whether an entrance fires now: the element is seen, the tab is visible and no skipped
    /// subtree holds it. The owner latches the first `true`.
    pub(super) fn entered(tab_hidden: bool, seen: bool, skipped: bool) -> bool {
        !tab_hidden && seen && !skipped
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn sighting(is_intersecting: bool, ratio: f64, height: f64, root: f64) -> Sighting {
            Sighting {
                is_intersecting,
                ratio,
                intersection_height: height,
                root_height: root,
            }
        }

        #[test]
        fn a_loop_is_unknown_until_the_observer_answers() {
            assert_eq!(loop_active(false, None, false), None);
            assert_eq!(loop_active(false, None, true), None);
        }

        #[test]
        fn a_hidden_tab_is_an_answer_without_the_observer() {
            assert_eq!(loop_active(true, None, false), Some(false));
            assert_eq!(loop_active(true, Some(true), false), Some(false));
        }

        #[test]
        fn a_loop_runs_only_when_seen_and_not_skipped() {
            assert_eq!(loop_active(false, Some(true), false), Some(true));
            assert_eq!(loop_active(false, Some(false), false), Some(false));
            assert_eq!(loop_active(false, Some(true), true), Some(false));
        }

        #[test]
        fn the_document_subscription_follows_the_tab_alone() {
            assert!(document_active(false));
            assert!(!document_active(true));
        }

        #[test]
        fn a_loop_counts_any_intersection_with_the_margin_as_seen() {
            assert!(loop_seen(sighting(true, 0.0, 0.0, 800.0)));
            assert!(!loop_seen(sighting(false, 0.0, 0.0, 800.0)));
        }

        #[test]
        fn an_entrance_needs_forty_percent_of_the_element() {
            assert!(enter_seen(sighting(true, 0.4, 10.0, 800.0)));
            assert!(!enter_seen(sighting(true, 0.39, 10.0, 800.0)));
        }

        #[test]
        fn a_tall_element_enters_at_forty_percent_of_the_root_height() {
            assert!(enter_seen(sighting(true, 0.1, 320.0, 800.0)));
            assert!(!enter_seen(sighting(true, 0.1, 319.0, 800.0)));
        }

        #[test]
        fn a_non_intersecting_entry_is_never_seen() {
            assert!(!enter_seen(sighting(false, 1.0, 900.0, 800.0)));
        }

        #[test]
        fn an_entrance_waits_for_a_visible_tab_and_an_unskipped_element() {
            assert!(entered(false, true, false));
            assert!(!entered(true, true, false));
            assert!(!entered(false, true, true));
            assert!(!entered(false, false, false));
        }

        #[test]
        fn the_thresholds_step_by_a_tenth_up_to_one() {
            assert_eq!(ENTER_THRESHOLDS.len(), 11);
            assert_eq!(ENTER_THRESHOLDS[0], 0.0);
            assert_eq!(ENTER_THRESHOLDS[10], 1.0);
            assert!(ENTER_THRESHOLDS
                .windows(2)
                .all(|w| (w[1] - w[0] - 0.1).abs() < 1e-9));
        }
    }
}

/// Which signals decide `active`.
#[derive(Clone, Copy, PartialEq)]
enum Mode {
    /// Viewport + `content-visibility` + tab.
    Element,
    /// The tab only.
    Document,
    /// One-shot: true once, when the element is first seen past a threshold.
    Enter,
}

/// Escapes `id` for use inside `[id="..."]`.
fn id_selector(id: &str) -> String {
    format!("[id=\"{}\"]", id.replace('\\', "\\\\").replace('"', "\\\""))
}

/// The page-wide facts every subscriber shares, kept by ONE subscriber at a time (the leader, the
/// first of [`LISTENERS`]): whether the tab is hidden, and a counter that moves whenever a
/// `content-visibility: auto` subtree starts or stops being skipped. One pair of document listeners
/// serves the whole page, so the cost does not grow with the number of subscribers.
static TAB_HIDDEN: GlobalSignal<bool> = Global::new(|| false);
/// Whether the tab's visibility has been read at least once.
static TAB_KNOWN: GlobalSignal<bool> = Global::new(|| false);
/// Moves on every `contentvisibilityautostatechange`.
static CV_EPOCH: GlobalSignal<u64> = Global::new(|| 0);
/// The mounted subscribers, in mount order; the first one holds the document listeners.
static LISTENERS: GlobalSignal<Vec<usize>> = Global::new(Vec::new);

fn next_subscriber() -> usize {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

/// Reads the tab's visibility into [`TAB_HIDDEN`].
async fn read_tab_visibility() {
    let Ok(props) = bridge::get(Target::Document, [Prop::VisibilityState]).await else {
        return;
    };
    let Some(state) = props.string(&Prop::VisibilityState) else {
        return;
    };
    let hidden = state == "hidden";
    if *TAB_HIDDEN.peek() != hidden {
        *TAB_HIDDEN.write() = hidden;
    }
    if !*TAB_KNOWN.peek() {
        *TAB_KNOWN.write() = true;
    }
}

/// Whether a `content-visibility: auto` subtree skips the element `selector` names; `None` when the
/// page cannot answer (no `checkVisibility`, no element, no page).
async fn read_skipped(selector: String) -> Option<bool> {
    let props = bridge::get(Target::selector(selector), [Prop::CheckVisibility])
        .await
        .ok()?;
    props.bool(&Prop::CheckVisibility).map(|visible| !visible)
}

/// Subscribes the calling component to the activity of the element `selector()` names (`None`:
/// nothing to observe yet, the motion stays active). Re-subscribes when a signal `selector`
/// reads changes.
fn use_activity(
    selector: impl FnMut() -> Option<String> + 'static,
    mode: Mode,
    mut active: Signal<bool>,
    mut known: Signal<bool>,
) {
    let selector = use_memo(selector);
    let mut seen: Signal<Option<bool>> = use_signal(|| None);
    let mut skipped = use_signal(|| false);

    // The document facts: one subscriber holds the two listeners for the page.
    let id = use_hook(next_subscriber);
    use_effect(move || LISTENERS.write().push(id));
    use_drop(move || LISTENERS.write().retain(|other| *other != id));
    let leader = use_memo(move || LISTENERS.read().first() == Some(&id));
    bridge::use_event_listener(
        move || leader().then_some(Target::Document),
        EventKind::Visibilitychange,
        ListenOptions::default(),
        &[],
        move |_: serde_json::Value| {
            spawn(read_tab_visibility());
        },
    );
    bridge::use_event_listener(
        move || leader().then_some(Target::Document),
        EventKind::Contentvisibilityautostatechange,
        ListenOptions::CAPTURE,
        &[],
        move |_: serde_json::Value| *CV_EPOCH.write() += 1,
    );
    use_effect(move || {
        if leader() {
            spawn(read_tab_visibility());
        }
    });

    // A new element starts unseen.
    use_effect(move || {
        selector();
        seen.set(None);
        skipped.set(false);
    });

    // The element's own visibility, from an observer of its own.
    let options = match mode {
        Mode::Enter => ObserveOptions {
            threshold: decide::ENTER_THRESHOLDS.to_vec(),
            ..ObserveOptions::default()
        },
        _ => ObserveOptions {
            root_margin: Some(ROOT_MARGIN.into()),
            ..ObserveOptions::default()
        },
    };
    bridge::use_intersection_observer(
        move || {
            if mode == Mode::Document {
                None
            } else {
                selector()
            }
            .map(Target::selector)
        },
        options,
        move |entry: Intersection| {
            let sighting = decide::Sighting {
                is_intersecting: entry.is_intersecting,
                ratio: entry.ratio,
                intersection_height: entry.intersection_height,
                root_height: entry.root_height,
            };
            let now = if mode == Mode::Enter {
                decide::enter_seen(sighting)
            } else {
                decide::loop_seen(sighting)
            };
            if *seen.peek() != Some(now) {
                seen.set(Some(now));
            }
            // Only a seen element's skipped state matters; read it as the element comes into view.
            if now {
                if let Some(selector) = selector.peek().clone() {
                    spawn(async move {
                        if let Some(now) = read_skipped(selector).await {
                            if *skipped.peek() != now {
                                skipped.set(now);
                            }
                        }
                    });
                }
            }
        },
    );

    // A skipped-subtree change re-reads the elements that are in view.
    use_effect(move || {
        CV_EPOCH();
        if *seen.peek() == Some(true) {
            if let Some(selector) = selector.peek().clone() {
                spawn(async move {
                    if let Some(now) = read_skipped(selector).await {
                        if *skipped.peek() != now {
                            skipped.set(now);
                        }
                    }
                });
            }
        }
    });

    // The decision.
    use_effect(move || {
        let hidden = TAB_HIDDEN();
        let tab_known = TAB_KNOWN();
        let seen = seen();
        let skipped = skipped();
        if selector().is_none() {
            // An entrance keeps its answer (the owner latches it); the others fall back to the
            // optimistic default.
            if mode != Mode::Enter {
                if !*active.peek() {
                    active.set(true);
                }
                if *known.peek() {
                    known.set(false);
                }
            }
            return;
        }
        let now = match mode {
            Mode::Document => tab_known.then(|| decide::document_active(hidden)),
            Mode::Element => decide::loop_active(hidden, seen, skipped),
            Mode::Enter => decide::entered(hidden, seen == Some(true), skipped).then_some(true),
        };
        let Some(now) = now else { return };
        if *active.peek() != now {
            active.set(now);
        }
        if !*known.peek() {
            known.set(true);
        }
    });
}

/// The handle [`use_motion`] returns.
#[derive(Clone, Copy)]
pub struct Motion {
    key: Signal<String>,
    active: Signal<bool>,
    known: Signal<bool>,
    enabled: Signal<bool>,
}

impl Motion {
    /// Whether the element's motion should be running. Read it in a task, an effect or a
    /// `use_interval_while`; it is a signal, so reading it in render subscribes the component.
    pub fn active(&self) -> ReadSignal<bool> {
        self.active.into()
    }

    /// Whether the observer has answered at least once (until then [`active`](Self::active) is
    /// the optimistic default `true`). A one-shot "play when first seen" effect must wait for
    /// this, or it fires on the default.
    pub fn known(&self) -> ReadSignal<bool> {
        self.known.into()
    }

    /// The attributes to spread on the element that carries the motion: the lookup key, and
    /// `data-dx-motion="paused"` while the motion is inactive (the theme stylesheet pauses every
    /// CSS animation on and under it). Reads `active`, so the component re-renders when it flips.
    pub fn attributes(&self) -> Vec<Attribute> {
        let paused = !(self.active)();
        let key = self.key.cloned();
        attributes!(span {
            "data-dx-motion-key": key,
            "data-dx-motion": paused.then_some("paused"),
        })
    }

    /// Whether this handle is currently observing anything (`false` while [`use_motion_when`]'s
    /// condition is false).
    pub fn enabled(&self) -> bool {
        (self.enabled)()
    }
}

/// Tracks whether the element the caller spreads [`Motion::attributes`] onto is visible. See the
/// [module docs](self).
pub fn use_motion() -> Motion {
    use_motion_when(|| true)
}

/// Like [`use_motion`], but only observes while `enabled()` is true (an indeterminate progress
/// bar, an avatar that is still loading): while it is false no script runs and the motion is
/// reported active. `enabled` runs in an effect, so signals it reads re-subscribe.
pub fn use_motion_when(enabled: impl FnMut() -> bool + 'static) -> Motion {
    let (key, enabled_signal) = use_enabled_key(enabled);
    let active = use_signal(|| true);
    let known = use_signal(|| false);
    use_activity(
        move || key_selector(key, enabled_signal),
        Mode::Element,
        active,
        known,
    );
    Motion {
        key,
        active,
        known,
        enabled: enabled_signal,
    }
}

fn key_selector(key: Signal<String>, enabled: Signal<bool>) -> Option<String> {
    enabled().then(|| format!("[data-dx-motion-key=\"{}\"]", key.read().replace('"', "")))
}

fn use_enabled_key(mut enabled: impl FnMut() -> bool + 'static) -> (Signal<String>, Signal<bool>) {
    let key = crate::use_unique_id();
    let mut enabled_signal = use_signal(&mut enabled);
    use_effect(move || {
        let now = enabled();
        if *enabled_signal.peek() != now {
            enabled_signal.set(now);
        }
    });
    (key, enabled_signal)
}

/// The handle [`use_entered_view_when`] returns.
#[derive(Clone, Copy)]
pub struct EnterView {
    key: Signal<String>,
    entered: Signal<bool>,
}

impl EnterView {
    /// `false` until the element has been seen, then `true` (and it stays `true`: the owner
    /// reads it once to start its entrance and never needs to un-play it).
    pub fn entered(&self) -> ReadSignal<bool> {
        self.entered.into()
    }

    /// The lookup key to spread on the element. No `data-dx-motion` here: an entrance is not
    /// a loop and is never paused.
    pub fn attributes(&self) -> Vec<Attribute> {
        let key = self.key.cloned();
        attributes!(span { "data-dx-motion-key": key })
    }
}

/// One-shot entrances: `entered()` turns `true` once, when the element is actually on screen
/// (at least [`ENTER_RATIO`] of it, no pre-roll margin) and the tab is visible, and never
/// while the tab is hidden or the element is inside a skipped `content-visibility` subtree. An
/// element already in view at mount enters as soon as the browser has laid it out. After it
/// fires the shared observer lets go of the element. Charts play their load animation on it.
///
/// `enabled` false (no animation wanted) subscribes to nothing; turning it false after the
/// entrance has fired (the usual "latch it" pattern) tears the subscription down and keeps
/// the answer.
pub fn use_entered_view_when(enabled: impl FnMut() -> bool + 'static) -> EnterView {
    let (key, enabled_signal) = use_enabled_key(enabled);
    let entered = use_signal(|| false);
    let known = use_signal(|| false);
    use_activity(
        move || key_selector(key, enabled_signal),
        Mode::Enter,
        entered,
        known,
    );
    EnterView { key, entered }
}

/// Like [`use_motion`], for an element the component does not render itself: observes the
/// element whose DOM `id` is `element_id` (re-subscribes when it changes; an empty id observes
/// nothing). Used by the carousel's autoplay, which lives beside the scroller it should watch.
pub fn use_motion_active_for(element_id: impl Into<ReadSignal<String>>) -> ReadSignal<bool> {
    let element_id: ReadSignal<String> = element_id.into();
    let active = use_signal(|| true);
    let known = use_signal(|| false);
    use_activity(
        move || {
            let id = element_id.cloned();
            (!id.is_empty()).then(|| id_selector(&id))
        },
        Mode::Element,
        active,
        known,
    );
    active.into()
}

/// Whether the tab is visible. Ignores where the element is on the page: for a timer the user is
/// waiting on (a toast's auto-dismiss), which must keep counting while scrolled away but not while
/// nobody can read the page.
pub fn use_document_visible() -> ReadSignal<bool> {
    let active = use_signal(|| true);
    let known = use_signal(|| false);
    use_activity(|| Some(String::new()), Mode::Document, active, known);
    active.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_id_selector_escapes_quotes_and_backslashes() {
        assert_eq!(id_selector("a\"b\\c"), "[id=\"a\\\"b\\\\c\"]");
    }
}
