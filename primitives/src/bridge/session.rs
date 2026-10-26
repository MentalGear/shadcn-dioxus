//! Subscriptions: listeners, observers, media queries, sampling loops and named behaviours whose
//! lifetime is a scope. One [`Session`] is one long-lived eval running the page's `__dxBridge.serve`;
//! opening it yields a [`Subscription`] whose `Drop` makes the page undo everything the session
//! registered, in reverse order. The hooks tie that drop to the calling component.
//!
//! A session streams ONE message type to Rust: the fields its commands name. Open one session per
//! message type (the hooks do).
//!
//! **Frame-rate events go through here.** Scroll, resize, pointermove, wheel and touchmove are consumed
//! with [`use_event_listener`], [`use_resize_observer`], [`use_intersection_observer`] or
//! [`use_sample`], never with a vdom handler: on desktop every vdom event is a blocking synchronous
//! request, so a frame-rate stream queues behind itself. The page coalesces a [`hot`](EventKind::hot)
//! event to one message per animation frame, and a [`use_sample`] field flushes per frame or per event
//! task ([`Flush`]).

use dioxus::document::Eval;
use dioxus::prelude::*;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use super::events::{EventField, EventKind};
use super::transport;
use super::{Behaviour, Epoch, Target};
use crate::js_listener::{receive, TEARDOWN};

/// The options of a streaming listener. A streaming listener can never cancel an event, so it is
/// always passive and there is no `passive` option (a decision made inside the event is a
/// [`Behaviour`], which declares its own passivity).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct ListenOptions {
    /// Listen in the capture phase.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub capture: bool,
    /// Remove the listener after it fires once.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub once: bool,
}

impl ListenOptions {
    /// The capture phase.
    pub const CAPTURE: ListenOptions = ListenOptions {
        capture: true,
        once: false,
    };
}

/// Which observer.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ObserverKind {
    /// `ResizeObserver`: messages are [`Size`].
    Resize,
    /// `IntersectionObserver`: messages are [`Intersection`].
    Intersection,
    /// `MutationObserver`: messages are [`Mutation`].
    Mutation,
}

/// Which box a `ResizeObserver` reports.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum ResizeBox {
    /// The content box.
    #[serde(rename = "content-box")]
    ContentBox,
    /// The border box.
    #[serde(rename = "border-box")]
    BorderBox,
    /// The device-pixel content box.
    #[serde(rename = "device-pixel-content-box")]
    DevicePixelContentBox,
}

/// Observer options; a field applies only to the kind it belongs to.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObserveOptions {
    /// Resize: which box to observe.
    #[serde(rename = "box", skip_serializing_if = "Option::is_none")]
    pub box_model: Option<ResizeBox>,
    /// Intersection: the root element (the viewport when absent).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root: Option<Target>,
    /// Intersection: `rootMargin`, e.g. `"0px 0px 120px 0px"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_margin: Option<String>,
    /// Intersection: `threshold`.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub threshold: Vec<f64>,
    /// Mutation: observe attribute changes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<bool>,
    /// Mutation: only these attributes.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attribute_filter: Vec<String>,
    /// Mutation: observe child additions and removals.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub child_list: Option<bool>,
    /// Mutation: observe the whole subtree.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtree: Option<bool>,
    /// Mutation: observe text changes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub character_data: Option<bool>,
}

/// When a batched sample field is delivered.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Flush {
    /// With the frame's one message (the entries of the whole frame).
    Frame,
    /// In a message of its own at the end of the event's task, carrying only task-flushed batched
    /// fields: a passive stream that must not wait for the frame (a wheel gesture's phase).
    Task,
}

/// One field of a [`Command::Sample`].
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(untagged)]
pub enum SampleField {
    /// Coalesced: the property `get` reads (a [`Prop`](super::Prop) wire name), its value at the end of
    /// the frame, sent only when it changed.
    Read(String),
    /// Batched: every `event` the target gets, each as `{ timeStamp, ...pick }`, in order, as the array
    /// `name` of the message ([`Flush`] says which one; an empty array when a frame saw none). A
    /// task-flushed message carries only the task-flushed arrays, so the receiving struct gives the
    /// frame's fields `#[serde(default)]`.
    Batched {
        /// The key of the array in the message.
        name: String,
        /// Always `"batched"`.
        mode: &'static str,
        /// The event to collect.
        event: EventKind,
        /// Event fields to keep, besides `timeStamp`.
        pick: Vec<EventField>,
        /// Listen in the capture phase.
        capture: bool,
        /// When the entries are delivered.
        flush: Flush,
    },
}

