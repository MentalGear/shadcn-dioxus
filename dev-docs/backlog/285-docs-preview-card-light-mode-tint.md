---
id: 285
title: "Docs preview card: a visible light-mode tint without breaking Toggle hover contrast"
status: "OPEN. Needs the owner's call on the light-mode tint strength once the Toggle/Pagination hover is…"
size: "S"
area: preview
updated: 2026-10-19
---

# Row 285: Docs preview card: a visible light-mode tint without breaking Toggle hover contrast

## Item

Owner asked (2026-10-19) for the docs preview cards (`.dx-component-preview-frame`) to sit on a soft in-between of the page
background and the card token (`--dx-docs-preview-surface`, `preview/assets/main.css`). In dark mode that works
(50% page / 50% card). In light mode page and card are both `#fff`, so the mix is with `--dx-muted`, and PR #108 had to
keep it at 85/15 (about `#fefefe`, barely visible): a stronger tint (50% gave `#fafafa`) made a hovered Toggle
(`rgb(248,248,248)`) only 5/255 from the surface where `theme-preset-contrast.spec.ts` requires 6.

## Status / trigger

**OPEN.** Needs the owner's call on the light-mode tint strength once the Toggle/Pagination hover is adjusted.

## Size

S

## Detail

### Options

- Darken Toggle's light hover/pressed fill slightly (component change, then tint the surface to taste); or
- loosen the spec's distinctness threshold for the preview surface (weaker guard); or
- accept the near-white light surface.

Recommendation: darken the Toggle hover mix toward shadcn's base-nova value and re-check Pagination's hover
(`dark-surface-contrast.spec.ts` hit the same 5/255 limit at 50%), then pick a visible light tint (about 50/50 with muted).
