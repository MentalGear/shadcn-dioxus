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

use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap};
use std::rc::Rc;

use dioxus::document::Eval;
use dioxus::prelude::*;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::behaviours::{Inbound, StandDown};
use super::events::{EventField, EventKind};
use super::transport;
use super::{Behaviour, Epoch, Prop, Target};
use crate::dioxus_core::{provide_root_context, try_consume_context, Runtime, ScopeId};
use crate::js_listener::{receive, TEARDOWN};

/// How many page sequence numbers the pump holds ahead of a gap. A number still missing when more than this many
/// later ones wait is declared lost and the prefix moves past it: a page transition that was lost that way could
/// let a stale Rust write through the fence, which is the price of bounded memory.
const AHEAD_MAX: usize = 1024;

/// What the page sends for one inbound fact: its page-minted sequence number, and either the message (`m`) or the
/// slot whose page-caused transition it reports (`t`). A frame is the only shape a session's page-to-Rust message
/// takes, so every fact is ordered against every other one the webview sent.
#[derive(Deserialize)]
struct Frame {
    seq: u64,
    /// The subscription the frame belongs to.
    sub: u64,
    #[serde(default)]
    m: Option<Value>,
}

/// The inbound pump's per-webview state, kept in the root context: which page sequence numbers Rust has taken,
/// and the install epochs it has handed out.
#[derive(Clone, Default)]
struct Pump(Rc<RefCell<PumpState>>);

struct PumpState {
    /// The next page sequence number the contiguous prefix is waiting for.
    next: u64,
    /// Numbers taken ahead of `next` (a frame of another eval arrived first).
    ahead: BTreeSet<u64>,
    /// The install epochs handed out in this webview.
    epochs: u64,
    /// The subscription ids handed out in this webview.
    subs: u64,
    /// Where a subscription's messages go; an entry leaves when its [`Subscription`] drops.
    entries: HashMap<u64, Entry>,
}

type Entry = Rc<RefCell<Box<dyn FnMut(Value)>>>;

impl Default for PumpState {
    fn default() -> Self {
        PumpState {
            next: 1,
            ahead: BTreeSet::new(),
            epochs: 0,
            subs: 0,
            entries: HashMap::new(),
        }
    }
}

impl Pump {
    /// This webview's pump, created on first use together with its one inbound eval and the root task that
    /// reads it. Only inside the Dioxus runtime.
    fn current() -> Pump {
        try_consume_context::<Pump>().unwrap_or_else(|| {
            let pump = Pump::default();
            provide_root_context(pump.clone());
            let mut eval = transport::open(transport::Program::Inbound);
            let root = pump.clone();
            Runtime::current().spawn(ScopeId::ROOT, async move {
                receive(&mut eval, move |frame: Frame| root.dispatch(frame)).await;
            });
            pump
        })
    }

    /// Takes the frame's number, then hands its message to the entry of its subscription, if it still has one.
    fn dispatch(&self, frame: Frame) {
        self.take(frame.seq);
        let entry = self.0.borrow().entries.get(&frame.sub).cloned();
        if let (Some(entry), Some(message)) = (entry, frame.m) {
            (entry.borrow_mut())(message);
        }
    }

    /// Registers a subscription's handler and returns its id.
    fn register(&self, handler: Entry) -> u64 {
        let mut state = self.0.borrow_mut();
        state.subs += 1;
        let id = state.subs;
        state.entries.insert(id, handler);
        id
    }

    fn unregister(&self, id: u64) {
        self.0.borrow_mut().entries.remove(&id);
    }

    /// Takes the page's sequence number `seq`.
    fn take(&self, seq: u64) {
        let mut guard = self.0.borrow_mut();
        let state = &mut *guard;
        if seq < state.next {
            return;
        }
        state.ahead.insert(seq);
        // past the bound the oldest gap is declared lost: its number never arrived and holding every later one
        // for it would grow without limit
        if state.ahead.len() > AHEAD_MAX {
            if let Some(&first) = state.ahead.first() {
                state.next = first;
            }
        }
        while state.ahead.remove(&state.next) {
            state.next += 1;
        }
    }

