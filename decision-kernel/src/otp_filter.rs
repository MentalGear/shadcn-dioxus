//! The one-time-passcode field's filter: what an edit of the field's text is allowed to leave in it.
//!
//! The behaviour is installed three times on one text control, once per event (`event` is a parameter of the kind):
//!
//! * `beforeinput` (the edit has not happened): the text the edit would insert is filtered to the accepted characters
//!   and bounded to the room left in the field. When that differs from what the browser would insert, the decision
//!   cancels the edit and writes the accepted part over the selection itself. A paste, a keystroke, a drop and a
//!   replacement text are one rule; a deletion carries no text and passes.
//! * `input` (the edit happened, from a source that carried no `beforeinput` text: autofill, undo): the whole value is
//!   filtered and bounded, and the caret stays after the accepted characters that preceded it.
//! * `compositionend` (an input method committed text): the same rule as `input` over the committed text.
//!
//! A decision never writes while an input method composes: the browser owns the text until the composition ends, a write
//! would destroy the composition, and a write held until the end would be decided from text that has changed since. The
//! write is dropped (the decision is empty) and `compositionend` decides again from the committed text.
//!
//! The accepted characters are the closed `ranges` of the field's pattern: a string of `(low, high)` character pairs.

use crate::decision::{Decision, ValueWrite};
use crate::value::Value;
use crate::wire_input;

wire_input! {
    /// The behaviour's parameters (the keys of the Rust `Behaviour::OtpFilter` the decision reads).
    pub struct Config {
        /// The event the install listens to: `beforeinput`, `input` or `compositionend`.
        pub event: String = "event",
        /// The accepted characters as `(low, high)` pairs, concatenated: `"09"` is the digits, `"AZaz09"` the
        /// alphanumerics.
        pub ranges: String = "ranges",
        /// The most characters the field holds.
        pub max: f64 = "max",
    }
}

wire_input! {
    /// The attribute reads taken before the event: none.
    pub struct Snapshot {}
}

wire_input! {
    /// The event fields the decision reads.
    pub struct Event {
        /// The event's `type`.
        pub kind: String = "type",
        /// The input event's `inputType` (`null` for a composition event).
        pub input_type: Option<String> = "inputType",
        /// The text the edit inserts (`null` for a deletion).
        pub data: Option<String> = "data",
        /// Whether an input method is composing (`null` for an event that has no such member).
        pub composing: Option<bool> = "isComposing",
        /// The field's value now.
        pub value: String = "value",
        /// The start of the selection, in UTF-16 units.
        pub start: f64 = "selectionStart",
        /// The end of the selection, in UTF-16 units.
        pub end: f64 = "selectionEnd",
    }
}

/// The accepted characters of a `ranges` string.
struct Accepted(Vec<(char, char)>);

impl Accepted {
    fn parse(ranges: &str) -> Self {
        let chars: Vec<char> = ranges.chars().collect();
        Accepted(chars.chunks_exact(2).map(|p| (p[0], p[1])).collect())
    }

    fn accepts(&self, c: char) -> bool {
        self.0.iter().any(|&(low, high)| low <= c && c <= high)
    }

    /// The accepted characters of `text`, at most `room` of them.
    fn keep(&self, text: impl Iterator<Item = char>, room: usize) -> String {
        text.filter(|&c| self.accepts(c)).take(room).collect()
    }
}

/// The length of `chars` in UTF-16 units.
fn utf16(chars: &[char]) -> usize {
    chars.iter().map(|c| c.len_utf16()).sum()
}

/// The index in `chars` of the UTF-16 offset `offset` (the end when it is past the text).
fn char_index(chars: &[char], offset: f64) -> usize {
    let mut units = 0.0;
    for (i, c) in chars.iter().enumerate() {
        if units >= offset {
            return i;
        }
        units += c.len_utf16() as f64;
    }
    chars.len()
}

/// The decision. Pure: the same inputs give the same answer.
pub fn decide(config: &Config, _memory: &Value, _snapshot: &Snapshot, event: &Event) -> Decision {
    if event.composing == Some(true) {
        return Decision::default();
    }
    let accepted = Accepted::parse(&config.ranges);
    let max = config.max.max(0.0) as usize;
    match event.kind.as_str() {
        "beforeinput" => edit(&accepted, max, event),
        "input" => settle(&accepted, max, event, false),
        "compositionend" => settle(&accepted, max, event, true),
        _ => Decision::default(),
    }
}

/// An edit that carries text: cancelled and replaced by its accepted part when the browser would insert more.
fn edit(accepted: &Accepted, max: usize, event: &Event) -> Decision {
    let Some(data) = event.data.as_deref().filter(|d| !d.is_empty()) else {
        return Decision::default();
    };
    if event.input_type.as_deref() == Some("insertCompositionText") {
        return Decision::default();
    }
    let chars: Vec<char> = event.value.chars().collect();
    let (a, b) = (char_index(&chars, event.start), char_index(&chars, event.end));
    let (from, to) = (a.min(b), a.max(b));
    let room = max.saturating_sub(chars.len() - (to - from));
    let text = accepted.keep(data.chars(), room);
    if text == data {
        return Decision::default();
    }
    let decision = Decision {
        prevent: true,
        ..Decision::default()
    };
    if text.is_empty() {
        return decision;
    }
    let caret = utf16(&chars[..from]) + text.encode_utf16().count();
    decision.write_value(&ValueWrite {
        from: utf16(&chars[..from]),
        to: utf16(&chars[..to]),
        text,
        caret,
        input: true,
    })
}

/// The value as it stands: filtered and bounded as a whole, the caret kept after the accepted characters before it.
fn settle(accepted: &Accepted, max: usize, event: &Event, input: bool) -> Decision {
    let chars: Vec<char> = event.value.chars().collect();
    let filtered = accepted.keep(chars.iter().copied(), max);
    if filtered.chars().eq(chars.iter().copied()) {
        return Decision::default();
    }
    let before = &chars[..char_index(&chars, event.start)];
    let kept = accepted.keep(before.iter().copied(), max);
    Decision::default().write_value(&ValueWrite {
        from: 0,
        to: utf16(&chars),
        text: filtered,
        caret: kept.encode_utf16().count(),
        input,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wire::FromValue;

    #[test]
    fn the_keys_are_the_declared_wire_names() {
        assert_eq!(Config::KEYS, ["event", "ranges", "max"]);
        assert!(Snapshot::KEYS.is_empty());
        assert_eq!(
            Event::KEYS,
            [
                "type",
                "inputType",
                "data",
                "isComposing",
                "value",
                "selectionStart",
                "selectionEnd"
            ]
        );
    }

    #[test]
    fn utf16_offsets_map_to_characters() {
        let chars: Vec<char> = "a\u{1F600}b".chars().collect();
        assert_eq!(char_index(&chars, 0.0), 0);
        assert_eq!(char_index(&chars, 1.0), 1);
        assert_eq!(char_index(&chars, 3.0), 2);
        assert_eq!(char_index(&chars, 99.0), 3);
        assert_eq!(utf16(&chars), 4);
    }

    #[test]
    fn an_odd_trailing_range_character_is_ignored() {
        assert!(Accepted::parse("09a").accepts('5'));
        assert!(!Accepted::parse("09a").accepts('a'));
    }
}
