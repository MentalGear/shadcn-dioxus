//! The one transport seam. Every byte between Rust and the page's bridge goes through this file, and
//! it is the only file of `primitives/src` that names `document::eval` (`scripts/check-bridge-behaviours.sh`).
//! What it runs is a closed [`Program`] with its arguments as data, never a script string a caller
//! composed: the session loop, the ordered channel, a behaviour module, and the quarantined
//! [`legacy`](super::legacy) body of `js_listener`. Verbs and hooks reach it as [`fire`] and [`ask`]
//! (ops on the ordered channel), [`ensure_module`] and a session's [`open`].
//!
//! # Timing contract per renderer
//!
//! What the bridge can promise depends on how the renderer runs an eval and applies DOM edits. The
//! first column is verified (the specs run on Chromium web); desktop and mobile are the documented
//! behaviour of Dioxus 0.7.9's webview renderer (read from its sources, not measured here).
//!
//! | | web (wasm in the page) | desktop / mobile (webview over IPC) |
//! |---|---|---|
//! | `document::eval` runs the script | synchronously, in the calling turn | after an IPC hop, never in the calling turn |
//! | a reply (`dioxus.send`) reaches Rust | a microtask later | an IPC hop later |
//! | DOM edits of a render apply | at the end of the render's turn | in a `requestAnimationFrame`, at most one render per frame |
//! | an eval issued from a handler runs | before that render's edits | before that render's edits (they wait for the frame) |
//! | `MountedData` reads | synchronous | an IPC round trip |
//! | a vdom event handler | synchronous | a blocking synchronous request per event, routed to a vdom that may be one render ahead of the DOM |
//! | the ack gate | none: effects run after the render's edits | `poll_vdom` does not run the vdom (so no effect) while the last render's edits are unacknowledged |
//! | Rust to page | a call | one `evaluate_script` per message, with the JSON inside the script text; the pump's FIFO is the webview's order of those calls |
//! | a bridge flush (an effect) | after the render | after the render's edits are applied and acknowledged, so a handler's write waits for the frame |
//! | an exception while applying edits (rAF) | the render is lost | the ack is never sent and the runtime stalls for good: only the page's own fail-opens (leases, the watchdog) recover |
//!
//! Consequences the verbs are built around:
//!
//! * A decision inside an event's dispatch cannot wait for Rust on any renderer: it is a named
//!   [`Behaviour`](super::Behaviour) that Rust published beforehand.
//! * Pointer capture and `focus` are not verbs: a call arrives after the event's dispatch, and capture or
//!   focus after that moment is too late for the gesture or the focus handoff. They are effects of a named
//!   behaviour (Dioxus has no pointer-capture API, and its `set_focus` is a webview query that runs after
//!   the handler); no behaviour does either yet.
//! * Only element operations share the pump. Each [`Session`](super::Session) is its own eval, as is each
//!   `js_listener` script and each module load, so there is no order between a session's commands and a
//!   pump message beyond the order of the `evaluate_script` calls.
//! * Show-then-measure is one [`Transaction`](super::Transaction) (see
//!   [`show_popover_measured`](super::show_popover_measured)): the rects come back from the same JS turn
//!   that showed the element, so no render or frame can fall between the show and the read.
//! * A frame-rate stream is `listen` / `observe` / `sample`, never a vdom handler.
//!
//! What lives here, so that it lives once:
//!
//! * **Loading.** The bridge bundle ([`BRIDGE_JS`]) is evaluated through the same eval queue as the
//!   scripts that use it, once per webview, so a verb never runs before the bridge exists. The
//!   alternative, `asset!` plus `document::Script` from a root provider (upstream's shape for its
//!   focus trap), was not adopted: it needs every consuming app to mount a provider, the `<script>` is
//!   inserted by a queued head effect while an eval runs immediately (DioxusLabs/dioxus#5919: eval
//!   before a same-render script on desktop), and the committed bundle is reached by name from the
//!   page, where a bundler pass may strip a function nothing references at top level (#5183). Cost of
//!   the chosen route: the page needs `new Function` (see the CSP note in [`super`]).
//! * **The ready handshake.** Every script starts with a bounded wait (about 2 s) for
//!   `window.__dxBridge`, then fails loudly instead of calling a missing function. "Installed" is
//!   remembered in the root context of the VirtualDom, so it is per webview (a second desktop window
//!   installs its own copy, a reload starts a new VirtualDom), never a process-wide flag.
//! * **The slow-channel mode.** Every `dioxus` object a body sees is already the bridge's `channel`
//!   wrapper, so `window.__dxBridgeDelayFrames` covers every message, in both directions, without a
//!   caller doing anything.
//! * **One FIFO channel for element operations.** The pump is ONE long-lived eval per webview. Every element write and read
//!   ([`fire`], [`ask`]) is a message `{ id, ops }` on it, run in the order received and answered
//!   `{ id, results }` when it asked, because ordering inside one channel is the only ordering Dioxus
//!   gives; separate evals have none on desktop, so a read is ordered after the writes before it.
//! * **Flushed after the render that issued them.** A verb called from a handler, an effect or a task only
//!   queues its ops; one effect on the root scope (`queue_effect`, which runs after the next render's
//!   mutations are written) sends the whole outbox. The element an op names is then in the DOM on the web, and
//!   the first verb of a webview creates the pump in that effect, never inside a drop handler.
//! * **Teardown never starts an eval.** Dropping a subscription sends one message on the channel it
//!   already has (DioxusLabs/dioxus#4064: a new eval inside `use_drop` failed an assertion). A verb in a
//!   drop handler only queues ops, and the pump is created by the flush effect, never in the handler.
//! * **"Unsupported".** A renderer without JavaScript (Blitz) answers `EvalError::Unsupported`;
//!   [`is_unsupported`] turns true the first time the platform says so, requests fail with
//!   [`BridgeError::Unsupported`] and subscriptions end quietly, so a
//!   caller degrades instead of panicking.
//! * **The seam for another backend.** A future in-process backend (a web-sys/wasm-bindgen one, if
//!   DioxusLabs/dioxus#5194 lands) replaces [`open`], [`send`] and the channel behind [`fire`] / [`ask`];
//!   nothing above them names eval.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll, Waker};

