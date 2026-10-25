//! The one scanner for `#[derive(Props)]` structs (backlog row 165).
//!
//! Two callers include this file with `#[path]`, so what the docs page prints and what the gate
//! demands come from the same parse and cannot drift apart:
//! - `preview/build.rs` renders the props reference table of every component docs page from it;
//! - `scripts/props-scan/main.rs` (compiled by `scripts/check-props-docs.sh`) lists every public prop
//!   field with no doc comment, so a prop cannot ship undocumented.
//!
//! It is a small scan, not `syn`: only column-0 `#[derive(.. Props ..)] pub struct` items are read (a
//! struct inside `mod tests` is indented and is not a public prop surface), and a field is whatever sits
//! between top-level commas of the struct body. `std` only, so the gate can build it with bare `rustc`.
#![allow(dead_code)]

/// One field of a props struct (pub or not: the builder setter is public either way).
#[derive(Debug, Clone)]
pub struct PropField {
    pub name: String,
    /// The type exactly as written (whitespace collapsed).
    pub ty: String,
    /// The expression of `#[props(default = ..)]`, `Some("")` for a bare `#[props(default)]`.
    pub default: Option<String>,
    /// `#[props(extends = X)]`: not a prop but a spread of X's attributes.
    pub extends: Option<String>,
    /// The `///` lines, joined with newlines. Empty when the field has none.
    pub doc: String,
    /// 1-based line of the field in its file.
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct PropsStruct {
    pub name: String,
    /// `#[component] fn Name(args)` rather than a `#[derive(Props)]` struct.
    pub is_fn: bool,
    pub doc: String,
    pub line: usize,
    pub fields: Vec<PropField>,
}

/// Every top-level `#[derive(.. Props ..)] pub struct` in `source`.
pub fn scan_props(source: &str) -> Vec<PropsStruct> {
    let lines: Vec<&str> = source.lines().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if line.starts_with("#[component") {
            if let Some((item, next)) = scan_component_fn(&lines, i) {
                out.push(item);
                i = next;
                continue;
            }
        }
        if !(line.starts_with("#[derive(") && line.contains("Props")) {
            i += 1;
            continue;
        }
        // Doc lines directly above the derive (attributes in between are skipped).
        let mut doc_lines = Vec::new();
        let mut j = i;
        while j > 0 {
            let above = lines[j - 1].trim_start();
            if let Some(text) = above.strip_prefix("///") {
                doc_lines.push(text.strip_prefix(' ').unwrap_or(text).to_string());
            } else if !above.starts_with("#[") {
                break;
            }
            j -= 1;
        }
        doc_lines.reverse();
        // Walk to the `pub struct` line.
        let mut k = i + 1;
        while k < lines.len() && !lines[k].starts_with("pub struct ") {
            if !lines[k].trim_start().starts_with("#[") && !lines[k].trim_start().starts_with("///")
            {
                break;
            }
            k += 1;
        }
        if k >= lines.len() || !lines[k].starts_with("pub struct ") {
            i += 1;
            continue;
        }
        let header = lines[k];
        let name: String = header["pub struct ".len()..]
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !header.contains('{') {
            i = k + 1; // unit or tuple struct: no named fields
            continue;
        }
        // Body: from after the `{` of the header to its matching `}`.
        let mut body = String::new();
        let mut body_lines: Vec<usize> = Vec::new(); // line number per body char start of each line
        let mut depth = 0i32;
        let mut m = k;
        let mut finished = false;
        while m < lines.len() && !finished {
            let text = lines[m];
            let mut start = 0;
            if m == k {
                start = text.find('{').unwrap() + 1;
                depth = 1;
            }
            let mut piece = String::new();
            let mut in_str = false;
            let mut prev = ' ';
            for (idx, ch) in text.char_indices().skip_while(|(idx, _)| *idx < start) {
                if in_str {
                    if ch == '"' && prev != '\\' {
                        in_str = false;
                    }
                } else if ch == '"' {
                    in_str = true;
                } else if ch == '{' {
                    depth += 1;
                } else if ch == '}' {
                    depth -= 1;
                    if depth == 0 {
                        piece.push_str("");
                        finished = true;
                        let _ = idx;
                        break;
                    }
                }
                piece.push(ch);
                prev = ch;
            }
            body_lines.push(body.len());
            body.push_str(&piece);
            body.push('\n');
            m += 1;
        }
        let first_body_line = k + 1;
        out.push(PropsStruct {
            name,
            is_fn: false,
            doc: doc_lines.join("\n"),
            line: i + 1,
            fields: parse_fields(&body, &body_lines, first_body_line),
        });
        i = m;
    }
    out
}

