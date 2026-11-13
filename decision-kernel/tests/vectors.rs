//! The shared vectors (`primitives/conformance/behaviours.json`) against the Rust `decide`, once called
//! directly and once through the byte route the page uses.

use decision_kernel::fixture::{self, Config, Event, Snapshot};
use decision_kernel::value::Value;
use decision_kernel::wire::FromValue;
use decision_kernel::{decide_named, KINDS};

const VECTORS: &str = include_str!("../../primitives/conformance/behaviours.json");

/// Just enough JSON to read the vectors (this crate has no dependencies).
struct Json<'a> {
    s: &'a [u8],
    at: usize,
}

impl Json<'_> {
    fn ws(&mut self) {
        while self.s[self.at].is_ascii_whitespace() {
            self.at += 1;
        }
    }
    fn eat(&mut self, c: u8) {
        self.ws();
        assert_eq!(
            self.s[self.at], c,
            "expected {:?} at {}",
            c as char, self.at
        );
        self.at += 1;
    }
    fn peek(&mut self) -> u8 {
        self.ws();
        self.s[self.at]
    }
    fn string(&mut self) -> String {
        self.eat(b'"');
        let start = self.at;
        while self.s[self.at] != b'"' {
            assert_ne!(self.s[self.at], b'\\', "the vectors use no escapes");
            self.at += 1;
        }
        let out = std::str::from_utf8(&self.s[start..self.at])
            .unwrap()
            .to_owned();
        self.at += 1;
        out
    }
    fn value(&mut self) -> Value {
        match self.peek() {
            b'"' => Value::Str(self.string()),
            b'{' => {
                self.eat(b'{');
                let mut pairs = Vec::new();
                while self.peek() != b'}' {
                    let key = self.string();
                    self.eat(b':');
                    pairs.push((key, self.value()));
                    if self.peek() == b',' {
                        self.eat(b',');
                    }
                }
                self.eat(b'}');
                Value::Obj(pairs)
            }
            b'[' => {
                self.eat(b'[');
                let mut items = Vec::new();
                while self.peek() != b']' {
                    items.push(self.value());
                    if self.peek() == b',' {
                        self.eat(b',');
                    }
                }
                self.eat(b']');
                Value::Arr(items)
            }
            _ => {
                let rest = &self.s[self.at..];
                for (word, value) in [
                    ("true", Value::Bool(true)),
                    ("false", Value::Bool(false)),
                    ("null", Value::Null),
                ] {
                    if rest.starts_with(word.as_bytes()) {
                        self.at += word.len();
                        return value;
                    }
                }
                let end = rest
                    .iter()
                    .position(|b| !b"-+.0123456789eE".contains(b))
                    .unwrap();
                let n = std::str::from_utf8(&rest[..end]).unwrap().parse().unwrap();
                self.at += end;
                Value::Num(n)
            }
        }
    }
}

/// One vector: (name, behaviour, event, state, memory, expect, largest).
type Vector = (String, Value, Value, Value, Value, Value, bool);

/// Every vector of `kind`.
fn vectors_of(kind: &str) -> Vec<Vector> {
    let all = Json {
        s: VECTORS.as_bytes(),
        at: 0,
    }
    .value();
    let Some(Value::Arr(vectors)) = all.get(kind) else {
        panic!("no {kind} vectors");
    };
    vectors
        .iter()
        .map(|v| {
            (
                v.get("name").unwrap().as_str().unwrap().to_owned(),
                v.get("behaviour").unwrap().clone(),
                v.get("event").unwrap().clone(),
                v.get("state").unwrap().clone(),
                v.get("memory").cloned().unwrap_or(Value::Obj(vec![])),
                v.get("expect").unwrap().clone(),
                v.get("largest").and_then(Value::as_bool).unwrap_or(false),
            )
        })
        .collect()
}

