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

**Found by** the computed-style snapshot review: the natural-language date picker's calendar cells were 24 px instead of 28 px and the
chevrons 16 px, because `input_group`'s addon `button` and `svg` descendant rules reached into the calendar popup. **Class:** a popover
or dialog renders in the top layer but remains a DOM descendant of its trigger, so an ancestor component's descendant selector on a bare
element restyles it.

**Landed in four pieces.**

1. *2026-10-15 (lane `ancestorleak` `900f394`, main loop `526b51d`, `beaf3b0`).* The `svg` rule bounded with
   `:not(:where([popover], dialog) *)`, the `button` rule deleted, 7 more free-content rules hardened, new gate
   `scripts/check-popup-boundary.sh` and `popup-style-isolation.spec.ts`; the gate needed integration fixes after the p9parts merge
   (`alert/style.css` bounded, avatar and toggle leaf `svg` rules allowlisted with reasons).
2. *Same-component nesting, Tabs (`lane/p5tabs`).* The docs page's DEMO/CODE line Tabs' `[data-variant="line"] .dx-tabs-*` descendant
   selectors restyled every Tabs demo inside it; fixed by every Tabs root re-declaring all variant slots, with a spec.
3. *Typography half, 2026-10-21: row 259.* One zero-specificity theme rule resets every inherited property at the popup root, a gate
   keeps it complete (`check-popup-root-reset.sh`), and the spec dropped its `ALLOWED` font-size entries (this also closed the
   2026-10-20 "site-home" recurrence, where the docs home cards became 14 px `.dx-card` and the popups inside them inherited it).
4. *The variant-slot gate, 2026-10-23 (branch `claude/variant-scope`).* `scripts/check-variant-scope.sh` (in `run-gates.sh` after
   `check-popup-boundary`) flags, in every component `style.css`, a variant attribute (`data-variant`, `data-size`,
   `data-orientation`, `data-spacing`, `data-align`, `:not()` included) on a compound followed by a DESCENDANT combinator, a `:has()`
   reaching one through a descendant, or a bare `[data-variant=...]` compound. Red on main with 68 sites in 16 components; fixed by the
   Tabs construction (the root declares a slot variable for every variant, the default included; the variant rule re-declares it on
   the root; the part reads it) or, where the component renders the part as a direct child, a `>`; the 2 remaining sites
   (`.dx-item-media[data-variant=icon|image] svg|img`, a user's leaf) are allowlisted with the reason. The browser half is
   `playwright/variant-nesting.spec.ts`: on the page of every component whose sheet styles by a variant attribute (derived from the
   CSS, 31 pages), each other-signature instance is cloned into a neutral `all: initial` host inside and beside each variant instance
   and every computed property must agree. It was red on main on 14 pages and also found leaks with no descendant selector at all,
   fixed here: Button's `--dx-button-radius`, Toggle's and ToggleGroup's `--dx-toggle-*` slots (not re-declared by a default root),
   ButtonGroup's seam corners (a nested group's end controls inherited the outer seam's flat corners; the `nested` demo's inner groups
   now round their outer corners), and Item's `:has(description) .dx-item-media`.

**Verified** on a debug SSG build: the spec 32/32; a before/after computed-style snapshot of every page in both schemes (checked
against a main-versus-main control that came out identical) differs only in the ButtonGroup `nested` demo (intended), the email
client's row avatars 2 px up (which `email_client.css` asks for and lost on main only by sheet order against the old (0,3,0) Item
rule), and one `/charts/` open-state coverage difference seen in both directions; fast gates 70/70; `test:changed` (94 specs) had no
red the branch caused.

*Trimmed in row 266 (repo cleanup). The full dated narrative of this row is in git: `git show 3c584cc2:dev-docs/backlog/done/218-popup-style-isolation-top-layer-popups-stay-dom.md`.*
