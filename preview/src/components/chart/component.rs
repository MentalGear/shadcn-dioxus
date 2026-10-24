use dioxus::prelude::*;
use dioxus_primitives::chart::{self, ChartContainerProps, ChartLegendProps, ChartProps, ChartTooltipProps};
use dioxus_primitives::dioxus_attributes::attributes;
use dioxus_primitives::merge_attributes;

// Plain, render-nothing value/data types a caller configures a chart with
// (dev-docs/preview-composition.md's allowlist rationale: "plain value
// types/enums that render nothing themselves have no theme for a wrapper to
// attach"). Re-exported here -- rather than left for demo code to import
// raw from `dioxus_primitives::chart` -- so every demo composes exclusively
// through `crate::components::chart::*`, never a raw `dioxus_primitives::`
// path outside this file (`scripts/check-preview-composition.sh`). `Curve`
// is on this list for the per-family gallery packages (`area_chart`,
// `line_chart`, ...): their own `component.rs` is a thin `pub use
// crate::components::chart::*;` (the installable package stays `chart`
// alone), so anything a variant demo names -- `Curve::Linear`/`Curve::Step`
// for e.g. `area_chart`'s `linear`/`step` demos -- has to be reachable from
// here, transitively, rather than each gallery reaching past this module
// into `dioxus_primitives::chart::Curve` directly (which
// `check-preview-composition.sh` would then have to special-case per
// gallery instead of once, here, at the seam this file already is).
//
// `BarOptions`/`PieOptions`/`RadarOptions`/`RadialOptions` (and
// `LineOptions`/`AreaOptions`) are the per-family option structs behind
// `ChartProps`' own `bar`/`pie`/`radar`/`radial`/`line`/`area` fields, so
// each family's own gallery can write `<Family>Options { .. }` in a
// demo with no edit to this shared file needed first.
// `ChartSeries`/`ChartIcon`/`StackMode` are on this list for `area_chart`'s
// own `icons`/`stacked_expand` variants (setting a series' `icon` field
// directly, and `AreaOptions.stack_mode`, respectively).
#[allow(unused_imports)]
pub use dioxus_primitives::chart::{
    AreaOptions, BarOptions, BarRadius, ChartConfig, ChartDatum, ChartIcon, ChartKind,
    ChartMargin, ChartSeries, Curve, LegendAlign, LineOptions, PieLabels, PieOptions,
    RadarOptions, RadialOptions, StackMode,
};

/// The themed chart container: sets the `--color-<key>` CSS variables
/// generated from `config` as an inline `style` on its own element, so they
/// are scoped to this instance by inheritance. Always
/// the outermost chart piece -- `Chart`/`ChartTooltip`/`ChartLegend` are
/// composed as its children. The focusable/keyboard-navigable root
/// (`Chart`'s `keyboard` prop) is `Chart`'s own `[data-slot="chart"]` div,
/// not this container's -- see `Chart`'s doc comment below (an
/// `Element`-typed child has no way to attach
/// attributes to its already-rendered parent without a post-mount effect,
/// which would make them absent from the first SSR render).
///
/// Forwards via `..props` (the
/// same shape as `Chart` below) rather than a hand-listed field-by-field
/// call, so a newly added `ChartContainerProps` field forwards for free
/// instead of silently compiling away.
#[component]
pub fn ChartContainer(props: ChartContainerProps) -> Element {
    let base = attributes!(div {
        class: "dx-chart", "data-dx-root": "",
    });
    // `.clone()`, not a move: `..props` below needs `props` intact --
    // moving `props.attributes` out first (as the non-spread `Chart`/
    // `ChartTooltip`/`ChartLegend` bodies safely could not, either, if they
    // needed to override `attributes` the same way) would make `props` a
    // partially-moved value, which struct-update syntax rejects (E0382).
    let merged = merge_attributes(vec![base, props.attributes.clone()]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/chart/style.css") }
        chart::ChartContainer { attributes: merged, ..props }
    }
}

