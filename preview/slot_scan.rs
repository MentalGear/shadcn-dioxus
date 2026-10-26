//! The one scanner for a component's slot variables (backlog row 433).
//!
//! A SLOT is a custom property a component's sheet reads under its own prefix (`--dx-<dir with dashes>-`) with
//! a fallback (`var(--dx-label-gap, var(--dx-space-2))`) and never declares itself: the composing component or
//! the app sets it, the sheet's fallback is its default. Two callers include this file with `#[path]`, so what the
//! docs page prints and what the gate demands come from the same parse:
//! - `preview/build.rs` renders every component page's "Styling (slot variables)" table from it;
//! - `scripts/slot-scan/main.rs` (compiled by `scripts/check-slot-docs.sh` and `check-css-vars-defined.sh`) lists
//!   the slots and the notes that describe them.
//!
//! The description is hand-written, once, in the sheet next to the read:
//! `/* slot: --dx-label-gap  Space between the label text and its control. */`. A variable a script writes
//! (so no caller sets it) says `/* slot-internal: --dx-drawer-drag  <who writes it> */` and stays off the page.
//!
//! It is a small scan, not a CSS parser: comments and strings are skipped, a declaration is whatever sits between
//! the last `;`, `{` or `}` and the next `;`, and a custom property is declared when a declaration's property
//! starts with `--`. `std` only, so a gate can build it with bare `rustc`.
#![allow(
    dead_code,
    reason = "two callers include this file and each uses a different subset"
)]

/// The `/* slot: .. */` or `/* slot-internal: .. */` comment that describes one slot.
#[derive(Debug, Clone)]
pub struct SlotNote {
    /// `slot-internal`: a script writes it, a caller does not set it.
    pub internal: bool,
    /// The variable the comment names; empty when the comment has no `--name`.
    pub name: String,
    /// The description with whitespace collapsed; empty when the comment gives none.
    pub text: String,
    /// 1-based line of the comment.
    pub line: usize,
}

/// A slot read of one sheet.
#[derive(Debug, Clone)]
pub struct Slot {
    pub name: String,
    /// The fallback of the first read that has one, whitespace collapsed (`var(--dx-space-2)`, `1.25rem`).
    pub fallback: String,
    /// The distinct properties the reads feed, in source order (`gap`, `--dx-button-icon-size`).
    pub properties: Vec<String>,
    /// 1-based line of the first read.
    pub line: usize,
    pub note: Option<SlotNote>,
}

#[derive(Debug, Clone, Default)]
pub struct SheetSlots {
    pub slots: Vec<Slot>,
    /// Notes that name no slot of this sheet: documented but not read, or read without a fallback, or declared.
    pub orphan_notes: Vec<SlotNote>,
    /// A second note for a variable that already has one.
    pub duplicate_notes: Vec<SlotNote>,
}

/// `dropdown_menu` -> `--dx-dropdown-menu-`.
pub fn component_prefix(dir: &str) -> String {
    format!("--dx-{}-", dir.replace('_', "-"))
}

