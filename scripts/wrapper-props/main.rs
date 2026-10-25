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

/// Every primitive component, by its top-level module (`select/components/x.rs` is module `select`).
fn catalog(root: &Path) -> (Catalog, Catalog) {
    let mut files = Vec::new();
    rs_files(root, &mut files);
    let mut structs: HashMap<String, HashMap<String, Vec<String>>> = HashMap::new();
    let mut fns: HashMap<String, Vec<(String, Option<String>, Vec<String>)>> = HashMap::new();
    for path in &files {
        let rel = path.strip_prefix(root).unwrap();
        let module = rel
            .components()
            .next()
            .unwrap()
            .as_os_str()
            .to_string_lossy()
            .trim_end_matches(".rs")
            .to_string();
        let source = std::fs::read_to_string(path).unwrap();
        let props = |fields: &[props_scan::PropField]| -> Vec<String> {
            fields
                .iter()
                .filter(|f| f.extends.is_none() && f.name != "children" && f.name != "attributes")
                .map(|f| f.name.trim_start_matches("r#").to_string())
                .collect()
        };
        for item in props_scan::scan_props(&source) {
            if item.is_fn {
                fns.entry(module.clone()).or_default().push((item.name, None, props(&item.fields)));
            } else {
                structs.entry(module.clone()).or_default().insert(item.name, props(&item.fields));
            }
        }
        // `#[component] pub fn Name(props: SomeProps)`: the derive form, which props_scan leaves to the struct.
        let lines: Vec<&str> = source.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if !line.starts_with("#[component") {
                continue;
            }
            let mut k = i + 1;
            while k < lines.len() && (lines[k].trim_start().starts_with("#[") || lines[k].trim_start().starts_with("///")) {
                k += 1;
            }
            let Some(rest) = lines.get(k).and_then(|h| h.strip_prefix("pub fn ")) else { continue };
            let name = ident(rest);
            let rest = rest.replacen("(mut props: ", "(props: ", 1);
            if let Some(p) = rest.find("(props: ") {
                let ty = ident(&rest[p + "(props: ".len()..]);
                fns.entry(module.clone()).or_default().push((name, Some(ty), Vec::new()));
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

fn main() {
    let mut args = std::env::args().skip(1);
    let root = args.next().expect("usage: wrapper-props <primitives/src> <component.rs>...");
    let (cat, structs) = catalog(Path::new(&root));
    for path in args {
        let source = std::fs::read_to_string(&path).unwrap();
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
        for line in source.lines() {
            let t = line.trim_start();
            let Some(rest) = ["pub fn ", "pub(crate) fn ", "fn "].iter().find_map(|p| t.strip_prefix(p)) else { continue };
            let name = ident(rest);
            let Some(at) = rest.find("(props: ").or_else(|| rest.find("(mut props: ")) else { continue };
            let ty = rest[at..].split_once(": ").unwrap().1;
            let ty: String = ty.chars().take_while(|c| c.is_alphanumeric() || *c == '_' || *c == ':').collect();
            let resolved = match ty.rsplit_once("::") {
                Some((m, t)) => modules.get(m.rsplit("::").next().unwrap()).map(|m| (m.clone(), t.to_string())),
                None => names.get(&ty).cloned(),
            };
            if let Some(fields) = resolved.and_then(|(m, t)| structs.get(&m).and_then(|s| s.get(&t))) {
                local.insert(name, fields.clone());
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
                                    println!("{path}\t{module}::{comp}.{p}");
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
    }
}
