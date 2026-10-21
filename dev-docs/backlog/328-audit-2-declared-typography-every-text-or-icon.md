---
id: 328
title: "Audit 2: declared typography -- every text- or icon-bearing root declares its size, only font-family, colour and direction inherit"
status: "PARTLY LANDED 2026-10-21 (lane typography: measured; static root gate check-declared-typography +…"
size: "M"
area: theme
updated: 2026-10-21
---

# Row 328: Audit 2: declared typography -- every text- or icon-bearing root declares its size, only font-family, colour and direction inherit

## Item

Audit 2: declared typography -- every text- or icon-bearing root declares its size, only font-family, colour and direction inherit.

## Status / trigger

**PARTLY LANDED 2026-10-21 (lane typography: measured; static root gate `check-declared-typography` + shrink-only baseline of 38 font-size / 45 line-height rows; burn-down and the line-height token open)**

## Size

M

## Detail

Audit section 4: of ~200 text/icon-bearing parts, 68 declare no `font-size` and 151 no `line-height`; of 64 component roots 20 declare a size and 5 a line-height; 12 of 17 popup roots declare no size. A part that inherits its size is right only by accident (Select's scroll buttons at 14px in a card footer, row 268). Construction: a gate reading roots from `component.rs` (outermost element per `#[component]`, plus `popover`/`dialog`/`role=menu|listbox|dialog|tooltip` elements) that requires each root's class to declare font-size and line-height through a token or slot, shrink-only baseline, `typography-inherit: <reason>` marker. Complements the popup-root reset (row 218 half). Shrinks popup-style-isolation further and touch-focus-zoom.

### Measurement (lane typography, 2026-10-21, debug SSG of dd14365e)

Probe: `dev-docs/research/typography-hostile-host-probe.spec.ts.txt` (copy to `playwright/`, run against an SSG). For all 83 component
pages it sets a hostile value on every demo frame (`#component-preview-frame*`, the host of the component) and re-reads
every `dx-` element's computed style and box: font-size 23px, line-height 3, weight 800 (second run, realistic host: 18px,
1.8, 600). 374 distinct `dx-` part classes rendered (329 in main demos). "Follows host" = the computed value changed;
"leaf" = an element that owns text (or is an `<svg>`); "visible" = its box changed by >= 1 px.

| Property | Part classes that follow the host | First undeclared link (frontier) | Text/icon leaf classes | Leaf visibly changes (23px / 3 / 800) | Same at 18px / 1.8 / 600 |
|---|---|---|---|---|---|
| font-size | 147 (main demo only: 124) | 74 | 32 | 15 (10 in a component sheet, 7 once demo wrappers are removed) | 14 |
| line-height | 260 (213) | 89 | 94 | 63 | 59 |
| font-weight | 264 (222) | 89 | 95 | 48 | 44 |

Against the audit's static estimate: roots whose class is `dx-<dir>` and which are rendered: 61; 40 follow the host's size,
52 its line height. Static: 19 declare a size and 5 a line height (audit: 20 of 64 and 5). The size exposures that visibly
matter are few: of the 7, six are slots for the caller's content (form field, hover-card and tooltip trigger, resizable
panel, tabs panel, radio group demo text) where shadcn also inherits, and one is a component part (the input-otp
separator's glyph). Icons follow the host's font-size but keep their box (the SVG has a pixel size), which is why Select's
chevrons only misbehaved through the popup's own inheritance, now cut by row 259. Line height and weight are the broad
exposure: 34 components have a leaf whose box moves with the host's line height, 29 with its weight.

How well a static root check predicts it (component level, ground truth = "some text/icon leaf of this component follows the
host"): font-size, roots lacking it 42 (after popup roots), of which 14 exposed and 28 inert (the root is a holder whose
children declare their own size: Select, DropdownMenu, Menubar, Switch, Slider ...): precision 33 %, recall 88 %.
line-height: 56 flagged, 36 exposed, 20 inert: precision 64 %, recall 100 %. Narrowing the check to roots that style text
(`color`, `font-weight`, ...) raises precision but recall for size drops to 1 of 14, because the exposed roots are wrappers
with no text rules of their own. So a static root gate can enforce "declared or deliberately inherited" but it cannot tell
the inert from the exposed: the baseline carries an `evidence` column from this probe for that reason.

### What landed

- `scripts/check-declared-typography.sh` (in `run-gates.sh`): namesake roots from `component.rs` (`dx-<dir>`, 61 rendered,
  18 components have no namesake root, the 4 `<dialog>`-panel roots are skipped because the popup-root rule resets them),
  font-size through a `var()` and line-height with any non-`inherit` value, `typography-inherit(<root>[, font-size|line-height]):
  <reason>` markers, shrink-only baseline `scripts/check-declared-typography.baseline.tsv` (`arm`, component, class, evidence),
  stale rows fail, a 7-mutation self-test. Not every `#[component]` is a root: that definition gave 405 classes, 258 failing.
- Burned down: line-height paired with the declared size (`calc(1.25 / 0.875)` for `--dx-text-sm`, `calc(1 / 0.75)` for
  `--dx-text-xs`, a ratio so a larger child stays proportional) on avatar, badge, card, item, marker, message and table; four
  markers (tabs, tooltip, hover_card, carousel: wrappers around the caller's content).

### Open

- Baseline: font-size 38 rows (10 exposed: chart, command, drag_and_drop_list, field, form, message_scroller, pagination,
  radio_group, toolbar, combobox), line-height 45 (25 exposed). `combobox` belongs to the select/combobox lane.
- The theme has no line-height token, so "through a token" is only enforceable for font-size; a `--dx-leading-*` pairing for
  the `--dx-text-*` scale (theme sheet, row 330's file) would let the line-height arm demand a token too.
- Weight is not an arm: 29 components follow the host's weight, but shadcn inherits it as well (only the parts that style it
  declare one), so a gate would demand more than the reference does.
- No browser half was added: the exposed size cases are caller-content slots, the popup half has `popup-style-isolation`, and
  one more spec over 83 pages costs ~100 s of suite time for no remaining defect class. Rerun the probe when a root changes.
