//! The `/docs` page's "Utilities" section: the two plain-CSS utilities of `assets/dx-effects.css`
//! (`dx-scroll-fade`, `dx-shimmer`), ported from shadcn/ui's utilities, in shadcn's page order
//! (scroll-fade, then shimmer): class table, variable table, one live demo each.
//!
//! The shimmer demo is the recipe the page documents: the element that carries `dx-shimmer` also
//! spreads `use_motion().attributes()`, so the sweep pauses while it is off screen
//! (`scripts/check-motion-gating.sh`, check C, requires the gate in any file naming the class).

use crate::components::{
    button::{Button, ButtonVariant},
    table::{Table, TableBody, TableCell, TableHead, TableHeader, TableRow},
};
use crate::DocsPre;
use dioxus::prelude::*;
use dioxus_primitives::activity::use_motion;

/// Renders `text` with every `` `backtick span` `` as `<code>`.
fn inline_code(text: &str) -> Element {
    let parts: Vec<(usize, String)> = text
        .split('`')
        .enumerate()
        .map(|(i, part)| (i, part.to_string()))
        .collect();
    rsx! {
        for (i , part) in parts {
            if i % 2 == 1 {
                code { key: "{i}", "{part}" }
            } else {
                Fragment { key: "{i}", "{part}" }
            }
        }
    }
}

fn utility_table(head: &[&str], rows: &[[&str; 2]]) -> Element {
    let head: Vec<String> = head.iter().map(|h| h.to_string()).collect();
    let rows: Vec<[String; 2]> = rows
        .iter()
        .map(|r| [r[0].to_string(), r[1].to_string()])
        .collect();
    rsx! {
        Table {
            TableHeader {
                TableRow {
                    for h in head {
                        TableHead { key: "{h}", "{h}" }
                    }
                }
            }
            TableBody {
                for row in rows {
                    TableRow { key: "{row[0]}",
                        TableCell { {inline_code(&row[0])} }
                        TableCell { {inline_code(&row[1])} }
                    }
                }
            }
        }
    }
}

const SCROLL_FADE_CLASSES: &[[&str; 2]] = &[
    [
        "`dx-scroll-fade`, `dx-scroll-fade-y`",
        "both block edges of an `overflow-y` box",
    ],
    [
        "`dx-scroll-fade-x`",
        "both inline edges of an `overflow-x` box; follows the reading direction",
    ],
    [
        "`dx-scroll-fade-xy`",
        "all four edges of a box that scrolls on both axes (the block and inline masks intersected); inline edges follow the reading direction",
    ],
    [
        "`dx-scroll-fade-t`, `dx-scroll-fade-b`",
        "one physical block edge (top, bottom)",
    ],
    [
        "`dx-scroll-fade-l`, `dx-scroll-fade-r`",
        "one physical inline edge (left, right)",
    ],
    [
        "`dx-scroll-fade-s`, `dx-scroll-fade-e`",
        "one logical inline edge (start, end; mirrors in RTL)",
    ],
    [
        "`dx-scroll-fade-none`",
        "turns the fade off; works in any class order",
    ],
];

const SCROLL_FADE_VARS: &[[&str; 2]] = &[
    [
        "`--dx-scroll-fade-size`",
        "depth of every fade; default `min(12%, var(--dx-space-10))`, i.e. 12% of the box capped at 40px",
    ],
    [
        "`--dx-scroll-fade-t-size`, `-b-size`, `-s-size`, `-e-size`",
        "depth of one edge; overrides `--dx-scroll-fade-size`",
    ],
    [
        "`--dx-scroll-fade-reveal`",
        "scroll distance over which a fade eases in or out; default `96px`",
    ],
];

const SHIMMER_CLASSES: &[[&str; 2]] = &[
    [
        "`dx-shimmer`",
        "the effect: `background-clip: text` plus a looping sweep (gate it, see below)",
    ],
    [
        "`dx-shimmer-once`",
        "plays a single sweep instead of looping",
    ],
    [
        "`dx-shimmer-reverse`",
        "sweeps the other way (RTL already follows the reading direction)",
    ],
    [
        "`dx-shimmer-none`",
        "renders the text normally; the stateful off switch",
    ],
];

