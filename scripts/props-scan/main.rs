//! `scripts/check-props-docs.sh` helper: prints `path:line: Struct.field` for every public prop field
//! of a `#[derive(Props)]` struct that has no doc comment. Shares its parser with `preview/build.rs`.
#[path = "../../preview/props_scan.rs"]
mod props_scan;

fn main() {
    let mut missing = 0usize;
    let mut structs = 0usize;
    let mut fields = 0usize;
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
                    missing += 1;
                    println!("{path}:{}: {}.{} has no doc comment", f.line, s.name, f.name);
                }
            }
        }
    }
    eprintln!("props-scan: {structs} props structs, {fields} prop fields, {missing} undocumented");
    std::process::exit(if missing == 0 { 0 } else { 1 });
}
