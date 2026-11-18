---
id: 528
title: "HoverSurface: Tooltip and HoverCard as one root, trigger and content, configured by one struct"
status: "APPROVED 2026-11-14 (owner: \"I approve your recommendation for generalizing the components and…"
size: "M: one new module beside tooltip.rs and…"
area: overlays
updated: 2026-11-18
---

# Row 528: HoverSurface, Tooltip and HoverCard as one surface

## Item

`tooltip.rs` (670 lines) and `hover_card.rs` (601 lines) are two copies of one hover-opened surface: a root that owns
the open state, a trigger that opens it on hover or focus after a delay, and content that `top_layer::use_anchored_panel`
places (row 519). Both already share the timers (`hover_timers.rs`, row 519) and the panel hook. Build one `HoverSurface`
root, trigger and content with a config struct (delays, which inputs open it, whether the content is hoverable, the ARIA
relation: `aria-describedby` for a tooltip, none for a hover card), and make `Tooltip` and `HoverCard` thin wrappers
over it with their public APIs unchanged. The first of [row 519](./done/519-anchored-panel-recipe-one-hook.md)'s
"Further candidates" (about 250 lines, about 5 h, low risk, inferred there). Row
[523](./523-edge-and-floating-close-mid-entry-under-a-keyframe.md) touches the same recipes and goes in the same lane.
Row [517](./517-chart-tooltip-on-the-tooltip-layer.md) (the chart tooltip on the `Tooltip`'s content layer with a point
anchor) follows it and builds on the content half.

## Status / trigger

APPROVED 2026-11-14 (owner: "I approve your recommendation for generalizing the components and tooltip as well").
Not started. Waits for the owner's go: the owner stopped the run at the end of phase 0 (2026-11-14, "you can finish
phase 0, then wait and report").

## Size

M: one new module beside `tooltip.rs` and `hover_card.rs`, the two components reduced to wrappers, no public API change
(`scripts/check-public-api.sh` stays unchanged), the install smoke for both components.

## Detail

- **What pins behaviour:** `playwright/tooltip.spec.ts`, `hover-card.spec.ts`, `hover-card-force-mount.spec.ts`,
  `dismiss-parity.spec.ts`, `anchored-close-clip.spec.ts`, `oracle/tier2-html/top-layer.spec.ts`; the unit tests in
  `tooltip.rs` (an SSR open tooltip with one folded `style`) and `hover_card.rs`. The lane reads them before it moves
  code.
- **Proof:** those specs and unit tests stay green on a release build, the public API snapshot is unchanged, the
  install smoke passes for `tooltip` and `hover_card`, and the two component files hold no root, trigger or content logic
  of their own.
