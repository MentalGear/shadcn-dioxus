//! What every decision answers: the shape the page's adapter applies.

use crate::value::Value;

/// A decision. Every field is what the decision sets; the default is "do nothing".
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Decision {
    /// Cancel the default action (honoured only for a behaviour declared non-passive).
    pub prevent: bool,
    /// Stop propagation (honoured only for a behaviour declared non-passive, at an element in the capture phase).
    pub stop: bool,
    /// Attributes to set on the host (`data-*` and `aria-*` only: the page refuses any other).
    pub attrs: Vec<(String, String)>,
    /// A message for Rust.
    pub send: Option<Value>,
    /// Per-install state merged into the `memory` the next decision of this install receives.
    pub remember: Option<Value>,
}

impl Decision {
    /// The value the page reads: an object holding only what the decision sets.
    pub fn to_value(&self) -> Value {
        let mut pairs = Vec::new();
        if self.prevent {
            pairs.push(("prevent".to_owned(), Value::Bool(true)));
        }
        if self.stop {
            pairs.push(("stop".to_owned(), Value::Bool(true)));
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
        if let Some(remember) = &self.remember {
            pairs.push(("remember".to_owned(), remember.clone()));
        }
        Value::Obj(pairs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_decision_sets_nothing() {
        assert_eq!(Decision::default().to_value(), Value::Obj(vec![]));
    }

    #[test]
    fn a_decision_holds_only_what_it_sets() {
        let decision = Decision {
            prevent: true,
            stop: true,
            attrs: vec![("data-x".into(), "y".into())],
            send: Some(Value::Null),
            remember: Some(Value::Obj(vec![("n".into(), Value::Num(1.0))])),
        };
        let value = decision.to_value();
        let Value::Obj(pairs) = &value else {
            panic!("an object")
        };
        let keys: Vec<&str> = pairs.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(keys, ["prevent", "stop", "attrs", "send", "remember"]);
        assert_eq!(
            value.get("attrs").and_then(|a| a.get("data-x")),
            Some(&Value::Str("y".into()))
        );
    }
}
