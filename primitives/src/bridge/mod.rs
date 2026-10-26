//! The generic JS bridge (backlog row 434 step 3): the one small typed layer between Rust and the
//! browser APIs Dioxus does not wrap, the same on web, desktop and mobile because it goes only
//! through `document::eval` (no `web-sys`).
//!
//! The JavaScript half is `primitives/src/ts/bridge.ts` (bundled to `src/js/bridge.js`, evaluated once
//! per webview by the transport and then reached as `window.__dxBridge`). It has generic verbs and
//! no component logic. This module is its Rust face:
//!
//! | JS verb | Rust |
//! |---|---|
//! | `call(el, method, args)` | [`ElementExt`] methods ([`show_popover`](ElementExt::show_popover), [`hide_popover`](ElementExt::hide_popover), [`show_modal`](ElementExt::show_modal), [`set_pointer_capture`](ElementExt::set_pointer_capture), [`focus_prevent_scroll`](ElementExt::focus_prevent_scroll), ...) and [`Method`] |
//! | `get(el, props)` | [`ElementExt::get`] / [`get`] with [`Prop`], answered as [`Props`] |
//! | `set(el, --var, value)` | [`ElementExt::set_var`] |
//! | `listen(target, type, options, fields)` | [`use_event_listener`] (window, document or element), [`use_window_listener`], [`use_document_listener`], [`use_element_listener`], [`use_window_size`] |
//! | `observe(el, kind, opts)` | [`use_resize_observer`], [`use_intersection_observer`], [`use_mutation_observer`], [`use_observer`] |
//! | `matchMedia(query)` | [`use_media_query`] |
//! | `sample(el, fields, until)` | [`use_sample`], with coalesced and batched fields ([`SampleField`]) |
//! | `rules.install` / `data-dx-rule-*` | [`use_rule`], [`Rule`] |
//! | `transaction(ops)`, `commit()` | [`Transaction`], [`commit`] |
//!
//! Elements are addressed by a [`Target`]: the window, the document, an element by `id` or by
//! selector, or the active element. There is no JS handle across renderers, so an element that the
//! bridge acts on carries an `id`.
//!
//! The hook names follow leptos-use and dioxus-sdk; there is no dioxus-sdk dependency. Each hook has a
//! deterministic initial value that is the same on the server and in the first client render
//! (`false`, a zero [`WindowSize`]); the real value arrives from the page after mount.
//!
//! # Use Dioxus's own API first
//!
//! The bridge is for what Dioxus has no API for. Where it has one, use it: `onresize` and `onvisible`
//! for a plain element's size and visibility (on desktop `onresize` bubbles, so stop its propagation in
//! the child), `onscrollend`, `MountedData` (`get_client_rect`, `scroll_to`, `set_focus`) and, from 0.8,
//! `onbeforeinput`. The bridge's observers are for what those cannot express: a `rootMargin` or
//! `threshold`, a `MutationObserver`, a non-element target, a listener option (`capture`, `passive`,
//! `once`) or a `window`/`document` listener.
//!
//! # How the page loads it, and the CSP consequence
//!
//! The transport evaluates the bundle through the eval queue, once per webview, behind a bounded
//! ready handshake (the reasons `asset!` plus `document::Script` was not adopted are in the transport
//! module). On the web, `document::eval` compiles its script with `new Function`, so an app served
//! under a Content-Security-Policy without `'unsafe-eval'` cannot use the bridge, `js_listener` or any
//! other eval-driven primitive (upstream dioxus-primitives has the same limit).
//!
//! # One ordered write channel per webview
//!
//! Fire-and-forget verbs ([`ElementExt::call`], [`ElementExt::set_var`], [`Transaction::fire`]) share one
//! long-lived eval per webview that runs what it receives in order; separate evals have no ordering on
//! desktop. A read ([`get`], [`commit`]) is its own request, so two things that must be ordered against a
//! read go in one [`Transaction`].
//!
//! # What the bridge can and cannot carry
//!
//! A message from the page reaches Rust asynchronously: a microtask later on web, an IPC round trip on
//! desktop and mobile. So the bridge carries work that may be late (subscriptions, reads, calls). It
//! cannot carry a decision made inside an event's dispatch or a read-then-write that must land in one
//! frame; those are [`Rule`]s that Rust publishes beforehand and the page's interpreter evaluates.
//!
//! # Listener ownership
//!
//! Every subscription lives in a [`Subscription`], whose `Drop` tells the page to remove everything the
//! subscription registered (listeners, observers, media queries, sampling loops, rules); the message goes
//! over the channel the subscription already has, never a new eval (DioxusLabs/dioxus#4064). A
//! registration on an element also ends on the page when the element leaves the document, without
//! waiting for Rust (there is no merged `onunmounted`). The hooks tie
//! that drop to the calling component, exactly as [`use_js_listeners`](crate::js_listener::use_js_listeners)
//! does; [`js_listener`](crate::js_listener) itself runs on the same scope object, so there is one
//! registry (`window.__dxJsListeners.live`) and one removal rule.
//!
//! # The slow-channel test mode
//!
//! Every spec runs on Chromium web, where the eval channel costs a microtask and hides each latency bug
//! of a webview. Setting `window.__dxBridgeDelayFrames = N` defers every message that passes a bridge
//! channel by N animation frames, in both directions, to model desktop (zero cost when unset). It
//! covers the reply of [`get`], every subscription message and every message a `js_listener` script
//! sends. `"never"` instead drops every message to Rust (the answer that never arrives; commands from
//! Rust still arrive), which is how a lane proves that a pending state still ends visible when Rust
//! never answers. It does NOT delay `MountedData` reads, which are synchronous on web: code that must behave on
//! desktop reads through [`get`] or follows its reads with [`commit`] so the mode models it.

