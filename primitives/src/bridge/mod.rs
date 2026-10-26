//! The generic JS bridge (backlog row 434 step 3): the one small typed layer between Rust and the
//! browser APIs Dioxus does not wrap, the same on web, desktop and mobile because it goes only
//! through `document::eval` (no `web-sys`).
//!
//! The JavaScript half is `primitives/src/ts/bridge.ts` (bundled to `src/js/bridge.js`, evaluated once
//! per webview by the transport and then reached as `window.__dxBridge`) plus one module per named
//! behaviour (`src/ts/behaviours/<kind>.ts`). It has generic verbs and no component logic. This module
//! is its Rust face. **The surface is closed**: Rust sends closed enums ([`Method`], [`Prop`],
//! [`EventKind`], [`EventField`], [`ObserverKind`], [`Behaviour`]) and the page maps each through a closed
//! own-property table with a receiver kind per row, so there is no `el[method](...)` and a name outside
//! a table is an error, never a property read.
//!
//! | JS verb | Rust |
//! |---|---|
//! | `call(el, method, args)` | [`ElementExt`] methods ([`show_popover`](ElementExt::show_popover), [`hide_popover`](ElementExt::hide_popover), [`show_modal`](ElementExt::show_modal), [`scroll_into_view`](ElementExt::scroll_into_view), ...) and [`Method`] |
//! | `get(el, props)` | [`ElementExt::get`] / [`get`] with [`Prop`], answered as [`Props`] |
//! | `set(el, --var, value)` | [`ElementExt::set_var`] |
//! | `listen(target, event, fields)` | [`use_event_listener`] (window, document or element), [`use_window_listener`], [`use_document_listener`], [`use_element_listener`], [`use_window_size`] |
//! | `observe(el, kind, opts)` | [`use_resize_observer`], [`use_intersection_observer`], [`use_mutation_observer`], [`use_observer`] |
//! | `matchMedia(query)` | [`use_media_query`] |
//! | `sample(el, fields, until)` | [`use_sample`], with coalesced and batched fields ([`SampleField`], [`Flush`]) |
//! | `behaviour(target, kind, ...)` | [`use_behaviour`], [`Behaviour`] (see [`behaviours`]) |
//! | `transaction(ops)` | [`Transaction`], [`show_popover_measured`], [`show_modal_measured`] |
//!
//! Elements are addressed by a [`Target`]: the window, the document, an element by `id` or by
//! selector, or the active element. There is no JS handle across renderers, so an element that the
//! bridge acts on carries an `id`.
//!
//! The hook names follow leptos-use and dioxus-sdk; there is no dioxus-sdk dependency. Each hook has a
//! deterministic initial value that is the same on the server and in the first client render
//! (`false`, a zero [`WindowSize`]); the real value arrives from the page after mount.
//!
//! # Where work goes
//!
//! Decide the placement first; each rung is the right one only when the rung above cannot do it.
//!
//! 1. **An rsx handler.** A target inside a Dioxus element and an event Dioxus supports: the handler
//!    decides synchronously with full Rust state, on every renderer (`prevent_default()` works).
//! 2. **A named [`Behaviour`].** A window, document or outside target, an event Dioxus lacks
//!    (`beforeinput` before 0.8, `close`), or the capture phase: the decision must be made inside the
//!    event's dispatch on the page's thread, so it is a pure `decide()` in a TS module that Rust installs
//!    by name beforehand.
//! 3. **`listen` / `observe` / `sample`.** A continuous stream (scroll, resize, pointermove, wheel,
//!    touchmove), or any work that may be late. Never a vdom handler: on desktop every vdom event is a
//!    blocking synchronous request, so a frame-rate stream queues behind itself.
//!
//! # Use Dioxus's own API first
//!
//! The bridge is for what Dioxus has no API for. Where it has one, use it: `onresize` and `onvisible`
//! for a plain element's size and visibility (on desktop `onresize` bubbles, so stop its propagation in
//! the child), `onscrollend`, `MountedData` (`get_client_rect`, `scroll_to`, `set_focus`) and, from 0.8,
//! `onbeforeinput`. The bridge's observers are for what those cannot express: a `rootMargin` or
//! `threshold`, a `MutationObserver`, a non-element target, a listener option (`capture`, `once`) or a
//! `window`/`document` listener.
//!
//! # How the page loads it, and the CSP consequence
//!
//! The transport evaluates the bundle through the eval queue, once per webview, behind a bounded
//! ready handshake (the reasons `asset!` plus `document::Script` was not adopted are in the transport
//! module). On the web, `document::eval` compiles its script with `new Function`, so an app served
//! under a Content-Security-Policy without `'unsafe-eval'` cannot use the bridge, `js_listener` or any
//! other eval-driven primitive (upstream dioxus-primitives has the same limit).
//!
//! # One FIFO channel for element operations, flushed after the render
//!
//! Every verb on an element, fire-and-forget ([`ElementExt::call`], [`ElementExt::set_var`],
//! [`Transaction::fire`]) or answered ([`get`], [`ElementExt::call_value`], [`Transaction::run`]), is a
//! message on ONE long-lived eval per webview that runs what it receives in order and answers the
//! asking messages on the same channel; separate evals have no ordering on desktop. So a write is
//! ordered before the read that follows it, and a read and a write that must land in one JS turn go in
//! one [`Transaction`]. A verb only queues its ops: one effect after the render that is running (or
//! about to) sends the queue, so an element the render created is in the DOM when its op runs on the
//! web. A verb called outside the Dioxus runtime does nothing, and an answered one is an error. Only
//! element operations are on this channel: each [`Session`] is its own eval (its order against the pump is
//! the order of the platform's script evaluations), and a stream is at most one message per frame per
//! stream with no acknowledgement.
//!
//! # What the bridge can and cannot carry
//!
//! A message from the page reaches Rust asynchronously: a microtask later on web, an IPC round trip on
//! desktop and mobile (the per-renderer contract is a table in the transport module). So the bridge
//! carries work that may be late (subscriptions, reads, calls). It cannot carry a decision made inside
//! an event's dispatch or a read-then-write that must land in one frame; the first is a [`Behaviour`]
//! that Rust publishes beforehand, the second a [`Transaction`]. Pointer capture and `focus` are not verbs
//! for the same reason: a call lands after the event's dispatch, so they are effects of a named behaviour.
//!
//! # Epochs
//!
//! An [`Epoch`] on a write, a read or a behaviour install makes the page drop it when a newer one for
//! the same key has already run on that target (a late command of a superseded request), and every
//! reply echoes the epoch it answers; a dropped op answers `dropped: true`.
//!
//! # Listener ownership
//!
//! Every subscription lives in a [`Subscription`], whose `Drop` tells the page to remove everything the
//! subscription registered (listeners, observers, media queries, sampling loops, behaviours); the message
//! goes over the channel the subscription already has, never a new eval (DioxusLabs/dioxus#4064). A
//! registration on an element also ends on the page when the element leaves the document, without
//! waiting for Rust (there is no merged `onunmounted`). The hooks tie that drop to the calling
//! component, exactly as [`use_js_listeners`](crate::js_listener::use_js_listeners) does;
//! [`js_listener`](crate::js_listener) itself runs on the same scope object, so there is one registry
//! (`window.__dxJsListeners.live`) and one removal rule.
//!
//! # The bridge-latency harness
//!
//! This is a bridge-latency harness, not a model of desktop. Every spec runs on Chromium web, where the
//! eval channel costs a microtask and hides each latency bug of a webview. Setting
//! `window.__dxBridgeDelayFrames = N` defers every message that passes a bridge channel by N animation
//! frames, in both directions (zero cost when unset). It covers the reply of [`get`], every subscription
//! message and every message a `js_listener` script sends. `"never"` instead drops every message to Rust
//! (the answer that never arrives; commands from Rust still arrive), which is how a lane proves that a
//! pending state still ends visible when Rust never answers. It does NOT delay `MountedData` reads, which
//! are synchronous on web (code that must behave on desktop reads through [`get`]), and it does not reproduce desktop's cadence (edits applied in a frame, one render
//! per frame, an eval from a handler before its render's edits); `channelDelay` in `bridge.ts` is the one
//! seam where a "desktop cadence" mode would plug in, and until that mode and a real-webview pass exist
//! a row says "web-verified; desktop inferred".