/// The kinds the vectors file names.
fn vector_kinds() -> Vec<String> {
    let all = Json {
        s: VECTORS.as_bytes(),
        at: 0,
    }
    .value();
    let Value::Obj(pairs) = all else {
        panic!("an object keyed by kind")
    };
    pairs.into_iter().map(|(k, _)| k).collect()
}

/// An object compared without regard to key order (the page reads members by name).
fn same(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Obj(x), Value::Obj(y)) => {
            x.len() == y.len() && x.iter().all(|(k, v)| b.get(k).is_some_and(|w| same(v, w)))
        }
        (Value::Arr(x), Value::Arr(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(p, q)| same(p, q))
        }
        _ => a == b,
    }
}

fn request(behaviour: &Value, memory: &Value, state: &Value, event: &Value) -> Vec<u8> {
    let mut bytes = Vec::new();
    Value::Arr(vec![
        behaviour.clone(),
        memory.clone(),
        state.clone(),
        event.clone(),
    ])
    .encode(&mut bytes);
    bytes
}

/// The vectors and the kind table are the same set, and each kind has a vector at its largest planned snapshot.
#[test]
fn every_kind_has_vectors_and_every_vector_has_a_kind() {
    let mut kinds: Vec<String> = KINDS.iter().map(|k| k.name.to_owned()).collect();
    let mut named = vector_kinds();
    kinds.sort();
    named.sort();
    assert_eq!(kinds, named, "the kernel's kinds and the vectors' kinds");
    for kind in &kinds {
        assert!(
            vectors_of(kind).iter().any(|v| v.6),
            "{kind}: no vector marked `largest` (its largest planned snapshot)"
        );
    }
}

/// The Rust `decide` called directly (the route of a renderer with no page).
#[test]
fn the_rust_decide_meets_every_fixture_vector() {
    let vectors = vectors_of("fixture");
    assert!(vectors.len() >= 3);
    for (name, behaviour, event, state, memory, expect, _) in vectors {
        let decision = fixture::decide(
            &Config::from_value(&behaviour).unwrap(),
            &memory,
            &Snapshot::from_value(&state).unwrap(),
            &Event::from_value(&event).unwrap(),
        );
        assert!(
            same(&decision.to_value(), &expect),
            "{name}: {:?} != {expect:?}",
            decision.to_value()
        );
    }
}

/// Every vector of every kind through the byte route the page uses.
#[test]
fn the_byte_route_meets_every_vector() {
    for kind in vector_kinds() {
        for (name, behaviour, event, state, memory, expect, _) in vectors_of(&kind) {
            let answer = decide_named(&kind, &request(&behaviour, &memory, &state, &event))
                .unwrap_or_else(|| panic!("{kind}: {name}: no answer"));
            let decoded = Value::decode(&answer).unwrap();
            assert!(
                same(&decoded, &expect),
                "{kind}: {name}: {decoded:?} != {expect:?}"
            );
        }
    }
}

/// The wire keys the vectors carry are exactly the kind's declared keys (an event or a state with a key the
/// type does not read, or without one it does, is a vector that tests another contract than the page's).
#[test]
fn the_vectors_carry_exactly_the_declared_keys() {
    let keys = |value: &Value| -> Vec<String> {
        let Value::Obj(pairs) = value else {
            panic!("an object")
        };
        let mut keys: Vec<String> = pairs.iter().map(|(k, _)| k.clone()).collect();
        keys.sort();
        keys
    };
    for spec in KINDS {
        let sorted = |list: &[&str]| {
            let mut keys: Vec<String> = list.iter().map(|k| (*k).to_owned()).collect();
            keys.sort();
            keys
        };
        for (name, behaviour, event, state, ..) in vectors_of(spec.name) {
            assert_eq!(keys(&event), sorted(spec.fields), "{}: {name}: event", spec.name);
            assert_eq!(keys(&state), sorted(spec.reads), "{}: {name}: state", spec.name);
            for key in spec.config {
                assert!(behaviour.get(key).is_some(), "{}: {name}: config key {key}", spec.name);
            }
        }
    }
}

