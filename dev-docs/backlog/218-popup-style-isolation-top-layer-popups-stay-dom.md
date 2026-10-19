---
id: 218
title: "Popup style isolation: top-layer popups stay DOM descendants of their trigger, so free-content rules in an…"
status: "NARROWED 2026-10-19 (backlog review): PARTLY LANDED 2026-10-15 (lane ancestorleak 900f394 +…"
size: "M"
area: overlays
updated: 2026-10-19
---

# Row 218: Popup style isolation: top-layer popups stay DOM descendants of their trigger, so free-content rules in an…

## Item

Popup style isolation: top-layer popups stay DOM descendants of their trigger, so free-content rules in an ancestor's sheet restyle them

## Status / trigger

**NARROWED 2026-10-19 (backlog review): PARTLY LANDED 2026-10-15 (lane `ancestorleak` 900f394 + main-loop 526b51d, beaf3b0); open: the typography reset and the same-component-nesting generalisation; remaining: the typography reset for popup roots and the variant-slot gate**

## Size

M

## Detail

**Found by** the computed-style snapshot review: the natural-language date picker's calendar cells were 24px instead of 28px and the chevrons 16px, because `input_group`'s addon `button` and `svg` descendant rules reached into the calendar popup. **Class:** a popover or dialog renders in the top layer but remains a DOM descendant of its trigger, so an ancestor component's descendant selector on a bare element (`button`, `svg`) restyles it. **Fixed:** the `svg` rule bounded with `:not(:where([popover], dialog) *)`, the `button` rule deleted, 7 more free-content rules hardened, a new gate `scripts/check-popup-boundary.sh` and `popup-style-isolation.spec.ts`. The gate also needed integration fixes after the p9parts merge: `alert/style.css` rules bounded (526b51d) and the avatar and toggle leaf `svg` rules given reasons in the allowlist (526b51d, beaf3b0; the tabs docs h2 was renamed). **Sibling class (same-component nesting), fixed for Tabs by `lane/p5tabs`:** the docs page's DEMO/CODE line Tabs' `[data-variant="line"] .dx-tabs-*` descendant selectors restyled every Tabs demo inside it; fixed by every Tabs root re-declaring all variant slots, with a spec. `check-popup-boundary.sh` does not cover `dx-` class subjects. **Still open:** (1) generalise the rule: variant rules must target the component's own root through slot variables, never descendants of an ancestor variant (gate candidate beside `check-cross-component-overrides.sh`); (2) popup panel roots inherit the host's `font-size` (DropdownMenu or Select inside a breadcrumb, card or item renders 14px against 16px alone): one theme rule `:where([popover], dialog[class^="dx-"])` resetting inherited typography.