pub mod behaviours;
pub mod events;
pub(crate) mod legacy;
mod session;
pub(crate) mod transport;

use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::document::EvalError;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub use behaviours::{Behaviour, Declared, DeclaredTarget};
pub use events::{EventField, EventKind};
pub use session::{
    use_behaviour, use_document_listener, use_element_listener, use_event_listener,
    use_intersection_observer, use_media_query, use_mutation_observer, use_observer,
    use_resize_observer, use_sample, use_window_listener, use_window_size, Command, Flush,
    Intersection, ListenOptions, MediaChange, Mutation, ObserveOptions, ObserverKind, ResizeBox,
    SampleField, Sampled, Session, Size, Subscription, Until, WindowSize,
};
pub use transport::{is_unsupported, BRIDGE_JS};

/// A generation for an install, a write or a read: the page drops one older than the newest it has
/// seen on the same target for the same `key`, an equal one renews, and a reply echoes it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Epoch {
    /// What the generation counts (a behaviour kind, a request name).
    pub key: String,
    /// The generation; later is larger.
    pub n: u64,
}

impl Epoch {
    /// The next generation of `key`: monotonic across the process, so a later call is always newer.
    pub fn next(key: impl Into<String>) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        Epoch {
            key: key.into(),
            n: NEXT.fetch_add(1, Ordering::Relaxed),
        }
    }
}

