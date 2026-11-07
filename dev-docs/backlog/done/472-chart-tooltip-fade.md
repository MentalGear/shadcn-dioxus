---
id: 472
title: "Chart tooltips fade in and out with the theme's floating motion tokens"
status: "DONE 2026-10-26, landed in PR #155 (lane lane/chart-tooltip-472; owner request: \"chart tooltips…"
size: "S"
area: components
updated: 2026-10-26
---

# Row 472: chart tooltip fade

## Item

The chart tooltip (`.dx-chart-tooltip`, `preview/src/components/chart/style.css` "Tooltip" section, `ChartTooltip` in `chart_tooltip`) appears and disappears without a transition. It should fade in and out like the library's other floating surfaces, using a standard design token rather than a literal duration: the theme's `--dx-floating-duration` (and the shared `dx-fade-*` motion through the `--dx-floating-enter`/`--dx-floating-exit` slots where that fits a non-popover surface), with reduced motion handled by the theme's one reduced-motion rule.

## Status / trigger

DONE 2026-10-26, landed in PR #155 (lane `lane/chart-tooltip-472`; owner request: "chart tooltips should have a fade in/out transition (use a standard design token)").

Fix: `preview/src/components/chart/style.css` "Tooltip": opacity and a discrete `display` transition on `--dx-floating-duration` with the
floating ease tokens, `@starting-style` for the open fade, `visibility: hidden` only while open and unplaced, closed `opacity: 0`
with the exit curve, and reduced motion as `transition-duration: var(--dx-motion-duration-reduced)` in the sheet's existing block
(the theme's shared reduced-motion layer does not reach a non-interactive chart tooltip).

Root cause of the follow-spec reds the fade first exposed (measured on the built page): a closing tooltip stays displayed for the
whole fade, and the closed render emptied its content (no active datum) and re-anchored it at the cursor-less fallback. The
emptied box shrank, its `onresize` recorded that smaller size as `tip_size`, and a reopen inside the fade placed by the stale size
(wrong flip/clamp, x off by 58 px and by 117 to 144 px at the right edge) and then slid to the right place once the content
re-measured. Before the fade a closed tooltip was `display: none`, so every open re-measured from 0x0 while hidden.

Construction (`primitives/src/chart/components/tooltip.rs`): (1) while closed the tooltip keeps rendering the last open datum at the
last open placement, so the fade-out neither collapses nor moves; (2) every closed to open transition clears `placed` and measures
the box again (`onmounted` rect, no JS) before the tooltip may show, because a reopen inside the fade produces no 0x0 resize and
the new datum's label can change the width. The first visible frame of any open is therefore at its target.

Spec: `playwright/chart_tooltip_fade.spec.ts` (closed at rest with opacity 0 and the token's duration; opens to 1 and fades out;
both reduced-motion settings). `playwright/chart_tooltip_follow.spec.ts` "prefers-reduced-motion turns the transition off" no longer
expects the literal `0s`: the theme's `--dx-motion-duration-reduced` token is the single source of truth for "off" (a near-zero
duration, not zero), so the assertion reads the token's computed value from the page and requires every transition duration to
equal it. The no-fly-in and first-open cases are unchanged.

Proof: both specs 37/37 green on a debug SSG build (twice); red before the Rust fix (3 follow cases).

## Size

S

## Detail

- The tooltip moves while it is shown (it follows the pointer or the touch scrub along the category axis), so only `opacity` fades; position updates must stay instant, never transitioned.
- Fade out needs the element to stay mounted (or stay painted) until the transition ends; prefer a `data-state` attribute with an opacity transition over unmounting on leave.
- Gates that apply: `check-duplicate-keyframes.sh` (no pasted fade keyframes), `check-css-state-recipes.sh`, `check-infinite-animations.sh`; a spec reads the computed `transition-duration`/opacity at rest under both `reducedMotion` settings (the `scrim-reduced-motion.spec.ts` pattern).
