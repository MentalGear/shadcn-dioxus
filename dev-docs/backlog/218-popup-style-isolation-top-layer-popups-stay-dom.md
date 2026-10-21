---
id: 218
title: "Popup style isolation: top-layer popups stay DOM descendants of their trigger, so free-content rules in an…"
status: "NARROWED 2026-10-21: PARTLY LANDED 2026-10-15 (lane ancestorleak 900f394 + main-loop 526b51d…"
size: "M"
area: overlays
updated: 2026-10-20
---

# Row 218: Popup style isolation: top-layer popups stay DOM descendants of their trigger, so free-content rules in an…

## Item

Popup style isolation: top-layer popups stay DOM descendants of their trigger, so free-content rules in an ancestor's sheet restyle them

## Status / trigger

**NARROWED 2026-10-21: PARTLY LANDED 2026-10-15 (lane `ancestorleak` 900f394 + main-loop 526b51d, beaf3b0); the typography-reset half LANDED 2026-10-21 as [row 259](done/259-popup-root-typography-reset.md) (the `Popup root` theme rule + `scripts/check-popup-root-reset.sh`); open: the variant-slot gate (variant rules target the component's own root through slot variables, never descendants of an ancestor variant) and the same-component-nesting generalisation**

## Size

M

## Detail

**Found by** the computed-style snapshot review: the natural-language date picker's calendar cells were 24px instead of 28px and the chevrons 16px, because `input_group`'s addon `button` and `svg` descendant rules reached into the calendar popup. **Class:** a popover or dialog renders in the top layer but remains a DOM descendant of its trigger, so an ancestor component's descendant selector on a bare element (`button`, `svg`) restyles it. **Fixed:** the `svg` rule bounded with `:not(:where([popover], dialog) *)`, the `button` rule deleted, 7 more free-content rules hardened, a new gate `scripts/check-popup-boundary.sh` and `popup-style-isolation.spec.ts`. The gate also needed integration fixes after the p9parts merge: `alert/style.css` rules bounded (526b51d) and the avatar and toggle leaf `svg` rules given reasons in the allowlist (526b51d, beaf3b0; the tabs docs h2 was renamed). **Sibling class (same-component nesting), fixed for Tabs by `lane/p5tabs`:** the docs page's DEMO/CODE line Tabs' `[data-variant="line"] .dx-tabs-*` descendant selectors restyled every Tabs demo inside it; fixed by every Tabs root re-declaring all variant slots, with a spec. `check-popup-boundary.sh` does not cover `dx-` class subjects. **Still open:** (1) generalise the rule: variant rules must target the component's own root through slot variables, never descendants of an ancestor variant (gate candidate beside `check-cross-component-overrides.sh`); (2) ~~popup panel roots inherit the host's `font-size`~~ -- DONE, split out as row 259 (see below). The remaining open item is (1) only.

RECURRED 2026-10-20 (verify6): popup-style-isolation site-home -- the docs home cards became `.dx-card` (14px, 6be1887d) and the toast region, color-picker, combobox list and alert-dialog inside them inherit it (14 vs 16 px), the same class as the spec's ALLOWED font-size entries; the theme's typography reset at the popup boundary would close it and let those ALLOWED entries go.

## Split 2026-10-21 (row 259)

The typography-reset half (item 2 above and the 2026-10-20 "RECURRED ... site-home" note) is row 259, FIXED: one zero-specificity
theme rule resets every inherited property at the popup root, a gate keeps it complete, and the spec dropped its `ALLOWED`
font-size entries. Row 218 stays open for the variant-slot gate only (item 1), which another session owns; nothing in row 259
touches `check-cross-component-overrides.sh` or the Tabs-style variant rules.