/// What a bridge verb acts on.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Target {
    /// `window`.
    Window,
    /// `document`.
    Document,
    /// The element with this `id`.
    Id {
        /// The element's `id` attribute.
        id: String,
    },
    /// The first element matching this CSS selector.
    Selector {
        /// A CSS selector.
        selector: String,
    },
    /// `document.activeElement`.
    Active,
}

impl Target {
    /// The element with this `id`.
    pub fn id(id: impl Into<String>) -> Self {
        Target::Id { id: id.into() }
    }

    /// The first element matching `selector`.
    pub fn selector(selector: impl Into<String>) -> Self {
        Target::Selector {
            selector: selector.into(),
        }
    }
}

impl From<&str> for Target {
    fn from(id: &str) -> Self {
        Target::id(id)
    }
}

impl From<String> for Target {
    fn from(id: String) -> Self {
        Target::Id { id }
    }
}

impl From<&String> for Target {
    fn from(id: &String) -> Self {
        Target::id(id.as_str())
    }
}

/// Where `scrollIntoView` aligns the element on one axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ScrollAlign {
    /// Align to the start edge.
    Start,
    /// Center it.
    Center,
    /// Align to the end edge.
    End,
    /// Scroll the least that makes it visible.
    Nearest,
}

/// How a scroll moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ScrollMotion {
    /// The element's `scroll-behavior`.
    Auto,
    /// Jump.
    Instant,
    /// Animate.
    Smooth,
}

/// `scrollIntoView`'s options.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct ScrollIntoView {
    /// Vertical alignment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block: Option<ScrollAlign>,
    /// Horizontal alignment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline: Option<ScrollAlign>,
    /// Jump or animate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub behavior: Option<ScrollMotion>,
}

/// A method the bridge may call on an element: a closed list, never `el[name](...)`. The popover and
/// dialog methods tolerate the state they are called in (showing an open popover is not an error).
#[derive(Clone, Debug, PartialEq)]
pub enum Method {
    /// `showPopover()`.
    ShowPopover,
    /// `hidePopover()`.
    HidePopover,
    /// `togglePopover()`.
    TogglePopover,
    /// `showModal()` on a `<dialog>`.
    ShowModal,
    /// `close(returnValue)` on a `<dialog>`.
    Close(Option<String>),
    /// `blur()`.
    Blur,
    /// `scrollIntoView(options)`.
    ScrollIntoView(ScrollIntoView),
    /// `scrollTo({ left, top, behavior })`.
    ScrollTo {
        /// Horizontal offset, if it changes.
        left: Option<f64>,
        /// Vertical offset, if it changes.
        top: Option<f64>,
        /// Jump or animate.
        behavior: Option<ScrollMotion>,
    },
    /// `scrollBy({ left, top, behavior })`.
    ScrollBy {
        /// Horizontal delta.
        left: Option<f64>,
        /// Vertical delta.
        top: Option<f64>,
        /// Jump or animate.
        behavior: Option<ScrollMotion>,
    },
    /// `setSelectionRange(start, end, direction)` on a text control.
    SetSelectionRange {
        /// Selection start.
        start: u32,
        /// Selection end.
        end: u32,
    },
    /// `select()` on a text control.
    Select,
}

fn scroll_options(left: Option<f64>, top: Option<f64>, behavior: Option<ScrollMotion>) -> Value {
    let mut options = serde_json::Map::new();
    if let Some(left) = left {
        options.insert("left".into(), json!(left));
    }
    if let Some(top) = top {
        options.insert("top".into(), json!(top));
    }
    if let Some(behavior) = behavior {
        options.insert("behavior".into(), json!(behavior));
    }
    Value::Object(options)
}

impl Method {
    /// The JS method name and its arguments, as the bridge's `call` takes them.
    pub fn wire(&self) -> (&'static str, Vec<Value>) {
        match self {
            Method::ShowPopover => ("showPopover", vec![]),
            Method::HidePopover => ("hidePopover", vec![]),
            Method::TogglePopover => ("togglePopover", vec![]),
            Method::ShowModal => ("showModal", vec![]),
            Method::Close(None) => ("close", vec![]),
            Method::Close(Some(value)) => ("close", vec![json!(value)]),
            Method::Blur => ("blur", vec![]),
            Method::ScrollIntoView(options) => ("scrollIntoView", vec![json!(options)]),
            Method::ScrollTo {
                left,
                top,
                behavior,
            } => ("scrollTo", vec![scroll_options(*left, *top, *behavior)]),
            Method::ScrollBy {
                left,
                top,
                behavior,
            } => ("scrollBy", vec![scroll_options(*left, *top, *behavior)]),
            Method::SetSelectionRange { start, end } => {
                ("setSelectionRange", vec![json!(start), json!(end)])
            }
            Method::Select => ("select", vec![]),
        }
    }
}

