use crate::components::{
    button::{Button, ButtonSize, ButtonVariant},
    drawer::{
        Drawer, DrawerClose, DrawerContent, DrawerDescription, DrawerFooter, DrawerHandle,
        DrawerHeader, DrawerSide, DrawerTitle, SnapPoint,
    },
};
use dioxus::prelude::*;
use dioxus_primitives::AsChild;

/// shadcn's `drawer-snap-points` (Base UI `snapPoints`): the drawer rests at 148px, 355px or fully open.
/// A drag settles on the nearest point (a flick projects further), dragging below 148px closes it. The
/// active point is controlled here, so the line in the header and the footer button follow it.
const SNAP_POINTS: [SnapPoint; 3] = [
    SnapPoint::Px(148.0),
    SnapPoint::Px(355.0),
    SnapPoint::Fraction(1.0),
];

fn label(point: Option<SnapPoint>) -> &'static str {
    match point {
        Some(SnapPoint::Px(355.0)) => "355px",
        Some(SnapPoint::Fraction(_)) => "full",
        _ => "148px",
    }
}

#[component]
pub fn Demo() -> Element {
    let mut open = use_signal(|| false);
    let mut active = use_signal(|| Some(SnapPoint::Px(148.0)));

    rsx! {
        Button {
            variant: ButtonVariant::Outline,
            onclick: move |_| open.set(true),
            "Open Drawer"
        }
        Drawer {
            open: open(),
            on_open_change: move |v| open.set(v),
            side: DrawerSide::Bottom,
            snap_points: SNAP_POINTS.to_vec(),
            active_snap_point: active(),
            on_active_snap_point_change: move |point| active.set(Some(point)),
            DrawerContent {
                DrawerHandle {}
                DrawerHeader {
                    DrawerTitle { "Snap points" }
                    DrawerDescription {
                        "Drag the handle to a snap point. Active: "
                        span { "data-testid": "active-snap", "{label(active())}" }
                    }
                    // In the header, so it stays in view at the lowest snap point.
                    Button {
                        size: ButtonSize::Sm,
                        align_self: "flex-start",
                        onclick: move |_| active.set(Some(SnapPoint::Fraction(1.0))),
                        "Expand"
                    }
                }
                div {
                    display: "flex",
                    flex_direction: "column",
                    gap: "0.75rem",
                    padding: "0 1rem",
                    color: "var(--dx-muted-foreground)",
                    font_size: "0.875rem",
                    for n in 1..=24 {
                        p { margin: "0", "Paragraph {n}: the content scrolls once the drawer is fully open." }
                    }
                }
                DrawerFooter {
                    DrawerClose {
                        as: |el: AsChild| rsx! {
                            Button { variant: ButtonVariant::Outline, attributes: el.attributes, {el.children} }
                        },
                        "Close"
                    }
                }
            }
        }
    }
}
