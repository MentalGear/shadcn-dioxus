---
id: 198
title: "Point-anchor and fallback clamps measured a shrink-to-fit panel at the position they were replacing"
status: "LANDED 2026-10-14"
size: "S (done)"
area: overlays
updated: 2026-10-14
---

# Row 198: Point-anchor and fallback clamps measured a shrink-to-fit panel at the position they were replacing

## Item

Point-anchor and fallback clamps measured a shrink-to-fit panel at the position they were replacing

## Status / trigger

**LANDED 2026-10-14**

## Size

S (done)

## Detail

A context menu opened at x=1263 measured 148px (squeezed) instead of 160, was placed with it and grew back to 0px from the edge instead of 4 (hidden while `min-width` was 220; the menu density change exposed it; the old spec read the box mid-scale-animation). `top_layer.rs` `measureAtFullWidth` measures at the viewport origin, then positions in the same synchronous block; applied to `use_point_anchor_clamp` and `use_anchor_position_fallback`. Note for specs: `getBoundingClientRect` of a popup can keep the entry animation's start scale after it finishes until the next style change (`menu-indicator-gap.spec.ts` commits and cancels finished animations).
