//! `scripts/check-wrapper-props.sh` helper (row 391): prints `path<TAB>module::Component.prop` for every prop of a
//! primitive component that a themed wrapper renders without passing. Shares its parser with `preview/build.rs`, so
//! the props it checks are the rows of the primitive's props table on the component's docs page.
//!
//! Usage: `wrapper-props <primitives/src> <component.rs>...`
//!
//! A wrapper call is `module::Component { .. }` (or a name imported from a primitive module) in a
//! `preview/src/components/*/component.rs`, where `module` resolves through that file's `use dioxus_primitives::..`
//! lines. The call passes a prop when the prop's name is a key at the call's top level (`name: ..` or the `name,`
//! shorthand). `children`, the `#[props(extends = ..)]` spreads and `attributes` are not props a call passes by name.
//!
//! A component's header is read by `fn_signatures` from the token stream (comments and literals masked), never from
//! one line of text: `pub fn X(mut props: XProps)` is recognised across any number of lines, with `///` docs or
//! attributes on the argument, with or without `mut`, with generics, and a `#[cfg(test)] mod` is skipped by its
//! attribute, not by its indentation. `cargo test`-less: `rustc --test` runs the unit tests (`check-wrapper-props.sh`).
#[path = "../../preview/props_scan.rs"]
mod props_scan;

use std::collections::{BTreeSet, HashMap};
use std::path::Path;

/// module -> component (or props struct) -> prop names.
type Catalog = HashMap<String, HashMap<String, Vec<String>>>;

fn rs_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let p = entry.path();
        if p.is_dir() {
            rs_files(&p, out);
        } else if p.extension().is_some_and(|e| e == "rs") {
            out.push(p);
        }
    }
}

fn ident(s: &str) -> String {
    s.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect()
}

/// One parameter of a fn header: the bound name (no attributes, `mut` or `ref`) and its type as written.
#[derive(Debug, PartialEq)]
struct Param {
    name: String,
    ty: String,
}

/// A fn item: its name, the attributes written on it (the text between `#[` and `]`) and its parameters.
#[derive(Debug)]
struct FnSig {
    name: String,
    attrs: Vec<String>,
    params: Vec<Param>,
}

/// `#[component]`, `#[component(..)]` or a path ending in `component`.
fn is_component_attr(attr: &str) -> bool {
    let head = attr.split('(').next().unwrap_or("").trim();
    head.rsplit("::").next() == Some("component")
}

/// `src` with every comment (doc comments included) and the body of every string and char literal replaced by
/// spaces, newlines kept, so a token walk never reads a brace, comma or `fn` that is text.
fn mask(src: &str) -> Vec<char> {
    let c: Vec<char> = src.chars().collect();
    let mut out = c.clone();
    let blank = |out: &mut Vec<char>, from: usize, to: usize| {
        for ch in out.iter_mut().take(to).skip(from) {
            if *ch != '\n' {
                *ch = ' ';
            }
        }
    };
    let (mut i, n) = (0, c.len());
    while i < n {
        let prev_ident = i > 0 && (c[i - 1].is_alphanumeric() || c[i - 1] == '_');
        match c[i] {
            '/' if c.get(i + 1) == Some(&'/') => {
                let end = (i..n).find(|&k| c[k] == '\n').unwrap_or(n);
                blank(&mut out, i, end);
                i = end;
            }
            '/' if c.get(i + 1) == Some(&'*') => {
                let (mut depth, mut k) = (1, i + 2);
                while k < n && depth > 0 {
                    if c[k] == '/' && c.get(k + 1) == Some(&'*') {
                        depth += 1;
                        k += 2;
                    } else if c[k] == '*' && c.get(k + 1) == Some(&'/') {
                        depth -= 1;
                        k += 2;
                    } else {
                        k += 1;
                    }
                }
                blank(&mut out, i, k);
                i = k;
            }
            // r"..", r#".."#, br"..": no escapes, closed by a quote and as many hashes as opened it.
            'r' if !prev_ident && matches!(c.get(i + 1), Some('"') | Some('#')) => {
                let hashes = c[i + 1..].iter().take_while(|&&x| x == '#').count();
                if c.get(i + 1 + hashes) != Some(&'"') {
                    i += 1;
                    continue;
                }
                let open = i + 2 + hashes;
                let close = (open..n)
                    .find(|&k| c[k] == '"' && (1..=hashes).all(|h| c.get(k + h) == Some(&'#')))
                    .unwrap_or(n - 1);
                blank(&mut out, open, close);
                i = (close + 1 + hashes).min(n);
            }
            '"' => {
                let mut k = i + 1;
                while k < n && c[k] != '"' {
                    k += if c[k] == '\\' { 2 } else { 1 };
                }
                blank(&mut out, i + 1, k.min(n));
                i = k + 1;
            }
            // A char literal (`'x'`, `'\n'`, `'"'`); anything else after a quote is a lifetime.
            '\'' => {
                let end = if c.get(i + 1) == Some(&'\\') {
                    (i + 2..n.min(i + 12)).find(|&k| c[k] == '\'')
                } else if c.get(i + 2) == Some(&'\'') {
                    Some(i + 2)
                } else {
                    None
                };
                match end {
                    Some(e) => {
                        blank(&mut out, i + 1, e);
                        i = e + 1;
                    }
                    None => i += 1,
                }
            }
            _ => i += 1,
        }
    }
    out
}

