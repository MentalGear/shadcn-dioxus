//! Defines the `Chart` component and its sub-components: a pure-RSX SVG
//! chart engine (Area, Bar, Line, Pie, Radar and RadialBar), styled and
//! composed exactly like every other primitive in this crate -- no new
//! dependency, and data-driven props rather than children-as-configuration.
//!
//! ## Provenance
//!
//! This module ports shadcn/ui's *conceptual* chart layer (`ChartConfig` ->
//! `--color-<key>` CSS variables (an inline `style` on the container,
//! which also carries `data-chart="<id>"`),
//! `ChartContainer`/`ChartTooltip`/`ChartLegend`) rather than its literal
//! React composition API, which has no Dioxus equivalent -- see
//! `dev-docs/archive/research/chart-2026-09-19.md` §1 (especially §1.4) for why a
//! `Chart { XAxis {...} Bar {...} }` children-introspection API is not
//! portable, and §6 for the design this module implements. Its
//! accessibility contract (§2 of the same report) follows: `role="img"` +
//! `<title>`/`<desc>` on the SVG root rather than Recharts'
//! `role="application"`, a real hidden `<table>` as the actual
//! screen-reader-facing data path, and an *optional* sighted-keyboard
//! stepping layer mirroring Recharts' own `accessibilityLayer`
//! (recharts/recharts @ 86ad3632ff3f83a742001ae6fc079dd27961a2a4,
//! `src/state/keyboardEventsMiddleware.ts`). Legend swatches use
//! `role="graphics-symbol"` per the W3C WAI-ARIA Graphics Module 1.0
//! (`https://www.w3.org/TR/graphics-aria-1.0/` -- an atomic, presentational-
//! children image role, the textbook fit for a color swatch).
//!
//! ## An engine, and a components layer built on it
//!
//! [`engine`] holds every pure computation and the plain chart data model,
//! with no dependency on this crate's other modules and no Dioxus types --
//! see its own module doc for why (in short: it is meant to be offered
//! upstream to `dioxus-community/dioxus-charts` once this component has
//! proven itself here). [`Chart`], [`ChartContainer`],
//! [`ChartTooltip`], and [`ChartLegend`] (this module's `components`, plus
//! [`ChartContext`]/[`use_chart`] bridging `engine`'s data into this
//! crate's reactivity, and `config` holding [`ChartConfig`]/
//! [`ChartSeries`] -- see that module's own doc for why the series model
//! specifically lives beside `context`, not under `engine`) are the
//! Dioxus-specific layer built on top of it. This split is purely
//! internal: every public item from every layer is re-exported flatly here
//! (`dioxus_primitives::chart::LinearScale`,
//! `dioxus_primitives::chart::ChartConfig`, ...), so it changes nothing
//! about how this crate's own consumers use the module.
//!
//! The layers: [`engine`] (unit-tested in isolation, no `dioxus` types),
//! [`ChartContext`]/[`use_chart`] (so a themed `preview` package can build
//! against the data model), and the rendering components, split per mark
//! family into `components::series::{area,bar,line,pie,radar,radial}`
//! behind `components::layout`'s shared plot geometry.
//!
//! ## Example
//!
//! ```rust
//! use dioxus_primitives::chart::{Curve, LinearScale, line_path};
//!
//! let y = LinearScale { domain: (0.0, 100.0), range: (200.0, 0.0) }.nice();
//! let path = line_path(&[(0.0, y.scale(10.0)), (100.0, y.scale(90.0))], Curve::Linear);
//! assert!(path.starts_with('M'));
//! ```

mod components;
mod config;
mod context;
pub mod engine;

// Glob re-export: `components` (including its own `series::*`, e.g.
// `AreaOptions`/`LineOptions`/...) is where each mark family lands its
// public types, so re-exporting it by name here would need a
// `chart/mod.rs` edit per family. `config`/`context` are not per-family
// extension points, so glob-exporting them too costs nothing and is
// simpler than hand-listing their few items.
pub use components::*;
pub use config::*;
pub use context::*;
// NOT a glob, deliberately, unlike the three above: `engine` declares a
// `pub mod` per math file (`curve`, `data`, `geometry`, `scale`, `stack`,
// `table`, `polar`, `radar`) alongside its actual
// re-exported items, and `pub use engine::*;` would re-export those
// SUBMODULE NAMES too, not just their contents. `engine::radar` and
// `components::series::radar` both
// then surfacing as bare `radar` collides the moment both sides are
// blanket-globbed into this same flat `chart::` namespace --
// `error: ambiguous glob re-exports`. Hand-listing
// `engine`'s actual value-level items sidesteps it entirely: no
// `engine`-level submodule shares a name with a `components`-level one
// (`curve`/`data`/`geometry`/`scale`/`stack`/
// `table`/`polar`/`radar` vs. `chart`/`container`/`legend`/`series`/
// `tooltip`/`area`/`bar`/`line`/`pie`/`radial` -- `radar` is the only
// overlap). A genuinely new top-level export name from an
// existing `engine` file is added to this list by hand; every name it
// already carries (`stack`, `nice_domain`, `BandScale`, `LinearScale`,
// `Curve`, `ChartDatum`, `ChartKind`, the three path builders) is part of
// the public surface.
pub use engine::{
    area_between_path, area_path, line_path, nice_domain, nice_ticks, stack, stack_with_mode,
    BandScale, CategoryScale, ChartDatum, ChartKind, Curve, LinearScale, PointScale, StackMode,
};
