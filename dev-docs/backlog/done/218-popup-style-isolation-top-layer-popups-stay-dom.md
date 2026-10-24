---
id: 218
title: "Popup style isolation: top-layer popups stay DOM descendants of their trigger, so free-content rules in an…"
status: "DONE 2026-10-23 (branch claude/variant-scope): item (1), the variant-slot gate, LANDED as…"
size: "M"
area: overlays
updated: 2026-10-23
---

# Row 218: Popup style isolation: top-layer popups stay DOM descendants of their trigger, so free-content rules in an…

## Item

Popup style isolation: top-layer popups stay DOM descendants of their trigger, so free-content rules in an ancestor's sheet restyle them

## Status / trigger

**DONE 2026-10-23 (branch `claude/variant-scope`): item (1), the variant-slot gate, LANDED as `scripts/check-variant-scope.sh` + `playwright/variant-nesting.spec.ts` (see "Fix 2026-10-23" below). Before that, NARROWED 2026-10-21: PARTLY LANDED 2026-10-15 (lane `ancestorleak` 900f394 + main-loop 526b51d, beaf3b0); the typography-reset half LANDED 2026-10-21 as [row 259](259-popup-root-typography-reset.md) (the `Popup root` theme rule + `scripts/check-popup-root-reset.sh`); open: the variant-slot gate (variant rules target the component's own root through slot variables, never descendants of an ancestor variant) and the same-component-nesting generalisation**

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

## Fix 2026-10-23 (item 1: the variant-slot gate)

**Gate** `scripts/check-variant-scope.sh` (in `run-gates.sh` after `check-popup-boundary`): in every component
`style.css`, a variant attribute (`data-variant`, `data-size`, `data-orientation`, `data-spacing`, `data-align`,
`:not()` included) on a compound followed by a DESCENDANT combinator, a `:has()` that reaches one through a
descendant, or a bare `[data-variant=...]` compound. Red on main with 68 sites in 16 components. Fixed by the Tabs
construction (the root declares a slot variable for every variant, the default included; the variant rule
re-declares it on the root; the part reads it) or, where the component itself renders the part as a direct
child, a `>`; the 2 remaining sites (`.dx-item-media[data-variant=icon|image] svg|img`, a user's leaf) are
allowlisted with the reason.

**Spec** `playwright/variant-nesting.spec.ts` (the browser half, and what the gate cannot see: slot variables
INHERITED from an outer variant root): on the page of every component whose sheet styles by a variant attribute
(derived from the CSS, 31 pages), each other-signature instance is cloned into a neutral `all: initial` host inside
and beside each variant instance and every computed property must agree. Red on main on 14 pages; it also found
leaks with no descendant selector at all, fixed here: Button's `--dx-button-radius`, Toggle's and ToggleGroup's
`--dx-toggle-*` slots (not re-declared by a default root), ButtonGroup's seam corners (a nested group's own end
controls inherited the outer seam's flat corners through the inner root; the `nested` demo's inner groups now
round their outer corners), and Item's `:has(description) .dx-item-media`.

**Verified** on a debug SSG build: the spec 32/32; a full before/after computed-style snapshot of every page in
both schemes (then the differing pages again, quiet, against a main-vs-main control that came out identical)
differs in: the ButtonGroup `nested` demo (intended, above); the email client's row avatars, 2px up (`transform`
`translateY(0.125rem)` -> `none`), which is what `email_client.css` asks for (`.ec-row:has(description) > media
{ transform: none }`) and lost on main only by sheet order against the old (0,3,0) Item rule; and the `/charts/`
open-state pass reaching a trigger in one run and not the other (coverage, "only in A/B", seen in both directions);
fast gates 70/70; `test:changed` (94 specs) had no red the branch caused (11 reds, each red on main too or green
alone).