/// The text between `open` (an opening bracket) and its match, and the index after the match. A `>` that closes
/// nothing (`->`, `=>`) is not counted when the brackets are angle brackets.
fn balanced(c: &[char], open: usize) -> (String, usize) {
    let (o, cl) = match c[open] {
        '(' => ('(', ')'),
        '[' => ('[', ']'),
        '{' => ('{', '}'),
        _ => ('<', '>'),
    };
    let (mut depth, mut k) = (0i32, open);
    while k < c.len() {
        if c[k] == o {
            depth += 1;
        } else if c[k] == cl && !(cl == '>' && k > 0 && (c[k - 1] == '-' || c[k - 1] == '=')) {
            depth -= 1;
            if depth == 0 {
                return (c[open + 1..k].iter().collect(), k + 1);
            }
        }
        k += 1;
    }
    (c[open + 1..].iter().collect(), c.len())
}

/// `text` cut at the commas outside every bracket pair (`->` and `=>` do not close an angle bracket).
fn split_commas(text: &str) -> Vec<String> {
    let c: Vec<char> = text.chars().collect();
    let (mut depth, mut seg, mut out) = (0i32, String::new(), Vec::new());
    for (k, &ch) in c.iter().enumerate() {
        match ch {
            '(' | '[' | '{' | '<' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            '>' if k == 0 || !(c[k - 1] == '-' || c[k - 1] == '=') => depth -= 1,
            ',' if depth == 0 => {
                out.push(std::mem::take(&mut seg));
                continue;
            }
            _ => {}
        }
        seg.push(ch);
    }
    out.push(seg);
    out.into_iter().filter(|s| !s.trim().is_empty()).collect()
}

/// One parameter, `#[attr] mut name: Type`, or `None` for a pattern that binds no single name.
fn parse_param(text: &str) -> Option<Param> {
    let mut rest: Vec<char> = text.trim().chars().collect();
    loop {
        while rest.first().is_some_and(|ch| ch.is_whitespace()) {
            rest.remove(0);
        }
        if rest.first() == Some(&'#') && rest.get(1) == Some(&'[') {
            let (_, after) = balanced(&rest, 1);
            rest.drain(..after);
            continue;
        }
        let word: String = rest.iter().take_while(|ch| ch.is_alphanumeric() || **ch == '_').collect();
        if (word == "mut" || word == "ref") && rest.get(word.len()).is_some_and(|ch| ch.is_whitespace()) {
            rest.drain(..word.len());
            continue;
        }
        break;
    }
    let text: String = rest.iter().collect();
    let name = ident(&text);
    let after = text[name.len()..].trim_start();
    let ty = after.strip_prefix(':').filter(|t| !t.starts_with(':'))?;
    (!name.is_empty()).then(|| Param {
        name: name.trim_start_matches("r#").to_string(),
        ty: ty.split_whitespace().collect::<Vec<_>>().join(" "),
    })
}