/// `#[component] pub fn Name(a: A, #[props(default)] b: B, ..)`: the arguments are the props, with the same
/// attribute and doc-comment shapes as a struct field. `fn Name(props: NameProps)` is the derive form
/// (covered by its struct) and yields nothing. Returns the item and the line to resume at.
fn scan_component_fn(lines: &[&str], at: usize) -> Option<(PropsStruct, usize)> {
    let mut k = at + 1;
    while k < lines.len()
        && (lines[k].trim_start().starts_with("#[") || lines[k].trim_start().starts_with("///"))
    {
        k += 1;
    }
    let header = *lines.get(k)?;
    let rest = header
        .strip_prefix("pub fn ")
        .or_else(|| header.strip_prefix("fn "))?;
    let name: String = rest
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if !name.chars().next()?.is_uppercase() {
        return None;
    }
    let mut body = String::new();
    let mut line_starts = Vec::new();
    let mut depth = 0i32;
    let mut m = k;
    let mut opened = false;
    let mut done = false;
    while m < lines.len() && !done {
        let text = lines[m];
        let mut piece = String::new();
        let mut in_str = false;
        let mut prev = ' ';
        for ch in text.chars() {
            if in_str {
                if ch == '"' && prev != '\\' {
                    in_str = false;
                }
            } else if ch == '"' {
                in_str = true;
            } else if ch == '(' {
                depth += 1;
                if !opened {
                    opened = true;
                    prev = ch;
                    continue;
                }
            } else if ch == ')' {
                depth -= 1;
                if opened && depth == 0 {
                    done = true;
                    break;
                }
            }
            if opened {
                piece.push(ch);
            }
            prev = ch;
        }
        if opened {
            line_starts.push(body.len());
            body.push_str(&piece);
            body.push('\n');
        }
        m += 1;
    }
    let first_line = if lines[k].contains('(') { k + 1 } else { k + 2 };
    let fields = parse_fields(&body, &line_starts, first_line);
    if fields.len() == 1 && fields[0].name == "props" {
        return None;
    }
    let mut doc_lines = Vec::new();
    let mut j = at;
    while j > 0 {
        let above = lines[j - 1].trim_start();
        if let Some(text) = above.strip_prefix("///") {
            doc_lines.push(text.strip_prefix(' ').unwrap_or(text).to_string());
        } else if !above.starts_with("#[") {
            break;
        }
        j -= 1;
    }
    doc_lines.reverse();
    // Doc lines between `#[component]` and `fn` count too (`#[component]` is often below the doc).
    Some((
        PropsStruct {
            name,
            is_fn: true,
            doc: doc_lines.join("\n"),
            line: at + 1,
            fields,
        },
        m,
    ))
}

