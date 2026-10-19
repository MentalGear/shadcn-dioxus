use super::{ComponentDemoData, ComponentType, ComponentVariantDemoData, HighlightedCode};
// Only `css_highlight!`'s debug-only arm needs `asset!` -- gating the
// import too keeps a release build (which never calls it) from a bare
// `unused_imports` warning.
#[cfg(debug_assertions)]
use dioxus::prelude::*;

/// Builds this codebase's `CssHighlight` (`main.rs`) for one component's
/// CSS file (`style.css` or a Block-kind variant's `demo.css`). See
/// `CssHighlight`'s own doc comment for the full why: release/SSG builds
/// keep the previous compile-time `dioxus_code::code!()` embed unchanged;
/// debug builds (`dx serve`'s dev loop) carry only the file's own
/// `asset!()` URL instead, so an edit hot-reloads via the asset pipeline
/// rather than forcing a full rebuild through `code!()`'s
/// `include_str!(path)` (which makes rustc -- and in turn `dx serve`'s own
/// file-change classifier, which reads the compiled crate's rustc
/// dep-info -- treat the `.css` file as a source dependency of this
/// crate). Measured before/after: `dev-docs/dev-loop.md`'s CSS section.
///
/// Two full, cfg-gated definitions rather than one macro with a cfg'd
/// struct-literal field: `CssHighlight`'s own two fields are themselves
/// `#[cfg]`-gated (release-only `embedded`, debug-only `asset`), so each
/// mode's definition here can only ever construct the field that exists
/// in it.
#[cfg(not(debug_assertions))]
macro_rules! css_highlight {
    ($path:expr) => {
        crate::CssHighlight {
            embedded: crate::HighlightedCode {
                source: dioxus_code::code!($path),
            },
        }
    };
}

#[cfg(debug_assertions)]
macro_rules! css_highlight {
    ($path:expr) => {
        crate::CssHighlight {
            asset: asset!($path),
        }
    };
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ComponentCategory {
    Forms,
    Navigation,
    Overlays,
    Feedback,
    Disclosure,
    DataDisplay,
    Chat,
    Charts,
}

impl ComponentCategory {
    pub const ALL: &'static [Self] = &[
        Self::Forms,
        Self::Navigation,
        Self::Overlays,
        Self::Feedback,
        Self::Disclosure,
        Self::DataDisplay,
        Self::Chat,
        Self::Charts,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Forms => "Forms",
            Self::Navigation => "Navigation",
            Self::Overlays => "Overlays",
            Self::Feedback => "Feedback",
            Self::Disclosure => "Disclosure",
            Self::DataDisplay => "Data display",
            Self::Chat => "Chat",
            Self::Charts => "Charts",
        }
    }
}

pub fn category_of(name: &str) -> ComponentCategory {
    match name {
        "button" | "input" | "textarea" | "label" | "checkbox" | "switch" | "radio_group"
        | "toggle" | "toggle_group" | "select" | "slider" | "calendar" | "date_picker"
        | "color_picker" | "form" | "button_group" | "field" | "input_group" | "input_otp"
        | "native_select" | "questionnaire" => ComponentCategory::Forms,
        "navbar" | "sidebar" | "tabs" | "pagination" | "menubar" | "toolbar" | "context_menu"
        | "dropdown_menu" | "breadcrumb" | "navigation_menu" => ComponentCategory::Navigation,
        "dialog" | "alert_dialog" | "sheet" | "drawer" | "popover" | "tooltip" | "hover_card"
        | "top_layer" | "command" => ComponentCategory::Overlays,
        "toast" | "progress" | "skeleton" | "badge" | "alert" | "empty" | "spinner" => {
            ComponentCategory::Feedback
        }
        "accordion" | "collapsible" => ComponentCategory::Disclosure,
        "avatar" | "card" | "separator" | "aspect_ratio" | "item" | "drag_and_drop_list"
        | "virtual_list" | "scroll_area" | "tag_group" | "table" | "data_table" | "resizable" => {
            ComponentCategory::DataDisplay
        }
        "message" | "bubble" | "marker" | "attachment" | "message_scroller" | "markdown" => {
            ComponentCategory::Chat
        }
        "chart" | "area_chart" | "bar_chart" | "line_chart" | "pie_chart" | "radar_chart"
        | "radial_chart" | "chart_tooltip" => ComponentCategory::Charts,
        _ => ComponentCategory::DataDisplay,
    }
}