/// Every fn item of `source` with its attributes and parameters, read from the token stream: the header may span
/// any number of lines, its arguments may carry `///` docs and attributes, `mut` and generics are fine, and the
/// body is skipped. A `#[cfg(test)] mod` is skipped by its attribute.
fn fn_signatures(source: &str) -> Vec<FnSig> {
    let c = mask(source);
    let n = c.len();
    let (mut out, mut pending, mut i) = (Vec::new(), Vec::<String>::new(), 0usize);
    while i < n {
        let ch = c[i];
        if ch.is_whitespace() {
            i += 1;
        } else if ch == '#' {
            let mut k = i + 1;
            let inner = c.get(k) == Some(&'!');
            if inner {
                k += 1;
            }
            while c.get(k).is_some_and(|x| x.is_whitespace()) {
                k += 1;
            }
            if c.get(k) == Some(&'[') {
                let (attr, after) = balanced(&c, k);
                if !inner {
                    pending.push(attr.split_whitespace().collect::<Vec<_>>().join(" "));
                }
                i = after;
            } else {
                i += 1;
            }
        } else if ch.is_alphabetic() || ch == '_' {
            let word: String = c[i..].iter().take_while(|x| x.is_alphanumeric() || **x == '_').collect();
            i += word.len();
            match word.as_str() {
                // Modifiers and visibility keep the attributes pending (`pub(crate)`'s parens are skipped below).
                "pub" | "async" | "const" | "unsafe" | "extern" | "default" => {}
                "mod" if pending.iter().any(|a| a.replace(' ', "").contains("cfg(test)")) => {
                    while i < n && c[i] != '{' && c[i] != ';' {
                        i += 1;
                    }
                    if i < n && c[i] == '{' {
                        i = balanced(&c, i).1;
                    }
                    pending.clear();
                }
                "fn" => {
                    let mut k = i;
                    while c.get(k).is_some_and(|x| x.is_whitespace()) {
                        k += 1;
                    }
                    let name: String = c[k..].iter().take_while(|x| x.is_alphanumeric() || **x == '_').collect();
                    k += name.len();
                    while c.get(k).is_some_and(|x| x.is_whitespace()) {
                        k += 1;
                    }
                    if c.get(k) == Some(&'<') {
                        k = balanced(&c, k).1;
                        while c.get(k).is_some_and(|x| x.is_whitespace()) {
                            k += 1;
                        }
                    }
                    if name.is_empty() || c.get(k) != Some(&'(') {
                        // `fn(A) -> B` type, not an item.
                        pending.clear();
                        continue;
                    }
                    let (args, after) = balanced(&c, k);
                    let params = split_commas(&args).iter().filter_map(|a| parse_param(a)).collect();
                    out.push(FnSig {
                        name: name.trim_start_matches("r#").to_string(),
                        attrs: std::mem::take(&mut pending),
                        params,
                    });
                    // Skip the return type and where clause, then the body.
                    i = after;
                    while i < n && c[i] != '{' && c[i] != ';' {
                        i += 1;
                    }
                    if i < n && c[i] == '{' {
                        i = balanced(&c, i).1;
                    }
                }
                _ => pending.clear(),
            }
        } else if ch == '(' && !pending.is_empty() {
            // `pub(crate)` / `pub(in path)`.
            i = balanced(&c, i).1;
        } else {
            if matches!(ch, '{' | '}' | ';') {
                pending.clear();
            }
            i += 1;
        }
    }
    out
}


/// The `(module, source)` of every `.rs` under the primitives root (`select/components/x.rs` is module `select`).
fn load_sources(root: &Path) -> Vec<(String, String)> {
    let mut files = Vec::new();
    rs_files(root, &mut files);
    files
        .iter()
        .map(|path| {
            let rel = path.strip_prefix(root).unwrap();
            let module = rel.components().next().unwrap().as_os_str().to_string_lossy().trim_end_matches(".rs").to_string();
            (module, std::fs::read_to_string(path).unwrap())
        })
        .collect()
}

