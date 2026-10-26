//! `scripts/check-slot-docs.sh` and `check-css-vars-defined.sh` helper (backlog row 433): prints one tab-separated
//! line per fact about the component sheets given as arguments (`preview/src/components/<dir>/style.css`), and
//! shares its parser with `preview/build.rs`.
//!
//!   SLOT    <path> <name> <line> <doc|internal|none>   a slot the sheet reads
//!   ORPHAN  <path> <name> <line> <text>                a note that names no slot of its sheet
//!   DUP     <path> <name> <line>                       a second note for one slot
//!   EMPTY   <path> <name> <line>                       a note with no description
#[path = "../../preview/slot_scan.rs"]
mod slot_scan;

fn main() {
    let mut slots = 0usize;
    for path in std::env::args().skip(1) {
        let source = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
        let dir = std::path::Path::new(&path)
            .parent()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let sheet = slot_scan::scan_sheet(&dir, &source);
        for slot in &sheet.slots {
            slots += 1;
            let kind = match &slot.note {
                None => "none",
                Some(note) if note.internal => "internal",
                Some(_) => "doc",
            };
            println!("SLOT\t{path}\t{}\t{}\t{kind}", slot.name, slot.line);
            if let Some(note) = &slot.note {
                if note.text.is_empty() {
                    println!("EMPTY\t{path}\t{}\t{}", note.name, note.line);
                }
            }
        }
        for note in &sheet.orphan_notes {
            println!("ORPHAN\t{path}\t{}\t{}\t{}", note.name, note.line, note.text);
        }
        for note in &sheet.duplicate_notes {
            println!("DUP\t{path}\t{}\t{}", note.name, note.line);
        }
    }
    eprintln!("slot-scan: {slots} slots");
}
