//! `scripts/check-props-docs.sh` helper: prints `path:line: Name.field` for every prop with no doc comment,
//! as `STRUCT` (a `#[derive(Props)]` field) or `FN` (an argument of a `#[component] fn`); the gate fails on
//! either, with no baseline. Shares its parser with `preview/build.rs`.
#[path = "../../preview/props_scan.rs"]
mod props_scan;

fn main() {
    let (mut structs, mut fields) = (0usize, 0usize);
    for path in std::env::args().skip(1) {
        let source = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
        for s in props_scan::scan_props(&source) {
            structs += 1;
            for f in s.fields {
                // A spread (`extends = ..`) is documented once, as the table's "plus the global attributes" line.
                if f.extends.is_some() {
                    continue;
                }
                fields += 1;
                if f.doc.trim().is_empty() {
                    let kind = if s.is_fn { "FN" } else { "STRUCT" };
                    println!("{kind}\t{path}\t{}.{}\t{}", s.name, f.name, f.line);
                }
            }
        }
    }
    eprintln!("props-scan: {structs} props structs/component fns, {fields} props");
}