/// Every primitive component, by its top-level module.
fn catalog(sources: &[(String, String)]) -> (Catalog, Catalog) {
    let mut structs: HashMap<String, HashMap<String, Vec<String>>> = HashMap::new();
    let mut fns: HashMap<String, Vec<(String, Option<String>, Vec<String>)>> = HashMap::new();
    for (module, source) in sources {
        let props = |fields: &[props_scan::PropField]| -> Vec<String> {
            fields
                .iter()
                .filter(|f| f.extends.is_none() && f.name != "children" && f.name != "attributes")
                .map(|f| f.name.trim_start_matches("r#").to_string())
                .collect()
        };
        for item in props_scan::scan_props(source) {
            if item.is_fn {
                fns.entry(module.clone()).or_default().push((item.name, None, props(&item.fields)));
            } else {
                structs.entry(module.clone()).or_default().insert(item.name, props(&item.fields));
            }
        }
        // `#[component] pub fn Name(props: SomeProps)`: the derive form, which props_scan leaves to the struct.
        for sig in fn_signatures(source) {
            if !sig.attrs.iter().any(|a| is_component_attr(a)) {
                continue;
            }
            if let Some(first) = sig.params.first().filter(|p| p.name == "props") {
                fns.entry(module.clone()).or_default().push((sig.name, Some(ident(&first.ty)), Vec::new()));
            }
        }
    }
    let mut out: Catalog = HashMap::new();
    for (module, list) in fns {
        for (name, ty, args) in list {
            let fields = match ty {
                Some(ty) => match structs.get(&module).and_then(|s| s.get(&ty)) {
                    Some(f) => f.clone(),
                    None => continue,
                },
                None => args,
            };
            out.entry(module.clone()).or_default().insert(name, fields);
        }
    }
    (out, structs)
}

/// The `{ .. }` block starting at `open`, without its braces; strings and nested blocks skipped as text.
fn block(chars: &[char], open: usize) -> String {
    let (mut depth, mut i, mut in_str) = (0i32, open, false);
    let mut out = String::new();
    while i < chars.len() {
        let c = chars[i];
        if in_str {
            if c == '"' && chars[i - 1] != '\\' {
                in_str = false;
            }
        } else if c == '"' {
            in_str = true;
        } else if c == '{' {
            depth += 1;
            if depth == 1 {
                i += 1;
                continue;
            }
        } else if c == '}' {
            depth -= 1;
            if depth == 0 {
                break;
            }
        }
        out.push(c);
        i += 1;
    }
    out
}

/// The keys at the top level of a call block (`name: ..` and the `name,` shorthand), and whether the block spreads a
/// props struct (`..props`; `..attributes`-style spreads of an attribute list do not count).
fn top_keys(body: &str) -> (BTreeSet<String>, bool) {
    let chars: Vec<char> = body.chars().collect();
    let (mut depth, mut in_str, mut seg) = (0i32, false, String::new());
    let mut segs = Vec::new();
    for (i, &c) in chars.iter().enumerate() {
        if in_str {
            if c == '"' && chars[i - 1] != '\\' {
                in_str = false;
            }
            seg.push(c);
            continue;
        }
        match c {
            '"' => in_str = true,
            '{' | '(' | '[' => depth += 1,
            '}' | ')' | ']' => depth -= 1,
            ',' | '\n' if depth == 0 => {
                segs.push(std::mem::take(&mut seg));
                continue;
            }
            _ => {}
        }
        seg.push(c);
    }
    segs.push(seg);
    let mut keys = BTreeSet::new();
    let mut spreads = false;
    for s in segs {
        let s = s.trim();
        if let Some(rest) = s.strip_prefix("..") {
            let name = ident(rest);
            spreads |= !name.is_empty() && !["attributes", "merged", "attrs"].contains(&name.as_str());
            continue;
        }
        let s = s.strip_prefix("r#").unwrap_or(s);
        let name = ident(s);
        if name.is_empty() {
            continue;
        }
        let after = s[name.len()..].trim_start();
        if after.is_empty() || (after.starts_with(':') && !after.starts_with("::")) {
            keys.insert(name);
        }
    }
    (keys, spreads)
}

