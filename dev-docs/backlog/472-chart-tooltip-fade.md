---
id: 472
title: "Chart tooltips fade in and out with the theme's floating motion tokens"
status: "BACKLOG. Owner request 2026-10-26 (backlog thread): \"chart tooltips should have a fade in/out…"
size: "S"
area: components
updated: 2026-10-26
---

# Row 472: chart tooltip fade

## Item

The chart tooltip (`.dx-chart-tooltip`, `preview/src/components/chart/style.css` "Tooltip" section, `ChartTooltip` in `chart_tooltip`) appears and disappears without a transition. It should fade in and out like the library's other floating surfaces, using a standard design token rather than a literal duration: the theme's `--dx-floating-duration` (and the shared `dx-fade-*` motion through the `--dx-floating-enter`/`--dx-floating-exit` slots where that fits a non-popover surface), with reduced motion handled by the theme's one reduced-motion rule.

## Status / trigger

BACKLOG. Owner request 2026-10-26 (backlog thread): "chart tooltips should have a fade in/out transition (use a standard design token)".

## Size

S

## Detail

- The tooltip moves while it is shown (it follows the pointer or the touch scrub along the category axis), so only `opacity` fades; position updates must stay instant, never transitioned.
- Fade out needs the element to stay mounted (or stay painted) until the transition ends; prefer a `data-state` attribute with an opacity transition over unmounting on leave.
- Gates that apply: `check-duplicate-keyframes.sh` (no pasted fade keyframes), `check-css-state-recipes.sh`, `check-infinite-animations.sh`; a spec reads the computed `transition-duration`/opacity at rest under both `reducedMotion` settings (the `scrim-reduced-motion.spec.ts` pattern).