const SHIMMER_VARS: &[[&str; 2]] = &[
    [
        "`--dx-shimmer-color`",
        "highlight colour; default derived from `currentColor`, brighter in dark mode",
    ],
    ["`--dx-shimmer-duration`", "one sweep; default `2s`"],
    [
        "`--dx-shimmer-spread`",
        "width of the highlight band; default `3ch + 40px`",
    ],
    ["`--dx-shimmer-angle`", "tilt of the band; default `20deg`"],
];

const INSTALL_ADD: &str =
    "# the smallest component that lists dx-effects.css in its globalAssets\n\
shadcn-dioxus add scroll_area\n\
# or, with the Dioxus CLI\n\
dx components add scroll_area";

const INSTALL_LINK: &str = r#"document::Link { rel: "stylesheet", href: asset!("/assets/dx-components-theme.css") }
document::Link { rel: "stylesheet", href: asset!("/assets/dx-effects.css") }"#;

const FADE_Y_RSX: &str = r#"div {
    class: "dx-scroll-fade",
    style: "height: 12rem; overflow-y: auto;",
    for i in 0..30 {
        p { "Row {i}" }
    }
}"#;

const FADE_X_RSX: &str = r#"div {
    class: "dx-scroll-fade-x",
    style: "display: flex; gap: 0.5rem; overflow-x: auto; scrollbar-width: none;",
    for i in 0..20 {
        span { "Chip {i}" }
    }
}"#;

const SHIMMER_RSX: &str = r#"#[component]
fn Streaming(done: bool) -> Element {
    let motion = use_motion();
    rsx! {
        span {
            role: "status",
            class: if done { "dx-shimmer dx-shimmer-none" } else { "dx-shimmer" },
            ..motion.attributes(),
            "Reading 12 files..."
        }
    }
}"#;

/// The demo box: a bordered, padded frame; the scroller inside carries the utility class.
const FRAME: &str = "border: 1px solid var(--border, currentColor); border-radius: var(--radius, 0.5rem); padding: 0.5rem; max-width: 24rem;";

#[component]
fn FadeYDemo() -> Element {
    rsx! {
        div { "data-demo": "util-fade-y", style: "{FRAME}",
            div {
                class: "dx-scroll-fade",
                "data-scroller": "y",
                tabindex: "0",
                style: "height: 12rem; overflow-y: auto;",
                for i in 1..=30 {
                    p { key: "{i}", style: "margin: 0; padding: 0.375rem 0.5rem;", "List item {i}" }
                }
            }
        }
    }
}

#[component]
fn FadeXDemo() -> Element {
    rsx! {
        div { "data-demo": "util-fade-x", style: "{FRAME}",
            div {
                class: "dx-scroll-fade-x",
                "data-scroller": "x",
                tabindex: "0",
                style: "display: flex; gap: 0.5rem; overflow-x: auto; scrollbar-width: none;",
                for i in 1..=20 {
                    span {
                        key: "{i}",
                        style: "flex: none; padding: 0.25rem 0.75rem; border: 1px solid var(--border, currentColor); border-radius: 999px;",
                        "Chip {i}"
                    }
                }
            }
        }
    }
}

/// The recipe: `dx-shimmer` on an element that also spreads `use_motion().attributes()`.
#[component]
fn ShimmerDemo() -> Element {
    let mut done = use_signal(|| false);
    let motion = use_motion();
    rsx! {
        div {
            "data-demo": "util-shimmer",
            style: "{FRAME} display: flex; align-items: center; justify-content: space-between; gap: 1rem;",
            span {
                role: "status",
                "data-shimmer": "line",
                class: if done() { "dx-shimmer dx-shimmer-none" } else { "dx-shimmer" },
                ..motion.attributes(),
                if done() {
                    "Read 12 files"
                } else {
                    "Reading 12 files..."
                }
            }
            Button {
                variant: ButtonVariant::Outline,
                r#type: "button",
                onclick: move |_| done.toggle(),
                if done() {
                    "Restart"
                } else {
                    "Finish"
                }
            }
        }
    }
}