/// `path<TAB>module::Component.prop` for every prop of a primitive that a call in `source` (the wrapper at `path`)
/// neither passes by name nor spreads.
fn dropped(path: &str, source: &str, cat: &Catalog, structs: &Catalog) -> Vec<String> {
    let mut out = Vec::new();
    // alias -> module, and a bare name imported from a module -> (module, component).
    let mut modules: HashMap<String, String> = HashMap::new();
    let mut names: HashMap<String, (String, String)> = HashMap::new();
    let flat = source.replace('\n', " ");
    for stmt in flat.split(';') {
        let Some(rest) = stmt.trim().strip_prefix("use dioxus_primitives::") else { continue };
        let rest = rest.trim();
        let (head, list) = match rest.find('{') {
            Some(b) => (rest[..b].trim_end_matches("::").trim().to_string(), Some(rest[b + 1..].trim_end_matches('}').to_string())),
            None => (rest.to_string(), None),
        };
        match list {
            None => {
                let (m, alias) = match head.split_once(" as ") {
                    Some((m, a)) => (m.trim().to_string(), a.trim().to_string()),
                    None => (head.clone(), head.rsplit("::").next().unwrap().to_string()),
                };
                let top = m.split("::").next().unwrap().to_string();
                if cat.contains_key(&top) && !m.contains("::") {
                    modules.insert(alias, top);
                } else if let Some((mm, n)) = m.rsplit_once("::") {
                    names.insert(alias, (mm.split("::").next().unwrap().to_string(), n.to_string()));
                }
            }
            Some(list) => {
                for item in list.split(',') {
                    let item = item.trim();
                    if item.is_empty() {
                        continue;
                    }
                    let (n, alias) = match item.split_once(" as ") {
                        Some((n, a)) => (n.trim().to_string(), a.trim().to_string()),
                        None => (item.to_string(), item.rsplit("::").next().unwrap().to_string()),
                    };
                    if head.is_empty() {
                        if cat.contains_key(&n) {
                            modules.insert(alias, n);
                        }
                    } else if n == "self" {
                        modules.insert(if item.contains(" as ") { alias } else { head.clone() }, head.split("::").next().unwrap().to_string());
                    } else {
                        names.insert(alias, (head.split("::").next().unwrap().to_string(), n));
                    }
                }
            }
        }
    }
    // A local `fn Name(props: module::XProps)` / `fn Name(props: XProps)` (XProps imported from a primitive module)
    // takes the primitive's own props, so a call of it must pass them on too.
    let mut local: HashMap<String, Vec<String>> = HashMap::new();
    for sig in fn_signatures(source) {
        let Some(first) = sig.params.first().filter(|p| p.name == "props") else { continue };
        let ty: String = first.ty.chars().take_while(|c| c.is_alphanumeric() || *c == '_' || *c == ':').collect();
        let resolved = match ty.rsplit_once("::") {
            Some((m, t)) => modules.get(m.rsplit("::").next().unwrap()).map(|m| (m.clone(), t.to_string())),
            None => names.get(&ty).cloned(),
        };
        if let Some(fields) = resolved.and_then(|(m, t)| structs.get(&m).and_then(|s| s.get(&t))) {
            local.insert(sig.name, fields.clone());
        }
    }
    // Calls inside the unit tests are fixtures, not the wrapper's surface.
    let code = source.split("\n#[cfg(test)]\nmod tests").next().unwrap();
    let chars: Vec<char> = code.chars().collect();
    let mut seen = BTreeSet::new();
    let mut i = 0;
    while i < chars.len() {
        // A call: `Ident {` or `module::Ident {`, not part of a longer path or a `fn`/`struct` header.
        if chars[i].is_uppercase() && (i == 0 || !(chars[i - 1].is_alphanumeric() || chars[i - 1] == '_')) {
            let name: String = chars[i..].iter().take_while(|c| c.is_alphanumeric() || **c == '_').collect();
            let end = i + name.len();
            let mut j = end;
            while j < chars.len() && chars[j] == ' ' {
                j += 1;
            }
            if j < chars.len() && chars[j] == '{' {
                let before: String = chars[..i].iter().collect();
                let target = if let Some(prefix) = before.strip_suffix("::") {
                    let m: String = prefix.chars().rev().take_while(|c| c.is_alphanumeric() || *c == '_').collect::<Vec<_>>().into_iter().rev().collect();
                    modules.get(&m).map(|m| (m.clone(), name.clone()))
                } else {
                    let prev_word: String = before.trim_end().chars().rev().take_while(|c| c.is_alphanumeric()).collect::<Vec<_>>().into_iter().rev().collect();
                    if ["struct", "enum", "impl", "for", "fn", "trait"].contains(&prev_word.as_str()) { None } else { names.get(&name).cloned() }
                };
                let target = target.or_else(|| {
                    let local_call = !before.ends_with("::") && local.contains_key(&name);
                    local_call.then(|| ("local".to_string(), name.clone()))
                });
                if let Some((module, comp)) = target {
                    let props = if module == "local" { local.get(&comp) } else { cat.get(&module).and_then(|c| c.get(&comp)) };
                    if let Some(props) = props {
                        // `..props` passes the whole props struct (the wrapper takes the primitive's own).
                        let (keys, spreads_all) = top_keys(&block(&chars, j));
                        for p in props.iter().filter(|_| !spreads_all) {
                            if !keys.contains(p) && seen.insert(format!("{module}::{comp}.{p}")) {
                                out.push(format!("{path}\t{module}::{comp}.{p}"));
                            }
                        }
                    }
                }
            }
            i = end;
            continue;
        }
        i += 1;
    }
    out
}