impl SampleField {
    /// A coalesced read of `prop`.
    pub fn read(prop: &super::Prop) -> Self {
        SampleField::Read(prop.wire())
    }

    /// A batched event stream, flushed with the frame.
    pub fn batched(
        name: impl Into<String>,
        event: EventKind,
        pick: impl IntoIterator<Item = EventField>,
    ) -> Self {
        SampleField::Batched {
            name: name.into(),
            mode: "batched",
            event,
            pick: pick.into_iter().collect(),
            capture: false,
            flush: Flush::Frame,
        }
    }

    /// The same field, flushed in the event's task instead of at the frame.
    #[must_use]
    pub fn flushed_per_task(mut self) -> Self {
        if let SampleField::Batched { flush, .. } = &mut self {
            *flush = Flush::Task;
        }
        self
    }
}

/// How long a sampling loop runs.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Until {
    /// Until the subscription ends.
    Forever,
    /// For this many frames.
    Frames {
        /// The number of frames.
        n: u32,
    },
    /// For this many milliseconds.
    Ms {
        /// The duration.
        ms: f64,
    },
    /// While the element has the attribute (and, with `value`, while it equals it).
    Attr {
        /// The attribute.
        name: String,
        /// Its required value.
        #[serde(skip_serializing_if = "Option::is_none")]
        value: Option<String>,
    },
}

/// A command a session sends to the page's `serve`.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "cmd", rename_all = "kebab-case")]
pub enum Command {
    /// `listen(target, event, options, fields)`: streams the named event fields (a value that is not a
    /// string, number or bool is null). Always passive; a [`hot`](EventKind::hot) event arrives at most
    /// once per animation frame (the latest event's fields).
    Listen {
        /// What to listen on.
        target: Target,
        /// The event.
        event: EventKind,
        /// The listener's options.
        options: ListenOptions,
        /// The event fields to send; the message has one key per field (its wire name).
        fields: Vec<EventField>,
        /// Also send once at registration, as if an event with `type: "initial"` and the target as
        /// `target` and `currentTarget` fired (`"target.innerWidth"` of a window reads its size now).
        #[serde(skip_serializing_if = "std::ops::Not::not")]
        initial: bool,
    },
    /// `observe(target, kind, options)`.
    Observe {
        /// The element.
        target: Target,
        /// Which observer.
        kind: ObserverKind,
        /// Its options.
        options: ObserveOptions,
    },
    /// `matchMedia(query)`: sends [`MediaChange`] now and on every change.
    Media {
        /// The media query.
        query: String,
    },
    /// `sample(target, fields, until)`.
    Sample {
        /// The element.
        target: Target,
        /// The fields.
        fields: Vec<SampleField>,
        /// How long to sample.
        until: Until,
        /// Send a frame only when a coalesced field changed or a batched one saw an event.
        #[serde(rename = "onlyChanged")]
        only_changed: bool,
    },
    /// Installs a named [`Behaviour`] on `target` for the life of the session (see [`Session::behaviour`]).
    Behaviour {
        /// The host element, window or document.
        target: Target,
        /// The behaviour and its parameters.
        behaviour: Behaviour,
        /// What the behaviour declares (target kind, event, phase, passivity, lease); derived from
        /// `behaviour`, sent so the page can check it against the resolved target.
        declared: super::behaviours::Declared,
        /// Install generation: older than one the target saw is dropped, equal renews.
        epoch: Epoch,
    },
}

/// A `matchMedia` answer.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
pub struct MediaChange {
    /// Whether the query matches.
    pub matches: bool,
}

/// A `ResizeObserver` entry.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Size {
    /// Content-box width.
    pub width: f64,
    /// Content-box height.
    pub height: f64,
    /// Border-box width.
    pub border_width: f64,
    /// Border-box height.
    pub border_height: f64,
}

/// An `IntersectionObserver` entry.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Intersection {
    /// Whether the target intersects the root.
    pub is_intersecting: bool,
    /// The intersection ratio.
    pub ratio: f64,
    /// The target's bounding box: top.
    pub top: f64,
    /// The target's bounding box: left.
    pub left: f64,
    /// The target's bounding box: width.
    pub width: f64,
    /// The target's bounding box: height.
    pub height: f64,
}