/// The chart's own visual: axes, grid, one mark per configured series (area
/// fill, bar, or line, per the container's `kind`), the hover hit-bands, and
/// the visually-hidden data table that carries the same series/category/
/// value data for assistive tech. Must be rendered inside a `ChartContainer`
/// -- reads its `kind`/`config`/`data` from that context, not from its own
/// props. Renders its own top-level `div[data-slot="chart"]` (wrapping the
/// svg + hidden table) that carries `tabindex`/`role="group"`/
/// `aria-roledescription="chart"`/`aria-label` and the keyboard handler when
/// `keyboard` is on. No base class
/// of its own: every mark (including this wrapper div) is unstyled by the
/// primitive and selected off `[data-slot="..."]` inside the container's
/// own `.dx-chart` scope (see `style.css`), so there is nothing new to
/// attach here beyond forwarding attributes through untouched.
///
/// Forwards via `..props` (a plain struct-update spread -- precedented in
/// this repo, e.g. `primitives/src/toast.rs`'s own `Toast { ..props }`),
/// not a hand-listed field-by-field call: `props` here IS `chart::Chart`'s
/// own `ChartProps` (imported directly, not a separate preview-defined
/// props type), so every field forwards with no mapping needed. This is a
/// fix by construction: a hand-listed forward silently drops any field
/// the list doesn't (yet) name, with no compile error (a demo setting
/// `x_label`/`max_x_ticks` would be silently ignored). `ChartTooltip`/
/// `ChartLegend`/`ChartContainer` use the identical shape.
#[component]
pub fn Chart(props: ChartProps) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/chart/style.css") }
        chart::Chart { ..props }
    }
}

/// The hover tooltip: one row per series at the active data point, plus a
/// `role="graphics-symbol"` swatch per row. Rendered even when closed
/// (`data-state="closed"`, hidden by CSS) so there is nothing to attach
/// post-hydration. Optional -- omit it for a chart that doesn't need one.
///
/// Forwards via `..props`: a plain struct-update spread rather than
/// a hand-listed field-by-field call, so a newly added `ChartTooltipProps`
/// field (e.g. `indicator`/`label_key`/`name_key`/`formatter`) reaches the
/// real primitive without an edit here. This also folds `children` back
/// into the spread instead of a separately named field -- `..props` moves
/// the whole struct (`Option<Element>` included) in one shot, so there is
/// no separate "trailing brace sugar always populates `Some`" pitfall to
/// avoid: nothing here is written as trailing braces.
#[component]
pub fn ChartTooltip(props: ChartTooltipProps) -> Element {
    let base = attributes!(div {
        class: "dx-chart-tooltip",
    });
    // `.clone()` -- see `ChartContainer`'s identical comment above: `..props`
    // below needs `props` intact, not partially moved.
    let merged = merge_attributes(vec![base, props.attributes.clone()]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/chart/style.css") }
        chart::ChartTooltip { attributes: merged, ..props }
    }
}

/// The legend: one `role="graphics-symbol"` swatch + label per configured
/// series, in `config` order. Optional -- a single/dual-series chart
/// usually doesn't need one.
///
/// Forwards via `..props`, the same shape as `ChartTooltip` above --
/// so a newly added `ChartLegendProps` field (e.g.
/// `hide_icon`) reaches `chart::ChartLegend`.
#[component]
pub fn ChartLegend(props: ChartLegendProps) -> Element {
    let base = attributes!(ul {
        class: "dx-chart-legend",
    });
    // `.clone()` -- see `ChartContainer`'s identical comment above: `..props`
    // below needs `props` intact, not partially moved.
    let merged = merge_attributes(vec![base, props.attributes.clone()]);

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/src/components/chart/style.css") }
        chart::ChartLegend { attributes: merged, ..props }
    }
}
