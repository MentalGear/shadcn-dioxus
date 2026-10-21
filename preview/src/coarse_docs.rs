//! The `/docs` page's "Touch sizing" section: `data-dx-coarse="auto|on|off"` (backlog rows 349, 350).
//!
//! The theme's sizes that differ by pointer (menu rows, text-entry font size, slider thumb, resize hit
//! area, hover-revealed actions) are variables, and `data-dx-coarse` on `<html>` or any container
//! switches them. The demo renders the same four controls in a container forced `off` and one forced `on`,
//! so the two can be compared on a desktop with a mouse; the theme picker's "Touch sizing" control
//! (`theme.rs`) writes the same attribute on `<html>`.

use crate::components::{
    button::{Button, ButtonVariant},
    command::{Command, CommandGroup, CommandItem},
    input::Input,
    slider::Slider,
    table::{Table, TableBody, TableCell, TableHead, TableHeader, TableRow},
    textarea::Textarea,
};
use crate::DocsPre;
use dioxus::prelude::*;

const SIZES: &[[&str; 3]] = &[
    ["Menu rows (dropdown, context menu, menubar, select, combobox, command)", "28px rows, 14px text", "44px rows, 16px text"],
    ["Menu bar entries (menubar, navbar, navigation menu)", "no minimum height", "44px minimum height"],
    ["Text entry (input, textarea, combobox, command, native select, date picker, calendar selects)", "14px text", "16px text (no iOS zoom on focus)"],
    ["Slider thumb", "12px thumb, 28px target", "20px thumb, 44px target"],
    ["Resizable handle", "12px hit area", "24px hit area"],
    ["Code block actions", "shown on hover or focus", "always shown"],
];

const ATTRIBUTE_RSX: &str = r#"// Everything on the page, whatever the device (the docs site's theme picker does this):
document::eval("document.documentElement.setAttribute('data-dx-coarse', 'on')");

// One subtree. `auto` inside an `on` subtree is the media query again.
div { "data-dx-coarse": "on",
    Input { placeholder: "16px text, 44px slider target" }
    Slider { default_value: 40.0 }
}"#;

const TOKENS_CSS: &str = r#":root {
  /* Retune one size for every touch device. Declare it after the theme sheet. */
  --dx-menu-item-min-height: 3rem;
}"#;

#[component]
pub fn CoarseDocs() -> Element {
    let rows: Vec<[String; 3]> = SIZES
        .iter()
        .map(|r| [r[0].to_string(), r[1].to_string(), r[2].to_string()])
        .collect();
    rsx! {
        p {
            "Dense desktop sizes are right for a mouse and too small for a finger, so the theme keeps every size that differs by pointer in one set of variables and switches the whole set with one attribute. "
            code { "data-dx-coarse" }
            " goes on "
            code { "<html>" }
            " or on any container: "
            code { "auto" }
            " (the default) follows "
            code { "(pointer: coarse)" }
            ", the device's primary pointer, so a laptop with a touchscreen keeps the desktop sizes; "
            code { "on" }
            " and "
            code { "off" }
            " force touch or desktop sizing for that subtree. A nested container wins over its ancestors. The \"Touch sizing\" control in this site's theme picker sets it page-wide."
        }
        Table {
            TableHeader {
                TableRow {
                    TableHead { "Sizes" }
                    TableHead { "Fine" }
                    TableHead { "Coarse" }
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
        h3 { id: "touch-sizing-demo", "Fine and coarse, side by side" }
        div {
            "data-demo": "coarse-compare",
            style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(16rem, 1fr)); gap: 1rem;",
            CoarsePane { mode: "off", title: "data-dx-coarse=\"off\"" }
            CoarsePane { mode: "on", title: "data-dx-coarse=\"on\"" }
        }
        h3 { id: "touch-sizing-use", "Using it" }
        DocsPre {
            code { "{ATTRIBUTE_RSX}" }
        }
        p {
            "Component sheets read the variables and never carry a "
            code { "pointer: coarse" }
            " rule of their own (a check fails the build if one does), so an app retunes a size, or adds its own, by declaring the variable:"
        }
        DocsPre {
            code { "{TOKENS_CSS}" }
        }
    }
}

/// One half of the comparison: a container forced to `mode` holding a text field, a textarea, a slider, a
/// button and a short command list (menu rows).
#[component]
fn CoarsePane(mode: &'static str, title: &'static str) -> Element {
    rsx! {
        div {
            "data-coarse-pane": mode,
            "data-dx-coarse": mode,
            style: "display: flex; flex-direction: column; gap: 0.75rem; border: 1px solid var(--dx-border); border-radius: var(--dx-radius-lg); padding: 0.75rem;",
            p { class: "dx-docs-muted", style: "margin: 0;", code { "{title}" } }
            Input { placeholder: "Text field", aria_label: "Text field ({mode})" }
            Textarea { placeholder: "Text area", aria_label: "Text area ({mode})", rows: "2" }
            Slider {
                label: "Slider ({mode})",
                horizontal: true,
                min: 0.0,
                max: 100.0,
                step: 1.0,
                default_value: 40.0,
            }
            div {
                Button { r#type: "button", variant: ButtonVariant::Outline, "A button" }
            }
            Command::<String> {
                input_aria_label: "Search ({mode})",
                placeholder: "Search...",
                list_aria_label: "Commands ({mode})",
                CommandGroup {
                    for (i , label) in ["Calendar", "Search", "Settings"].iter().enumerate() {
                        CommandItem::<String> {
                            index: i,
                            value: label.to_string(),
                            text_value: label.to_string(),
                            "{label}"
                        }
                    }
                }
            }
        }
    }
}