use crate::dioxus_core::{
    provide_root_context, queue_effect, try_consume_context, Runtime, ScopeId,
};
use dioxus::document::{self, Eval, EvalError};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::BridgeError;

/// The bundled bridge script, `primitives/src/js/bridge.js` (built from `src/ts/bridge.ts` by
/// `scripts/build-js-bundle.sh`, checked by `scripts/check-js-bundle.sh`).
pub const BRIDGE_JS: &str = include_str!("../js/bridge.js");

/// The bounded wait of the handshake, then the bridge as `__dxB`.
const READY: &str = "let __dxB = window.__dxBridge;\n\
    for (let i = 0; !__dxB && i < 125; i++) { await new Promise((r) => setTimeout(r, 16)); __dxB = window.__dxBridge; }\n\
    if (!__dxB) throw new Error('the dx bridge did not load');\n";

static UNSUPPORTED: AtomicBool = AtomicBool::new(false);

/// True once the platform has said it cannot run JavaScript (Blitz): every bridge verb is then a no-op
/// or an error, and a caller may choose another path.
pub fn is_unsupported() -> bool {
    UNSUPPORTED.load(Ordering::Relaxed)
}

/// Records what an eval error says about the platform.
pub(in crate::bridge) fn note_error(error: &EvalError) {
    if matches!(error, EvalError::Unsupported) {
        UNSUPPORTED.store(true, Ordering::Relaxed);
    }
}

/// What the transport can run in the page: a closed list. Each one is a fixed body that reaches the
/// bridge as `__dxB` and its channel as `dioxus`; what varies is data sent as messages, never script
/// text.
pub(in crate::bridge) enum Program<'a> {
    /// A [`Session`](super::Session)'s loop: setup commands in, the session's messages out.
    Serve,
    /// The webview's ordered channel (see [`fire`] and [`ask`]).
    Pump,
    /// The body of a `js_listener` script, quarantined in [`legacy`](super::legacy) and gone with that
    /// module.
    Legacy(&'a str),
}

impl Program<'_> {
    fn script(&self) -> String {
        match self {
            Program::Serve => format!("{READY}await __dxB.serve(__dxB.channel(dioxus));"),
            Program::Pump => format!("{READY}await __dxB.pump(__dxB.channel(dioxus));"),
            Program::Legacy(body) => {
                format!("{READY}await (async (dioxus) => {{\n{body}\n}})(__dxB.channel(dioxus));")
            }
        }
    }
}