/// A summary of one `MutationObserver` callback.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct Mutation {
    /// How many records the callback got.
    pub count: u32,
    /// The attributes that changed.
    pub attributes: Vec<String>,
    /// Nodes added.
    pub added: u32,
    /// Nodes removed.
    pub removed: u32,
}

/// One sampled frame: the fields, and whether this is the loop's last message.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Sampled<T> {
    /// The sampled fields.
    #[serde(flatten)]
    pub fields: T,
    /// True on the message that ends the loop.
    #[serde(rename = "$done", default)]
    pub done: bool,
}

/// A set of commands to run in one session.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Session {
    commands: Vec<Command>,
}

impl Session {
    /// An empty session.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a command.
    #[must_use]
    pub fn command(mut self, command: Command) -> Self {
        self.commands.push(command);
        self
    }

    /// Streams `fields` of `event` on `target`.
    #[must_use]
    pub fn listen(
        self,
        target: impl Into<Target>,
        event: EventKind,
        options: ListenOptions,
        fields: &[EventField],
    ) -> Self {
        self.command(Command::Listen {
            target: target.into(),
            event,
            options,
            fields: fields.to_vec(),
            initial: false,
        })
    }

    /// Installs the named `behaviour` on `target` for the life of the session, under `epoch`.
    #[must_use]
    pub fn behaviour(self, target: impl Into<Target>, behaviour: Behaviour, epoch: Epoch) -> Self {
        let declared = behaviour.declared();
        self.command(Command::Behaviour {
            target: target.into(),
            behaviour,
            declared,
            epoch,
        })
    }

    /// The commands, in order.
    pub fn commands(&self) -> &[Command] {
        &self.commands
    }

    /// Runs the session; `handler` gets every message it streams, in a task owned by the calling
    /// component's scope. Drop the returned [`Subscription`] to end it.
    pub fn open<T: DeserializeOwned + 'static>(
        self,
        handler: impl FnMut(T) + 'static,
    ) -> Subscription {
        // each behaviour's module is evaluated, once per webview, before the session that installs it
        for command in &self.commands {
            if let Command::Behaviour { behaviour, .. } = command {
                transport::ensure_module(behaviour.kind(), behaviour.module());
            }
        }
        let eval = transport::open(transport::Program::Serve);
        transport::send(&eval, &self.commands);
        // a lease is the page's fail-open: the session renews each behaviour that carries one
        let renew: Vec<Command> = self
            .commands
            .iter()
            .filter(
                |c| matches!(c, Command::Behaviour { declared, .. } if declared.lease_ms.is_some()),
            )
            .cloned()
            .collect();
        let renewal = renew
            .iter()
            .filter_map(|c| match c {
                Command::Behaviour { declared, .. } => declared.lease_ms,
                _ => None,
            })
            .min()
            .map(|lease| {
                let renew_eval = eval;
                spawn(async move {
                    // motion-ok: a lease renewal is a heartbeat the page's fail-open depends on, not motion
                    loop {
                        crate::sleep::sleep(std::time::Duration::from_millis(
                            u64::from(lease / 2).max(1),
                        ))
                        .await;
                        transport::send(&renew_eval, &renew);
                    }
                })
            });
        let mut task_eval = eval;
        spawn(async move {
            receive(&mut task_eval, handler).await;
        });
        Subscription { eval, renewal }
    }
}

/// A running session. Dropping it removes every listener, observer, media query, sampling loop and
/// behaviour the session registered.
#[must_use = "dropping a `Subscription` ends it at once"]
pub struct Subscription {
    eval: Eval,
    /// The lease-renewal task of a session that installed a behaviour with a lease.
    renewal: Option<crate::dioxus_core::Task>,
}

impl Drop for Subscription {
    fn drop(&mut self) {
        if let Some(renewal) = self.renewal.take() {
            renewal.cancel();
        }
        // On the channel the session already has: a new eval inside a drop handler is unsafe (#4064).
        transport::send(&self.eval, TEARDOWN);
    }
}

