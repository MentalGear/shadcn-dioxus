---
id: 243
title: "Button hover fills 3 levels off shadcn"
status: "LANDED 2026-10-19 (lane theme: hover recipe now in shadcn's oklch space)"
size: "S"
area: theme
updated: 2026-10-18
---

# Row 243: Button hover fills 3 levels off shadcn

## Item

Button hover fills 3 levels off shadcn

## Status / trigger

**LANDED 2026-10-19 (lane theme: hover recipe now in shadcn's oklch space)**

## Size

S

## Detail

After row 231's token fix, 148/156 Button values match base-nova; besides the deliberate solid destructive, the secondary hover fill is #e8e8e8 vs #e5e5e5 (light) and #2d2d2d vs #2f2f2f (dark) -- from the hover recipe, not the tokens. Table: lane docs-site's `button-table-final.md` (session scratch).

## Update 2026-10-19

Root cause: shadcn base-nova writes the secondary hover as `hover:bg-[color-mix(in_oklch,var(--secondary),var(--foreground)_5%)]`
(style-nova.css `.cn-button-variant-secondary`); ours was the same mix `in oklab`, which differs by 3 levels on a grey
(#e8e8e8 vs #e5e5e5 light, #2d2d2d vs #2f2f2f dark). Fix by construction: one theme token `--dx-hover-mix-space: oklch`
(dx-components-theme.css), read by every Button hover that steps a fill toward `--dx-foreground` (Secondary 5%, the solid
Destructive 8%): `color-mix(in var(--dx-hover-mix-space), <fill>, var(--dx-foreground) N%)`. Spec
`playwright/button-hover.spec.ts` computes shadcn's formula in the page from the resolved tokens and asserts the hover fill
of Secondary, Destructive and Primary in light and dark, plus the two literal anchors #e5e5e5 / #2f2f2f.
Not touched (not in this lane): the same oklab step recipe in alert_dialog (destructive action, 8%), toggle/toggle_group
(15%), calendar, navigation_menu; they should read `--dx-hover-mix-space` too if they are meant to follow shadcn.
