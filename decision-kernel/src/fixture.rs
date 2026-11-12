//! The test fixture behaviour's decision: on its event it sets one attribute on the host and tells Rust, unless the
//! host carries `data-dx-fixture-block`; with `cancel` it also asks to cancel the event.

use crate::value::Value;

/// The attribute Rust mirrors onto the host to make the behaviour stand down.
pub const BLOCK_ATTR: &str = "data-dx-fixture-block";

/// The behaviour's parameters.
#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    /// The event the behaviour acts on (the wire name).
    pub event: String,
    /// The attribute to set.
    pub attr: String,
    /// Its value.
    pub value: String,
    /// Whether the decision asks to cancel the event.
    pub cancel: bool,
}

/// The attribute reads taken before the event.
#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    /// The value of [`BLOCK_ATTR`] on the host, `None` when absent.
    pub block: Option<String>,
}

/// The event fields the decision reads.
#[derive(Clone, Debug, PartialEq)]
pub struct Event {
    /// The event's `type`.
    pub kind: String,
}

/// What the page applies: the same shape the TypeScript adapter reads.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Decision {
    /// Cancel the default action (honoured only for a behaviour declared non-passive).
    pub prevent: bool,
    /// Attributes to set on the host.
    pub attrs: Vec<(String, String)>,
    /// A message for Rust.
    pub send: Option<Value>,
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
    }
}

impl Config {
    /// Reads the behaviour object the page sends (`kind`, `event`, `attr`, `value`, `cancel`, ...).
    pub fn from_value(value: &Value) -> Option<Config> {
        Some(Config {
            event: value.get("event")?.as_str()?.to_owned(),
            attr: value.get("attr")?.as_str()?.to_owned(),
            value: value.get("value")?.as_str()?.to_owned(),
            cancel: value
                .get("cancel")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        })
    }
}

impl Snapshot {
    /// Reads the attribute reads the page sends: an object of attribute name to string or null.
    pub fn from_value(value: &Value) -> Option<Snapshot> {
        let block = match value.get(BLOCK_ATTR)? {
            Value::Str(s) => Some(s.clone()),
            _ => None,
        };
        Some(Snapshot { block })
    }
}

impl Event {
    /// Reads the event fields the page sends.
    pub fn from_value(value: &Value) -> Option<Event> {
        Some(Event {
            kind: value.get("type")?.as_str()?.to_owned(),
        })
    }
}

impl Decision {
    /// The value the page reads: an object holding only what the decision sets.
    pub fn to_value(&self) -> Value {
        let mut pairs = Vec::new();
        if self.prevent {
            pairs.push(("prevent".to_owned(), Value::Bool(true)));
        }
        if !self.attrs.is_empty() {
            let attrs = self
                .attrs
                .iter()
                .map(|(k, v)| (k.clone(), Value::Str(v.clone())))
                .collect();
            pairs.push(("attrs".to_owned(), Value::Obj(attrs)));
        }
        if let Some(send) = &self.send {
            pairs.push(("send".to_owned(), send.clone()));
        }
        Value::Obj(pairs)
    }
}