/// Where a component comes from, relative to the shadcn/ui catalog this
/// library tracks (`dev-docs/component-backlog.md`'s scope rule).
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Origin {
    /// A page of shadcn/ui's catalog.
    Shadcn,
    /// A shadcn-dioxus addition that shadcn/ui does not have. Rendered
    /// with an "Extra" badge (`ExtraBadge`, `main.rs`). The payload is WHY it
    /// is an extra: one short, user-facing sentence that says what the thing
    /// is and why it is here. It is part of the variant, so an extra without
    /// a reason does not compile; the badge tooltip, the sidebar/card `title`
    /// and the explainer line under the component page's heading all read it.
    Extra(&'static str),
}

/// THE classification: one row per `preview/src/components/<dir>`, naming
/// whether it is a shadcn/ui catalog page or one of our extras. Everything
/// that needs to know (the "Extra" badge on the component page, the sidebar
/// entry and the homepage card) reads it through [`origin_of`] / [`extra_reason`].
///
/// Why both kinds are listed, not just the extras: a list of extras alone
/// would make "unclassified" indistinguishable from "shadcn", so a new
/// component would silently skip the badge. With every directory tagged
/// either way, a new one is absent from this list until someone decides, and
/// `scripts/check-component-catalog.sh` (run by `scripts/run-gates.sh`) and
/// the `catalog_classifies_every_demo` test below both fail on it. Until it
/// is decided [`extra_reason`] is `None`, so the UI never claims "not part of
/// shadcn/ui" for something nobody has classified.
///
/// The catalog is `shadcn-ui/ui@295a1f1`, base flavour (64 pages), as audited
/// in `dev-docs/research/shadcn-catalog-2026-10-04.md`, whose section 2
/// is the name-by-name comparison. Directory names are the shadcn page name
/// with `-` as `_`. `scripts/check-component-catalog.sh` reads the
/// `("name", Origin::...)` rows of this block, so keep one tuple per row
/// (`#[rustfmt::skip]` below stops rustfmt splitting a long `Extra` reason row).
#[rustfmt::skip] // one tuple per line: the check script and the Playwright spec read the rows
pub const CATALOG: &[(&str, Origin)] = &[
    // shadcn/ui base catalog pages (one per component).
    ("accordion", Origin::Shadcn),
    ("alert", Origin::Shadcn),
    ("alert_dialog", Origin::Shadcn),
    ("aspect_ratio", Origin::Shadcn),
    ("attachment", Origin::Shadcn),
    ("avatar", Origin::Shadcn),
    ("badge", Origin::Shadcn),
    ("breadcrumb", Origin::Shadcn),
    ("bubble", Origin::Shadcn),
    ("button", Origin::Shadcn),
    ("button_group", Origin::Shadcn),
    ("calendar", Origin::Shadcn),
    ("card", Origin::Shadcn),
    ("carousel", Origin::Shadcn),
    ("chart", Origin::Shadcn),
    ("checkbox", Origin::Shadcn),
    ("collapsible", Origin::Shadcn),
    ("combobox", Origin::Shadcn),
    ("command", Origin::Shadcn),
    ("context_menu", Origin::Shadcn),
    ("data_table", Origin::Shadcn),
    ("date_picker", Origin::Shadcn),
    ("dialog", Origin::Shadcn),
    ("direction", Origin::Shadcn),
    ("drawer", Origin::Shadcn),
    ("dropdown_menu", Origin::Shadcn),
    ("empty", Origin::Shadcn),
    ("field", Origin::Shadcn),
    ("hover_card", Origin::Shadcn),
    ("input", Origin::Shadcn),
    ("input_group", Origin::Shadcn),
    ("input_otp", Origin::Shadcn),
    ("item", Origin::Shadcn),
    ("kbd", Origin::Shadcn),
    ("label", Origin::Shadcn),
    ("marker", Origin::Shadcn),
    ("menubar", Origin::Shadcn),
    ("message", Origin::Shadcn),
    ("message_scroller", Origin::Shadcn),
    ("native_select", Origin::Shadcn),
    ("navigation_menu", Origin::Shadcn),
    ("pagination", Origin::Shadcn),
    ("popover", Origin::Shadcn),
    ("progress", Origin::Shadcn),
    ("questionnaire", Origin::Shadcn),
    ("radio_group", Origin::Shadcn),
    ("resizable", Origin::Shadcn),
    ("scroll_area", Origin::Shadcn),
    ("select", Origin::Shadcn),
    ("separator", Origin::Shadcn),
    ("sheet", Origin::Shadcn),
    ("sidebar", Origin::Shadcn),
    ("skeleton", Origin::Shadcn),
    ("slider", Origin::Shadcn),
    ("spinner", Origin::Shadcn),
    ("switch", Origin::Shadcn),
    ("table", Origin::Shadcn),
    ("tabs", Origin::Shadcn),
    ("textarea", Origin::Shadcn),
    ("toast", Origin::Shadcn),
    ("toggle", Origin::Shadcn),
    ("toggle_group", Origin::Shadcn),
    ("tooltip", Origin::Shadcn),
    // The chart gallery pages: shadcn's own `chart` page carries these per-type sections
    // (`/charts/area`, `/charts/bar`, ...), so they are not additions.
    ("area_chart", Origin::Shadcn),
    ("bar_chart", Origin::Shadcn),
    ("chart_tooltip", Origin::Shadcn),
    ("line_chart", Origin::Shadcn),
    ("pie_chart", Origin::Shadcn),
    ("radar_chart", Origin::Shadcn),
    ("radial_chart", Origin::Shadcn),
    // NOT in shadcn/ui's catalog: these get the "Extra" badge, and say why.
    ("color_picker", Origin::Extra("A color picker from the upstream Dioxus components; shadcn/ui has no color picker.")),
    ("drag_and_drop_list", Origin::Extra("A list you can reorder by drag, touch or keyboard, from upstream Dioxus; shadcn/ui has none.")),
    ("form", Origin::Extra("A test fixture for native form submission; shadcn/ui replaced its Form with Field.")),
    ("markdown", Origin::Extra("Renders model output as safe, streaming-friendly Markdown; shadcn/ui has no Markdown component.")),
    ("navbar", Origin::Extra("A menubar-style navigation bar from upstream Dioxus; shadcn/ui's Navigation Menu is a different component.")),
    ("tag_group", Origin::Extra("A focusable group of tags with selection and removal, from upstream Dioxus; not in shadcn/ui.")),
    ("toolbar", Origin::Extra("A keyboard-navigable toolbar for grouping controls, from upstream Dioxus; not in shadcn/ui.")),
    ("top_layer", Origin::Extra("A test page for overlay stacking, not an installable component.")),
    ("virtual_grid", Origin::Extra("A data grid or gallery that renders only the cells in view, rows and columns; shadcn/ui has no virtualized grid.")),
    ("virtual_list", Origin::Extra("A long list that renders only the visible rows, from upstream Dioxus; not in shadcn/ui.")),
];

/// The [`Origin`] of a component directory name, or `None` while it is
/// unclassified (no row in [`CATALOG`]).
pub fn origin_of(name: &str) -> Option<Origin> {
    CATALOG
        .iter()
        .find(|(catalog_name, _)| *catalog_name == name)
        .map(|(_, origin)| *origin)
}

/// Why `name` is an extra, or `None` for a shadcn/ui page or a component
/// nobody has classified yet: the one lookup behind every "Extra" badge and
/// explainer.
pub fn extra_reason(name: &str) -> Option<&'static str> {
    match origin_of(name) {
        Some(Origin::Extra(reason)) => Some(reason),
        Some(Origin::Shadcn) | None => None,
    }
}