/// A value `get` can read.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Prop {
    /// A computed style value (`getComputedStyle(el).getPropertyValue(name)`), a string.
    Style(String),
    /// An attribute's value, or null.
    Attr(String),
    /// Whether the element matches a selector, a bool.
    Matches(String),
    /// `offsetWidth`.
    OffsetWidth,
    /// `offsetHeight`.
    OffsetHeight,
    /// `offsetTop`.
    OffsetTop,
    /// `offsetLeft`.
    OffsetLeft,
    /// `clientWidth`.
    ClientWidth,
    /// `clientHeight`.
    ClientHeight,
    /// `clientTop`.
    ClientTop,
    /// `clientLeft`.
    ClientLeft,
    /// `scrollWidth`.
    ScrollWidth,
    /// `scrollHeight`.
    ScrollHeight,
    /// `scrollTop`.
    ScrollTop,
    /// `scrollLeft`.
    ScrollLeft,
    /// `getBoundingClientRect()`, a [`Rect`].
    Rect,
    /// `document.activeElement` as `{ id, tagName, within }` (`within`: focus is inside the target).
    ActiveElement,
    /// `window.visualViewport` as `{ width, height, offsetTop, offsetLeft, pageTop, pageLeft, scale }`.
    VisualViewport,
    /// `window.innerWidth`.
    InnerWidth,
    /// `window.innerHeight`.
    InnerHeight,
    /// `window.scrollX`.
    ScrollX,
    /// `window.scrollY`.
    ScrollY,
    /// `devicePixelRatio`.
    DevicePixelRatio,
    /// A form control's `value`.
    Value,
    /// A checkbox or radio's `checked`.
    Checked,
    /// A text control's `selectionStart`.
    SelectionStart,
    /// A text control's `selectionEnd`.
    SelectionEnd,
    /// Whether a `<dialog>` is open.
    Open,
    /// Whether a popover is open (`:popover-open`).
    PopoverOpen,
}

impl Prop {
    /// The name the bridge's `get` takes.
    pub fn wire(&self) -> String {
        match self {
            Prop::Style(name) => format!("style:{name}"),
            Prop::Attr(name) => format!("attr:{name}"),
            Prop::Matches(selector) => format!("matches:{selector}"),
            Prop::OffsetWidth => "offsetWidth".into(),
            Prop::OffsetHeight => "offsetHeight".into(),
            Prop::OffsetTop => "offsetTop".into(),
            Prop::OffsetLeft => "offsetLeft".into(),
            Prop::ClientWidth => "clientWidth".into(),
            Prop::ClientHeight => "clientHeight".into(),
            Prop::ClientTop => "clientTop".into(),
            Prop::ClientLeft => "clientLeft".into(),
            Prop::ScrollWidth => "scrollWidth".into(),
            Prop::ScrollHeight => "scrollHeight".into(),
            Prop::ScrollTop => "scrollTop".into(),
            Prop::ScrollLeft => "scrollLeft".into(),
            Prop::Rect => "rect".into(),
            Prop::ActiveElement => "activeElement".into(),
            Prop::VisualViewport => "visualViewport".into(),
            Prop::InnerWidth => "innerWidth".into(),
            Prop::InnerHeight => "innerHeight".into(),
            Prop::ScrollX => "scrollX".into(),
            Prop::ScrollY => "scrollY".into(),
            Prop::DevicePixelRatio => "devicePixelRatio".into(),
            Prop::Value => "value".into(),
            Prop::Checked => "checked".into(),
            Prop::SelectionStart => "selectionStart".into(),
            Prop::SelectionEnd => "selectionEnd".into(),
            Prop::Open => "open".into(),
            Prop::PopoverOpen => "popoverOpen".into(),
        }
    }
}

/// `getBoundingClientRect()` as `get` returns it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Deserialize)]
pub struct Rect {
    /// Left edge, like `left`.
    pub x: f64,
    /// Top edge, like `top`.
    pub y: f64,
    /// Width.
    pub width: f64,
    /// Height.
    pub height: f64,
    /// Top edge.
    pub top: f64,
    /// Left edge.
    pub left: f64,
    /// Right edge.
    pub right: f64,
    /// Bottom edge.
    pub bottom: f64,
}

/// What [`get`] read: the requested props and their values. A prop the page could not read (a missing
/// element, a property the target does not have) is JSON null.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(transparent)]
pub struct Props(pub serde_json::Map<String, Value>);

impl Props {
    /// The raw JSON value of `prop`, if it was requested.
    pub fn value(&self, prop: &Prop) -> Option<&Value> {
        self.0.get(&prop.wire())
    }

