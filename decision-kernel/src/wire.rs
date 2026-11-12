//! One request in, one decision out, as bytes: the function the page calls and `cargo test` calls alike.

use crate::fixture;
use crate::value::Value;

/// Decodes the request, runs the fixture's `decide` and encodes the decision. The request is an array of
/// `[config, memory, snapshot, event]`. Malformed input is `None` (the page then stands down).
pub fn decide_fixture(request: &[u8]) -> Option<Vec<u8>> {
    let Value::Arr(parts) = Value::decode(request)? else {
        return None;
    };
    let [config, memory, snapshot, event] = parts.as_slice() else {
        return None;
    };
    let decision = fixture::decide(
        &fixture::Config::from_value(config)?,
        memory,
        &fixture::Snapshot::from_value(snapshot)?,
        &fixture::Event::from_value(event)?,
    );
    let mut out = Vec::with_capacity(96);
    decision.to_value().encode(&mut out);
    Some(out)
}
