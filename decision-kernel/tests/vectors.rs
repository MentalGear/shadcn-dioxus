//! The shared vectors (`primitives/conformance/behaviours.json`) against the Rust `decide`, once called
//! directly and once through the byte route the page uses.

use decision_kernel::fixture::{self, Config, Event, Snapshot};
use decision_kernel::value::Value;
use decision_kernel::wire;

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
        assert_eq!(self.s[self.at], c, "expected {:?} at {}", c as char, self.at);
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
        let out = std::str::from_utf8(&self.s[start..self.at]).unwrap().to_owned();
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
                for (word, value) in [("true", Value::Bool(true)), ("false", Value::Bool(false)), ("null", Value::Null)] {
                    if rest.starts_with(word.as_bytes()) {
                        self.at += word.len();
                        return value;
                    }
                }
                let end = rest.iter().position(|b| !b"-+.0123456789eE".contains(b)).unwrap();
                let n = std::str::from_utf8(&rest[..end]).unwrap().parse().unwrap();
                self.at += end;
                Value::Num(n)
            }
        }
    }
}

/// Every fixture vector as (name, behaviour, event, state, memory, expect).
fn fixture_vectors() -> Vec<(String, Value, Value, Value, Value, Value)> {
    let all = Json { s: VECTORS.as_bytes(), at: 0 }.value();
    let Some(Value::Arr(vectors)) = all.get("fixture") else {
        panic!("no fixture vectors");
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
            )
        })
        .collect()
}

/// An object compared without regard to key order (the page reads members by name).
fn same(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Obj(x), Value::Obj(y)) => {
            x.len() == y.len() && x.iter().all(|(k, v)| b.get(k).is_some_and(|w| same(v, w)))
        }
        (Value::Arr(x), Value::Arr(y)) => x.len() == y.len() && x.iter().zip(y).all(|(p, q)| same(p, q)),
        _ => a == b,
    }
}

#[test]
fn the_rust_decide_meets_every_vector() {
    let vectors = fixture_vectors();
    assert!(vectors.len() >= 3);
    for (name, behaviour, event, state, memory, expect) in vectors {
        let decision = fixture::decide(
            &Config::from_value(&behaviour).unwrap(),
            &memory,
            &Snapshot::from_value(&state).unwrap(),
            &Event::from_value(&event).unwrap(),
        );
        assert!(same(&decision.to_value(), &expect), "{name}: {:?} != {expect:?}", decision.to_value());
    }
}

#[test]
fn the_byte_route_meets_every_vector() {
    for (name, behaviour, event, state, memory, expect) in fixture_vectors() {
        let mut request = Vec::new();
        Value::Arr(vec![behaviour, memory, state, event]).encode(&mut request);
        let answer = wire::decide_fixture(&request).unwrap_or_else(|| panic!("{name}: no answer"));
        let decoded = Value::decode(&answer).unwrap();
        assert!(same(&decoded, &expect), "{name}: {decoded:?} != {expect:?}");
    }
}

#[test]
fn malformed_requests_have_no_answer_and_never_panic() {
    let mut good = Vec::new();
    let (_, behaviour, event, state, memory, _) = fixture_vectors().remove(0);
    Value::Arr(vec![behaviour, memory, state, event]).encode(&mut good);
    assert!(wire::decide_fixture(&good).is_some());
    for cut in 0..good.len() {
        assert!(wire::decide_fixture(&good[..cut]).is_none(), "prefix {cut}");
    }
    let mut trailing = good.clone();
    trailing.push(0);
    assert!(wire::decide_fixture(&trailing).is_none());
    assert!(wire::decide_fixture(&[0xff; 64]).is_none());
    // a count far past the input
    assert!(wire::decide_fixture(&[5, 0xff, 0xff, 0xff, 0x7f]).is_none());
    // nesting past the limit
    assert!(wire::decide_fixture(&[5, 1, 0, 0, 0].repeat(40)).is_none());
}

#[test]
fn values_round_trip() {
    let value = Value::Obj(vec![
        ("a".into(), Value::Arr(vec![Value::Null, Value::Bool(true), Value::Bool(false), Value::Num(-1.5)])),
        ("ключ".into(), Value::Str("значение \u{1F600}".into())),
    ]);
    let mut bytes = Vec::new();
    value.encode(&mut bytes);
    assert_eq!(Value::decode(&bytes), Some(value));
}

/// Reported, not asserted: the native call's cost (the route a renderer with no page takes).
#[test]
fn the_native_call_is_cheap() {
    let (_, behaviour, event, state, memory, _) = fixture_vectors().remove(2);
    let config = Config::from_value(&behaviour).unwrap();
    let snapshot = Snapshot::from_value(&state).unwrap();
    let event = Event::from_value(&event).unwrap();
    let mut request = Vec::new();
    Value::Arr(vec![behaviour, memory.clone(), state, Value::Obj(vec![("type".into(), Value::Str("keydown".into()))])]).encode(&mut request);
    let mut direct = Vec::with_capacity(10_000);
    let mut bytes = Vec::with_capacity(10_000);
    for _ in 0..10_000 {
        let t = std::time::Instant::now();
        std::hint::black_box(fixture::decide(&config, &memory, &snapshot, &event));
        direct.push(t.elapsed().as_nanos());
        let t = std::time::Instant::now();
        std::hint::black_box(wire::decide_fixture(std::hint::black_box(&request)));
        bytes.push(t.elapsed().as_nanos());
    }
    for (label, samples) in [("direct", &mut direct), ("bytes", &mut bytes)] {
        samples.sort_unstable();
        println!("native {label}: p50 {} ns, p99 {} ns, max {} ns over {}", samples[samples.len() / 2], samples[samples.len() * 99 / 100], samples[samples.len() - 1], samples.len());
    }
    assert!(bytes[bytes.len() * 99 / 100] < 500_000, "p99 is under 0.5 ms natively as well");
}
