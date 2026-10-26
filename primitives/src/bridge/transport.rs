//! The one transport seam. Every byte between Rust and the page's bridge goes through this file, and
//! it is the only file of `bridge/` that names `document::eval` (`scripts/check-bridge-rules.sh`):
//! hooks and verbs ask for a script body ([`open`]), a request ([`request`]) or an ordered write
//! ([`pump`]) and never touch the platform.
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
//!   [`BridgeError::Unsupported`](super::BridgeError::Unsupported) and subscriptions end quietly, so a
//!   caller degrades instead of panicking.
//! * **The seam for another backend.** A future in-process backend (a web-sys/wasm-bindgen one, if
//!   DioxusLabs/dioxus#5194 lands) replaces the three functions below; nothing above them names eval.

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