fn collapse(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn is_ident(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_'
}

fn parse_note(comment: &str, line: usize) -> Option<SlotNote> {
    let lines: Vec<&str> = comment
        .lines()
        .map(|l| l.trim().trim_start_matches('*').trim())
        .collect();
    let body = collapse(&lines.join(" "));
    let (internal, rest) = match body.strip_prefix("slot-internal:") {
        Some(rest) => (true, rest),
        None => (false, body.strip_prefix("slot:")?),
    };
    let rest = rest.trim();
    let name_len = if rest.starts_with("--") {
        rest.bytes().take_while(|b| is_ident(*b)).count()
    } else {
        0
    };
    let name = rest[..name_len].to_string();
    let text = rest[name_len..]
        .trim_start_matches(|c: char| c.is_whitespace() || matches!(c, ':' | '-' | '\u{2014}'))
        .trim()
        .to_string();
    Some(SlotNote {
        internal,
        name,
        text,
        line,
    })
}

/// `css` with every comment replaced by spaces (newlines kept, so offsets and lines are the source's), and the
/// slot notes the comments held. A comment opener inside a string is text.
fn blank_comments(css: &str) -> (String, Vec<SlotNote>) {
    let b = css.as_bytes();
    let n = b.len();
    let mut out = String::with_capacity(n);
    let mut notes = Vec::new();
    let (mut i, mut line, mut start) = (0usize, 1usize, 0usize);
    while i < n {
        let c = b[i];
        if c == b'\n' {
            line += 1;
            i += 1;
        } else if c == b'/' && b.get(i + 1) == Some(&b'*') {
            let end = css[i + 2..].find("*/").map_or(n, |e| i + 2 + e + 2);
            out.push_str(&css[start..i]);
            let body_end = if end >= i + 4 && css[..end].ends_with("*/") {
                end - 2
            } else {
                end
            };
            if let Some(note) = parse_note(&css[i + 2..body_end.max(i + 2)], line) {
                notes.push(note);
            }
            for ch in css[i..end].chars() {
                out.push(if ch == '\n' { '\n' } else { ' ' });
            }
            line += css[i..end].matches('\n').count();
            i = end;
            start = end;
        } else if c == b'"' || c == b'\'' {
            i += 1;
            while i < n && b[i] != c && b[i] != b'\n' {
                i += if b[i] == b'\\' { 2 } else { 1 };
            }
            i = (i + 1).min(n);
        } else {
            i += 1;
        }
    }
    out.push_str(&css[start..n.min(css.len())]);
    (out, notes)
}

struct Read {
    name: String,
    fallback: Option<String>,
    property: String,
    line: usize,
}

/// The slots of `css`, the sheet of component directory `dir`.
pub fn scan_sheet(dir: &str, css: &str) -> SheetSlots {
    let prefix = component_prefix(dir);
    let (clean, mut notes) = blank_comments(css);
    let css = clean.as_str();
    let b = css.as_bytes();
    let n = b.len();
    let mut reads: Vec<Read> = Vec::new();
    let mut declared: std::collections::HashSet<String> = std::collections::HashSet::new();
    let (mut i, mut line, mut depth) = (0usize, 1usize, 0usize);
    let mut decl_start = 0usize;
    let mut property: Option<String> = None;
    while i < n {
        let c = b[i];
        if c == b'\n' {
            line += 1;
            i += 1;
        } else if c == b'"' || c == b'\'' {
            i += 1;
            while i < n && b[i] != c && b[i] != b'\n' {
                i += if b[i] == b'\\' { 2 } else { 1 };
            }
            i = (i + 1).min(n);
        } else if c == b'(' {
            // `var(` : the name, then a fallback to the matching paren. The scan carries on INSIDE the
            // fallback, so a `var()` nested in it is read too.
            if i >= 3 && &b[i - 3..i] == b"var" && (i == 3 || !is_ident(b[i - 4])) {
                let mut j = i + 1;
                while j < n && b[j].is_ascii_whitespace() {
                    j += 1;
                }
                let name_len = if css[j..].starts_with("--") {
                    b[j..].iter().take_while(|x| is_ident(**x)).count()
                } else {
                    0
                };
                if name_len > 0 {
                    let name = css[j..j + name_len].to_string();
                    let mut k = j + name_len;
                    while k < n && b[k].is_ascii_whitespace() {
                        k += 1;
                    }
                    let fallback = if b.get(k) == Some(&b',') {
                        let (mut d, mut m) = (1usize, k + 1);
                        while m < n && d > 0 {
                            match b[m] {
                                b'(' => d += 1,
                                b')' => d -= 1,
                                _ => {}
                            }
                            m += 1;
                        }
                        Some(collapse(&css[k + 1..m.saturating_sub(1).max(k + 1)]))
                    } else {
                        None
                    };
                    reads.push(Read {
                        name,
                        fallback,
                        property: property.clone().unwrap_or_default(),
                        line,
                    });
                }
            }
            depth += 1;
            i += 1;
        } else if c == b')' {
            depth = depth.saturating_sub(1);
            i += 1;
        } else if depth == 0 && matches!(c, b';' | b'{' | b'}') {
            decl_start = i + 1;
            property = None;
            i += 1;
        } else if depth == 0 && c == b':' && property.is_none() {
            let text = collapse(&css[decl_start..i]);
            if text.starts_with("--") {
                declared.insert(text.clone());
            }
            property = Some(text);
            i += 1;
        } else {
            i += 1;
        }
    }

    let mut out = SheetSlots::default();
    let mut order: Vec<String> = Vec::new();
    for read in &reads {
        if read.name.starts_with(&prefix)
            && !declared.contains(&read.name)
            && !order.contains(&read.name)
        {
            order.push(read.name.clone());
        }
    }
    for name in order {
        let of: Vec<&Read> = reads.iter().filter(|r| r.name == name).collect();
        let Some(fallback) = of.iter().find_map(|r| r.fallback.clone()) else {
            continue; // read bare: a variable another file always sets, not a knob with a default
        };
        let mut properties: Vec<String> = Vec::new();
        for read in &of {
            if !read.property.is_empty() && !properties.contains(&read.property) {
                properties.push(read.property.clone());
            }
        }
        out.slots.push(Slot {
            name,
            fallback,
            properties,
            line: of[0].line,
            note: None,
        });
    }
    for note in notes.drain(..) {
        match out.slots.iter_mut().find(|s| s.name == note.name) {
            Some(slot) if slot.note.is_none() => slot.note = Some(note),
            Some(_) => out.duplicate_notes.push(note),
            None => out.orphan_notes.push(note),
        }
    }
    out
}