fn main() {
    let mut args = std::env::args().skip(1);
    let root = args.next().expect("usage: wrapper-props <primitives/src> <component.rs>...");
    let (cat, structs) = catalog(&load_sources(Path::new(&root)));
    for path in args {
        let source = std::fs::read_to_string(&path).unwrap();
        for line in dropped(&path, &source, &cat, &structs) {
            println!("{line}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sig(src: &str, name: &str) -> FnSig {
        fn_signatures(src).into_iter().find(|s| s.name == name).unwrap_or_else(|| panic!("no fn {name} in {src:?}"))
    }

    fn first(src: &str, name: &str) -> Option<Param> {
        sig(src, name).params.into_iter().next()
    }

    fn props(ty: &str) -> Option<Param> {
        Some(Param { name: "props".into(), ty: ty.into() })
    }

    /// Every spelling of the header of a component that takes its props struct.
    const HEADERS: &[(&str, &str)] = &[
        ("single line", "pub fn Foo(props: FooProps) -> Element { rsx! {} }"),
        ("single line, mut", "pub fn Foo(mut props: FooProps) -> Element { rsx! {} }"),
        ("multi-line", "pub fn Foo(\n    props: FooProps,\n) -> Element {\n    rsx! {}\n}"),
        ("multi-line, mut", "pub fn Foo(\n    mut props: FooProps,\n) -> Element {\n    rsx! {}\n}"),
        ("doc comment on the argument", "pub fn Foo(\n    /// The props.\n    /// Two lines.\n    mut props: FooProps,\n) -> Element {\n    rsx! {}\n}"),
        ("attribute on the argument", "pub fn Foo(\n    #[allow(unused_mut)]\n    mut props: FooProps,\n) -> Element {\n    rsx! {}\n}"),
        ("doc and attribute", "pub fn Foo(\n    /// doc\n    #[allow(unused_mut)]\n    /// more doc\n    props: FooProps\n) -> Element {\n    rsx! {}\n}"),
        ("block comment", "pub fn Foo(/* the props */ props: FooProps) -> Element { rsx! {} }"),
        ("pub(crate) and generics", "pub(crate) fn Foo<T: Fn() -> u8>(\n    props: FooProps,\n) -> Element\nwhere\n    T: Clone,\n{\n    rsx! {}\n}"),
        ("name and paren on separate lines", "pub fn Foo\n(\n    props: FooProps) -> Element { rsx! {} }"),
    ];

    #[test]
    fn a_component_header_is_read_however_it_is_formatted() {
        for (label, header) in HEADERS {
            let src = format!("/// Docs.\n#[component]\n{header}\n");
            let s = sig(&src, "Foo");
            assert_eq!(s.params.len(), 1, "{label}: {:?}", s.params);
            assert_eq!(s.params.into_iter().next(), props("FooProps"), "{label}");
            assert!(s.attrs.iter().any(|a| is_component_attr(a)), "{label}: {:?}", s.attrs);
        }
    }

    #[test]
    fn every_formatting_reports_the_same_dropped_props() {
        let primitive = |header: &str| {
            format!("#[derive(Props, Clone, PartialEq)]\npub struct FooProps {{\n    /// a\n    pub a: u8,\n    /// b\n    pub b: u8,\n    children: Element,\n}}\n\n#[component]\n{header}\n")
        };
        let wrapper = "use dioxus_primitives::foo::{self, Foo};\n\n#[component]\npub fn Themed(a: u8) -> Element {\n    rsx! {\n        foo::Foo {\n            a,\n            \"x\"\n        }\n    }\n}\n";
        for (label, header) in HEADERS {
            let sources = vec![("foo".to_string(), primitive(header))];
            let (cat, structs) = catalog(&sources);
            assert!(cat["foo"].contains_key("Foo"), "{label}: Foo is not in the catalog");
            assert_eq!(dropped("w.rs", wrapper, &cat, &structs), vec!["w.rs\tfoo::Foo.b".to_string()], "{label}");
        }
    }

    #[test]
    fn a_local_component_taking_the_primitives_props_is_checked_too() {
        let primitive = "#[derive(Props, Clone, PartialEq)]\npub struct FooProps {\n    /// a\n    pub a: u8,\n    /// b\n    pub b: u8,\n}\n\n#[component]\npub fn Foo(props: FooProps) -> Element { rsx! {} }\n";
        let wrapper = "use dioxus_primitives::foo::{Foo, FooProps};\n\n#[component]\npub fn Inner(\n    /// The primitive's props.\n    mut props: FooProps,\n) -> Element {\n    rsx! { Foo { a: props.a } }\n}\n\n#[component]\npub fn Outer() -> Element {\n    rsx! {\n        Inner {\n            a: 1,\n        }\n    }\n}\n";
        let (cat, structs) = catalog(&[("foo".to_string(), primitive.to_string())]);
        let got = dropped("w.rs", wrapper, &cat, &structs);
        assert!(got.contains(&"w.rs\tfoo::Foo.b".to_string()), "{got:?}");
        assert!(got.contains(&"w.rs\tlocal::Inner.b".to_string()), "{got:?}");
    }

    #[test]
    fn text_that_looks_like_a_header_is_not_one() {
        let src = "// pub fn Ghost(props: GhostProps) -> Element {}\n/* fn Block(props: BlockProps) {} */\nconst S: &str = \"fn Str(props: StrProps) {}\";\nconst R: &str = r#\"fn Raw(props: RawProps) {}\"#;\nconst C: char = '\"';\n/// fn Doc(props: DocProps) {}\npub fn Real(props: RealProps) -> Element { let f: fn(u8) -> u8 = |x| x; rsx! {} }\n";
        let names: Vec<String> = fn_signatures(src).into_iter().map(|s| s.name).collect();
        assert_eq!(names, vec!["Real".to_string()]);
    }

    #[test]
    fn the_body_and_a_test_module_are_skipped() {
        let src = "#[component]\npub fn A(props: AProps) -> Element {\n    fn inner(props: InnerProps) {}\n    rsx! {}\n}\n\n#[cfg(test)]\nmod tests {\n    #[component]\n    fn B(props: BProps) -> Element { rsx! {} }\n}\n\n#[component]\npub fn C(props: CProps) -> Element { rsx! {} }\n";
        let names: Vec<String> = fn_signatures(src).into_iter().map(|s| s.name).collect();
        assert_eq!(names, vec!["A".to_string(), "C".to_string()]);
    }

    #[test]
    fn arguments_are_read_by_name_and_type() {
        let src = "#[component]\npub fn Foo(\n    /// doc, with a comma\n    a: Option<(u8, u8)>,\n    #[props(default)] mut b: Callback<u8, ()>,\n    c: Box<dyn Fn(u8) -> u8>,\n    (x, y): (u8, u8),\n) -> Element { rsx! {} }\n";
        let p = sig(src, "Foo").params;
        let got: Vec<(&str, &str)> = p.iter().map(|p| (p.name.as_str(), p.ty.as_str())).collect();
        assert_eq!(got, vec![("a", "Option<(u8, u8)>"), ("b", "Callback<u8, ()>"), ("c", "Box<dyn Fn(u8) -> u8>")]);
        assert_eq!(first("fn g(mut props: P) {}", "g"), props("P"));
    }

    #[test]
    fn component_attributes() {
        assert!(is_component_attr("component"));
        assert!(is_component_attr("component(no_case_check)"));
        assert!(is_component_attr("dioxus::prelude::component"));
        assert!(!is_component_attr("allow(clippy::component)"));
        assert!(!is_component_attr("derive(Props)"));
    }
}
