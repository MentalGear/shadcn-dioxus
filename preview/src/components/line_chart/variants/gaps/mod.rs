//! A missing value (`None`) breaks the line: it is a gap, not a bridge across the month.

use super::super::component::*;
use crate::components::card::{Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle};
use crate::components::icons::TrendingUp;
use dioxus::prelude::*;

/// The chart's rows, `(month, desktop)`.
fn chart_data() -> Vec<ChartDatum> {
    const ROWS: [(&str, Option<f64>); 6] = [
        ("January", Some(186.0)),
        ("February", Some(305.0)),
        ("March", None),
        ("April", Some(73.0)),
        ("May", Some(209.0)),
        ("June", Some(214.0)),
    ];
    ROWS.iter()
        .map(|&(label, desktop)| ChartDatum {
            label: label.to_string(),
            values: vec![desktop],
            ..Default::default()
        })
        .collect()
}

/// The one series drawn.
fn chart_config() -> ChartConfig {
    ChartConfig::new().series("desktop", "Desktop", "var(--dx-chart-1)")
}

#[component]
pub fn Demo() -> Element {
    rsx! {
        Card {
            CardHeader {
                CardTitle { "Line Chart - Gaps" }
                CardDescription { "January - June 2024" }
            }
            CardContent {
                ChartContainer { config: chart_config(), data: chart_data(), kind: ChartKind::Line,
                    Chart {
                        aria_label: "Visitors by month, desktop",
                        margin: ChartMargin { left: 12.0, right: 12.0, ..ChartMargin::NONE },
                        curve: Curve::Natural,
                        cursor: false,
                        line: LineOptions {
                            dots: true,
                            ..Default::default()
                        },
                    }
                    ChartTooltip { hide_label: true }
                }
            }
            CardFooter { class: "dx-chart-footer",
                div { class: "dx-chart-footer-trend",
                    "Trending up by 5.2% this month"
                    TrendingUp { size: "16px" }
                }
                div { class: "dx-chart-footer-caption", "Showing total visitors for the last 6 months" }
            }
        }
    }
}