    /// `prop` as a number, if it is one.
    pub fn f64(&self, prop: &Prop) -> Option<f64> {
        self.value(prop).and_then(Value::as_f64)
    }

    /// `prop` as a bool, if it is one.
    pub fn bool(&self, prop: &Prop) -> Option<bool> {
        self.value(prop).and_then(Value::as_bool)
    }

    /// `prop` as a string, if it is one.
    pub fn string(&self, prop: &Prop) -> Option<&str> {
        self.value(prop).and_then(Value::as_str)
    }

    /// [`Prop::Rect`] as a [`Rect`].
    pub fn rect(&self) -> Option<Rect> {
        self.value(&Prop::Rect)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }
}

/// Why a request to the page failed.
#[derive(Debug)]
pub enum BridgeError {
    /// The eval channel failed (the page is gone, or the answer did not deserialize).
    Eval(EvalError),
    /// The page threw while running the verb (an unknown method, a bad selector); the message is the
    /// error's text.
    Js(String),
    /// The renderer cannot run JavaScript (Blitz); see [`is_unsupported`].
    Unsupported,
}

impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BridgeError::Eval(error) => write!(f, "bridge channel: {error}"),
            BridgeError::Js(message) => write!(f, "bridge verb failed: {message}"),
            BridgeError::Unsupported => write!(f, "this renderer cannot run JavaScript"),
        }
    }
}

impl std::error::Error for BridgeError {}

/// An operation of a [`Transaction`], as the page's `transaction` takes it.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum Op {
    /// A [`Method`] on a target.
    Call {
        /// The element.
        target: Target,
        /// The method name (see [`Method::wire`]).
        method: &'static str,
        /// Its arguments.
        args: Vec<Value>,
        /// Dropped when a newer epoch for its key already ran on the target.
        #[serde(skip_serializing_if = "Option::is_none")]
        epoch: Option<Epoch>,
    },
    /// A [`get`].
    Get {
        /// The element.
        target: Target,
        /// The props, by wire name.
        props: Vec<String>,
        /// Dropped when a newer epoch for its key already ran on the target; the reply echoes it.
        #[serde(skip_serializing_if = "Option::is_none")]
        epoch: Option<Epoch>,
    },
    /// A custom-property write ([`ElementExt::set_var`]).
    Set {
        /// The element.
        target: Target,
        /// The custom property, `--dx-name`.
        name: String,
        /// The value; `None` removes the property.
        value: Option<String>,
        /// Dropped when a newer epoch for its key already ran on the target.
        #[serde(skip_serializing_if = "Option::is_none")]
        epoch: Option<Epoch>,
    },
}

/// The result of one [`Op`].
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum OpResult {
    /// The op ran (or was dropped); its value (null for a `call` with no result).
    Ok {
        /// The op's value.
        ok: Value,
        /// A newer epoch had already run on the target, so the op did nothing.
        #[serde(default)]
        dropped: bool,
        /// The epoch the op carried, echoed.
        #[serde(default)]
        epoch: Option<Epoch>,
    },
    /// The op threw.
    Err {
        /// The error's text.
        err: String,
    },
}

/// Several ops run in ONE JS turn, in order: a write and the read that must see it, or two calls that
/// must not be separated by a render.
///
/// ```rust,ignore
/// let results = Transaction::new()
///     .call("panel", Method::ShowPopover)
///     .get("panel", [Prop::Rect])
///     .run()
///     .await?;
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Transaction {
    ops: Vec<Op>,
}

impl Transaction {
    /// An empty transaction.
    pub fn new() -> Self {
        Self::default()
    }

    fn new_op(op: Op) -> Self {
        Self { ops: vec![op] }
    }

    /// Adds a [`Method`] call.
    #[must_use]
    pub fn call(mut self, target: impl Into<Target>, method: Method) -> Self {
        let (name, args) = method.wire();
        self.ops.push(Op::Call {
            target: target.into(),
            method: name,
            args,
            epoch: None,
        });
        self
    }

    /// Adds a read of `props`.
    #[must_use]
    pub fn get(mut self, target: impl Into<Target>, props: impl IntoIterator<Item = Prop>) -> Self {
        self.ops.push(Op::Get {
            target: target.into(),
            props: props.into_iter().map(|p| p.wire()).collect(),
            epoch: None,
        });
        self
    }

    /// Adds a custom-property write; `None` removes it.
    #[must_use]
    pub fn set_var(
        mut self,
        target: impl Into<Target>,
        name: impl Into<String>,
        value: Option<String>,
    ) -> Self {
        self.ops.push(Op::Set {
            target: target.into(),
            name: name.into(),
            value,
            epoch: None,
        });
        self
    }