fn use_subscription<T: DeserializeOwned + 'static>(
    mut build: impl FnMut() -> Option<Session> + 'static,
    handler: impl FnMut(T) + 'static,
) {
    let handler = use_callback(handler);
    crate::use_effect_with_cleanup(move || {
        let subscription =
            build().map(|session| session.open(move |message: T| handler.call(message)));
        move || drop(subscription)
    });
}

/// Streams `fields` of `event` on the target `target()` names (window, document or an element), from mount until unmount or until a
/// signal `target` read changes (the listener is then removed and added again). `target` returning
/// `None` listens to nothing for that state (an element that is not rendered yet).
///
/// `T` is the message: a struct with one field per entry of `fields`, named by its
/// [`wire`](EventField::wire) name (`#[serde(rename = "target.id")]` for a dotted one). Messages reach
/// `handler` asynchronously and the listener is passive: this hook cannot cancel an event or read the
/// DOM at the event's moment (that is a named [`Behaviour`]). A [`hot`](EventKind::hot) event arrives at
/// most once per animation frame.
pub fn use_event_listener<T: DeserializeOwned + 'static>(
    mut target: impl FnMut() -> Option<Target> + 'static,
    event: EventKind,
    options: ListenOptions,
    fields: &'static [EventField],
    handler: impl FnMut(T) + 'static,
) {
    use_subscription(
        move || target().map(|t| Session::new().listen(t, event, options, fields)),
        handler,
    );
}

/// [`use_event_listener`] on `window`.
pub fn use_window_listener<T: DeserializeOwned + 'static>(
    event: EventKind,
    options: ListenOptions,
    fields: &'static [EventField],
    handler: impl FnMut(T) + 'static,
) {
    use_event_listener(|| Some(Target::Window), event, options, fields, handler);
}

/// [`use_event_listener`] on `document`.
pub fn use_document_listener<T: DeserializeOwned + 'static>(
    event: EventKind,
    options: ListenOptions,
    fields: &'static [EventField],
    handler: impl FnMut(T) + 'static,
) {
    use_event_listener(|| Some(Target::Document), event, options, fields, handler);
}

/// [`use_event_listener`] on the element whose `id` `id()` returns.
pub fn use_element_listener<T: DeserializeOwned + 'static>(
    mut id: impl FnMut() -> Option<String> + 'static,
    event: EventKind,
    options: ListenOptions,
    fields: &'static [EventField],
    handler: impl FnMut(T) + 'static,
) {
    use_event_listener(
        move || id().map(Target::from),
        event,
        options,
        fields,
        handler,
    );
}

/// Whether `query` matches, kept current. `false` until the page's first answer (which arrives right
/// after mount) and on the server.
pub fn use_media_query(query: impl Into<String>) -> ReadSignal<bool> {
    let query = query.into();
    let mut matches = use_signal(|| false);
    use_subscription(
        move || {
            Some(Session::new().command(Command::Media {
                query: query.clone(),
            }))
        },
        move |change: MediaChange| matches.set(change.matches),
    );
    matches.into()
}

/// Observes the target `target()` names. `T` is [`Size`], [`Intersection`] or [`Mutation`] to match
/// `kind`. Re-subscribes when a signal `target` read changes.
pub fn use_observer<T: DeserializeOwned + 'static>(
    mut target: impl FnMut() -> Option<Target> + 'static,
    kind: ObserverKind,
    options: ObserveOptions,
    handler: impl FnMut(T) + 'static,
) {
    use_subscription(
        move || {
            target().map(|target| {
                Session::new().command(Command::Observe {
                    target,
                    kind: kind.clone(),
                    options: options.clone(),
                })
            })
        },
        handler,
    );
}

/// Samples the target `target()` names once per frame while `until` holds: coalesced fields
/// (the last value of the frame) and batched ones (every event of the frame with its timestamp), one
/// message per frame. `T` is the fields; the messages arrive as [`Sampled<T>`](Sampled).
pub fn use_sample<T: DeserializeOwned + 'static>(
    mut target: impl FnMut() -> Option<Target> + 'static,
    fields: Vec<SampleField>,
    until: Until,
    handler: impl FnMut(Sampled<T>) + 'static,
) {
    use_subscription(
        move || {
            target().map(|target| {
                Session::new().command(Command::Sample {
                    target,
                    fields: fields.clone(),
                    until: until.clone(),
                    only_changed: true,
                })
            })
        },
        handler,
    );
}