mod session;
pub(crate) mod transport;

pub mod rules;

use dioxus::document::EvalError;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub use rules::Rule;
pub use session::{
    use_document_listener, use_element_listener, use_event_listener, use_intersection_observer,
    use_media_query, use_mutation_observer, use_observer, use_resize_observer, use_rule,
    use_sample, use_window_listener, use_window_size, Command, Intersection, ListenOptions,
    MediaChange, Mutation, ObserveOptions, ObserverKind, SampleField, Sampled, Session, Size,
    Subscription, Until, WindowSize,
};
pub use transport::{is_unsupported, BRIDGE_JS};

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
    /// `setPointerCapture(pointerId)`.
    SetPointerCapture(i32),
    /// `releasePointerCapture(pointerId)`.
    ReleasePointerCapture(i32),
    /// `focus({ preventScroll })`.
    Focus {
        /// Do not scroll the focused element into view.
        prevent_scroll: bool,
    },
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
            Method::SetPointerCapture(id) => ("setPointerCapture", vec![json!(id)]),
            Method::ReleasePointerCapture(id) => ("releasePointerCapture", vec![json!(id)]),
            Method::Focus { prevent_scroll } => {
                ("focus", vec![json!({ "preventScroll": prevent_scroll })])
            }
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
    },
    /// A [`get`].
    Get {
        /// The element.
        target: Target,
        /// The props, by wire name.
        props: Vec<String>,
    },
    /// A custom-property write ([`ElementExt::set_var`]).
    Set {
        /// The element.
        target: Target,
        /// The custom property, `--name`.
        name: String,
        /// The value; `None` removes the property.
        value: Option<String>,
    },
    /// [`commit`]: wait for a frame and a style flush before the next op.
    Commit,
}

/// The result of one [`Op`].
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum OpResult {
    /// The op ran; its value (null for a `call` with no result).
    Ok {
        /// The op's value.
        ok: Value,
    },
    /// The op threw.
    Err {
        /// The error's text.
        err: String,
    },
}

/// Several ops run in ONE JS turn, in order (up to the first [`commit`], which waits a frame): a
/// write and the read that must see it, or two calls that must not be separated by a render.
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
        });
        self
    }

    /// Adds a read of `props`.
    #[must_use]
    pub fn get(mut self, target: impl Into<Target>, props: impl IntoIterator<Item = Prop>) -> Self {
        self.ops.push(Op::Get {
            target: target.into(),
            props: props.into_iter().map(|p| p.wire()).collect(),
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
        });
        self
    }

    /// Adds a [`commit`]: the ops after it run after a frame and a forced style flush.
    #[must_use]
    pub fn commit(mut self) -> Self {
        self.ops.push(Op::Commit);
        self
    }

    /// The ops, in order.
    pub fn ops(&self) -> &[Op] {
        &self.ops
    }

    /// Runs the ops and waits for every result.
    pub async fn run(self) -> Result<Vec<OpResult>, BridgeError> {
        transport::request(&format!("__dxBridge.transaction({})", to_json(&self.ops))).await
    }

    /// Sends the ops on the webview's ordered write channel without waiting for a result: they run after
    /// every earlier [`fire`](Self::fire), [`call`](ElementExt::call) and
    /// [`set_var`](ElementExt::set_var) of this webview, in order.
    pub fn fire(self) {
        transport::pump(&self.ops);
    }
}

fn to_json(value: &impl Serialize) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "null".into())
}

