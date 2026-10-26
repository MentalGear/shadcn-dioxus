//! The closed set of rules Rust can hand the page's one interpreter (row 434 step 3, "rules as data,
//! one interpreter").
//!
//! A decision that must happen INSIDE an event's dispatch (cancel this wheel, ignore this key) or
//! within one frame cannot wait for the bridge: a bridge message reaches Rust after the event's
//! default action is committed and after the frame is painted. So Rust publishes the rule and the
//! state it reads *beforehand*, and `primitives/src/ts/rules/index.ts` decides locally at event or
//! frame time. A rule is a [`Rule`] value, serialised as JSON onto an element in one of two ways:
//!
//! * [`Session::rule`](super::Session::rule) / [`use_rule`](super::use_rule): installed once, for
//!   static configuration, and removed with the owning scope;
//! * [`Rule::attribute`]: a `data-dx-rule-<kind>` attribute rendered with the element, for state that
//!   changes with renders; [`Session::sync_rules`](super::Session::sync_rules) installs, replaces and
//!   removes what the attributes now say.
//!
//! The enum is closed on purpose. Every variant is one named rule with fixed parameters, no variables,
//! loops or expressions; a decision that needs those stays a typed JS module with a stated exit
//! (backlog row 434). `scripts/check-bridge-rules.sh` keeps this enum and the interpreter's `switch`
//! (no default arm) one to one, requires a `bun test` case for every kind, and the four marked regions
//! below are where the later lanes put their families. A new kind needs a backlog row.

use serde::{Deserialize, Serialize};

/// One rule for the page's interpreter. Serialised as `{ "kind": "<kebab-case variant>", ...fields }`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Rule {
    // region:B1 begin (event gates: a decision inside the listener call)
    // region:B1 end
    // region:B2 begin (drag and swipe recogniser)
    // region:B2 end
    // region:B3 begin (measurement publisher)
    // region:B3 end
    // region:B5 begin (reveal and settle)
    // region:B5 end
    // region:FIXTURE begin (test fixtures only, no component uses these)
    /// On `event` at the host element, sets `attr` to `value` and sends Rust `{ rule: "fixture",
    /// fired: n }`. Exists to prove install, replace, remove and scope ownership end to end.
    Fixture {
        /// The DOM event type to react to.
        event: String,
        /// The attribute to write on the host element.
        attr: String,
        /// The value written.
        value: String,
    },
    // region:FIXTURE end
}

/// The prefix of an attribute that carries a rule (`data-dx-rule-<kind>`).
pub const RULE_ATTR_PREFIX: &str = "data-dx-rule-";

impl Rule {
    /// The rule's wire name (`"fixture"`): the serde tag, so it can never drift from the JSON.
    pub fn kind(&self) -> String {
        match serde_json::to_value(self) {
            Ok(serde_json::Value::Object(mut map)) => match map.remove("kind") {
                Some(serde_json::Value::String(kind)) => kind,
                _ => String::new(),
            },
            _ => String::new(),
        }
    }

    /// The `(name, value)` of the attribute that mirrors this rule into the DOM:
    /// `data-dx-rule-<kind>` and the rule's JSON. One attribute per kind per element.
    pub fn attribute(&self) -> (String, String) {
        (
            format!("{RULE_ATTR_PREFIX}{}", self.kind()),
            serde_json::to_string(self).unwrap_or_default(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Rule {
        Rule::Fixture {
            event: "click".into(),
            attr: "data-hit".into(),
            value: "yes".into(),
        }
    }

    /// The JSON shape the interpreter reads: an internal `kind` tag, kebab-case, fields beside it.
    #[test]
    fn a_rule_is_a_tagged_object() {
        let json = serde_json::to_value(fixture()).expect("serialises");
        assert_eq!(
            json,
            serde_json::json!({"kind": "fixture", "event": "click", "attr": "data-hit", "value": "yes"})
        );
        let back: Rule = serde_json::from_value(json).expect("round-trips");
        assert_eq!(back, fixture());
    }

    #[test]
    fn the_attribute_mirrors_the_rule() {
        let (name, value) = fixture().attribute();
        assert_eq!(name, "data-dx-rule-fixture");
        let back: Rule =
            serde_json::from_str(&value).expect("the attribute value is the rule's JSON");
        assert_eq!(back, fixture());
        assert_eq!(fixture().kind(), "fixture");
    }

    /// The shared conformance vectors (`primitives/conformance/rules.json`, also run by `bun test` against
    /// the interpreter): every rule in them is a `Rule` that round-trips, and every expected message is JSON.
    #[test]
    fn the_conformance_vectors_are_rust_rules() {
        let file: serde_json::Value =
            serde_json::from_str(include_str!("../../conformance/rules.json")).expect("valid JSON");
        let vectors = file["vectors"].as_array().expect("a vectors array");
        assert!(!vectors.is_empty());
        for v in vectors {
            let rule: Rule = serde_json::from_value(v["rule"].clone())
                .unwrap_or_else(|e| panic!("{}: {e}", v["name"]));
            assert_eq!(
                serde_json::to_value(&rule).expect("serialises"),
                v["rule"],
                "{}",
                v["name"]
            );
            assert!(
                v["steps"].is_array() && v["expect"]["messages"].is_array(),
                "{}",
                v["name"]
            );
        }
    }

    /// An unknown kind is an error on this side, never a silently different rule.
    #[test]
    fn an_unknown_kind_does_not_parse() {
        assert!(serde_json::from_str::<Rule>(r#"{"kind":"from-the-future"}"#).is_err());
    }
}