fn parse_fields(body: &str, line_starts: &[usize], first_line: usize) -> Vec<PropField> {
    let chars: Vec<char> = body.chars().collect();
    // `line_starts` are byte offsets; fields only need an approximate line, so map via byte offsets.
    let line_of = |byte: usize| -> usize {
        let mut line = 0;
        for (n, start) in line_starts.iter().enumerate() {
            if *start <= byte {
                line = n;
            }
        }
        first_line + line
    };
    let byte_offsets: Vec<usize> = {
        let mut acc = 0;
        chars
            .iter()
            .map(|c| {
                let here = acc;
                acc += c.len_utf8();
                here
            })
            .collect()
    };

    let mut fields = Vec::new();
    let mut p = 0;
    let mut doc: Vec<String> = Vec::new();
    let mut default: Option<String> = None;
    let mut extends: Option<String> = None;
    while p < chars.len() {
        while p < chars.len() && chars[p].is_whitespace() {
            p += 1;
        }
        if p >= chars.len() {
            break;
        }
        if chars[p] == '/' && chars.get(p + 1) == Some(&'/') {
            let end = chars[p..]
                .iter()
                .position(|c| *c == '\n')
                .map_or(chars.len(), |n| p + n);
            let text: String = chars[p..end].iter().collect();
            if let Some(d) = text.strip_prefix("///") {
                doc.push(d.strip_prefix(' ').unwrap_or(d).to_string());
            }
            p = end;
            continue;
        }
        if chars[p] == '#' && chars.get(p + 1) == Some(&'[') {
            let (text, end) = read_balanced(&chars, p + 1, '[', ']');
            let inner = text.trim();
            if let Some(args) = inner
                .strip_prefix("props(")
                .and_then(|s| s.strip_suffix(')'))
            {
                for part in split_top_level(args, ',') {
                    let part = part.trim();
                    if part == "default" {
                        default = Some(String::new());
                    } else if let Some(expr) = part.strip_prefix("default") {
                        let expr = expr.trim_start();
                        if let Some(expr) = expr.strip_prefix('=') {
                            default = Some(collapse_ws(expr.trim()));
                        }
                    } else if let Some(rest) = part.strip_prefix("extends") {
                        if let Some(target) = rest.trim_start().strip_prefix('=') {
                            extends = Some(target.trim().to_string());
                        }
                    }
                }
            } else if let Some(d) = inner.strip_prefix("doc = ") {
                doc.push(d.trim().trim_matches('"').to_string());
            }
            p = end;
            continue;
        }
        // A field: up to the next top-level comma.
        let start = p;
        let mut depth = 0i32;
        let mut in_str = false;
        let mut prev = ' ';
        while p < chars.len() {
            let ch = chars[p];
            if in_str {
                if ch == '"' && prev != '\\' {
                    in_str = false;
                }
            } else {
                match ch {
                    '"' => in_str = true,
                    '(' | '[' | '{' | '<' => depth += 1,
                    ')' | ']' | '}' => depth -= 1,
                    '>' if prev != '-' => depth -= 1,
                    ',' if depth <= 0 => break,
                    _ => {}
                }
            }
            prev = ch;
            p += 1;
        }
        let text: String = chars[start..p].iter().collect();
        p += 1; // the comma
        let text = text.trim();
        // Visibility is not a filter: dioxus' generated builder setter is `pub` for every field, so a
        // private field is still a prop a caller can set.
        let rest = if let Some(r) = text.strip_prefix("pub(") {
            r.split_once(')').map_or("", |(_, r)| r).trim_start()
        } else if let Some(r) = text.strip_prefix("pub ") {
            r.trim_start()
        } else {
            text
        };
        {
            if let Some((name, ty)) = rest.split_once(':') {
                // `mut props: XProps` binds mutably but is still the argument `props`.
                let name = name.trim();
                let name = name.strip_prefix("mut ").unwrap_or(name).trim_start();
                fields.push(PropField {
                    name: name.trim_start_matches("r#").to_string(),
                    ty: collapse_ws(ty.trim()),
                    default: default.take(),
                    extends: extends.take(),
                    doc: doc.join("\n"),
                    line: line_of(byte_offsets[start]),
                });
            }
        }
        doc.clear();
        default = None;
        extends = None;
    }
    fields
}

/// `chars[open]` is the opening bracket; returns the text between the matching pair and the index
/// just past the closer.
fn read_balanced(chars: &[char], open: usize, o: char, c: char) -> (String, usize) {
    let mut depth = 0;
    let mut in_str = false;
    let mut prev = ' ';
    let mut p = open;
    let mut text = String::new();
    while p < chars.len() {
        let ch = chars[p];
        if in_str {
            if ch == '"' && prev != '\\' {
                in_str = false;
            }
        } else if ch == '"' {
            in_str = true;
        } else if ch == o {
            depth += 1;
            if depth == 1 {
                p += 1;
                prev = ch;
                continue;
            }
        } else if ch == c {
            depth -= 1;
            if depth == 0 {
                return (text, p + 1);
            }
        }
        text.push(ch);
        prev = ch;
        p += 1;
    }
    (text, p)
}

