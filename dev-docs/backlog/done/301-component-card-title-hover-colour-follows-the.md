---
id: 301
title: "Component-card title hover colour follows the theme; offer the default blue in the palette picker"
status: "LANDED 2026-10-20 (theme-touch lane 1ba0b2e5, spec theme-preset-contrast.spec.ts \"component-card…"
size: "S"
area: theme
updated: 2026-10-20
---

# Row 301: Component-card title hover colour follows the theme; offer the default blue in the palette picker

## Item

The `.dx-component-card-title` hover colour isn't a theme token. Sync it, and add that blue to the palette chooser.

## Status / trigger

**LANDED 2026-10-20 (theme-touch lane 1ba0b2e5, spec `theme-preset-contrast.spec.ts` "component-card title hover (row 301)").** Owner default taken: no re-routing of the other link users, just an extra swatch.

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

## Landed

- **Swatch "Docs blue"** (slug `docs`, first accent after "Default"): `theme.rs` `ACCENTS`, a `[data-theme-accent="docs"]` block in `theme-presets.css` (fill `oklch(0.562 0.242 260)` = #0065ff, in both modes because white on the dark-mode #4d94ff is 2.9:1; secondary and charts are shadcn's blue). The block is not shadcn data, so `dev-docs/archive/research/theme-2026-10-03/tools/gen_presets.mjs` emits it from a `SITE_ACCENTS` literal and a regeneration reproduces the file.
- **Ink**: `main.css` names the default pair once (`--dx-docs-blue`) and `:root[data-theme-accent="docs"]` points `--highlight-color-tertiary` at it, so picking the swatch gives exactly today's #0065ff / #4d94ff (the generic clamp, `min(l, 0.5)`, would have darkened the light value). The card title hover already followed every other accent (`main.css` derives it from `--dx-primary`), now asserted by the spec for default, Docs blue and orange in both modes.
- Contrast sweep (`theme-preset-contrast.spec.ts`, 18 accents x 9 bases x 2 modes) stays green; counts in `theme-preset.spec.ts` are 1 + 18.
- Screenshots: `/home/user/tgt/shots-theme-touch/301-picker-docs-blue-{light,dark}.png`.
