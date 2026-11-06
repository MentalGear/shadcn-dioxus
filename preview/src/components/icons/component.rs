//! The glyphs the other components draw, in one file that installs with them.
//!
//! lifted-from: dioxus-icons@0.1.0 (generated Lucide components; MIT AND ISC)
//!
//! Each glyph is a component that renders one inline `<svg>` (24 x 24 view box, a 2px round stroke in
//! `currentColor`), so it takes the text colour of its parent and the size of its `size` prop. A
//! component that needs a glyph imports it from here (`crate::components::icons::Check`); no icon
//! crate is installed with it.
//!
//! The paths are Lucide's (https://lucide.dev), distributed under the ISC licence:
//!
//! ```text
//! ISC License
//!
//! Copyright (c) 2026 Lucide Icons and Contributors
//!
//! Permission to use, copy, modify, and/or distribute this software for any
//! purpose with or without fee is hereby granted, provided that the above
//! copyright notice and this permission notice appear in all copies.
//!
//! THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
//! WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
//! MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
//! ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
//! WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
//! ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
//! OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
//! ```
//!
//! `arrow-down`, `arrow-up`, `check`, `chevron-down`, `chevron-left`, `chevron-right`, `chevron-up`,
//! `minus`, `search` and `x` are derived from the Feather project, under the MIT licence:
//!
//! ```text
//! The MIT License (MIT)
//!
//! Copyright (c) 2013-present Cole Bemis
//!
//! Permission is hereby granted, free of charge, to any person obtaining a copy
//! of this software and associated documentation files (the "Software"), to deal
//! in the Software without restriction, including without limitation the rights
//! to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//! copies of the Software, and to permit persons to whom the Software is
//! furnished to do so, subject to the following conditions:
//!
//! The above copyright notice and this permission notice shall be included in all
//! copies or substantial portions of the Software.
//!
//! THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//! IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//! FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//! AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//! LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//! OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
//! SOFTWARE.
//! ```
use dioxus::prelude::*;

/// The props every glyph takes.
#[derive(Clone, PartialEq, Props)]
pub struct IconProps {
    /// The glyph's width and height: a number of pixels (`"16"`) or any CSS length (`"1rem"`).
    /// Defaults to 24.
    #[props(into, default = "24".to_string())]
    pub size: String,
    #[props(extends = SvgAttributes)]
    pub attributes: Vec<Attribute>,
}

/// The `<svg>` every glyph shares: the Lucide defaults first, so an attribute the caller passes
/// (`stroke`, `class`, `aria-hidden`, ..) comes after and wins.
#[component]
fn Svg(
    /// The glyph's width and height.
    size: String,
    /// The caller's attributes.
    attributes: Vec<Attribute>,
    /// The glyph's shapes.
    children: Element,
) -> Element {
    let stroke = if attributes.iter().any(|a| a.name == "stroke") { None } else { Some("currentColor") };
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "{size}",
            height: "{size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke,
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            ..attributes,
            {children}
        }
    }
}

macro_rules! glyph {
    ($(#[$doc:meta])* $name:ident { $($shape:tt)* }) => {
        $(#[$doc])*
        #[component]
        #[allow(non_snake_case)]
        pub fn $name(props: IconProps) -> Element {
            rsx! {
                Svg { size: props.size, attributes: props.attributes, $($shape)* }
            }
        }
    };
}

glyph!(
    /// The Lucide `ArrowDown` glyph.
    ArrowDown {
        path { d: "M12 5v14" }
        path { d: "m19 12-7 7-7-7" }
    }
);

glyph!(
    /// The Lucide `ArrowUp` glyph.
    ArrowUp {
        path { d: "m5 12 7-7 7 7" }
        path { d: "M12 19V5" }
    }
);

glyph!(
    /// The Lucide `ArrowUpDown` glyph.
    ArrowUpDown {
        path { d: "m21 16-4 4-4-4" }
        path { d: "M17 20V4" }
        path { d: "m3 8 4-4 4 4" }
        path { d: "M7 4v16" }
    }
);

glyph!(
    /// The Lucide `BadgeCheck` glyph.
    BadgeCheck {
        path { d: "M3.85 8.62a4 4 0 0 1 4.78-4.77 4 4 0 0 1 6.74 0 4 4 0 0 1 4.78 4.78 4 4 0 0 1 0 6.74 4 4 0 0 1-4.77 4.78 4 4 0 0 1-6.75 0 4 4 0 0 1-4.78-4.77 4 4 0 0 1 0-6.76Z" }
        path { d: "m9 12 2 2 4-4" }
    }
);

glyph!(
    /// The Lucide `Check` glyph.
    Check {
        path { d: "M20 6 9 17l-5-5" }
    }
);

glyph!(
    /// The Lucide `ChevronDown` glyph.
    ChevronDown {
        path { d: "m6 9 6 6 6-6" }
    }
);

glyph!(
    /// The Lucide `ChevronLeft` glyph.
    ChevronLeft {
        path { d: "m15 18-6-6 6-6" }
    }
);

glyph!(
    /// The Lucide `ChevronRight` glyph.
    ChevronRight {
        path { d: "m9 18 6-6-6-6" }
    }
);

glyph!(
    /// The Lucide `ChevronUp` glyph.
    ChevronUp {
        path { d: "m18 15-6-6-6 6" }
    }
);

glyph!(
    /// The Lucide `ChevronsUpDown` glyph.
    ChevronsUpDown {
        path { d: "m7 15 5 5 5-5" }
        path { d: "m7 9 5-5 5 5" }
    }
);

glyph!(
    /// The Lucide `Copy` glyph.
    Copy {
        rect { x: "8", y: "8", width: "14", height: "14", rx: "2", ry: "2" }
        path { d: "M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2" }
    }
);

glyph!(
    /// The Lucide `Ellipsis` glyph.
    Ellipsis {
        circle { cx: "12", cy: "12", r: "1" }
        circle { cx: "19", cy: "12", r: "1" }
        circle { cx: "5", cy: "12", r: "1" }
    }
);

glyph!(
    /// The Lucide `GripVertical` glyph.
    GripVertical {
        circle { cx: "9", cy: "12", r: "1" }
        circle { cx: "9", cy: "5", r: "1" }
        circle { cx: "9", cy: "19", r: "1" }
        circle { cx: "15", cy: "12", r: "1" }
        circle { cx: "15", cy: "5", r: "1" }
        circle { cx: "15", cy: "19", r: "1" }
    }
);

glyph!(
    /// The Lucide `LoaderCircle` glyph.
    LoaderCircle {
        path { d: "M21 12a9 9 0 1 1-6.219-8.56" }
    }
);

glyph!(
    /// The Lucide `Minus` glyph.
    Minus {
        path { d: "M5 12h14" }
    }
);

glyph!(
    /// The Lucide `PanelLeft` glyph.
    PanelLeft {
        rect { x: "3", y: "3", width: "18", height: "18", rx: "2" }
        path { d: "M9 3v18" }
    }
);

glyph!(
    /// The Lucide `Search` glyph.
    Search {
        path { d: "m21 21-4.34-4.34" }
        circle { cx: "11", cy: "11", r: "8" }
    }
);

glyph!(
    /// The Lucide `TrendingUp` glyph.
    TrendingUp {
        path { d: "M16 7h6v6" }
        path { d: "m22 7-8.5 8.5-5-5L2 17" }
    }
);

glyph!(
    /// The Lucide `X` glyph.
    X {
        path { d: "M18 6 6 18" }
        path { d: "m6 6 12 12" }
    }
);
