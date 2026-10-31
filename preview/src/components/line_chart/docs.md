Line charts show how values change across an ordered set of categories, such as months or dates. This page is a gallery of line configurations built from the [Chart](/component/chart/) package; install `chart` and you have everything. The Chart page explains the shared `ChartConfig` and `ChartDatum` model, colors and accessibility.

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
        ChartContainer { config, data, kind: ChartKind::Line,
            Chart {
                aria_label: "Visitors by month",
                // Line-specific settings (dots, labels) live in `line: LineOptions { .. }`.
                curve: Curve::Natural,
            }
            ChartTooltip { hide_label: true }
            // ChartLegend lists one swatch and label per series; add it once there is more than one.
            ChartLegend {}
        }
    }
    CardFooter { "Showing total visitors for the last 6 months" }
}
```

The pieces are siblings inside the container, as on the [Chart](/component/chart/) page.

## Quick start

```rust
let config = ChartConfig::new().series("desktop", "Desktop", "var(--dx-chart-1)");
let data = vec![
    ChartDatum { label: "January".into(), values: vec![Some(186.0)], ..Default::default() },
    ChartDatum { label: "February".into(), values: vec![Some(305.0)], ..Default::default() },
    // ...
];

ChartContainer { config, data, kind: ChartKind::Line,
    Chart {
        aria_label: "Visitors by month, desktop",
        x_label: "Month",
    }
    ChartTooltip { hide_label: true } // one series does not need its name repeated
}
```

## Curve

`Chart`'s `curve` prop sets the interpolation: `Curve::Natural` is the smooth spline most shadcn demos use (it can overshoot a point slightly between data points), `Curve::Monotone` (the default) is smooth and never overshoots a data point, `Curve::Linear` draws straight segments, and `Curve::Step` draws midpoint steps -- each value holds until halfway to the next point (`StepBefore`/`StepAfter` put the riser at a point instead). A missing value (`None`) breaks the line at that point.

## Line options

Line-specific settings live in `line: LineOptions { .. }` on `Chart`:

- `dots: true` draws a circle at every defined point, in the series color or in `ChartDatum::color` when a datum sets one. The active point's circle is drawn twice as large. `dot_radius` sets the base radius (default `3`).
- `dot: Some(..)` replaces the circle with your own renderer. It receives a `DotContext` for each defined point, with the scaled position, the raw value and whether the point is active.
- `labels` draws a text label above each point: `LineLabels::Value` shows the point's value, and `LineLabels::Custom(callback)` calls your callback with the point index so you can label by category or anything else.
- `stroke_width` sets the line thickness (default `2`).