/// Marks a webview (a VirtualDom) as having the bridge evaluated.
#[derive(Clone, Copy)]
struct Installed;

/// The behaviour modules evaluated in this webview, by kind.
#[derive(Clone, Default)]
struct Loaded(Rc<RefCell<HashSet<&'static str>>>);

fn in_runtime() -> bool {
    Runtime::try_current().is_some()
}

/// Evaluates the bridge once in this webview. Only meaningful inside the Dioxus runtime.
fn ensure_installed() {
    if !in_runtime() || try_consume_context::<Installed>().is_some() {
        return;
    }
    provide_root_context(Installed);
    let _ = document::eval(BRIDGE_JS);
}

/// Evaluates the bundled module `js` of the behaviour `kind` once in this webview, after the bridge it
/// registers into (the eval queue keeps the order) and behind the same bounded ready handshake. Only
/// meaningful inside the Dioxus runtime.
pub(in crate::bridge) fn ensure_module(kind: &'static str, js: &'static str) {
    if !in_runtime() {
        return;
    }
    ensure_installed();
    let loaded = try_consume_context::<Loaded>().unwrap_or_else(|| {
        let loaded = Loaded::default();
        provide_root_context(loaded.clone());
        loaded
    });
    if loaded.0.borrow_mut().insert(kind) {
        let _ = document::eval(&format!("{READY}{js}"));
    }
}

/// Starts `program` in the page after the ready handshake. The returned [`Eval`] is its channel: send it
/// parameters, receive what it `dioxus.send`s. Only meaningful inside the Dioxus runtime.
pub(in crate::bridge) fn open(program: Program<'_>) -> Eval {
    ensure_installed();
    document::eval(&program.script())
}

/// Sends `message` on `eval`, noting what a failure says about the platform.
pub(in crate::bridge) fn send(eval: &Eval, message: impl Serialize) {
    if let Err(error) = eval.send(message) {
        note_error(&error);
    }
}

// ------------------------------------------------------------------ the ordered channel

/// One message of the ordered channel, before it is sent.
struct Outgoing {
    /// Set when the caller waits for the results.
    id: Option<u64>,
    ops: Vec<Value>,
}

/// The answer to an asking message.
#[derive(Deserialize)]
struct Answer {
    id: u64,
    results: Value,
}

/// A future for one answer, filled by the pump's reader.
#[derive(Default)]
struct Slot {
    value: Option<Result<Value, BridgeError>>,
    waker: Option<Waker>,
}

struct Pending(Rc<RefCell<Slot>>);

impl Future for Pending {
    type Output = Result<Value, BridgeError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut slot = self.0.borrow_mut();
        match slot.value.take() {
            Some(value) => Poll::Ready(value),
            None => {
                slot.waker = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}

fn settle(slot: &Rc<RefCell<Slot>>, value: Result<Value, BridgeError>) {
    let mut slot = slot.borrow_mut();
    slot.value = Some(value);
    if let Some(waker) = slot.waker.take() {
        waker.wake();
    }
}

#[derive(Default)]
struct Channel {
    /// The pump's eval, once the first flush has created it.
    eval: Option<Eval>,
    outbox: Vec<Outgoing>,
    /// A flush effect is queued.
    scheduled: bool,
    next_id: u64,
    waiting: HashMap<u64, Rc<RefCell<Slot>>>,
    /// The pump ended (the platform has no JavaScript, or the page is gone): later asks fail at once.
    closed: Option<bool>,
}

/// This webview's channel, in the root context.
#[derive(Clone, Default)]
struct Hub(Rc<RefCell<Channel>>);

fn hub() -> Hub {
    try_consume_context::<Hub>().unwrap_or_else(|| {
        let hub = Hub::default();
        provide_root_context(hub.clone());
        hub
    })
}

/// Queues `f` to run after the next render, owned by the root scope: an effect queued from a component
/// is dropped with that component, which would strand the outbox.
fn after_render(f: impl FnOnce() + 'static) {
    Runtime::current().in_scope(ScopeId::ROOT, || queue_effect(f));
}

fn enqueue(ops: Vec<Value>, asks: bool) -> Option<Pending> {
    let hub = hub();
    let mut channel = hub.0.borrow_mut();
    let mut pending = None;
    let mut id = None;
    if asks {
        if let Some(unsupported) = channel.closed {
            let slot = Rc::new(RefCell::new(Slot::default()));
            settle(&slot, Err(closed_error(unsupported)));
            return Some(Pending(slot));
        }
        channel.next_id += 1;
        let next = channel.next_id;
        id = Some(next);
        let slot = Rc::new(RefCell::new(Slot::default()));
        channel.waiting.insert(next, slot.clone());
        pending = Some(Pending(slot));
    } else if channel.closed.is_some() {
        return None;
    }
    channel.outbox.push(Outgoing { id, ops });
    if !channel.scheduled {
        channel.scheduled = true;
        let again = hub.clone();
        drop(channel);
        after_render(move || flush(&again));
    }
    pending
}

fn closed_error(unsupported: bool) -> BridgeError {
    if unsupported {
        BridgeError::Unsupported
    } else {
        BridgeError::Js("the bridge channel is closed".into())
    }
}

/// Fire-and-forget messages that sit next to each other travel as one; an asking message keeps its own
/// place, so the order of every op is the order it was queued in.
fn merge(batch: Vec<Outgoing>) -> Vec<Outgoing> {
    let mut merged: Vec<Outgoing> = Vec::with_capacity(batch.len());
    for message in batch {
        match merged.last_mut() {
            Some(last) if last.id.is_none() && message.id.is_none() => last.ops.extend(message.ops),
            _ => merged.push(message),
        }
    }
    merged
}

/// Sends the outbox, creating the pump on first use. Runs as an effect, so inside the runtime.
fn flush(hub: &Hub) {
    let (batch, eval) = {
        let mut channel = hub.0.borrow_mut();
        channel.scheduled = false;
        (std::mem::take(&mut channel.outbox), channel.eval)
    };
    if batch.is_empty() {
        return;
    }
    let eval = eval.unwrap_or_else(|| {
        let eval = open(Program::Pump);
        hub.0.borrow_mut().eval = Some(eval);
        read_answers(hub.clone(), eval);
        eval
    });
    for message in merge(batch) {
        send(&eval, json!({ "id": message.id, "ops": message.ops }));
    }
}

/// The task that hands every answer of the pump to the future waiting for it. It belongs to the root
/// scope, because the pump outlives the component whose verb created it.
fn read_answers(hub: Hub, mut eval: Eval) {
    // interval-ok: the reader of the webview's one pump lives as long as the pump and ends when its eval does
    crate::dioxus_core::spawn_forever(async move {
        loop {
            match eval.recv::<Answer>().await {
                Ok(answer) => {
                    let slot = hub.0.borrow_mut().waiting.remove(&answer.id);
                    if let Some(slot) = slot {
                        settle(&slot, Ok(answer.results));
                    }
                }
                Err(EvalError::Serialization(_)) => {}
                Err(error) => {
                    note_error(&error);
                    let unsupported = matches!(error, EvalError::Unsupported);
                    let waiting: Vec<_> = {
                        let mut channel = hub.0.borrow_mut();
                        channel.closed = Some(unsupported);
                        channel.waiting.drain().map(|(_, slot)| slot).collect()
                    };
                    for slot in waiting {
                        settle(&slot, Err(closed_error(unsupported)));
                    }
                    break;
                }
            }
        }
    });
}

/// Queues `ops` (bridge ops, as JSON) on the webview's ordered channel without waiting for a result.
/// They run after every earlier [`fire`] and [`ask`] of this webview, in order, after the next render.
/// Nothing is queued outside the Dioxus runtime.
pub(in crate::bridge) fn fire<T: Serialize>(ops: &[T]) {
    if in_runtime() {
        enqueue(values(ops), false);
    }
}

/// Queues `ops` like [`fire`] and answers with their results, one per op. A message that never gets an
/// answer (the page is gone) fails with an error instead of waiting for ever.
pub(in crate::bridge) async fn ask<T: Serialize, R: DeserializeOwned>(
    ops: &[T],
) -> Result<R, BridgeError> {
    if !in_runtime() {
        return Err(BridgeError::Unsupported);
    }
    let Some(pending) = enqueue(values(ops), true) else {
        return Err(BridgeError::Unsupported);
    };
    let results = pending.await?;
    serde_json::from_value(results).map_err(|error| BridgeError::Js(error.to_string()))
}

fn values<T: Serialize>(ops: &[T]) -> Vec<Value> {
    ops.iter()
        .map(|op| serde_json::to_value(op).unwrap_or(Value::Null))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus::prelude::*;
    use std::cell::Cell;

    fn out(id: Option<u64>, ops: &[i32]) -> Outgoing {
        Outgoing {
            id,
            ops: ops.iter().map(|n| json!(n)).collect(),
        }
    }

    #[test]
    fn neighbouring_fire_and_forget_messages_merge_and_an_ask_keeps_its_place() {
        let merged = merge(vec![
            out(None, &[1]),
            out(None, &[2, 3]),
            out(Some(7), &[4]),
            out(None, &[5]),
            out(Some(8), &[6]),
            out(Some(9), &[7]),
        ]);
        let shape: Vec<(Option<u64>, Vec<i64>)> = merged
            .iter()
            .map(|m| (m.id, m.ops.iter().filter_map(Value::as_i64).collect()))
            .collect();
        assert_eq!(
            shape,
            vec![
                (None, vec![1, 2, 3]),
                (Some(7), vec![4]),
                (None, vec![5]),
                (Some(8), vec![6]),
                (Some(9), vec![7]),
            ]
        );
    }

    #[test]
    fn a_pending_answer_resolves_when_settled_and_wakes_its_task() {
        let slot = Rc::new(RefCell::new(Slot::default()));
        let mut pending = Pending(slot.clone());
        let waker = std::task::Waker::noop();
        let mut cx = Context::from_waker(waker);
        assert!(Pin::new(&mut pending).poll(&mut cx).is_pending());
        settle(&slot, Ok(json!([1])));
        match Pin::new(&mut pending).poll(&mut cx) {
            Poll::Ready(Ok(value)) => assert_eq!(value, json!([1])),
            _ => panic!("a settled answer is ready"),
        }
    }

    /// What an `after_render` effect needs to prove: it runs after the work that queued it, whoever queued
    /// it, even when no render follows, and even when the component that queued it is gone by then.
    #[derive(Clone, Default)]
    struct Probe {
        ran: Rc<Cell<u32>>,
        show_child: Rc<Cell<bool>>,
        from_task: Rc<Cell<bool>>,
    }

    #[component]
    fn Child() -> Element {
        let probe = use_context::<Probe>();
        use_hook(|| {
            let ran = probe.ran.clone();
            after_render(move || ran.set(ran.get() + 1));
        });
        let ran = probe.ran.clone();
        use_drop(move || after_render(move || ran.set(ran.get() + 100)));
        rsx! { "child" }
    }

    #[component]
    fn Host() -> Element {
        let probe = use_context::<Probe>();
        let from_task = probe.from_task.clone();
        use_hook(|| {
            let ran = probe.ran.clone();
            spawn(async move {
                if from_task.get() {
                    after_render(move || ran.set(ran.get() + 10));
                }
            });
        });
        rsx! { if probe.show_child.get() { Child {} } }
    }

    #[test]
    fn after_render_runs_after_a_render_a_task_and_the_unmount_of_its_caller() {
        let probe = Probe::default();
        probe.show_child.set(true);
        let mut dom = VirtualDom::new(Host).with_root_context(probe.clone());
        dom.rebuild_in_place();
        dom.process_events();
        assert_eq!(probe.ran.get(), 1, "queued during a render");

        // the component that queues is unmounted in the same pass, before its effect would run
        let probe = Probe::default();
        probe.show_child.set(true);
        let mut dom = VirtualDom::new(Host).with_root_context(probe.clone());
        dom.rebuild_in_place();
        probe.show_child.set(false);
        dom.mark_dirty(ScopeId::APP);
        dom.render_immediate_to_vec();
        dom.process_events();
        assert_eq!(
            probe.ran.get(),
            101,
            "owned by the root scope: it survives its caller, and a drop handler may queue one"
        );

        // from a task, with no render following
        let probe = Probe::default();
        probe.from_task.set(true);
        let mut dom = VirtualDom::new(Host).with_root_context(probe.clone());
        dom.rebuild_in_place();
        dom.process_events();
        assert_eq!(probe.ran.get(), 10, "a task's effect runs without a render");
    }
}
