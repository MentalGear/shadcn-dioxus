---
id: 260
title: "Every hover color-mix reads the theme's mix space (finish row 243)"
status: "LANDED 2026-10-19 (lane hovermix: every hover step reads --dx-hover-mix-space; gate + spec)"
size: "S"
area: theme
updated: 2026-10-19
---

# Row 260: Every hover color-mix reads the theme's mix space (finish row 243)

## Item

Every hover `color-mix` that steps a fill toward `--dx-foreground` reads `--dx-hover-mix-space` (shadcn base-nova mixes `in oklch`).

## Status / trigger

**LANDED 2026-10-19 (lane hovermix: every hover step reads --dx-hover-mix-space; gate + spec)**

## Size

S

## Detail

Row 243 fixed Button's secondary and destructive hovers by adding the theme token `--dx-hover-mix-space: oklch`. The same
`color-mix(in oklab, <fill>, var(--dx-foreground) N%)` step is still hand-written in `alert_dialog` (destructive action, 8%),
`toggle`/`toggle_group` (15%), `calendar` and `navigation_menu`, so those fills are a few levels off shadcn in the same way.
Construction: switch every site to `in var(--dx-hover-mix-space)` and add the class to a gate (no literal `in oklab`/`in srgb`
hover step toward `--dx-foreground` in a component sheet, reasoned allowlist), so a new component cannot reintroduce it;
extend `playwright/button-hover.spec.ts`'s formula check to the other components' hover fills.

## Update 2026-10-19

Switched to `color-mix(in var(--dx-hover-mix-space), <fill>, var(--dx-foreground) N%)`: alert_dialog (action hover 8%),
toggle and toggle_group (pressed 15%), calendar (today hover 6%), navigation_menu (featured hover/focus 8%, two sites),
bubble (secondary and muted hover 5%; was a hand-written `in oklch`), and the theme's dark `--dx-accent-selected` (15%,
the same pressed recipe the toggle uses). shadcn comparison (style-nova.css): its only `color-mix` steps are the Button
secondary hover and the Bubble secondary/muted hover, both `in oklch ... 5%`, which these now match. Toggle on, calendar
today and the featured card have no shadcn formula (shadcn uses plain `bg-muted`/`bg-accent`); those are dx recipes
chosen for dark-mode contrast (rows 145/231), so only their space changed, not the weight. No `/80` or token swap applied.

Gate: new `scripts/check-hover-mix-space.sh` (registered in run-gates.sh) fails on any literal-space `color-mix` with no
`transparent` argument (a step, not an alpha tint) in a component sheet or `preview/assets/*.css`; three reasoned
allowlist entries (Button accent-preset primary hover, shimmer gradient, preset `--border-strong`). Seeded check: the
pre-change toggle sheet and a toggle sheet with the token swapped back to `in oklab` both exit 1. Spec:
`playwright/hover-mix-space.spec.ts` (formula computed in the page, light and dark, six components). Not mine and
untouched: drawer, toast, carousel sheets had no literal step (see lane report).
