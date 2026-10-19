---
id: 260
title: "Every hover color-mix reads the theme's mix space (finish row 243)"
status: "BACKLOG (found 2026-10-19 by lane theme, row 243)"
size: "S"
area: theme
updated: 2026-10-19
---

# Row 260: Every hover color-mix reads the theme's mix space (finish row 243)

## Item

Every hover `color-mix` that steps a fill toward `--dx-foreground` reads `--dx-hover-mix-space` (shadcn base-nova mixes `in oklch`).

## Status / trigger

**BACKLOG (found 2026-10-19 by lane theme, row 243)**

## Size

S

## Detail

Row 243 fixed Button's secondary and destructive hovers by adding the theme token `--dx-hover-mix-space: oklch`. The same
`color-mix(in oklab, <fill>, var(--dx-foreground) N%)` step is still hand-written in `alert_dialog` (destructive action, 8%),
`toggle`/`toggle_group` (15%), `calendar` and `navigation_menu`, so those fills are a few levels off shadcn in the same way.
Construction: switch every site to `in var(--dx-hover-mix-space)` and add the class to a gate (no literal `in oklab`/`in srgb`
hover step toward `--dx-foreground` in a component sheet, reasoned allowlist), so a new component cannot reintroduce it;
extend `playwright/button-hover.spec.ts`'s formula check to the other components' hover fills.