    /// Puts `epoch` on the LAST op added (a call, a read or a write): the page drops it when a newer
    /// epoch for the same key already ran on that target, and the reply echoes it.
    #[must_use]
    pub fn epoch(mut self, epoch: Epoch) -> Self {
        match self.ops.last_mut() {
            Some(Op::Call { epoch: slot, .. })
            | Some(Op::Get { epoch: slot, .. })
            | Some(Op::Set { epoch: slot, .. }) => *slot = Some(epoch),
            None => {}
        }
        self
    }

    /// The ops, in order.
    pub fn ops(&self) -> &[Op] {
        &self.ops
    }

    /// Runs the ops and waits for every result.
    pub async fn run(self) -> Result<Vec<OpResult>, BridgeError> {
        transport::ask(&self.ops).await
    }

    /// Sends the ops on the webview's ordered write channel without waiting for a result: they run after
    /// every earlier [`fire`](Self::fire), [`call`](ElementExt::call) and
    /// [`set_var`](ElementExt::set_var) of this webview, in order.
    pub fn fire(self) {
        transport::fire(&self.ops);
    }
}

/// Reads `props` of `target` in one JS turn. The answer arrives asynchronously (a microtask on web,
/// an IPC round trip on desktop).
pub async fn get(
    target: impl Into<Target>,
    props: impl IntoIterator<Item = Prop>,
) -> Result<Props, BridgeError> {
    let value = single(Transaction::new().get(target, props).run().await?)?;
    serde_json::from_value(value).map_err(|e| BridgeError::Js(e.to_string()))
}

/// The value of the only op of a one-op transaction.
fn single(results: Vec<OpResult>) -> Result<Value, BridgeError> {
    match results.into_iter().next() {
        Some(OpResult::Ok { ok, .. }) => Ok(ok),
        Some(OpResult::Err { err }) => Err(BridgeError::Js(err)),
        None => Err(BridgeError::Js(
            "the transaction answered fewer ops than sent".into(),
        )),
    }
}

/// Shows the popover `target` and reads `props` of it in ONE JS turn: the values are those of the element
/// as shown, with no render or frame between the show and the read.
pub async fn show_popover_measured(
    target: impl Into<Target>,
    props: impl IntoIterator<Item = Prop>,
) -> Result<Props, BridgeError> {
    measured(target.into(), Method::ShowPopover, props).await
}

/// [`show_popover_measured`] for a `<dialog>`: `showModal()`, then the read, in one JS turn.
pub async fn show_modal_measured(
    target: impl Into<Target>,
    props: impl IntoIterator<Item = Prop>,
) -> Result<Props, BridgeError> {
    measured(target.into(), Method::ShowModal, props).await
}

async fn measured(
    target: Target,
    method: Method,
    props: impl IntoIterator<Item = Prop>,
) -> Result<Props, BridgeError> {
    let results = Transaction::new()
        .call(target.clone(), method)
        .get(target, props)
        .run()
        .await?;
    let mut results = results.into_iter();
    if let Some(OpResult::Err { err }) = results.next() {
        return Err(BridgeError::Js(err));
    }
    match results.next() {
        Some(OpResult::Ok { ok, .. }) => {
            serde_json::from_value(ok).map_err(|e| BridgeError::Js(e.to_string()))
        }
        Some(OpResult::Err { err }) => Err(BridgeError::Js(err)),
        None => Err(BridgeError::Js(
            "the transaction answered fewer ops than sent".into(),
        )),
    }
}

/// The bridge's verbs on an element, window or document, by [`Target`], or by element `id` for a
/// `&str` / `String`. The calls are fire-and-forget; [`get`](ElementExt::get) and
/// [`call_value`](ElementExt::call_value) answer.
pub trait ElementExt {
    /// The target this value names.
    fn target(&self) -> Target;

    /// Calls `method` on the webview's ordered write channel (after every earlier write of this webview);
    /// any error is logged to the browser console.
    fn call(&self, method: Method) {
        let (name, args) = method.wire();
        Transaction::new_op(Op::Call {
            target: self.target(),
            method: name,
            args,
            epoch: None,
        })
        .fire();
    }

