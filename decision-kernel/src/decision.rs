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
    /// Writes to native slots the page owns, one `(slot, argument)` per write: the page hands each to the slot's owner
    /// (`focus`: focus the host without scrolling, the argument is `null`; `value`: replace a range of the host's text,
    /// the argument is a [`ValueWrite`]'s value). A slot the page has no owner for is
    /// refused there. Every native-slot write is one entry of this list, so a new slot adds no new field.
    pub slots: Vec<(String, Value)>,
    /// A message for Rust.
    pub send: Option<Value>,
    /// Per-install state merged into the `memory` the next decision of this install receives.
    pub remember: Option<Value>,
}

/// The write the `value` slot owner applies to a text control: the characters `from..to` (UTF-16 offsets, the unit of
/// the DOM's selection and `slice`) of its value become `text`, the caret goes to `caret`, and `input` says whether the
/// owner then dispatches a bubbling `input` event so every listener of the control hears the new value.
#[derive(Clone, Debug, PartialEq)]
pub struct ValueWrite {
    /// The start of the replaced range.
    pub from: usize,
    /// The end of the replaced range.
    pub to: usize,
    /// What replaces it.
    pub text: String,
    /// The caret position after the write.
    pub caret: usize,
    /// Whether the owner dispatches an `input` event after the write.
    pub input: bool,
}

impl ValueWrite {
    /// The `slots` argument the page's owner reads.
    pub fn to_value(&self) -> Value {
        Value::Obj(vec![
            ("from".to_owned(), Value::Num(self.from as f64)),
            ("to".to_owned(), Value::Num(self.to as f64)),
            ("text".to_owned(), Value::Str(self.text.clone())),
            ("caret".to_owned(), Value::Num(self.caret as f64)),
            ("input".to_owned(), Value::Bool(self.input)),
        ])
    }
}

impl Decision {
    /// Adds a write of the `value` slot.
    pub fn write_value(mut self, write: &ValueWrite) -> Self {
        self.slots.push(("value".to_owned(), write.to_value()));
        self
    }

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
        if !self.slots.is_empty() {
            pairs.push(("slots".to_owned(), Value::Obj(self.slots.clone())));
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
    fn a_value_write_is_one_entry_of_the_slots_list() {
        let write = ValueWrite {
            from: 1,
            to: 2,
            text: "7".into(),
            caret: 2,
            input: true,
        };
        let value = Decision::default().write_value(&write).to_value();
        let slot = value.get("slots").and_then(|s| s.get("value")).unwrap();
        assert_eq!(slot.get("text"), Some(&Value::Str("7".into())));
        assert_eq!(slot.get("from"), Some(&Value::Num(1.0)));
        assert_eq!(slot.get("input"), Some(&Value::Bool(true)));
    }

    #[test]
    fn a_decision_holds_only_what_it_sets() {
        let decision = Decision {
            prevent: true,
            stop: true,
            attrs: vec![("data-x".into(), "y".into())],
            slots: vec![("focus".into(), Value::Null)],
            send: Some(Value::Null),
            remember: Some(Value::Obj(vec![("n".into(), Value::Num(1.0))])),
        };
        let value = decision.to_value();
        let Value::Obj(pairs) = &value else {
            panic!("an object")
        };
        let keys: Vec<&str> = pairs.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(
            keys,
            ["prevent", "stop", "attrs", "slots", "send", "remember"]
        );
        assert_eq!(
            value.get("attrs").and_then(|a| a.get("data-x")),
            Some(&Value::Str("y".into()))
        );
        assert_eq!(
            value.get("slots").and_then(|a| a.get("focus")),
            Some(&Value::Null)
        );
    }
}
