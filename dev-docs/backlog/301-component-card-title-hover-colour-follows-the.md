---
id: 301
title: "Component-card title hover colour follows the theme; offer the default blue in the palette picker"
status: "OPEN (owner review 2026-10-19), partly already true (see below)."
size: "S"
area: theme
updated: 2026-10-19
---

# Row 301: Component-card title hover colour follows the theme; offer the default blue in the palette picker

## Item

The `.dx-component-card-title` hover colour isn't a theme token. Sync it, and add that blue to the palette chooser.

## Status / trigger

OPEN (owner review 2026-10-19), partly already true (see below).

## Size

S

## Detail

- **Today:** `.dx-component-card-title-link:hover` uses `--highlight-color-tertiary` (`main.css:2052-2056`). That is a fixed, contrast-tuned link blue (`#4d94ff` dark, `#0065ff` light, `main.css:35`, row 39).
- **When an accent is picked** it already follows: `main.css:57-60` derives it from `--dx-primary`.
- **The gap is the default state**, where it is that fixed blue and the picker has no swatch that matches it.
- `--dx-accent` is the wrong token for text. In shadcn it is the neutral hover background, so text in it would be near-invisible.
- **Target:**
  - Add the site blue as a palette swatch (`ACCENTS`, `theme.rs:69`). The existing "blue" accent is a different shade.
  - Route the card-title hover, and the other site links that use `--highlight-color-tertiary`, through the primary-derived token, so default and picked accents share one path.
  - Keep `theme-preset-contrast.spec.ts` green.
- Owner default (2026-10-19): no, just an extra swatch.