/// [`use_observer`] with a `ResizeObserver`: the element's [`Size`], from the page's first answer on.
/// For a plain vdom element prefer Dioxus's `onresize`.
pub fn use_resize_observer(
    target: impl FnMut() -> Option<Target> + 'static,
    options: ObserveOptions,
    handler: impl FnMut(Size) + 'static,
) {
    use_observer(target, ObserverKind::Resize, options, handler);
}

/// [`use_observer`] with an `IntersectionObserver` (`root_margin`, `threshold` in `options`): an
/// [`Intersection`] per callback. Where default options suffice, prefer Dioxus's `onvisible`.
pub fn use_intersection_observer(
    target: impl FnMut() -> Option<Target> + 'static,
    options: ObserveOptions,
    handler: impl FnMut(Intersection) + 'static,
) {
    use_observer(target, ObserverKind::Intersection, options, handler);
}

/// [`use_observer`] with a `MutationObserver`: a [`Mutation`] summary per callback.
pub fn use_mutation_observer(
    target: impl FnMut() -> Option<Target> + 'static,
    options: ObserveOptions,
    handler: impl FnMut(Mutation) + 'static,
) {
    use_observer(target, ObserverKind::Mutation, options, handler);
}

/// The window's inner size in CSS pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WindowSize {
    /// `window.innerWidth`.
    pub width: f64,
    /// `window.innerHeight`.
    pub height: f64,
}

/// The window's size, kept current. A zero [`WindowSize`] on the server and in the first client render
/// (so hydration matches); the real size arrives right after mount and on every `resize`.
pub fn use_window_size() -> ReadSignal<WindowSize> {
    #[derive(Deserialize)]
    struct Message {
        #[serde(rename = "target.innerWidth")]
        width: f64,
        #[serde(rename = "target.innerHeight")]
        height: f64,
    }
    let mut size = use_signal(WindowSize::default);
    use_subscription(
        || {
            Some(Session::new().command(Command::Listen {
                target: Target::Window,
                event: EventKind::Resize,
                options: ListenOptions::default(),
                fields: vec![EventField::TargetInnerWidth, EventField::TargetInnerHeight],
                initial: true,
            }))
        },
        move |m: Message| {
            size.set(WindowSize {
                width: m.width,
                height: m.height,
            })
        },
    );
    size.into()
}