    /// Calls `method` and answers with its result (null for a method that returns nothing).
    #[allow(
        async_fn_in_trait,
        reason = "the future is awaited in place by the caller; it is never spawned across threads"
    )]
    async fn call_value(&self, method: Method) -> Result<Value, BridgeError> {
        single(Transaction::new().call(self.target(), method).run().await?)
    }

    /// Reads `props` in one JS turn.
    #[allow(
        async_fn_in_trait,
        reason = "the future is awaited in place by the caller; it is never spawned across threads"
    )]
    async fn get(&self, props: impl IntoIterator<Item = Prop>) -> Result<Props, BridgeError> {
        get(self.target(), props).await
    }

    /// Writes the theme's CSS custom property `name` (`--dx-name`; the page refuses any other) on the
    /// element, for an element whose style Rust does not own; `None` removes it.
    fn set_var(&self, name: &str, value: Option<&str>) {
        Transaction::new_op(Op::Set {
            target: self.target(),
            name: name.into(),
            value: value.map(String::from),
            epoch: None,
        })
        .fire();
    }

    /// `showPopover()`; nothing if it is already open.
    fn show_popover(&self) {
        self.call(Method::ShowPopover);
    }

    /// `hidePopover()`; nothing if it is not open.
    fn hide_popover(&self) {
        self.call(Method::HidePopover);
    }

    /// `showModal()`; nothing if the dialog is already open.
    fn show_modal(&self) {
        self.call(Method::ShowModal);
    }

    /// `close()` on a dialog; nothing if it is closed.
    fn close_dialog(&self) {
        self.call(Method::Close(None));
    }

    /// `scrollIntoView(options)`.
    fn scroll_into_view(&self, options: ScrollIntoView) {
        self.call(Method::ScrollIntoView(options));
    }

    /// `setSelectionRange(start, end)`.
    fn set_selection_range(&self, start: u32, end: u32) {
        self.call(Method::SetSelectionRange { start, end });
    }
}

impl ElementExt for Target {
    fn target(&self) -> Target {
        self.clone()
    }
}

impl ElementExt for str {
    fn target(&self) -> Target {
        Target::id(self)
    }
}

