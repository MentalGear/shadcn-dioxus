//! The closed names the page's bridge accepts for events and event fields. Rust sends these enums and
//! `ts/bridge.ts` maps each through a closed own-property table (`EVENTS`, `FIELD_GETTERS`), so an
//! event or a field the table has no row for is an error on the page, never a property read. The
//! tests compare every row with the TypeScript, so the two cannot drift.

use serde::{Deserialize, Serialize, Serializer};

/// Which targets an event may be listened on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Receivers {
    /// `window`.
    pub window: bool,
    /// `document`.
    pub document: bool,
    /// An element.
    pub element: bool,
}

impl Receivers {
    const fn new(letters: &str) -> Self {
        let b = letters.as_bytes();
        let mut window = false;
        let mut document = false;
        let mut element = false;
        let mut i = 0;
        while i < b.len() {
            match b[i] {
                b'w' => window = true,
                b'd' => document = true,
                b'e' => element = true,
                _ => {}
            }
            i += 1;
        }
        Receivers {
            window,
            document,
            element,
        }
    }

    /// The letters the page's `EVENTS` table spells (`w`, `d`, `e`, in that order).
    pub fn letters(self) -> String {
        let mut out = String::new();
        if self.window {
            out.push('w');
        }
        if self.document {
            out.push('d');
        }
        if self.element {
            out.push('e');
        }
        out
    }
}

macro_rules! events {
    ($( $variant:ident => $wire:literal, $recv:literal, $hot:literal; )*) => {
        /// A DOM event the bridge may listen to.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum EventKind {
            $(
                #[doc = concat!("The `", $wire, "` event.")]
                $variant,
            )*
        }

        impl EventKind {
            /// Every event, in table order.
            pub const ALL: &'static [EventKind] = &[$(EventKind::$variant),*];

            /// The event's name, as `addEventListener` takes it.
            pub const fn wire(self) -> &'static str {
                match self { $(EventKind::$variant => $wire),* }
            }

            /// What the event may be listened on.
            pub const fn receivers(self) -> Receivers {
                match self { $(EventKind::$variant => Receivers::new($recv)),* }
            }

            /// A frame-rate event: the page's `listen` coalesces it to one message per animation
            /// frame, so it must be consumed through `listen`, `observe` or `sample`, never a vdom handler.
            pub const fn hot(self) -> bool {
                match self { $(EventKind::$variant => $hot),* }
            }
        }
    };
}

events! {
    Keydown => "keydown", "wde", false;
    Keyup => "keyup", "wde", false;
    Beforeinput => "beforeinput", "e", false;
    Input => "input", "e", false;
    Change => "change", "e", false;
    Compositionstart => "compositionstart", "e", false;
    Compositionend => "compositionend", "e", false;
    Click => "click", "wde", false;
    Dblclick => "dblclick", "wde", false;
    Contextmenu => "contextmenu", "wde", false;
    Pointerdown => "pointerdown", "wde", false;
    Pointerup => "pointerup", "wde", false;
    Pointermove => "pointermove", "wde", true;
    Pointercancel => "pointercancel", "wde", false;
    Mousedown => "mousedown", "wde", false;
    Mouseup => "mouseup", "wde", false;
    Mousemove => "mousemove", "wde", true;
    Touchstart => "touchstart", "wde", false;
    Touchmove => "touchmove", "wde", true;
    Touchend => "touchend", "wde", false;
    Touchcancel => "touchcancel", "wde", false;
    Wheel => "wheel", "wde", true;
    Scroll => "scroll", "wde", true;
    Resize => "resize", "w", true;
    Focusin => "focusin", "wde", false;
    Focusout => "focusout", "wde", false;
    Focus => "focus", "e", false;
    Blur => "blur", "e", false;
    Dragenter => "dragenter", "wde", false;
    Dragover => "dragover", "wde", true;
    Dragleave => "dragleave", "wde", false;
    Drop => "drop", "wde", false;
    Copy => "copy", "wde", false;
    Cut => "cut", "wde", false;
    Paste => "paste", "wde", false;
    Visibilitychange => "visibilitychange", "d", false;
    Selectionchange => "selectionchange", "d", false;
    Pagehide => "pagehide", "w", false;
    Pageshow => "pageshow", "w", false;
    Online => "online", "w", false;
    Offline => "offline", "w", false;
    Hashchange => "hashchange", "w", false;
    Popstate => "popstate", "w", false;
    Beforetoggle => "beforetoggle", "e", false;
    Toggle => "toggle", "e", false;
    Close => "close", "e", false;
    Cancel => "cancel", "e", false;
    Transitionend => "transitionend", "e", false;
    Animationend => "animationend", "e", false;
}

impl Serialize for EventKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.wire())
    }
}

impl<'de> Deserialize<'de> for EventKind {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let name = String::deserialize(deserializer)?;
        EventKind::ALL
            .iter()
            .copied()
            .find(|kind| kind.wire() == name)
            .ok_or_else(|| serde::de::Error::custom(format!("no such event: {name}")))
    }
}

