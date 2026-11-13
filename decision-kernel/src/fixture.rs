//! The test fixture behaviour's decision: on its event it sets one attribute on the host and tells Rust, unless the
//! host carries `data-dx-fixture-block`; with `cancel` it also asks to cancel the event.

use crate::decision::Decision;
use crate::value::Value;
use crate::wire_input;

/// The attribute Rust mirrors onto the host to make the behaviour stand down.
pub const BLOCK_ATTR: &str = "data-dx-fixture-block";

wire_input! {
    /// The behaviour's parameters (the keys of the Rust `Behaviour::Fixture` the decision reads).
    pub struct Config {
        /// The event the behaviour acts on (the wire name).
        pub event: String = "event",
        /// The attribute to set.
        pub attr: String = "attr",
        /// Its value.
        pub value: String = "value",
        /// Whether the decision asks to cancel the event.
        pub cancel: bool = "cancel",
    }
}

wire_input! {
    /// The attribute reads taken before the event (the host's attributes, `None` when absent).
    pub struct Snapshot {
        /// The value of [`BLOCK_ATTR`] on the host.
        pub block: Option<String> = "data-dx-fixture-block",
    }
}

wire_input! {
    /// The event fields the decision reads.
    pub struct Event {
        /// The event's `type`.
        pub kind: String = "type",
    }
}

/// The decision. Pure: the same inputs give the same answer.
pub fn decide(config: &Config, _memory: &Value, snapshot: &Snapshot, _event: &Event) -> Decision {
    if snapshot.block.is_some() {
        return Decision::default();
    }
    Decision {
        prevent: config.cancel,
        attrs: vec![(config.attr.clone(), config.value.clone())],
        send: Some(Value::Obj(vec![
            ("behaviour".into(), Value::Str("fixture".into())),
            ("event".into(), Value::Str(config.event.clone())),
        ])),
        ..Decision::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wire::FromValue;

    /// The wire keys are the attribute and field names the page reads, in declaration order.
    #[test]
    fn the_keys_are_the_declared_wire_names() {
        assert_eq!(Config::KEYS, ["event", "attr", "value", "cancel"]);
        assert_eq!(Snapshot::KEYS, [BLOCK_ATTR]);
        assert_eq!(Event::KEYS, ["type"]);
    }

    #[test]
    fn a_missing_or_mistyped_key_is_unreadable() {
        let obj = |pairs: &[(&str, Value)]| {
            Value::Obj(pairs.iter().map(|(k, v)| ((*k).into(), v.clone())).collect())
        };
        assert!(Snapshot::from_value(&obj(&[])).is_none());
        assert!(Snapshot::from_value(&obj(&[(BLOCK_ATTR, Value::Bool(true))])).is_none());
        assert_eq!(
            Snapshot::from_value(&obj(&[(BLOCK_ATTR, Value::Null)])),
            Some(Snapshot { block: None })
        );
        assert!(Event::from_value(&obj(&[("kind", Value::Str("x".into()))])).is_none());
    }
}
