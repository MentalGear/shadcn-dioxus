use super::super::component::*;
use dioxus::prelude::*;
use dioxus_primitives::direction::{Direction, DirectionProvider};

/// RTL fixture (dev-docs/backlog.md row 13) -- see `tabs/variants/rtl/mod.rs`'s
/// doc for the general shape, including why the items below
/// ("S"/"H"/"C") deliberately differ from the `main` variant's
/// ("B"/"I"/"U"): `toggle_group.spec.ts` and
/// `oracle/tier1-apg/keyboard-matrix.spec.ts` both query
/// `getByRole("button", { name: "B", exact: true })`, and while `exact`
/// rules out a same-named *substring* collision, it does nothing for an
/// outright identical label -- this variant renders alongside `main` on
/// the same page (`preview/src/site/component_page.rs`'s `ComponentHighlight`), so
/// reusing "B" verbatim would still make that locator match two buttons.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { dir: "rtl",
            DirectionProvider { direction: Direction::Rtl,
                ToggleGroup { horizontal: true, allow_multiple_pressed: true,
                    ToggleItem { index: 0usize,
                        value: "s",
                        s { "S" }
                    }
                    ToggleItem { index: 1usize,
                        value: "h",
                        mark { "H" }
                    }
                    ToggleItem { index: 2usize,
                        value: "c",
                        code { "C" }
                    }
                }
            }
        }
    }
}
