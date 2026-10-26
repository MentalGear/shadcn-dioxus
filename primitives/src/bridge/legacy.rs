//! The quarantine for `js_listener`'s bespoke scripts: the only place a caller-composed script body
//! reaches the transport. [`JsListeners`](crate::js_listener::JsListeners) is the one user, and it goes
//! away as the remaining bespoke blocks become named behaviours, bridge streams or CSS (row 434);
//! this module goes with it. `scripts/check-bridge-behaviours.sh` fails on any other user of it and on
//! any other construction of `Program::Legacy` (a private item of the transport).

use dioxus::document::{Eval, EvalError};
use serde::Serialize;

use super::transport::{self, Program};

/// Runs `body` as an async function body after the ready handshake; inside it `dioxus` is the bridge's
/// channel wrapper (so the slow-channel mode covers its messages) and `__dxB` is the bridge.
pub(crate) fn open(body: &str) -> Eval {
    transport::open(Program::Legacy(body))
}

/// Sends `message` on `eval`, noting what a failure says about the platform.
pub(crate) fn send(eval: &Eval, message: impl Serialize) {
    transport::send(eval, message);
}

/// Records what an eval error says about the platform.
pub(crate) fn note_error(error: &EvalError) {
    transport::note_error(error);
}