    fn seen(&self) -> u64 {
        self.0.borrow().next - 1
    }

    fn epoch(&self, key: impl Into<String>) -> Epoch {
        let mut state = self.0.borrow_mut();
        state.epochs += 1;
        Epoch {
            key: key.into(),
            n: state.epochs,
        }
    }
}

/// Boots this webview's bridge: opens its one inbound eval and waits (bounded, about two seconds) until the page
/// reports `html[data-dx-bridge="ready"]`, which it sets once the inbound eval runs and the frames held for it are
/// flushed. Answers whether the page got there; `false` when the platform runs no JavaScript or the page did not
/// answer in time. An app calls it from its root effect, before it reports itself ready; any session opened
/// earlier has already created the inbound eval.
pub async fn boot() -> bool {
    if Runtime::try_current().is_none() {
        return false;
    }
    Pump::current();
    let ready = Prop::Attr("data-dx-bridge".into());
    for _ in 0..125 {
        match super::get(
            Target::Selector {
                selector: "html".into(),
            },
            [ready.clone()],
        )
        .await
        {
            Ok(props) if props.value(&ready).and_then(Value::as_str) == Some("ready") => {
                return true
            }
            Ok(_) => crate::sleep::sleep(std::time::Duration::from_millis(16)).await,
            Err(_) => return false,
        }
    }
    false
}

/// The highest page sequence number such that Rust has taken every frame up to it: the `based_on` of a write
/// Rust decides now ([`Transaction::based_on`](super::Transaction::based_on)). A number the page minted whose
/// frame has not arrived holds it back, so a write decided before Rust saw a page transition carries a number
/// below that transition and the page drops it. `0` outside the Dioxus runtime.
pub fn page_seen() -> u64 {
    if Runtime::try_current().is_none() {
        return 0;
    }
    Pump::current().seen()
}

/// One row of the `publish` option of [`ListenOptions`]: the page copies one event field (a number) into the custom property `var` (`--dx-` only) of the observed element, as it observes it, so a
/// scroll-linked value is written in the frame it is measured and never crosses to Rust. This is transport:
/// a measurement and a unit go to a variable; the option has no condition and reads no component state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Publish {
    /// The custom property, `--dx-name`.
    pub var: &'static str,
    /// What is copied: the wire name of an [`EventField`].
    pub field: &'static str,
    /// Appended to the number (`"px"`); empty for a plain number.
    #[serde(skip_serializing_if = "str::is_empty")]
    pub unit: &'static str,
}

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
    /// Copy event fields into custom properties of the target element, in the page (see [`Publish`]). A
    /// listener that publishes and names no field to send sends nothing to Rust.
    #[serde(skip_serializing_if = "<[Publish]>::is_empty")]
    pub publish: &'static [Publish],
}