#[cfg(test)]
mod catalog_tests {
    use super::*;

    /// `CATALOG` and `DEMOS` name exactly the same components, once each: a
    /// new demo with no row (unclassified), a row for a demo that is gone, and
    /// a duplicated row all fail here, so the badge cannot drift from the
    /// demo list. (`scripts/check-component-catalog.sh` checks the same from
    /// the directories, without compiling.)
    #[test]
    fn catalog_classifies_every_demo() {
        let mut seen = std::collections::BTreeSet::new();
        for (name, _) in CATALOG {
            assert!(seen.insert(*name), "`{name}` has two rows in CATALOG");
            assert!(
                DEMOS.iter().any(|demo| demo.name == *name),
                "CATALOG row `{name}` has no demo in `examples!`"
            );
        }
        for demo in DEMOS {
            assert!(
                origin_of(demo.name).is_some(),
                "`{}` is not classified: add a `(\"{0}\", Origin::Shadcn | Origin::Extra)` row to CATALOG",
                demo.name
            );
        }
    }

    #[test]
    fn extra_badge_is_for_extras_only() {
        assert!(extra_reason("navbar").is_some());
        assert!(extra_reason("button").is_none());
        assert!(extra_reason("chart_tooltip").is_none());
        // Unclassified is not "extra": never claim what nobody decided.
        assert!(extra_reason("no_such_component").is_none());
    }

