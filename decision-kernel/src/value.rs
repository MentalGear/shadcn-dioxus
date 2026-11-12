//! The boundary's one data type and its binary encoding.
//!
//! Every value is a tag byte plus a payload; lengths and counts are `u32` little-endian:
//!
//! | tag | value | payload |
//! |---|---|---|
//! | 0 | null | none |
//! | 1 | false | none |
//! | 2 | true | none |
//! | 3 | number | `f64` little-endian |
//! | 4 | string | length, UTF-8 bytes |
//! | 5 | array | count, then the items |
//! | 6 | object | count, then (key as length plus UTF-8 bytes, value) pairs, in insertion order |
//!
//! Decoding never panics: malformed input is `None`.

/// A JSON-shaped value.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// `null`.
    Null,
    /// `true` or `false`.
    Bool(bool),
    /// A number (every JavaScript number is an `f64`).
    Num(f64),
    /// A string.
    Str(String),
    /// An array.
    Arr(Vec<Value>),
    /// An object with its keys in insertion order.
    Obj(Vec<(String, Value)>),
}

/// The deepest nesting `decode` accepts, so hostile input cannot exhaust the stack.
const MAX_DEPTH: usize = 16;

impl Value {
    /// The member `key` of an object.
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Obj(pairs) => pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    /// The string, if this is one.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }

    /// The boolean, if this is one.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Appends the encoding of this value to `out`.
    pub fn encode(&self, out: &mut Vec<u8>) {
        match self {
            Value::Null => out.push(0),
            Value::Bool(false) => out.push(1),
            Value::Bool(true) => out.push(2),
            Value::Num(n) => {
                out.push(3);
                out.extend_from_slice(&n.to_le_bytes());
            }
            Value::Str(s) => {
                out.push(4);
                put_str(out, s);
            }
            Value::Arr(items) => {
                out.push(5);
                out.extend_from_slice(&(items.len() as u32).to_le_bytes());
                for item in items {
                    item.encode(out);
                }
            }
            Value::Obj(pairs) => {
                out.push(6);
                out.extend_from_slice(&(pairs.len() as u32).to_le_bytes());
                for (key, value) in pairs {
                    put_str(out, key);
                    value.encode(out);
                }
            }
        }
    }

    /// Decodes one value that spans all of `bytes`.
    pub fn decode(bytes: &[u8]) -> Option<Value> {
        let mut at = 0;
        let value = read(bytes, &mut at, 0)?;
        (at == bytes.len()).then_some(value)
    }
}

fn put_str(out: &mut Vec<u8>, s: &str) {
    out.extend_from_slice(&(s.len() as u32).to_le_bytes());
    out.extend_from_slice(s.as_bytes());
}

fn take<'a>(bytes: &'a [u8], at: &mut usize, n: usize) -> Option<&'a [u8]> {
    let end = at.checked_add(n)?;
    let slice = bytes.get(*at..end)?;
    *at = end;
    Some(slice)
}

fn read_u32(bytes: &[u8], at: &mut usize) -> Option<usize> {
    let raw: [u8; 4] = take(bytes, at, 4)?.try_into().ok()?;
    Some(u32::from_le_bytes(raw) as usize)
}

fn read_str(bytes: &[u8], at: &mut usize) -> Option<String> {
    let len = read_u32(bytes, at)?;
    String::from_utf8(take(bytes, at, len)?.to_vec()).ok()
}

fn read(bytes: &[u8], at: &mut usize, depth: usize) -> Option<Value> {
    if depth > MAX_DEPTH {
        return None;
    }
    let tag = *take(bytes, at, 1)?.first()?;
    Some(match tag {
        0 => Value::Null,
        1 => Value::Bool(false),
        2 => Value::Bool(true),
        3 => Value::Num(f64::from_le_bytes(take(bytes, at, 8)?.try_into().ok()?)),
        4 => Value::Str(read_str(bytes, at)?),
        5 => {
            let count = read_u32(bytes, at)?;
            // every item is at least one byte: a count past the rest of the input is malformed
            if count > bytes.len() - *at {
                return None;
            }
            let mut items = Vec::with_capacity(count);
            for _ in 0..count {
                items.push(read(bytes, at, depth + 1)?);
            }
            Value::Arr(items)
        }
        6 => {
            let count = read_u32(bytes, at)?;
            if count > bytes.len() - *at {
                return None;
            }
            let mut pairs = Vec::with_capacity(count);
            for _ in 0..count {
                let key = read_str(bytes, at)?;
                pairs.push((key, read(bytes, at, depth + 1)?));
            }
            Value::Obj(pairs)
        }
        _ => return None,
    })
}