impl ListenOptions {
    /// The capture phase.
    pub const CAPTURE: ListenOptions = ListenOptions {
        capture: true,
        once: false,
        publish: &[],
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
    /// Coalesced: the property `get` reads (a [`Prop`] wire name), its value at the end of
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
    /// Tells the page which subscription its frames belong to; `open` sends it first, before any other command.
    Sub {
        /// The subscription's id.
        id: u64,
    },
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
    /// The open level Rust decided for the top-layer element `target`, applied by the page executor
    /// (`ts/lib/executor.ts`, row 449). `nonce` is the owner's command count: the page drops a command older than
    /// the newest it applied to the element, and stamps its reports with the nonce it had applied.
    Executor {
        /// The element's `id`.
        target: String,
        /// What the level opens.
        kind: super::executor::ExecutorKind,
        /// Whether a false level hides the element.
        mode: super::executor::ShowMode,
        /// Open (`true`) or closed.
        level: bool,
        /// The owner's command count.
        nonce: u64,
        /// The construction site the page's apply marker names.
        site: super::behaviours::Site,
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
    /// `intersectionRect.height`: how much of the target's height is inside the root.
    pub intersection_height: f64,
    /// The root's height (`rootBounds.height`; the window's inner height without a root box).
    pub root_height: f64,
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
        let pump = Pump::current();
        let mut handler = handler;
        let id = pump.register(Rc::new(RefCell::new(Box::new(move |message: Value| {
            if let Ok(message) = serde_json::from_value::<T>(message) {
                handler(message);
            }
        }))));
        let eval = transport::open(transport::Program::Serve);
        let mut commands = vec![Command::Sub { id }];
        commands.extend(self.commands.iter().cloned());
        transport::send(&eval, &commands);
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
        Subscription {
            eval,
            renewal,
            pump,
            id,
        }
    }
}

/// A running session. Dropping it removes every listener, observer, media query, sampling loop and
/// behaviour the session registered.
#[must_use = "dropping a `Subscription` ends it at once"]
pub struct Subscription {
    eval: Eval,
    /// The lease-renewal task of a session that installed a behaviour with a lease.
    renewal: Option<crate::dioxus_core::Task>,
    /// The webview's pump, whose entry for this subscription leaves on drop: a message already on its way when the
    /// subscription ends is taken (its number counts) and not handled.
    pump: Pump,
    id: u64,
}

impl Subscription {
    /// Sends more commands to the running session, on its own channel (the page executor's next level).
    pub(crate) fn send(&self, commands: &[Command]) {
        transport::send(&self.eval, commands);
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        if let Some(renewal) = self.renewal.take() {
            renewal.cancel();
        }
        self.pump.unregister(self.id);
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
    mut handler: impl FnMut(T) + 'static,
) {
    // A kind that cannot decide reports once ([`StandDown`]); the install is then torn down and not
    // installed again while this component lives: a resync would meet the same kernel, and a torn-down
    // install clears its memory (a gesture half done) and decides nothing more, so no later decision
    // can add to what earlier ones set.
    let mut stood_down = use_signal(|| false);
    use_subscription(
        move || {
            if stood_down() {
                return None;
            }
            target().map(|target| {
                let behaviour = behaviour();
                // one install per kind, target and event: a kind that decides for several events of one target is
                // installed once per event, and its installs do not replace each other
                let epoch = Pump::current().epoch(format!(
                    "behaviour:{}:{}",
                    behaviour.kind(),
                    behaviour.declared().event.wire()
                ));
                Session::new().behaviour(target, behaviour, epoch)
            })
        },
        move |message: Inbound<T>| {
            route(message, &mut handler, |_report| stood_down.set(true));
        },
    );
}

/// Sends a decision's message to `on_message` and a stand-down report to `on_report`.
fn route<T>(
    message: Inbound<T>,
    on_message: &mut impl FnMut(T),
    mut on_report: impl FnMut(StandDown),
) {
    match message {
        Inbound::Message(m) => on_message(m),
        Inbound::Report(r) => on_report(r.report),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::cell::{Cell, RefCell};
    use std::collections::VecDeque;
    use std::rc::Rc;

    fn fixture() -> Behaviour {
        Behaviour::Fixture {
            event: EventKind::Click,
            attr: "data-x".into(),
            value: "y".into(),
            capture: false,
            cancel: true,
            focus: false,
            lease_ms: Some(400),
        }
    }

    #[test]
    fn a_stand_down_report_reaches_the_owner_and_a_decision_does_not() {
        #[derive(Debug, PartialEq, Deserialize)]
        struct Mine {
            behaviour: String,
        }
        let parse = |v: serde_json::Value| serde_json::from_value::<Inbound<Mine>>(v);
        let mut got = Vec::new();
        let mut reports = Vec::new();
        for v in [
            json!({"behaviour": "fixture"}),
            json!({"kernel": "stand-down", "kind": "fixture", "reason": "trap"}),
            json!({"behaviour": "fixture"}),
        ] {
            route(
                parse(v).expect("parses"),
                &mut |m: Mine| got.push(m.behaviour),
                |r| reports.push(r),
            );
        }
        assert_eq!(got, ["fixture", "fixture"]);
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].kind, "fixture");
        assert_eq!(
            reports[0].reason,
            super::super::behaviours::StandDownReason::Trap
        );
        // neither a decision nor a report: skipped by `receive`
        assert!(parse(json!({"other": 1})).is_err());
        assert!(parse(json!("x")).is_err());
    }