    /// The reason is what the badge tooltip and the explainer line show, so it
    /// has to be a real, short sentence (the type already forbids a missing
    /// one; this forbids an empty or essay-length one).
    #[test]
    fn every_extra_has_a_short_reason() {
        let mut extras = 0;
        for (name, origin) in CATALOG {
            if let Origin::Extra(reason) = origin {
                extras += 1;
                assert!(
                    !reason.trim().is_empty(),
                    "`{name}` has an empty Extra reason"
                );
                assert!(
                    reason.len() < 120,
                    "`{name}`'s Extra reason is {} characters; keep it under 120: {reason}",
                    reason.len()
                );
                assert!(
                    reason.ends_with('.'),
                    "`{name}`'s Extra reason should be a sentence ending in a full stop: {reason}"
                );
                assert_eq!(extra_reason(name), Some(*reason));
            } else {
                assert_eq!(extra_reason(name), None, "`{name}` is a shadcn page");
            }
        }
        assert!(extras > 0, "CATALOG lists no extras at all");
        assert_eq!(extra_reason("no_such_component"), None);
    }
}

/// The `DEMOS` entries of one sidebar group, in sidebar order: `DEMOS` order
/// (alphabetical), except that a group's overview page leads it. Only
/// `chart` is one today; it would otherwise sort between `bar chart` and
/// `chart tooltip`, burying the page the other chart pages point to. A
/// stable sort on a boolean key keeps every other entry where it was, and
/// leaves the gallery/Demos grids (which iterate `DEMOS` directly) alone.
pub fn demos_in_category(cat: ComponentCategory) -> Vec<&'static ComponentDemoData> {
    let mut demos: Vec<_> = DEMOS
        .iter()
        .filter(|demo| category_of(demo.name) == cat)
        .collect();
    demos.sort_by_key(|demo| demo.name != "chart");
    demos
}

