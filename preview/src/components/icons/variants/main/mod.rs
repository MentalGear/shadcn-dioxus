use super::super::component::*;
use dioxus::prelude::*;

/// One cell of the sheet: the glyph over its component name.
#[component]
fn GlyphCell(
    /// The glyph's component name.
    name: &'static str,
    /// The glyph itself.
    glyph: Element,
) -> Element {
    rsx! {
        div {
            style: "display: flex; flex-direction: column; align-items: center; gap: 0.5rem; width: 7rem; font-size: 0.75rem; color: var(--dx-muted-foreground);",
            span { style: "color: var(--dx-foreground);", {glyph} }
            span { "{name}" }
        }
    }
}

/// Every glyph the components draw, at 24px in the text colour.
#[component]
pub fn Demo() -> Element {
    rsx! {
        div { style: "display: flex; flex-wrap: wrap; justify-content: center; gap: 1.25rem 0.5rem; width: 100%;",
            GlyphCell { name: "ArrowDown", glyph: rsx! { ArrowDown { size: "24" } } }
            GlyphCell { name: "ArrowUp", glyph: rsx! { ArrowUp { size: "24" } } }
            GlyphCell { name: "ArrowUpDown", glyph: rsx! { ArrowUpDown { size: "24" } } }
            GlyphCell { name: "BadgeCheck", glyph: rsx! { BadgeCheck { size: "24" } } }
            GlyphCell { name: "Check", glyph: rsx! { Check { size: "24" } } }
            GlyphCell { name: "ChevronDown", glyph: rsx! { ChevronDown { size: "24" } } }
            GlyphCell { name: "ChevronLeft", glyph: rsx! { ChevronLeft { size: "24" } } }
            GlyphCell { name: "ChevronRight", glyph: rsx! { ChevronRight { size: "24" } } }
            GlyphCell { name: "ChevronUp", glyph: rsx! { ChevronUp { size: "24" } } }
            GlyphCell { name: "ChevronsUpDown", glyph: rsx! { ChevronsUpDown { size: "24" } } }
            GlyphCell { name: "Copy", glyph: rsx! { Copy { size: "24" } } }
            GlyphCell { name: "Ellipsis", glyph: rsx! { Ellipsis { size: "24" } } }
            GlyphCell { name: "GripVertical", glyph: rsx! { GripVertical { size: "24" } } }
            GlyphCell { name: "LoaderCircle", glyph: rsx! { LoaderCircle { size: "24" } } }
            GlyphCell { name: "Minus", glyph: rsx! { Minus { size: "24" } } }
            GlyphCell { name: "PanelLeft", glyph: rsx! { PanelLeft { size: "24" } } }
            GlyphCell { name: "Search", glyph: rsx! { Search { size: "24" } } }
            GlyphCell { name: "TrendingUp", glyph: rsx! { TrendingUp { size: "24" } } }
            GlyphCell { name: "X", glyph: rsx! { X { size: "24" } } }
        }
    }
}