macro_rules! fields {
    ($( $variant:ident => $wire:literal; )*) => {
        /// An event field the bridge may read: a value that is a string, number or bool (anything
        /// else reads as null). A message from `listen` has one key per requested field, named by
        /// [`wire`](EventField::wire) (`#[serde(rename = "target.id")]` for a dotted one).
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum EventField {
            $(
                #[doc = concat!("The event's `", $wire, "`.")]
                $variant,
            )*
        }

        impl EventField {
            /// Every field, in table order.
            pub const ALL: &'static [EventField] = &[$(EventField::$variant),*];

            /// The field's name, as the page's `FIELD_GETTERS` spells it.
            pub const fn wire(self) -> &'static str {
                match self { $(EventField::$variant => $wire),* }
            }
        }
    };
}

fields! {
    Type => "type";
    Key => "key";
    Code => "code";
    ShiftKey => "shiftKey";
    CtrlKey => "ctrlKey";
    AltKey => "altKey";
    MetaKey => "metaKey";
    Repeat => "repeat";
    IsComposing => "isComposing";
    DefaultPrevented => "defaultPrevented";
    ClientX => "clientX";
    ClientY => "clientY";
    PointerId => "pointerId";
    PointerType => "pointerType";
    Button => "button";
    Buttons => "buttons";
    DeltaX => "deltaX";
    DeltaY => "deltaY";
    DeltaMode => "deltaMode";
    TimeStamp => "timeStamp";
    InputType => "inputType";
    Data => "data";
    NewState => "newState";
    OldState => "oldState";
    TouchCount => "touchCount";
    TargetId => "target.id";
    TargetTagName => "target.tagName";
    TargetValue => "target.value";
    TargetScrollTop => "target.scrollTop";
    TargetScrollLeft => "target.scrollLeft";
    TargetInnerWidth => "target.innerWidth";
    TargetInnerHeight => "target.innerHeight";
    CurrentTargetId => "currentTarget.id";
}

impl Serialize for EventField {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.wire())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BRIDGE_TS: &str = include_str!("../ts/bridge.ts");

    /// Every `EventKind` is exactly one row of the page's `EVENTS` table, with the same receivers and
    /// hot flag, and the table has no other row.
    #[test]
    fn the_events_table_is_this_enum() {
        let mut rows = 0;
        for kind in EventKind::ALL {
            let hot = if kind.hot() { ", hot: 1" } else { "" };
            let row = format!(
                "\"{}\": {{ recv: \"{}\"{hot} }},",
                kind.wire(),
                kind.receivers().letters()
            );
            assert!(BRIDGE_TS.contains(&row), "EVENTS row missing or different: {row}");
            rows += 1;
        }
        let start = BRIDGE_TS.find("const EVENTS").expect("EVENTS table");
        let end = start + BRIDGE_TS[start..].find("\n};").expect("end of EVENTS");
        let in_table = BRIDGE_TS[start..end].matches(": { recv: \"").count();
        assert_eq!(in_table, rows, "EVENTS has a row Rust does not name");
    }

    /// Every `EventField` is one getter of `FIELD_GETTERS`, and the table has no other.
    #[test]
    fn the_field_getters_are_this_enum() {
        for field in EventField::ALL {
            assert!(
                BRIDGE_TS.contains(&format!("\"{}\": (e) =>", field.wire())),
                "FIELD_GETTERS row missing: {}",
                field.wire()
            );
        }
        let start = BRIDGE_TS.find("const FIELD_GETTERS").expect("FIELD_GETTERS table");
        let end = start + BRIDGE_TS[start..].find("\n};").expect("end of FIELD_GETTERS");
        assert_eq!(
            BRIDGE_TS[start..end].matches("\": (e) =>").count(),
            EventField::ALL.len(),
            "FIELD_GETTERS has a row Rust does not name"
        );
    }

    #[test]
    fn names_are_unique_and_serialise_as_their_wire_name() {
        let mut seen = std::collections::HashSet::new();
        for kind in EventKind::ALL {
            assert!(seen.insert(kind.wire()), "{}", kind.wire());
        }
        let mut seen = std::collections::HashSet::new();
        for field in EventField::ALL {
            assert!(seen.insert(field.wire()), "{}", field.wire());
        }
        assert_eq!(
            serde_json::to_string(&EventKind::Keydown).expect("serialises"),
            "\"keydown\""
        );
        assert_eq!(
            serde_json::from_str::<EventKind>("\"wheel\"").expect("parses"),
            EventKind::Wheel
        );
        assert!(serde_json::from_str::<EventKind>("\"constructor\"").is_err());
    }

    #[test]
    fn frame_rate_events_are_hot() {
        for kind in [
            EventKind::Wheel,
            EventKind::Scroll,
            EventKind::Pointermove,
            EventKind::Touchmove,
            EventKind::Mousemove,
            EventKind::Resize,
        ] {
            assert!(kind.hot(), "{}", kind.wire());
        }
        assert!(!EventKind::Keydown.hot());
    }
}