/// The JS expression that calls `method` on `target`.
fn call_expression(target: &Target, method: &Method) -> String {
    let (name, args) = method.wire();
    format!(
        "__dxBridge.call({}, {}, {})",
        to_json(target),
        to_json(&name),
        to_json(&args)
    )
}

/// Reads `props` of `target` in one JS turn. The answer arrives asynchronously (a microtask on web,
/// an IPC round trip on desktop).
pub async fn get(
    target: impl Into<Target>,
    props: impl IntoIterator<Item = Prop>,
) -> Result<Props, BridgeError> {
    let props: Vec<String> = props.into_iter().map(|p| p.wire()).collect();
    transport::request(&format!(
        "__dxBridge.get({}, {})",
        to_json(&target.into()),
        to_json(&props)
    ))
    .await
}

/// Resolves after the page's next animation frame and a forced style flush: whatever Rust wrote before
/// is a style change of its own when this returns (the row-421 ordering, done from Rust).
pub async fn commit() -> Result<(), BridgeError> {
    transport::request::<Value>("__dxBridge.commit()")
        .await
        .map(|_| ())
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
        })
        .fire();
    }

    /// Calls `method` and answers with its result (null for a method that returns nothing).
    #[allow(
        async_fn_in_trait,
        reason = "the future is awaited in place by the caller; it is never spawned across threads"
    )]
    async fn call_value(&self, method: Method) -> Result<Value, BridgeError> {
        transport::request(&call_expression(&self.target(), &method)).await
    }

    /// Reads `props` in one JS turn.
    #[allow(
        async_fn_in_trait,
        reason = "the future is awaited in place by the caller; it is never spawned across threads"
    )]
    async fn get(&self, props: impl IntoIterator<Item = Prop>) -> Result<Props, BridgeError> {
        get(self.target(), props).await
    }

    /// Writes the CSS custom property `name` (`--name`) on the element, for an element whose style
    /// Rust does not own; `None` removes it.
    fn set_var(&self, name: &str, value: Option<&str>) {
        Transaction::new_op(Op::Set {
            target: self.target(),
            name: name.into(),
            value: value.map(String::from),
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

    /// `setPointerCapture(pointer_id)`.
    fn set_pointer_capture(&self, pointer_id: i32) {
        self.call(Method::SetPointerCapture(pointer_id));
    }

    /// `releasePointerCapture(pointer_id)`.
    fn release_pointer_capture(&self, pointer_id: i32) {
        self.call(Method::ReleasePointerCapture(pointer_id));
    }

    /// `focus({ preventScroll: true })`.
    fn focus_prevent_scroll(&self) {
        self.call(Method::Focus {
            prevent_scroll: true,
        });
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
            Method::SetPointerCapture(1),
            Method::ReleasePointerCapture(1),
            Method::Focus {
                prevent_scroll: true,
            },
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
            Method::Focus {
                prevent_scroll: true
            }
            .wire(),
            ("focus", vec![json!({"preventScroll": true})])
        );
        assert_eq!(
            Method::SetPointerCapture(7).wire(),
            ("setPointerCapture", vec![json!(7)])
        );
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
            .set_var("p", "--x", Some("1px".into()))
            .commit();
        assert_eq!(
            serde_json::to_value(tx.ops()).expect("serialises"),
            json!([
                {"op": "call", "target": {"kind": "id", "id": "p"}, "method": "showPopover", "args": []},
                {"op": "get", "target": {"kind": "id", "id": "p"}, "props": ["rect", "popoverOpen"]},
                {"op": "set", "target": {"kind": "id", "id": "p"}, "name": "--x", "value": "1px"},
                {"op": "commit"},
            ])
        );
    }

    #[test]
    fn op_results_are_ok_or_err() {
        let results: Vec<OpResult> =
            serde_json::from_value(json!([{"ok": null}, {"ok": {"a": 1}}, {"err": "boom"}]))
                .expect("results deserialize");
        assert_eq!(results[0], OpResult::Ok { ok: Value::Null });
        assert_eq!(results[2], OpResult::Err { err: "boom".into() });
    }

    #[test]
    fn a_call_is_an_expression_on_the_bridge() {
        assert_eq!(
            call_expression(&Target::id("a"), &Method::ShowPopover),
            r#"__dxBridge.call({"kind":"id","id":"a"}, "showPopover", [])"#
        );
    }

    /// The Rust `include_str!` embeds the committed bundle, which installs `window.__dxBridge`.
    #[test]
    fn the_bundle_installs_the_bridge() {
        assert!(BRIDGE_JS.contains("__dxBridge"));
        assert!(BRIDGE_JS.contains("__dxBridgeDelayFrames"));
        assert!(
            !BRIDGE_JS.contains("export{"),
            "a bundle that is eval'd must not be a module"
        );
    }
}
