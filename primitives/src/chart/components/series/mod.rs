//! One file per mark family -- the chart's extension points.
//! `components::chart::Chart` dispatches on `ChartKind` to exactly one
//! family's `super::layout::SeriesRenderContext`-consuming `render` per
//! render; everything else about how that family draws itself lives in its
//! own file, out of `Chart`'s own body, and out of every *other* family's
//! file.
//!
//! ## One file per family
//!
//! | File | Options | Also holds |
//! |---|---|---|
//! | `area.rs` | `AreaOptions` | |
//! | `bar.rs` | `BarOptions` | `components::layout` |
//! | `line.rs` | `LineOptions` | `engine::curve`, `engine::data` |
//! | `pie.rs` | `PieOptions` | `engine::polar` |
//! | `radial.rs` | `RadialOptions` | `engine::polar` |
//! | `radar.rs` | `RadarOptions` | `engine::radar` |
//!
//! `mod.rs` (this file), `../chart.rs`, `../container.rs`, `../mod.rs`,
//! `../../mod.rs` and `../../engine/mod.rs` hold the dispatch and the
//! shared structure; a family's own file holds its `*Options` and its
//! `render`.
//!
//! ## Shared vs. per-family props
//!
//! Two props apply to more than one family, and both stay on `ChartProps`
//! itself rather than being duplicated onto every family's own `*Options`
//! struct that could plausibly use them:
//! - **`stacked`** applies to Area and Bar only (`ChartKind::is_cartesian`'s
//!   three kinds minus Line -- there is no "stacked line chart"
//!   construction, matching shadcn/Recharts' own scope, so this is a
//!   modeling choice about Line specifically, not a Cartesian/polar split).
//!   `Chart` resolves it to one effective `bool` once
//!   (`props.stacked && matches!(kind, Area | Bar)`) and passes that
//!   resolved value into `SeriesRenderContext::stacked`, so neither
//!   `area.rs` nor `bar.rs` re-checks `kind` itself.
//! - **`curve`** applies to Area and Line (every family with a drawn edge
//!   to interpolate -- Bar has none). It reaches both through
//!   `SeriesRenderContext::curve` as given, never duplicated onto
//!   `AreaOptions`/`LineOptions`.
//!
//! One option lives on a single family's own options struct rather than on
//! `ChartProps`, since only that family uses it: the Line-only dots
//! switch is [`LineOptions::dots`] -- `line: LineOptions { dots: true,
//! ..Default::default() }`.
//!
//! Every family's per-series marks loop (`for (s, series) in
//! ctx.config.series.iter().enumerate() { g { "data-series": ..., ... } }`)
//! is intentionally duplicated across `area.rs`/`bar.rs`/`line.rs` rather
//! than factored into one shared helper here: each loop's *body* differs
//! enough (stacked-vs-not branching, grouped-bar sub-positioning, the dots
//! overlay) that a generic higher-order-function wrapper would cost more
//! clarity than the ~15 duplicated lines it would save. `series_values`
//! (pure data extraction, not drawing logic) IS shared, as a
//! `SeriesRenderContext` method -- see that type's own doc.

pub(crate) mod active_dot;
pub mod area;
pub mod bar;
pub mod line;
pub mod pie;
pub mod radar;
pub mod radial;

pub use area::AreaOptions;
pub use bar::{BarOptions, BarRadius};
// `line`'s own nested extension-point types (`DotContext`/`DotRenderer`/
// `LineLabels`), not just `LineOptions` itself: a
// caller-facing type gains no reachability at all from being `pub` inside
// `line.rs` alone -- `components`/`series` are both non-`pub` modules
// (`chart/mod.rs`/this file's own doc), so nothing outside `chart::`
// reaches past them except through exactly this line's own re-export
// list, however long. The `dots_custom`/`label`/`label_custom` gallery
// variants each name one of these types. One line per family keeps six
// families editing this shared file without conflict, the same shape as
// `preview/src/components/mod.rs`'s `examples!` list and the root
// `component.json`.
pub use line::{DotContext, DotRenderer, LineLabels, LineOptions};
// `PieLabels`, not just `PieOptions` itself: the `pie_chart` gallery's
// `label`/`label_list`/`label_custom` variants all name one of its variants
// directly (e.g. `PieLabels::Value`, `PieLabels::List(vec![...])`), and
// `RadialOptions::labels` (below) reuses this same enum -- same
// reachability rationale as `line`'s/`radar`'s own multi-type re-exports
// above.
pub use pie::{PieLabels, PieOptions};
// `RadarGrid`, not just `RadarOptions` itself: a caller-facing type (the
// `radar_chart` gallery's `grid_circle`/`grid_circle_fill`/
// `grid_circle_no_lines`/`grid_custom`/`grid_fill`/`grid_none` variants all
// name one of its variants directly, e.g. `RadarGrid::Circle`) gains no
// reachability from being `pub` inside `radar.rs` alone -- same rationale
// as `line`'s own multi-type re-export above.
pub use radar::{RadarGrid, RadarOptions};
pub use radial::RadialOptions;
