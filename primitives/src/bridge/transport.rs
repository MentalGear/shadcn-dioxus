//! The one transport seam. Every byte between Rust and the page's bridge goes through this file, and
//! it is the only file of `primitives/src` that names `document::eval` (`scripts/check-bridge-behaviours.sh`):
//! hooks and verbs ask for a script body ([`open`]), a request ([`request`]), an ordered write
//! ([`pump`]) or a behaviour module ([`ensure_module`]) and never touch the platform.
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
//! | [`commit`](super::commit) | a frame plus a forced style flush | no-op for ordering: edits and the eval share the frame, so use [`Transaction`](super::Transaction) |
//! | `MountedData` reads | synchronous | an IPC round trip |
//! | a vdom event handler | synchronous | a blocking synchronous request per event |
//!
//! Consequences the verbs are built around:
//!
//! * A decision inside an event's dispatch cannot wait for Rust on any renderer: it is a named
//!   [`Behaviour`](super::Behaviour) that Rust published beforehand.
//! * `setPointerCapture` and `focus` through [`call`](super::ElementExt::call) are NOT usable at
//!   `pointerdown`: the call arrives after the event's dispatch, and capture or focus after that moment
//!   is too late for the gesture or the focus handoff. They are behaviour effects or rsx handlers.
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
//! * **The ordered write channel.** [`pump`] is ONE long-lived eval per webview that takes arrays of
//!   ops and runs them in the order received, because ordering inside one channel is the only
//!   ordering Dioxus gives; separate evals have none on desktop.
//! * **Teardown never starts an eval.** Dropping a subscription sends one message on the channel it
//!   already has (DioxusLabs/dioxus#4064: a new eval inside `use_drop` failed an assertion). The pump is
//!   created at the first bridge use of a webview, so that first use must not be inside a drop handler.
//! * **"Unsupported".** A renderer without JavaScript (Blitz) answers `EvalError::Unsupported`;
//!   [`is_unsupported`] turns true the first time the platform says so, requests fail with
//!   [`BridgeError::Unsupported`] and subscriptions end quietly, so a
//!   caller degrades instead of panicking.
//! * **The seam for another backend.** A future in-process backend (a web-sys/wasm-bindgen one, if
//!   DioxusLabs/dioxus#5194 lands) replaces the three functions below; nothing above them names eval.

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::dioxus_core::{provide_root_context, try_consume_context};
use dioxus::document::{self, Eval, EvalError};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

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
pub(crate) fn note_error(error: &EvalError) {
    if matches!(error, EvalError::Unsupported) {
        UNSUPPORTED.store(true, Ordering::Relaxed);
    }
}

/// Marks a webview (a VirtualDom) as having the bridge evaluated.
#[derive(Clone, Copy)]
struct Installed;

/// The behaviour modules evaluated in this webview, by kind.
#[derive(Clone, Default)]
struct Loaded(Rc<RefCell<HashSet<&'static str>>>);

/// The webview's ordered write channel.
#[derive(Clone, Copy)]
struct Pump(Eval);

fn in_runtime() -> bool {
    crate::dioxus_core::Runtime::try_current().is_some()
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
pub(crate) fn ensure_module(kind: &'static str, js: &'static str) {
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

/// Runs `body` as an async function body in the page, after the ready handshake. Inside it `dioxus` is
/// the bridge's channel wrapper (`send`, `recv`, with the slow-channel mode applied), `__dxB` is
/// `window.__dxBridge`, and an early `return` or an `await` is allowed. The returned [`Eval`] is the
/// channel: send it parameters, receive what the body `dioxus.send`s.
pub(crate) fn open(body: &str) -> Eval {
    ensure_installed();
    document::eval(&format!(
        "{READY}await (async (dioxus) => {{\n{body}\n}})(__dxB.channel(dioxus));"
    ))
}

/// Sends `message` on `eval`, noting what a failure says about the platform.
pub(crate) fn send(eval: &Eval, message: impl Serialize) {
    if let Err(error) = eval.send(message) {
        note_error(&error);
    }
}

/// Evaluates the JS expression `expression` and answers with its value (a promise is awaited), through
/// the bridge channel, so the slow-channel mode delays the answer.
pub(crate) async fn request<T: DeserializeOwned>(expression: &str) -> Result<T, BridgeError> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Reply<T> {
        Ok { ok: T },
        Err { err: String },
    }
    let mut eval = open(&format!(
        "try {{ dioxus.send({{ ok: (await ({expression})) ?? null }}); }} \
         catch (e) {{ dioxus.send({{ err: String(e) }}); }}"
    ));
    match eval.recv::<Reply<T>>().await {
        Ok(Reply::Ok { ok }) => Ok(ok),
        Ok(Reply::Err { err }) => Err(BridgeError::Js(err)),
        Err(EvalError::Unsupported) => {
            note_error(&EvalError::Unsupported);
            Err(BridgeError::Unsupported)
        }
        Err(error) => Err(BridgeError::Eval(error)),
    }
}

/// Sends `ops` (an array of bridge ops) on the webview's ordered write channel. Nothing is sent outside
/// the Dioxus runtime.
pub(crate) fn pump(ops: &impl Serialize) {
    if !in_runtime() {
        return;
    }
    let eval = match try_consume_context::<Pump>() {
        Some(pump) => pump.0,
        None => {
            let eval = open("await __dxB.pump(dioxus);");
            provide_root_context(Pump(eval));
            eval
        }
    };
    send(&eval, ops);
}