macro_rules! examples {
    ($($name:ident $(($kind:ident))? $([$($variant:ident),*])?),* $(,)?) => {
        $(
            pub(crate) mod $name {
                mod component;
                #[allow(unused)]
                pub use component::*;
                pub(crate) mod variants {
                    pub(crate) mod main;
                    $(
                        $(
                            pub(crate) mod $variant;
                        )*
                    )?
                }
            }
        )*
        pub(crate) static DEMOS: &[ComponentDemoData] = &[
            $(
                examples!(@demo $name $( $kind )? $([$($variant),*])?),
            )*
        ];
    };

    (@kind) => { ComponentType::Normal };
    (@kind normal) => { ComponentType::Normal };
    (@kind block) => { ComponentType::Block };

    // Normal components: no variant-level css_highlighted
    (@demo $name:ident $([$($variant:ident),*])?) => {
        ComponentDemoData {
            name: stringify!($name),
            r#type: ComponentType::Normal,
            description: include_str!(concat!(
                env!("OUT_DIR"),
                "/",
                stringify!($name),
                "/description.txt"
            )),
            docs: include_str!(concat!(env!("OUT_DIR"), "/", stringify!($name), "/docs.html")),
            component: HighlightedCode {
                source: dioxus_code::code!(concat!("/src/components/", stringify!($name), "/component.rs")),
            },
            style: css_highlight!(concat!("/src/components/", stringify!($name), "/style.css")),
            variants: &[
                ComponentVariantDemoData {
                    name: "main",
                    rs_highlighted: HighlightedCode {
                        source: dioxus_code::code!(concat!("/src/components/", stringify!($name), "/variants/main/mod.rs")),
                    },
                    css_highlighted: None,
                    component: $name::variants::main::Demo,
                },
                $(
                    $(
                        ComponentVariantDemoData {
                            name: stringify!($variant),
                            rs_highlighted: HighlightedCode {
                                source: dioxus_code::code!(concat!("/src/components/", stringify!($name), "/variants/", stringify!($variant), "/mod.rs")),
                            },
                            css_highlighted: None,
                            component: $name::variants::$variant::Demo,
                        },
                    )*
                )?
            ],
        }
    };

    // Block components: rendered in iframe, with shared demo.css
    (@demo $name:ident block $([$($variant:ident),*])?) => {
        ComponentDemoData {
            name: stringify!($name),
            r#type: ComponentType::Block,
            description: include_str!(concat!(
                env!("OUT_DIR"),
                "/",
                stringify!($name),
                "/description.txt"
            )),
            docs: include_str!(concat!(env!("OUT_DIR"), "/", stringify!($name), "/docs.html")),
            component: HighlightedCode {
                source: dioxus_code::code!(concat!("/src/components/", stringify!($name), "/component.rs")),
            },
            style: css_highlight!(concat!("/src/components/", stringify!($name), "/style.css")),
            variants: &[
                ComponentVariantDemoData {
                    name: "main",
                    rs_highlighted: HighlightedCode {
                        source: dioxus_code::code!(concat!("/src/components/", stringify!($name), "/variants/main/mod.rs")),
                    },
                    css_highlighted: Some(css_highlight!(concat!("/src/components/", stringify!($name), "/variants/demo.css"))),
                    component: $name::variants::main::Demo,
                },
                $(
                    $(
                        ComponentVariantDemoData {
                            name: stringify!($variant),
                            rs_highlighted: HighlightedCode {
                                source: dioxus_code::code!(concat!("/src/components/", stringify!($name), "/variants/", stringify!($variant), "/mod.rs")),
                            },
                            css_highlighted: Some(css_highlight!(concat!("/src/components/", stringify!($name), "/variants/demo.css"))),
                            component: $name::variants::$variant::Demo,
                        },
                    )*
                )?
            ],
        }
    };
}

// The component and variant registry is DERIVED, not listed (backlog row 219): `build.rs`
// (`discover_components`) walks `src/components/<dir>/` (every directory with a `component.json`,
// sorted by name) and each one's `variants/<name>/` directories, and writes the whole list as the
// `with_registry!` macro in `OUT_DIR/examples.rs`. Adding a demo is a new `variants/<name>/mod.rs`
// and touches nothing outside that component's directory; the order after `main` is that
// directory's own `variants/order.txt` (names it does not list follow, sorted).
// `variants/demo.css` makes a component a Block (rendered in an iframe page of its own). A new
// COMPONENT still needs its `CATALOG` row above, which is also what re-runs `build.rs`.
include!(concat!(env!("OUT_DIR"), "/examples.rs"));
with_registry!(examples);

// Compile-time-dead mirror of the derived BLOCK rows. The Playwright specs (`all-routes`,
// `demo-forms-no-navigation`, `popup-style-isolation`) scrape `name(block)[variant, ..]` rows after
// this invocation's name to build the block demo routes. `cfg(any())` removes the item before macro
// resolution, so nothing is compiled from it; `build.rs` (`check_block_mirror`) fails the build when
// these rows differ from the directories, so the specs' route list cannot go quietly stale. Retire it
// when those specs read `variants/` instead.
#[cfg(any())]
examples!(
    sidebar(block)[floating, inset],
);