    /// The page, as far as the hook can see it: every eval the hook opens is recorded (its script and
    /// what Rust sent on it) and answers `recv` from a queue the test fills.
    #[derive(Default)]
    struct Page {
        evals: Vec<Rc<RefCell<Channel>>>,
        owners: Vec<dioxus::signals::Owner>,
    }

    #[derive(Default)]
    struct Channel {
        script: String,
        sent: Vec<serde_json::Value>,
        inbox: VecDeque<serde_json::Value>,
        waker: Option<std::task::Waker>,
        /// The page-minted sequence number of the last frame `deliver` sent.
        seq: u64,
        /// The webview's inbound eval, which carries every frame a session's scope sends.
        inbound: Option<Rc<RefCell<Channel>>>,
    }

    impl Channel {
        fn is_session(&self) -> bool {
            self.script.contains("__dxB.serve(")
        }

        /// The subscription id of the serve loop this eval runs, read from its `sub` command.
        fn sub(&self) -> Option<u64> {
            self.sent
                .iter()
                .filter_map(|m| m.as_array())
                .flatten()
                .find_map(|c| (c["cmd"] == "sub").then(|| c["id"].as_u64()).flatten())
        }
    }

    struct ScriptedEval(Rc<RefCell<Channel>>);

    impl dioxus::document::Evaluator for ScriptedEval {
        fn send(&self, data: serde_json::Value) -> Result<(), dioxus::document::EvalError> {
            self.0.borrow_mut().sent.push(data);
            Ok(())
        }
        fn poll_recv(
            &mut self,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<serde_json::Value, dioxus::document::EvalError>> {
            let mut channel = self.0.borrow_mut();
            match channel.inbox.pop_front() {
                Some(message) => std::task::Poll::Ready(Ok(message)),
                None => {
                    channel.waker = Some(cx.waker().clone());
                    std::task::Poll::Pending
                }
            }
        }
        fn poll_join(
            &mut self,
            _: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<serde_json::Value, dioxus::document::EvalError>> {
            std::task::Poll::Pending
        }
    }

    #[derive(Clone, Default)]
    struct ScriptedDocument(Rc<RefCell<Page>>);

    impl dioxus::document::Document for ScriptedDocument {
        fn eval(&self, js: String) -> Eval {
            let channel = Rc::new(RefCell::new(Channel {
                script: js,
                ..Channel::default()
            }));
            let owner = dioxus::signals::Owner::default();
            let eval = Eval::new(owner.insert(Box::new(ScriptedEval(channel.clone()))));
            let mut page = self.0.borrow_mut();
            if channel.borrow().is_session() {
                channel.borrow_mut().inbound = page
                    .evals
                    .iter()
                    .find(|c| c.borrow().script.contains("__dxB.inbound("))
                    .cloned();
            }
            page.evals.push(channel);
            page.owners.push(owner);
            eval
        }
        fn set_title(&self, _: String) {}
        fn create_meta(&self, _: dioxus::document::MetaProps) {}
        fn create_script(&self, _: dioxus::document::ScriptProps) {}
        fn create_style(&self, _: dioxus::document::StyleProps) {}
        fn create_link(&self, _: dioxus::document::LinkProps) {}
    }

    impl ScriptedDocument {
        /// The first eval whose script contains `needle`.
        fn eval_with(&self, needle: &str) -> Option<Rc<RefCell<Channel>>> {
            self.0
                .borrow()
                .evals
                .iter()
                .find(|c| c.borrow().script.contains(needle))
                .cloned()
        }

        fn has_eval(&self, needle: &str) -> bool {
            self.eval_with(needle).is_some()
        }

        /// The sessions the hook opened, in order.
        fn sessions(&self) -> Vec<Rc<RefCell<Channel>>> {
            let page = self.0.borrow();
            page.evals
                .iter()
                .filter(|c| c.borrow().is_session())
                .cloned()
                .collect()
        }
    }

    #[derive(Clone, Default)]
    struct Probe {
        got: Rc<RefCell<Vec<String>>>,
        generation: Rc<Cell<u32>>,
        unmounted: Rc<Cell<bool>>,
    }

    #[derive(Deserialize)]
    struct Decided {
        behaviour: String,
    }

    #[component]
    fn Host() -> Element {
        let probe: Probe = consume_context();
        rsx! { if !probe.unmounted.get() { Installer {} } }
    }

    #[component]
    fn Installer() -> Element {
        let probe: Probe = consume_context();
        let _ = probe.generation.get();
        use_behaviour(
            || Some(Target::Window),
            fixture,
            move |m: Decided| probe.got.borrow_mut().push(m.behaviour),
        );
        rsx! { "installed" }
    }

    /// Lets the effect, the receive task and a signal write it causes all settle (the effect needs
    /// a render pass to run, a write to a signal a render pass to re-run it).
    async fn settle(dom: &mut VirtualDom) {
        for _ in 0..3 {
            tokio::task::yield_now().await;
            dom.process_events();
            dom.render_immediate_to_vec();
        }
    }

    /// The page sends `message` as the next frame of its sequence.
    fn deliver(session: &Rc<RefCell<Channel>>, message: serde_json::Value) {
        let seq = {
            let mut channel = session.borrow_mut();
            channel.seq += 1;
            channel.seq
        };
        deliver_raw(session, json!({"seq": seq, "m": message}));
    }

    /// The page sends exactly `frame` (plus the session's `sub`) on the webview's inbound eval.
    fn deliver_raw(session: &Rc<RefCell<Channel>>, mut frame: serde_json::Value) {
        frame["sub"] = json!(session
            .borrow()
            .sub()
            .expect("the session registered a sub"));
        let inbound = session
            .borrow()
            .inbound
            .clone()
            .expect("the webview has an inbound eval");
        let mut channel = inbound.borrow_mut();
        channel.inbox.push_back(frame);
        if let Some(waker) = channel.waker.take() {
            waker.wake();
        }
    }

    /// Whether `session` was told to end (the teardown string rides the session's own channel).
    fn torn_down(session: &Rc<RefCell<Channel>>) -> bool {
        session.borrow().sent.iter().any(|m| m == &json!(TEARDOWN))
    }

    #[tokio::test(flavor = "current_thread", start_paused = true)]
    async fn use_behaviour_tears_the_install_down_on_a_stand_down_and_does_not_reinstall() {
        let document = ScriptedDocument::default();
        let probe = Probe::default();
        let mut dom = VirtualDom::new(Host)
            .with_root_context(Rc::new(document.clone()) as Rc<dyn dioxus::document::Document>)
            .with_root_context(probe.clone());
        dom.rebuild_in_place();
        settle(&mut dom).await;

        // installed once, on its own session, and live
        let sessions = document.sessions();
        assert_eq!(sessions.len(), 1, "one install");
        let first = sessions[0].clone();
        assert!(!torn_down(&first));

        // control: a decision's message reaches the handler and neither tears down nor reinstalls
        deliver(&first, json!({"behaviour": "fixture"}));
        settle(&mut dom).await;
        assert_eq!(*probe.got.borrow(), ["fixture"]);
        assert!(
            !torn_down(&first),
            "a decision does not tear the install down"
        );
        assert_eq!(
            document.sessions().len(),
            1,
            "a decision does not reinstall"
        );

        // the lease renews while the install lives
        let renewals =
            |c: &Rc<RefCell<Channel>>| c.borrow().sent.iter().filter(|m| m.is_array()).count();
        let before = renewals(&first);
        tokio::time::advance(std::time::Duration::from_millis(250)).await;
        settle(&mut dom).await;
        assert!(
            renewals(&first) > before,
            "the lease is renewed while installed"
        );

        // the stand-down report: the install is torn down, once, and nothing is installed in its place
        deliver(
            &first,
            json!({"kernel": "stand-down", "kind": "fixture", "reason": "missing-kind"}),
        );
        settle(&mut dom).await;
        assert!(torn_down(&first), "the session is told to end");
        assert_eq!(
            first
                .borrow()
                .sent
                .iter()
                .filter(|m| *m == &json!(TEARDOWN))
                .count(),
            1
        );
        assert_eq!(document.sessions().len(), 1, "no install replaces it");
        assert_eq!(
            *probe.got.borrow(),
            ["fixture"],
            "a report is not a decision"
        );

        // the lease heartbeat stopped with the install
        let after = renewals(&first);
        tokio::time::advance(std::time::Duration::from_millis(2000)).await;
        settle(&mut dom).await;
        assert_eq!(renewals(&first), after, "no renewal after the teardown");

        // later re-renders (the component re-runs, its closures re-read) do not install again
        for _ in 0..3 {
            probe.generation.set(probe.generation.get() + 1);
            dom.mark_dirty(ScopeId::APP);
            settle(&mut dom).await;
        }
        assert_eq!(
            document.sessions().len(),
            1,
            "a re-render does not reinstall"
        );
        assert_eq!(
            first
                .borrow()
                .sent
                .iter()
                .filter(|m| *m == &json!(TEARDOWN))
                .count(),
            1,
            "and the teardown is not repeated"
        );
    }

    /// A message the page had already sent when the subscription ended must not reach the handler.
    #[tokio::test(flavor = "current_thread", start_paused = true)]
    async fn a_message_after_the_subscription_ended_does_not_reach_the_handler() {
        for by_stand_down in [true, false] {
            let document = ScriptedDocument::default();
            let probe = Probe::default();
            let mut dom = VirtualDom::new(Host)
                .with_root_context(Rc::new(document.clone()) as Rc<dyn dioxus::document::Document>)
                .with_root_context(probe.clone());
            dom.rebuild_in_place();
            settle(&mut dom).await;
            let session = document.sessions()[0].clone();
            deliver(&session, json!({"behaviour": "fixture"}));
            settle(&mut dom).await;
            assert_eq!(*probe.got.borrow(), ["fixture"]);

            if by_stand_down {
                deliver(
                    &session,
                    json!({"kernel": "stand-down", "kind": "fixture", "reason": "trap"}),
                );
            } else {
                probe.unmounted.set(true);
                dom.mark_dirty(ScopeId::APP);
            }
            settle(&mut dom).await;
            assert!(torn_down(&session), "ended (stand-down: {by_stand_down})");

            deliver(&session, json!({"behaviour": "fixture"}));
            settle(&mut dom).await;
            assert_eq!(
                *probe.got.borrow(),
                ["fixture"],
                "a late decision is not handled (stand-down: {by_stand_down})"
            );
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
                               "capture": false, "cancel": true, "focus": false, "leaseMs": 400},
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
                once: true,
                publish: &[],
            })
            .expect("serialises"),
            json!({"once": true})
        );
    }