impl ElementExt for String {
    fn target(&self) -> Target {
        Target::id(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_json(value: &impl Serialize) -> String {
        serde_json::to_string(value).expect("serialises")
    }

    #[test]
    fn a_target_is_a_tagged_object() {
        assert_eq!(to_json(&Target::Window), r#"{"kind":"window"}"#);
        assert_eq!(to_json(&Target::Document), r#"{"kind":"document"}"#);
        assert_eq!(to_json(&Target::Active), r#"{"kind":"active"}"#);
        assert_eq!(to_json(&Target::id("a")), r#"{"kind":"id","id":"a"}"#);
        assert_eq!(
            to_json(&Target::selector("#a > b")),
            r##"{"kind":"selector","selector":"#a > b"}"##
        );
        assert_eq!(Target::from("x"), Target::id("x"));
        assert_eq!("x".target(), Target::id("x"));
        assert_eq!(String::from("x").target(), Target::id("x"));
    }

    #[test]
    fn methods_have_the_wire_names_the_bridge_whitelists() {
        let names: Vec<&str> = [
            Method::ShowPopover,
            Method::HidePopover,
            Method::TogglePopover,
            Method::ShowModal,
            Method::Close(None),
            Method::Blur,
            Method::ScrollIntoView(ScrollIntoView::default()),
            Method::ScrollTo {
                left: None,
                top: None,
                behavior: None,
            },
            Method::ScrollBy {
                left: None,
                top: None,
                behavior: None,
            },
            Method::SetSelectionRange { start: 0, end: 0 },
            Method::Select,
        ]
        .iter()
        .map(|m| m.wire().0)
        .collect();
        // Every name is a key of `METHODS` in `ts/bridge.ts`.
        let js = include_str!("../ts/bridge.ts");
        for name in names {
            assert!(
                js.contains(&format!("{name}(")) || js.contains(&format!("{name}:")),
                "{name}"
            );
        }
    }

    #[test]
    fn method_arguments_are_the_dom_arguments() {
        assert_eq!(
            Method::ScrollIntoView(ScrollIntoView {
                block: Some(ScrollAlign::Nearest),
                inline: None,
                behavior: Some(ScrollMotion::Instant),
            })
            .wire(),
            (
                "scrollIntoView",
                vec![json!({"block": "nearest", "behavior": "instant"})]
            )
        );
        assert_eq!(
            Method::ScrollTo {
                left: None,
                top: Some(10.0),
                behavior: None
            }
            .wire(),
            ("scrollTo", vec![json!({"top": 10.0})])
        );
        assert_eq!(
            Method::SetSelectionRange { start: 1, end: 3 }.wire(),
            ("setSelectionRange", vec![json!(1), json!(3)])
        );
        assert_eq!(
            Method::Close(Some("ok".into())).wire(),
            ("close", vec![json!("ok")])
        );
    }

    #[test]
    fn props_have_wire_names_and_props_read_back_by_them() {
        assert_eq!(
            Prop::Style("padding-top".into()).wire(),
            "style:padding-top"
        );
        assert_eq!(Prop::Attr("data-state".into()).wire(), "attr:data-state");
        assert_eq!(Prop::OffsetWidth.wire(), "offsetWidth");
        let props: Props = serde_json::from_value(json!({
            "offsetWidth": 100,
            "style:color": "red",
            "popoverOpen": true,
            "rect": {"x": 1, "y": 2, "width": 3, "height": 4, "top": 2, "left": 1, "right": 4, "bottom": 6},
            "attr:gone": null,
        }))
        .expect("props deserialize");
        assert_eq!(props.f64(&Prop::OffsetWidth), Some(100.0));
        assert_eq!(props.string(&Prop::Style("color".into())), Some("red"));
        assert_eq!(props.bool(&Prop::PopoverOpen), Some(true));
        assert_eq!(props.string(&Prop::Attr("gone".into())), None);
        assert_eq!(props.f64(&Prop::OffsetHeight), None);
        assert_eq!(props.rect().map(|r| r.height), Some(4.0));
    }

    #[test]
    fn a_transaction_serialises_to_the_ops_the_page_runs() {
        let tx = Transaction::new()
            .call("p", Method::ShowPopover)
            .get("p", [Prop::Rect, Prop::PopoverOpen])
            .set_var("p", "--dx-x", Some("1px".into()));
        assert_eq!(
            serde_json::to_value(tx.ops()).expect("serialises"),
            json!([
                {"op": "call", "target": {"kind": "id", "id": "p"}, "method": "showPopover", "args": []},
                {"op": "get", "target": {"kind": "id", "id": "p"}, "props": ["rect", "popoverOpen"]},
                {"op": "set", "target": {"kind": "id", "id": "p"}, "name": "--dx-x", "value": "1px"},
            ])
        );
    }

    #[test]
    fn an_epoch_goes_on_the_last_op_and_a_reply_echoes_it() {
        let tx = Transaction::new()
            .call("p", Method::ShowPopover)
            .get("p", [Prop::Rect])
            .epoch(Epoch {
                key: "measure".into(),
                n: 4,
            });
        let json = serde_json::to_value(tx.ops()).expect("serialises");
        assert!(json[0].get("epoch").is_none());
        assert_eq!(json[1]["epoch"], json!({"key": "measure", "n": 4}));
        let reply: OpResult = serde_json::from_value(
            json!({"ok": null, "dropped": true, "epoch": {"key": "measure", "n": 4}}),
        )
        .expect("reply");
        assert_eq!(
            reply,
            OpResult::Ok {
                ok: Value::Null,
                dropped: true,
                epoch: Some(Epoch {
                    key: "measure".into(),
                    n: 4
                })
            }
        );
        assert!(Epoch::next("k").n < Epoch::next("k").n);
    }

    /// Every non-parameterised `Prop` is one row of the page's `PROPS` table, and the table has no other.
    #[test]
    fn the_props_table_is_this_enum() {
        let props = [
            Prop::OffsetWidth,
            Prop::OffsetHeight,
            Prop::OffsetTop,
            Prop::OffsetLeft,
            Prop::ClientWidth,
            Prop::ClientHeight,
            Prop::ClientTop,
            Prop::ClientLeft,
            Prop::ScrollWidth,
            Prop::ScrollHeight,
            Prop::ScrollTop,
            Prop::ScrollLeft,
            Prop::Rect,
            Prop::ActiveElement,
            Prop::VisualViewport,
            Prop::InnerWidth,
            Prop::InnerHeight,
            Prop::ScrollX,
            Prop::ScrollY,
            Prop::DevicePixelRatio,
            Prop::Value,
            Prop::Checked,
            Prop::SelectionStart,
            Prop::SelectionEnd,
            Prop::Open,
            Prop::PopoverOpen,
        ];
        let js = include_str!("../ts/bridge.ts");
        let start = js.find("const PROPS").expect("PROPS table");
        let end = start + js[start..].find("\n};").expect("end of PROPS");
        let table = &js[start..end];
        for prop in &props {
            assert!(
                table.contains(&format!("\n  \"{}\": {{", prop.wire())),
                "{}",
                prop.wire()
            );
        }
        let rows = table
            .lines()
            .filter(|l| l.starts_with("  \"") && l.contains("\": {"))
            .count();
        assert_eq!(rows, props.len(), "PROPS has a row Rust does not name");
    }

    #[test]
    fn op_results_are_ok_or_err() {
        let results: Vec<OpResult> =
            serde_json::from_value(json!([{"ok": null}, {"ok": {"a": 1}}, {"err": "boom"}]))
                .expect("results deserialize");
        assert_eq!(
            results[0],
            OpResult::Ok {
                ok: Value::Null,
                dropped: false,
                epoch: None
            }
        );
        assert_eq!(results[2], OpResult::Err { err: "boom".into() });
    }

    /// The Rust `include_str!` embeds the committed bundle, which installs `window.__dxBridge`.
    #[test]
    fn the_bundle_installs_the_bridge() {
        assert!(BRIDGE_JS.contains("__dxBridge"));
        assert!(BRIDGE_JS.contains("__dxBridgeDelayFrames"));
        assert!(!BRIDGE_JS.contains("sync-rules"), "the rule layer is gone");
        assert!(
            !BRIDGE_JS.contains("export{"),
            "a bundle that is eval'd must not be a module"
        );
    }
}