fn split_top_level(text: &str, sep: char) -> Vec<String> {
    let mut parts = Vec::new();
    let mut cur = String::new();
    let mut depth = 0i32;
    let mut in_str = false;
    let mut prev = ' ';
    for ch in text.chars() {
        if in_str {
            if ch == '"' && prev != '\\' {
                in_str = false;
            }
        } else {
            match ch {
                '"' => in_str = true,
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => depth -= 1,
                c if c == sep && depth == 0 => {
                    parts.push(std::mem::take(&mut cur));
                    prev = ch;
                    continue;
                }
                _ => {}
            }
        }
        cur.push(ch);
        prev = ch;
    }
    parts.push(cur);
    parts
}

fn collapse_ws(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The type a reader passes: `ReadSignal<T>` and `Option<T>` show `T` (a plain `T` is accepted for a
/// `ReadSignal<T>`, and an optional prop is simply left out). Second value: the prop is optional by
/// type (`Option<T>` / `ReadSignal<Option<T>>`, which dioxus defaults to `None`).
pub fn display_type(ty: &str) -> (String, bool) {
    let mut ty = ty.trim().to_string();
    let mut optional = false;
    loop {
        let before = ty.clone();
        for wrapper in ["ReadSignal", "ReadOnlySignal", "Option"] {
            if let Some(inner) = ty
                .strip_prefix(wrapper)
                .and_then(|r| r.strip_prefix('<'))
                .and_then(|r| r.strip_suffix('>'))
            {
                let inner = split_top_level_angle(inner)
                    .into_iter()
                    .next()
                    .unwrap_or_default();
                if wrapper == "Option" {
                    optional = true;
                }
                ty = inner.trim().to_string();
            }
        }
        if ty == before {
            break;
        }
    }
    (ty, optional)
}

/// First-level comma split that respects `<..>` (for `ReadSignal<T, Storage>`).
fn split_top_level_angle(text: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut cur = String::new();
    let mut depth = 0i32;
    let mut prev = ' ';
    for ch in text.chars() {
        match ch {
            '<' | '(' | '[' => depth += 1,
            '>' if prev != '-' => depth -= 1,
            ')' | ']' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(std::mem::take(&mut cur));
                prev = ch;
                continue;
            }
            _ => {}
        }
        cur.push(ch);
        prev = ch;
    }
    parts.push(cur);
    parts
}

/// What the table's Default column says: the written default with `ReadSignal::new(Signal::new(..))`
/// peeled off, `None` for an optional type, a plain-words default for a bare `#[props(default)]`, and
/// "required" for a field with neither.
pub fn display_default(field: &PropField) -> String {
    let (inner, optional) = display_type(&field.ty);
    if let Some(expr) = &field.default {
        if expr.is_empty() {
            return bare_default(&inner, optional).to_string();
        }
        let mut expr = expr.as_str();
        while let Some(rest) = expr
            .strip_prefix("ReadSignal::new(Signal::new(")
            .and_then(|r| r.strip_suffix("))"))
        {
            expr = rest;
        }
        return expr.to_string();
    }
    if optional {
        return "None".to_string();
    }
    if field.name == "children" && inner == "Element" {
        return "empty".to_string();
    }
    "required".to_string()
}

fn bare_default(inner: &str, optional: bool) -> &'static str {
    if optional {
        return "None";
    }
    match inner {
        "bool" => "false",
        "String" => "\"\"",
        "Element" => "empty",
        "f32" | "f64" | "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "usize"
        | "isize" => "0",
        t if t.starts_with("Callback<") => "no-op",
        t if t.starts_with("Vec<") => "empty",
        _ => "Default::default()",
    }
}
