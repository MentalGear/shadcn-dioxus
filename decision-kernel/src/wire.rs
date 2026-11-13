//! The wire form of a kind's inputs, defined once, and the byte route both the wasm export and `cargo test` call.
//!
//! [`wire_input!`] declares an input type (`Config`, `Snapshot` or `Event`) together with the wire key each field is
//! read from. The type, its reader and its key list ([`FromValue::KEYS`]) come from that one declaration, so the key
//! list the page's adapter is generated from can never differ from the keys the Rust side reads. Every key is
//! required: a request that lacks one is malformed ([`run`] answers `None`, the page stands down and tells Rust),
//! so a renamed key is loud at the first event instead of a decision made on an absent value.

use crate::decision::Decision;
use crate::value::Value;

/// An input type read from the wire `Value` of an object.
pub trait FromValue: Sized {
    /// The wire keys, in field order. The page collects exactly these.
    const KEYS: &'static [&'static str];
    /// Reads the object `value`; `None` when a key is missing or has the wrong type.
    fn from_value(value: &Value) -> Option<Self>;
}

/// A field type and how it is read from the member of an object (`None`: the key is absent).
pub trait Field: Sized {
    /// Reads the member; `None` when it is absent or has the wrong type.
    fn read(member: Option<&Value>) -> Option<Self>;
}

impl Field for String {
    fn read(member: Option<&Value>) -> Option<Self> {
        member?.as_str().map(str::to_owned)
    }
}

impl Field for bool {
    fn read(member: Option<&Value>) -> Option<Self> {
        member?.as_bool()
    }
}

impl Field for Option<String> {
    /// A string or `null` (an attribute that is absent); the key itself must be present.
    fn read(member: Option<&Value>) -> Option<Self> {
        match member? {
            Value::Str(s) => Some(Some(s.clone())),
            Value::Null => Some(None),
            _ => None,
        }
    }
}

/// Declares an input type and its wire keys: `field: Type = "wire key"`.
///
/// ```
/// decision_kernel::wire_input! {
///     /// What the decision reads.
///     pub struct Snapshot { pub block: Option<String> = "data-dx-block" }
/// }
/// use decision_kernel::wire::FromValue;
/// assert_eq!(Snapshot::KEYS, ["data-dx-block"]);
/// ```
#[macro_export]
macro_rules! wire_input {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $( $(#[$fmeta:meta])* $fvis:vis $field:ident : $ty:ty = $key:literal ),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Debug, PartialEq)]
        $vis struct $name {
            $( $(#[$fmeta])* $fvis $field: $ty ),+
        }

        impl $crate::wire::FromValue for $name {
            const KEYS: &'static [&'static str] = &[$($key),+];

            fn from_value(value: &$crate::value::Value) -> Option<Self> {
                Some(Self {
                    $( $field: <$ty as $crate::wire::Field>::read(value.get($key))?, )+
                })
            }
        }
    };
}

/// Decodes the request (`[config, memory, snapshot, event]`), runs `decide` and encodes the decision. Malformed
/// input, a missing key or a key of the wrong type is `None` (the page then stands down and tells Rust).
pub fn run<C: FromValue, S: FromValue, E: FromValue>(
    request: &[u8],
    decide: fn(&C, &Value, &S, &E) -> Decision,
) -> Option<Vec<u8>> {
    let Value::Arr(parts) = Value::decode(request)? else {
        return None;
    };
    let [config, memory, snapshot, event] = parts.as_slice() else {
        return None;
    };
    let decision = decide(
        &C::from_value(config)?,
        memory,
        &S::from_value(snapshot)?,
        &E::from_value(event)?,
    );
    let mut out = Vec::with_capacity(96);
    decision.to_value().encode(&mut out);
    Some(out)
}
