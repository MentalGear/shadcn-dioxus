---
id: 385
title: "The focus ring follows the picked accent, clamped to 3:1 per mode"
status: "LANDED on main (merge 74c604ef, deployed by batch 28, main daf90a8f). DECIDED 2026-10-24 (owner)…"
size: "S"
area: theme
updated: 2026-10-26
---

# Row 385: the focus ring follows the picked accent

## Item

shadcn's accent presets set `--primary` (and secondary, charts, sidebar-primary) but never `--ring`, so the keyboard focus ring stayed the base colour's grey whatever accent the picker showed. The owner wants the ring to follow the accent ("clamped accent"): the accent's hue and chroma, lightness clamped per mode so every accent clears the 3:1 non-text minimum (WCAG 1.4.11); with no accent picked the ring stays exactly shadcn's (the base preset's or a host's `--ring`, else neutral's grey).

## Status / trigger

LANDED on main (merge 74c604ef, deployed by batch 28, main daf90a8f).

DECIDED 2026-10-24 (owner), BUILT on `lane/accent-ring`. With an accent picked the ring takes the accent's hue and chroma, lightness clamped to at most L 0.62 in light and at least L 0.6 in dark; with no accent it is untouched, to the byte.

Construction (`preview/assets/dx-components-theme.css`): one rule, `:root[data-theme-accent]`, provides the slot `--dx-accent-ring` as `--dx-primary` with `oklch(from ... min(l, var(--dx-accent-ring-l-max)) c h)` in light and `max(l, var(--dx-accent-ring-l-min))` in dark (the `--dx-primary-ink` construction, so a value already inside the bound is returned unchanged); `--dx-ring-source` reads `--dx-accent-ring` first, then `--ring`, then shadcn neutral's literal. With no accent the slot is undeclared and the chain is what it was. Every ring token derives from `--dx-ring-color`, so this is the only change; the "Contrast mode" block is untouched (its light clamp, 0.65, finds an accent ring already under it). The accent clamp does not read the contrast mode: an accent ring is not a shadcn look, so it clears 3:1 in `shadcn` and `wcag` alike; the no-accent grey keeps today's per-mode behaviour (shadcn's own under 3:1 in light by default, clamped in `wcag`). The `docs` accent (main.css, the docs link blue) is an accent like the others: its `--primary` is the input, no special case.

Measured (Chromium canvas sRGB, ring edge against `--dx-background`, `--dx-card` and `--dx-popover`, every base x accent, 10 x 18 combinations per cell): a plain copy of `--primary` is 1.54:1 (lime, light) and 1.96:1 (dark, 15 accents 1.96-2.79); with the clamp the minimum is 3.37:1 in light (lime on white; the bound's limit, a saturated yellow-green is brighter than a grey at the same L) and 3.84:1 in dark (fuchsia on the olive card), identical in `shadcn` and `wcag`. X = 0.62 is the largest ceiling that keeps >= 3.2:1 (0.64 gives 3.10). Y = 0.6: the 3:1 bound alone allows 0.55 (3.11:1, 0.57 gives 3.39, 0.58 gives 3.53), but the selected Tag's near-black ink on the ring fill needs 4.5:1 and fuchsia at L 0.58 gives 4.37, at 0.59 4.55, at 0.6 4.75, so 0.6. Hue: 16 of 18 light arms pass through unchanged (L 0.456-0.563), lime and yellow move 4 and 9 degrees (gamut mapping); dark arms move at most 2.3 degrees, 16 of 18 are raised to L 0.6 and lime and yellow pass through.

Knock-on: `tag_group` paints a selected tag with `--dx-ring-color` as its background and kept the page foreground as text, which a chromatic ring fill breaks (indigo in light 2.45:1, yellow in dark 1.83:1; 7.66 and 4.54 with no accent). Its sheet now derives the ink from the fill (near-white at or below L 0.575, near-black above, the `--dx-sidebar-primary-foreground` rule of the Contrast mode block): worst 4.67:1 in light and 4.51:1 in dark over every base x accent x contrast setting. Not changed, flagged: `main.rs` `BlockProfile`'s "Verified" icon is white on `--dx-ring-color` (an icon, 3:1): a lime or yellow dark ring (L 0.77-0.8) gives it about 1.5:1; the badge variant of the Badge page already clamps its fill with `min(l, 0.5)` and the same one-liner would fix this one.

Guards: `playwright/ring-contrast.spec.ts` (every accent x base x mode x contrast setting >= 3:1, hue tracks `--primary`, lightness only moves into the bound, no accent is byte-identical to the unclamped source, a host `--ring` never beats an accent).

## Size

S

## Detail

See the Status section. Tuning the two bounds is a two-number diff (`--dx-accent-ring-l-max`, `--dx-accent-ring-l-min`); `playwright/ring-contrast.spec.ts` carries the same two numbers as `ACCENT_L_MAX` / `ACCENT_L_MIN`.