    #[test]
    fn a_publish_option_is_data_the_page_copies() {
        const ROWS: &[Publish] = &[Publish {
            var: "--dx-scroll-top",
            field: "target.scrollTop",
            unit: "px",
        }];
        assert_eq!(
            serde_json::to_value(ListenOptions {
                publish: ROWS,
                ..ListenOptions::default()
            })
            .expect("serialises"),
            json!({"publish": [{"var": "--dx-scroll-top", "field": "target.scrollTop", "unit": "px"}]})
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
    /// `page_seen` from inside the dom's root scope.
    fn seen(dom: &mut VirtualDom) -> u64 {
        dom.in_scope(ScopeId::ROOT, page_seen)
    }

    #[tokio::test(flavor = "current_thread", start_paused = true)]
    async fn the_pump_takes_the_page_sequence_as_a_contiguous_prefix() {
        let document = ScriptedDocument::default();
        let probe = Probe::default();
        let mut dom = VirtualDom::new(Host)
            .with_root_context(Rc::new(document.clone()) as Rc<dyn dioxus::document::Document>)
            .with_root_context(probe.clone());
        dom.rebuild_in_place();
        settle(&mut dom).await;
        let session = document.sessions()[0].clone();
        assert_eq!(seen(&mut dom), 0, "nothing taken yet");

        deliver_raw(&session, json!({"seq": 1, "m": {"behaviour": "fixture"}}));
        settle(&mut dom).await;
        assert_eq!(seen(&mut dom), 1);

        // a frame ahead of a gap is dispatched at once but does not move the prefix
        deliver_raw(&session, json!({"seq": 3, "m": {"behaviour": "fixture"}}));
        settle(&mut dom).await;
        assert_eq!(seen(&mut dom), 1, "number 2 has not arrived");
        assert_eq!(
            *probe.got.borrow(),
            ["fixture", "fixture"],
            "dispatch does not wait for the gap"
        );

        // a transition frame carries no message and closes the gap
        deliver_raw(&session, json!({"seq": 2, "t": "data-dx-keyboard"}));
        settle(&mut dom).await;
        assert_eq!(seen(&mut dom), 3);
        assert_eq!(
            probe.got.borrow().len(),
            2,
            "a transition reaches no handler"
        );

        // a repeated number moves nothing
        deliver_raw(&session, json!({"seq": 2, "t": "data-dx-keyboard"}));
        settle(&mut dom).await;
        assert_eq!(seen(&mut dom), 3);
    }

    #[tokio::test(flavor = "current_thread", start_paused = true)]
    async fn an_install_epoch_counts_in_its_own_webview() {
        let epoch_of = |session: &Rc<RefCell<Channel>>| {
            session.borrow().sent[0][1]["epoch"]["n"]
                .as_u64()
                .expect("an install carries an epoch")
        };
        // two webviews in one process: neither count depends on the other
        let mut firsts = Vec::new();
        for _ in 0..2 {
            let document = ScriptedDocument::default();
            let mut dom = VirtualDom::new(Host)
                .with_root_context(Rc::new(document.clone()) as Rc<dyn dioxus::document::Document>)
                .with_root_context(Probe::default());
            dom.rebuild_in_place();
            settle(&mut dom).await;
            firsts.push(epoch_of(&document.sessions()[0]));
        }
        assert_eq!(firsts, [1, 1]);
    }
    /// A frame the page minted for a scope that has since ended still advances the prefix, and reaches no handler.
    #[tokio::test(flavor = "current_thread", start_paused = true)]
    async fn a_frame_of_an_ended_scope_advances_the_prefix_and_reaches_no_handler() {
        let document = ScriptedDocument::default();
        let probe = Probe::default();
        let mut dom = VirtualDom::new(Host)
            .with_root_context(Rc::new(document.clone()) as Rc<dyn dioxus::document::Document>)
            .with_root_context(probe.clone());
        dom.rebuild_in_place();
        settle(&mut dom).await;
        let session = document.sessions()[0].clone();
        probe.unmounted.set(true);
        dom.mark_dirty(ScopeId::APP);
        settle(&mut dom).await;
        assert!(
            torn_down(&session),
            "the component's unmount ends the session"
        );

        deliver_raw(&session, json!({"seq": 1, "m": {"behaviour": "fixture"}}));
        deliver_raw(&session, json!({"seq": 2, "t": "data-dx-keyboard"}));
        settle(&mut dom).await;
        assert_eq!(
            seen(&mut dom),
            2,
            "the numbers were taken although nobody listens"
        );
        assert!(probe.got.borrow().is_empty(), "and no dropped handler ran");
    }

    /// The numbers held ahead of a gap are bounded: past the bound the gap is declared lost.
    #[tokio::test(flavor = "current_thread", start_paused = true)]
    async fn the_numbers_held_ahead_of_a_gap_are_bounded() {
        let document = ScriptedDocument::default();
        let mut dom = VirtualDom::new(Host)
            .with_root_context(Rc::new(document.clone()) as Rc<dyn dioxus::document::Document>)
            .with_root_context(Probe::default());
        dom.rebuild_in_place();
        settle(&mut dom).await;
        let session = document.sessions()[0].clone();
        // number 1 never arrives
        for seq in 2..=(AHEAD_MAX as u64 + 1) {
            deliver_raw(&session, json!({"seq": seq, "t": "x"}));
        }
        settle(&mut dom).await;
        assert_eq!(seen(&mut dom), 0, "at the bound the gap still holds");
        deliver_raw(&session, json!({"seq": AHEAD_MAX as u64 + 2, "t": "x"}));
        settle(&mut dom).await;
        assert_eq!(
            seen(&mut dom),
            AHEAD_MAX as u64 + 2,
            "past it the lost number is given up"
        );
    }
    #[tokio::test(flavor = "current_thread", start_paused = true)]
    async fn boot_opens_the_inbound_eval_and_waits_for_the_ready_attribute() {
        #[derive(Clone, Default)]
        struct Booted(Rc<Cell<Option<bool>>>);
        #[component]
        fn Booter() -> Element {
            let booted: Booted = consume_context();
            use_effect(move || {
                let booted = booted.clone();
                spawn(async move { booted.0.set(Some(boot().await)) });
            });
            rsx! { "boot" }
        }
        let document = ScriptedDocument::default();
        let booted = Booted::default();
        let mut dom = VirtualDom::new(Booter)
            .with_root_context(Rc::new(document.clone()) as Rc<dyn dioxus::document::Document>)
            .with_root_context(booted.clone());
        dom.rebuild_in_place();
        settle(&mut dom).await;
        assert!(
            document.has_eval("__dxB.inbound("),
            "boot opens the inbound eval"
        );
        assert_eq!(booted.0.get(), None, "not ready until the page says so");

        // the page answers the ask on the ordered channel: not ready, then ready
        for (round, attr) in [json!(null), json!("ready")].into_iter().enumerate() {
            let pump = document
                .eval_with("__dxB.pump(")
                .expect("the ordered channel");
            let id = pump
                .borrow()
                .sent
                .iter()
                .filter_map(|m| m["id"].as_u64())
                .nth(round)
                .unwrap_or_else(|| {
                    panic!(
                        "round {round}: an ask is on the channel: {:?}",
                        pump.borrow().sent
                    )
                });
            {
                let mut channel = pump.borrow_mut();
                channel.inbox.push_back(
                    json!({"id": id, "results": [{"ok": {"attr:data-dx-bridge": attr}}]}),
                );
                if let Some(waker) = channel.waker.take() {
                    waker.wake();
                }
            }
            for _ in 0..3 {
                tokio::time::advance(std::time::Duration::from_millis(20)).await;
                settle(&mut dom).await;
            }
        }
        assert_eq!(booted.0.get(), Some(true));
    }
}
