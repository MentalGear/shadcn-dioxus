Radar charts compare several measures for the same item by laying categories out around a circle and drawing one polygon per series. This page covers what is specific to `kind: ChartKind::Radar`: the grid options, how series are drawn, and hover. Everything else (`ChartConfig`, colors, the data table, keyboard behavior) is shared with every chart; see [Chart](/component/chart/).

## Component Structure

```rust
// Card is the frame the demos use; a chart works without it.
Card {
    CardHeader {
        CardTitle { "Visitors" }
        CardDescription { "January - June 2024" }
    }
    CardContent {
        // ChartContainer, Chart, ChartTooltip and ChartLegend come from the Chart package this page
        // depends on; this page adds no piece of its own. Chart reads its kind from the container.
        ChartContainer { config, data, kind: ChartKind::Radar,
            Chart {
                aria_label: "Visitors by month",
                // The polygon, grid and axis settings are `radar: RadarOptions { .. }`.
                radar: RadarOptions { fill_opacity: vec![0.6], ..Default::default() },
            }
            ChartTooltip {}
            // ChartLegend lists one swatch and label per series; add it once there is more than one.
            ChartLegend {}
        }
    }
    CardFooter { "Showing total visitors for the last 6 months" }
}
```

Each `ChartDatum` is one axis around the circle and each series one polygon. The pieces are siblings inside the container, as on the [Chart](/component/chart/) page.

## Quick start

```rust
let config = ChartConfig::new()
    .series("desktop", "Desktop", "var(--dx-chart-1)")
    .series("mobile", "Mobile", "var(--dx-chart-2)");

// One ChartDatum per category, which becomes one axis around the circle.
let data = vec![
    ChartDatum { label: "January".into(), values: vec![Some(186.0), Some(80.0)], ..Default::default() },
    // ...
];

ChartContainer { config, data, kind: ChartKind::Radar,
    Chart {
        aria_label: "Visitors by month, desktop and mobile",
        radar: RadarOptions {
            fill_opacity: vec![0.6], // per series; unset series are opaque
            ..Default::default()
        },
    }
    ChartTooltip {}
    ChartLegend {}
}
```

The options follow Recharts' `RadarChart`, `Radar`, `PolarGrid`, `PolarAngleAxis` and `PolarRadiusAxis`, so a shadcn/ui example carries over unchanged. A polar chart is a square at most 250px wide, centred in its container, and lengths are CSS pixels. The outer radius is `Radius::Percent(80.0)` of the max radius by default (96px in a 250px chart); `outer_radius` takes a `Radius` to change it, and the `Chart`'s `margin` (a `ChartMargin`) shrinks the space it is measured from.

The radius axis always has five "nice" ticks starting at zero (Recharts' `tickCount` of 5): data up to 305 gives rings at 0, 80, 160, 240 and 320, and a vertex sits at `value / 320` of the outer radius.

## Grid

`RadarGrid` chooses the rings drawn at the radius axis' ticks:

- `Polygon` (default): straight-sided rings with a spoke to each category.
- `PolygonNoLines`: the same rings without spokes.
- `Circle`: circular rings, with spokes.
- `CircleNoLines`: circular rings, no spokes.
- `CircleFill` and `Fill`: circular or polygon rings and spokes tinted in the first series' color at 20% opacity, so the rings shade towards the centre.
- `None`: no grid.
- `Custom { polar_radius, spokes }`: rings at exactly the radii you give, in pixels (Recharts' `polarRadius`), and whether spokes are drawn.

## How series are drawn

Each series is one closed polygon (`path[data-slot="chart-radar-area"]`), filled with its color at its own `fill_opacity` entry, by series position. A series with no entry is opaque, as in Recharts, so `fill_opacity: vec![0.6]` over two series draws the first translucent under an opaque second. Polygons have no outline. `lines_only: true` draws only an outline: no fill and a 2px stroke. `dots: true` adds a 4px-radius circle (`circle[data-slot="chart-dot"]`) at every vertex.

A category with no value for a series (`None`) is skipped: its two neighbors are joined directly instead of drawing a made-up zero.

## Axes

- `axis_labels` (default `true`) prints each category's name 8px beyond the rim, anchored away from the centre: left-aligned on the right half, right-aligned on the left half, centred at the top and bottom.
- `ticks: Some(vec![RadarTick { .. }])` replaces those names with your own two-line labels: a first line of `(text, muted)` runs and an optional muted caption below.
- `radius_axis: Some(60.0)` prints the radius axis' tick values along the spoke at that angle (degrees, `0` at three o'clock, counter-clockwise), rotated to read along it.

The polygons grow from the centre when the chart first appears; nothing animates under `prefers-reduced-motion: reduce`.

## Hover

Each category owns an angular wedge reaching from the center to the outer edge. Hovering anywhere in a wedge activates that category, and the tooltip shows every series' value for it. Keyboard stepping activates categories in order.

## Accessibility

The hidden data table lists every category and every series' exact value regardless of the visual options above. The SVG is a named image and the wrapper is keyboard focusable; see [Chart](/component/chart/) for the full behavior.