#[test]
fn malformed_requests_have_no_answer_and_never_panic() {
    let (_, behaviour, event, state, memory, ..) = vectors_of("fixture").remove(0);
    let good = request(&behaviour, &memory, &state, &event);
    assert!(decide_named("fixture", &good).is_some());
    for cut in 0..good.len() {
        assert!(decide_named("fixture", &good[..cut]).is_none(), "prefix {cut}");
    }
    let mut trailing = good.clone();
    trailing.push(0);
    assert!(decide_named("fixture", &trailing).is_none());
    assert!(decide_named("fixture", &[0xff; 64]).is_none());
    // a count far past the input
    assert!(decide_named("fixture", &[5, 0xff, 0xff, 0xff, 0x7f]).is_none());
    // nesting past the limit
    assert!(decide_named("fixture", &[5, 1, 0, 0, 0].repeat(40)).is_none());
}

/// A request that lacks a key the kind reads is unreadable, not a decision made on an absent value.
#[test]
fn a_missing_key_is_a_stand_down_not_a_decision() {
    let (_, behaviour, event, state, memory, ..) = vectors_of("fixture").remove(0);
    let without = |value: &Value, key: &str| {
        let Value::Obj(pairs) = value else { panic!() };
        Value::Obj(pairs.iter().filter(|(k, _)| k != key).cloned().collect())
    };
    for key in ["event", "attr", "value", "cancel"] {
        let bytes = request(&without(&behaviour, key), &memory, &state, &event);
        assert!(decide_named("fixture", &bytes).is_none(), "config `{key}`");
    }
    let bytes = request(&behaviour, &memory, &without(&state, "data-dx-fixture-block"), &event);
    assert!(decide_named("fixture", &bytes).is_none(), "snapshot key");
    let bytes = request(&behaviour, &memory, &state, &without(&event, "type"));
    assert!(decide_named("fixture", &bytes).is_none(), "event field");
}

#[test]
fn values_round_trip() {
    let value = Value::Obj(vec![
        (
            "a".into(),
            Value::Arr(vec![
                Value::Null,
                Value::Bool(true),
                Value::Bool(false),
                Value::Num(-1.5),
            ]),
        ),
        ("ключ".into(), Value::Str("значение \u{1F600}".into())),
    ]);
    let mut bytes = Vec::new();
    value.encode(&mut bytes);
    assert_eq!(Value::decode(&bytes), Some(value));
}

/// Reported, not asserted tightly: the native call's cost (the route a renderer with no page takes).
#[test]
fn the_native_call_is_cheap() {
    let (_, behaviour, event, state, memory, ..) = vectors_of("fixture").remove(2);
    let config = Config::from_value(&behaviour).unwrap();
    let snapshot = Snapshot::from_value(&state).unwrap();
    let event_in = Event::from_value(&event).unwrap();
    let bytes_in = request(&behaviour, &memory, &state, &event);
    let mut direct = Vec::with_capacity(10_000);
    let mut bytes = Vec::with_capacity(10_000);
    for _ in 0..10_000 {
        let t = std::time::Instant::now();
        std::hint::black_box(fixture::decide(&config, &memory, &snapshot, &event_in));
        direct.push(t.elapsed().as_nanos());
        let t = std::time::Instant::now();
        std::hint::black_box(decide_named("fixture", std::hint::black_box(&bytes_in)));
        bytes.push(t.elapsed().as_nanos());
    }
    for (label, samples) in [("direct", &mut direct), ("bytes", &mut bytes)] {
        samples.sort_unstable();
        println!(
            "native {label}: p50 {} ns, p99 {} ns, max {} ns over {}",
            samples[samples.len() / 2],
            samples[samples.len() * 99 / 100],
            samples[samples.len() - 1],
            samples.len()
        );
    }
    assert!(
        bytes[bytes.len() * 99 / 100] < 500_000,
        "p99 is under 0.5 ms natively as well"
    );
}
