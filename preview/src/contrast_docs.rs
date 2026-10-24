//! The `/docs` page's "Contrast mode" section: `data-dx-contrast="wcag|shadcn"` on `<html>`.
//!
//! The theme's colours are shadcn's exact values by default. A few of them are below WCAG 2.2's thresholds
//! (shadcn's own, not ours), so the theme also carries one calibrated set behind one switch, defined once in
//! the "Contrast mode" block at the end of `dx-components-theme.css`. The theme picker's "Contrast" control
//! (`theme.rs`) writes the same attribute. A closing paragraph documents the interactive-element cursor option
//! (`data-dx-interactive-element-cursor`), which lives in the same theme sheet.

use crate::components::table::{Table, TableBody, TableCell, TableHead, TableHeader, TableRow};
use crate::DocsPre;
use dioxus::prelude::*;

/// `(what, shadcn, calibrated)` rows of the table.
const CHANGES: &[[&str; 3]] = &[
    [
        "Focus ring (light)",
        "shadcn's grey, 2.3-2.6:1 on a white surface",
        "lightness capped at 0.65, 3.2:1 (a preset's tint is kept; dark is already 3.5:1 or better and unchanged)",
    ],
    [
        "Input, Textarea, Select, Combobox, Checkbox, Radio edge",
        "light 1.2:1, dark 1.7:1",
        "light 3.2:1, dark 4.6:1 or better (WCAG 1.4.11, 3:1)",
    ],
    [
        "Unchecked Switch track",
        "light 1.2:1, dark 1.3:1",
        "light 3.2:1, dark 3.1:1 or better",
    ],
    [
        "Muted text (descriptions, placeholders)",
        "3.9-4.4:1 on a muted, accent or secondary surface in light, in eight of the nine base colours",
        "lightness capped at 0.54, 4.5:1 or better on every surface (WCAG 1.4.3)",
    ],
    [
        "Sidebar primary text (dark)",
        "3.5:1 on the Blue accent, 1.8:1 on Yellow",
        "near-white or near-black by the fill's lightness, 4.5:1 or better",
    ],
];

const ATTRIBUTE_RSX: &str = r#"// Calibrated for everyone:
document::eval("document.documentElement.setAttribute('data-dx-contrast', 'wcag')");

// shadcn's exact colours even when the user asks for more contrast:
document::eval("document.documentElement.setAttribute('data-dx-contrast', 'shadcn')");

// Or in the HTML shell, so the first paint is already right:
// <html data-dx-contrast="wcag">"#;

#[component]
pub fn ContrastDocs() -> Element {
    let rows: Vec<[String; 3]> = CHANGES
        .iter()
        .map(|r| [r[0].to_string(), r[1].to_string(), r[2].to_string()])
        .collect();
    rsx! {
        p {
            "Every theme colour defaults to shadcn's exact value, and a few of those are under WCAG 2.2's thresholds in shadcn itself. Contrast mode is one switch that replaces them with a calibrated set: "
            code { "data-dx-contrast" }
            " on "
            code { "<html>" }
            ". "
            code { "wcag" }
            " turns it on, "
            code { "shadcn" }
            " keeps shadcn's colours, and with no attribute the theme follows the system: calibrated under "
            code { "prefers-contrast: more" }
            ", shadcn's colours otherwise. The theme picker's \"Contrast\" control on this site sets it."
        }
        Table {
            TableHeader {
                TableRow {
                    TableHead { "Role" }
                    TableHead { "shadcn (default)" }
                    TableHead { "Calibrated" }
                }
            }
            TableBody {
                for row in rows {
                    TableRow { key: "{row[0]}",
                        TableCell { "{row[0]}" }
                        TableCell { "{row[1]}" }
                        TableCell { "{row[2]}" }
                    }
                }
            }
        }
        p {
            "Each calibrated value is a relative-colour lightness bound on the role's own source (a preset's tint and a pasted shadcn theme's colours still come through; only a value that fails is moved). Selected fills, slider and progress tracks and the Tabs pill stay shadcn's, because each carries a second cue. The status colours that have no shadcn original (the error Toast surface, the success green, the docs link blue) are fixed in both settings."
        }
        h3 { id: "contrast-mode-use", "Using it" }
        DocsPre {
            code { "{ATTRIBUTE_RSX}" }
        }
        p {
            "The values live in one block at the end of "
            code { "dx-components-theme.css" }
            " (\"Contrast mode\"); nothing else in the theme or in a component sheet reads the mode. To calibrate one more role in your app, declare it inside "
            code { ":root[data-dx-contrast=\"wcag\"]" }
            " after the theme sheet."
        }
        h3 { id: "interactive-element-cursor", "Cursor on interactive elements" }
        p {
            "Buttons, tabs, options, menu items, switches, checkboxes and the rest keep the default arrow, as in shadcn v4. To give every one of them the hand instead, set "
            code { "data-dx-interactive-element-cursor=\"pointer\"" }
            " on "
            code { "<html>" }
            " (or on any container); one rule in "
            code { "dx-components-theme.css" }
            " decides it for all of them, and the theme picker's \"Cursor\" control sets it. Text fields keep the text cursor, disabled controls their not-allowed cursor and drag handles their grab cursor, whichever you choose."
        }
    }
}
