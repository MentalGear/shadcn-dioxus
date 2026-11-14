---
id: 517
title: "Chart tooltips are confined to the chart box and get pushed around in small charts: build them on the Tooltip's content layer with a point anchor"
status: "APPROVED 2026-11-14. The tooltip escapes the chart box by default, hides on scroll where it cannot…"
size: "M (inferred)."
area: primitives
updated: 2026-11-14
---

# Row 517: the chart tooltip on the Tooltip's content layer

## Item

`primitives/src/chart/components/tooltip.rs` places the tooltip inside the chart container: beside the point, flipped
at the chart's right and bottom edge, then clamped into the chart box, from a rect it measures over the bridge on every
open. In a small chart the box is tight and the tooltip covers the point. The owner asks whether it can overlay above
its container and whether the library's `Tooltip` can be used directly.

## Status / trigger

APPROVED 2026-11-14. The tooltip escapes the chart box by default, hides on scroll where it cannot follow, and takes the `Tooltip`'s Escape scope. It follows rows 519 and 449, which shape the layer it sits on.

## Size

M (inferred).

## Detail

- **The construction.** The chart renders a point anchor (an element at the active point or the pointer) and the
  tooltip is the `Tooltip`'s content layer anchored to it: the top layer (`top_layer::use_popover_shown_while_mounted`),
  CSS anchor positioning with the library's fallback (`use_anchor_position_fallback`), the viewport as the boundary.
  The chart keeps what the `Tooltip` component does not fit: it drives `open` itself (no hover delay, no trigger
  element to describe) and its content changes per point. Where anchor positioning holds, the tooltip follows its
  anchor through a scroll and the per-open measurement goes away; in the fallback it hides on scroll (owner
  2026-11-14).
- **Open before the build.** Whether an anchor-positioned box can keep the slide between points (`data-motion`); the
  default (escaping the chart box deviates from Recharts' `allowEscapeViewBox: false`; the opt-out prop carries that
  name).
- **From the audit** ([overlay layering audit](../research/overlay-layering-audit-2026-11-14.md)). Three small props on
  `Tooltip` (an inert anchor, no ARIA linkage, no trigger handlers); the Escape-scope decision (the Tooltip's dismiss
  layer is document-wide); the fallback's anchor that moves while open (placement re-runs only on dependency change
  and does not track a moving anchor); and the open question of the slide.
- **Proof.** A spec with the chart inside a clipping ancestor and inside a scrolling one: the tooltip is fully
  visible, stays in the viewport, and hides or follows on scroll.