#[component]
pub(crate) fn UtilitiesDocs() -> Element {
    rsx! {
        p {
            {inline_code("Two small CSS utilities, ported from shadcn/ui's `scroll-fade` and `shimmer` and renamed into the `dx-` namespace. Both live in `assets/dx-effects.css`: plain CSS, no JavaScript, no scroll listeners. Tailwind's numeric modifiers (`scroll-fade-24`, `shimmer-color-...`) have no plain-CSS equivalent, so they are custom properties you set inline or on a class of your own.")}
        }

        h3 { id: "utilities-install", "Install" }
        p {
            {inline_code("`dx-effects.css` is not a component, so it has no `add` command of its own. The smallest existing path is to add a component that lists it in its `globalAssets`: `scroll_area`, `virtual_list`, `message_scroller`, `attachment` and `marker` all do, and `add` copies the file to `assets/dx-effects.css` next to the theme (an edited copy is kept, `--force` replaces it). Or copy the file by hand from the repository. Then link it once at the app root, after the theme, which it reads for its `--dx-space-*` tokens and `--light`/`--dark` toggles.")}
        }
        DocsPre {
            code { "{INSTALL_ADD}" }
        }
        DocsPre {
            code { "{INSTALL_LINK}" }
        }

        h3 { id: "utilities-scroll-fade", "Scroll fade" }
        p {
            {inline_code("A scroll-aware edge mask on a scroll container. The fade is a `mask-image` on the scroller itself, driven by CSS scroll-driven animations (`animation-timeline: scroll()`), so it adapts to any background. At rest the start edge is crisp and the end edge fades to hint at more content; scrolling fades the start edge in and, at the end, the end edge sharpens. No overflow, no fade. A browser without scroll-driven animations gets a static edge fade. Put the class on the element that scrolls, never on a wrapper (the mask would dissolve the border and background too), and set `overflow` yourself. A classic scrollbar stays opaque when the scroller goes through `use_scroll_fade` (which `ScrollArea`, `VirtualList`, `VirtualGrid` and `MessageScrollerViewport` do), because it writes the gutter sizes the mask leaves out; with the bare class and no script, or with an overlay scrollbar, the scrollbar fades with the content. Each edge's depth eases (150ms, ease-out) when it switches, on the registered `--dx-scroll-fade-t/-b/-s/-e` properties, and not under `prefers-reduced-motion: reduce`. In Rust, `ScrollArea`, `VirtualList` and `MessageScrollerViewport` take a `fade: ScrollFade` prop that adds the class to their scrolling element.")}
        }
        {utility_table(&["Class", "Edges"], SCROLL_FADE_CLASSES)}
        {utility_table(&["Variable", "Effect"], SCROLL_FADE_VARS)}
        p { class: "dx-docs-muted", "A long list with dx-scroll-fade:" }
        FadeYDemo {}
        DocsPre {
            code { "{FADE_Y_RSX}" }
        }
        p { class: "dx-docs-muted", "A horizontal strip with dx-scroll-fade-x:" }
        FadeXDemo {}
        DocsPre {
            code { "{FADE_X_RSX}" }
        }

        h3 { id: "utilities-shimmer", "Shimmer" }
        p {
            {inline_code("A highlight that sweeps across text, for streaming or in-progress status. Built on `currentColor`, so it adapts to the element. `prefers-reduced-motion` turns it off. Where relative colour syntax is missing the text can render transparent; gate it with `@supports (color: oklch(from white l c h))` if you target older browsers.")}
        }
        {utility_table(&["Class", "Effect"], SHIMMER_CLASSES)}
        {utility_table(&["Variable", "Effect"], SHIMMER_VARS)}
        h4 { id: "utilities-shimmer-gate", "Never ship a bare loop" }
        p {
            {inline_code("`dx-shimmer` is an infinite animation, and a class cannot know whether anyone can see it. `MarkerContent` and `AttachmentTitle` pause it for you; on any other element, spread `use_motion().attributes()` on the element that carries the class (or put `data-dx-motion` from a `use_motion` hook on an ancestor): while it is off screen, in a skipped `content-visibility` subtree or in a hidden tab the theme pauses every CSS animation on and under an element marked `data-dx-motion=\"paused\"`. When the work is finished, swap in `dx-shimmer-none` rather than removing the gate. `scripts/check-motion-gating.sh` fails a Rust file that names the class without the gate.")}
        }
        DocsPre {
            code { "{SHIMMER_RSX}" }
        }
        p { class: "dx-docs-muted", "A streaming status line with dx-shimmer, gated by use_motion:" }
        ShimmerDemo {}
    }
}