/// Installs the named behaviour `behaviour()` returns on the target `target()` names, for the life of
/// the component. It is replaced when a signal either closure reads changes (a fresh [`Epoch`] makes
/// the page drop a late command of the replaced install) and removed when the component unmounts. `handler`
/// gets what the behaviour's `decide` sends (nothing, for a behaviour that only sets attributes and
/// cancels). Read [`behaviours`](super::behaviours) for what a behaviour is and when to use one instead
/// of an rsx handler.
pub fn use_behaviour<T: DeserializeOwned + 'static>(
    mut target: impl FnMut() -> Option<Target> + 'static,
    mut behaviour: impl FnMut() -> Behaviour + 'static,
    handler: impl FnMut(T) + 'static,
) {
    use_subscription(
        move || {
            target().map(|target| {
                let behaviour = behaviour();
                let epoch = Epoch::next(format!("behaviour:{}", behaviour.kind()));
                Session::new().behaviour(target, behaviour, epoch)
            })
        },
        handler,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fixture() -> Behaviour {
        Behaviour::Fixture {
            event: EventKind::Click,
            attr: "data-x".into(),
            value: "y".into(),
            capture: false,
            cancel: true,
            lease_ms: Some(400),
        }
    }

    #[test]
    fn commands_have_the_shapes_the_page_reads() {
        let session = Session::new()
            .listen(
                Target::Window,
                EventKind::Keydown,
                ListenOptions::CAPTURE,
                &[EventField::Key, EventField::TargetId],
            )
            .command(Command::Media {
                query: "(hover: hover)".into(),
            })
            .behaviour(
                "a",
                fixture(),
                Epoch {
                    key: "behaviour:fixture".into(),
                    n: 3,
                },
            );
        assert_eq!(
            serde_json::to_value(session.commands()).expect("serialises"),
            json!([
                {"cmd": "listen", "target": {"kind": "window"}, "event": "keydown",
                 "options": {"capture": true}, "fields": ["key", "target.id"]},
                {"cmd": "media", "query": "(hover: hover)"},
                {"cmd": "behaviour", "target": {"kind": "id", "id": "a"},
                 "behaviour": {"kind": "fixture", "event": "click", "attr": "data-x", "value": "y",
                               "capture": false, "cancel": true, "leaseMs": 400},
                 "declared": {"target": "element", "event": "click", "capture": false,
                              "passive": false, "leaseMs": 400},
                 "epoch": {"key": "behaviour:fixture", "n": 3}},
            ])
        );
    }

    #[test]
    fn listen_options_omit_their_defaults_and_have_no_passive() {
        assert_eq!(
            serde_json::to_value(ListenOptions::default()).expect("serialises"),
            json!({})
        );
        assert_eq!(
            serde_json::to_value(ListenOptions {
                capture: false,
                once: true
            })
            .expect("serialises"),
            json!({"once": true})
        );
    }

    #[test]
    fn observers_and_samples_serialise_for_the_page() {
        let observe = Command::Observe {
            target: Target::id("a"),
            kind: ObserverKind::Intersection,
            options: ObserveOptions {
                root_margin: Some("0px 0px 100px 0px".into()),
                threshold: vec![0.0, 1.0],
                ..ObserveOptions::default()
            },
        };
        assert_eq!(
            serde_json::to_value(&observe).expect("serialises"),
            json!({"cmd": "observe", "target": {"kind": "id", "id": "a"}, "kind": "intersection",
                   "options": {"rootMargin": "0px 0px 100px 0px", "threshold": [0.0, 1.0]}})
        );
        assert_eq!(
            serde_json::to_value(ObserveOptions {
                box_model: Some(ResizeBox::BorderBox),
                ..ObserveOptions::default()
            })
            .expect("serialises"),
            json!({"box": "border-box"})
        );
        let sample = Command::Sample {
            target: Target::id("a"),
            fields: vec![
                SampleField::read(&super::super::Prop::ScrollTop),
                SampleField::batched(
                    "wheel",
                    EventKind::Wheel,
                    [EventField::DeltaX, EventField::DeltaY],
                )
                .flushed_per_task(),
            ],
            until: Until::Attr {
                name: "data-moving".into(),
                value: None,
            },
            only_changed: true,
        };
        assert_eq!(
            serde_json::to_value(&sample).expect("serialises"),
            json!({"cmd": "sample", "target": {"kind": "id", "id": "a"},
                   "fields": ["scrollTop",
                              {"name": "wheel", "mode": "batched", "event": "wheel",
                               "pick": ["deltaX", "deltaY"], "capture": false, "flush": "task"}],
                   "until": {"kind": "attr", "name": "data-moving"}, "onlyChanged": true})
        );
        assert_eq!(
            serde_json::to_value(Until::Frames { n: 3 }).expect("serialises"),
            json!({"kind": "frames", "n": 3})
        );
    }

    #[test]
    fn messages_deserialize_into_typed_values() {
        let size: Size = serde_json::from_value(
            json!({"width": 1.0, "height": 2.0, "borderWidth": 3.0, "borderHeight": 4.0}),
        )
        .expect("size");
        assert_eq!(size.border_height, 4.0);
        let change: MediaChange = serde_json::from_value(json!({"matches": true})).expect("media");
        assert!(change.matches);

        #[derive(Deserialize, PartialEq, Debug)]
        struct Frame {
            #[serde(rename = "scrollTop")]
            scroll_top: f64,
            wheel: Vec<serde_json::Value>,
        }
        let frame: Sampled<Frame> = serde_json::from_value(
            json!({"scrollTop": 7.0, "wheel": [{"timeStamp": 1}], "$done": true}),
        )
        .expect("sampled");
        assert!(frame.done);
        assert_eq!(frame.fields.wheel.len(), 1);
        let frame: Sampled<Frame> =
            serde_json::from_value(json!({"scrollTop": 7.0, "wheel": []})).expect("sampled");
        assert!(!frame.done);
    }

    /// The commands the page's `run` switch handles are exactly the kebab-case names above.
    #[test]
    fn every_command_name_is_a_case_of_the_pages_run_switch() {
        let js = include_str!("../ts/bridge.ts");
        for name in ["listen", "observe", "media", "sample", "behaviour"] {
            assert!(js.contains(&format!("case \"{name}\":")), "{name}");
        }
        assert!(!js.contains("case \"rule\":"), "the rule layer is gone");
    }
}
