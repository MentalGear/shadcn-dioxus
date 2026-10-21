---
id: 328
title: "Audit 2: declared typography -- every text- or icon-bearing root declares its size, only font-family, colour and direction inherit"
status: "LANDED 2026-10-21 (lane root-reset): the per-root gate of the first landing is replaced by a…"
size: "M"
area: theme
updated: 2026-10-21
---

# Row 328: Audit 2: declared typography -- every text- or icon-bearing root declares its size, only font-family, colour and direction inherit

## Item

Audit 2: declared typography -- every text- or icon-bearing root declares its size, only font-family, colour and direction inherit.

## Status / trigger

**LANDED 2026-10-21 (lane root-reset)**: the per-root gate of the first landing is replaced by a construction, see "Decision and measurement (root-reset)" below.

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

### Decision and measurement (root-reset, 2026-10-21, debug SSG; base ddd9d057 against the lane)

**Verdict: the construction is better than the per-root gate, and landed.** Every namesake root carries `"data-dx-root": ""` (57
roots; 4 wrapper roots opt out; the 4 `<dialog>`-panel roots stay with the popup-root rule; 18 components have no namesake
root) and ONE theme rule, "Component root" in `dx-components-theme.css`,
`:where([data-dx-root]:not([data-dx-root] *, button, input, select, textarea))`, resets 51 inherited text properties to the
theme base. `scripts/check-component-root-reset.sh` replaces `check-declared-typography.sh`; the 83-row baseline TSV is deleted.

Measured with the hostile-host probe (host 23px / line-height 3 / weight 800, 83 component pages; leaf = element that owns text
or an svg; "box moves" = width or height changes >= 1 px):

| property | leaf elements whose box moved, before -> after | part classes | components |
|---|---|---|---|
| line-height | 382 -> 54 | 38 -> 13 | 27 -> 10 |
| font-weight | 395 -> 53 | 69 -> 13 | 38 -> 9 |
| font-size | 26 -> 26 (not reset, see below) | 6 -> 6 | 6 -> 6 |

Same at a realistic host (18px / 1.8 / 600): lh 380 -> 54, fw 349 -> 52. (The row's first numbers, 63/48/15, counted classes
on dd14365e before the seven-root burn-down.) What is left is not the library's: the four opt-out wrappers (tabs content, hover-card and
tooltip trigger: caller content), demo scaffolding classes (`dx-avatar-label`, `dx-tasks-*`, `dx-virtual-list-*`,
`dx-form-hint`, `dx-top-layer-hint`, `dx-data-table-pagination-*`) and one probe artifact (textarea `dx-label`: computed weight
500 in both runs, the box moves because a flex item's width follows its container).

Screenshots, light theme, every prerendered route (100) at 1280 and 390 px, before against after, exact pixel compare: 190/200
identical; the 10 that differ are the capture's own noise (the same routes differ between two captures of the BASE build: charts,
radar, carousel, message-scroller, progress, a live render counter in virtual-list; `/charts/` at 390 px re-captured with a 1.5 s settle
is byte-identical, base against lane). The first lane build (no form-control exclusion) differed on the 7 `/charts/*` pages: an
icon-only `<button class="dx-button">` lost the UA `text-align: center` to the reset's `text-align: start` and its glyph moved 5 px.
That is the one real regression class the measurement found, fixed by construction (a UA-styled control is excluded from the rule;
the UA sheet already resets its `font`, `text-transform`, `letter-spacing`, and a `textarea`'s `pre-wrap` would have been lost too).

Variants measured on the same build (rule patched in the browser): (a) reset every root, nested ones included: 82/100 identical
desktop, regressions on bubble, message, chart-tooltip, field (a nested root loses its host component's declared typography) ->
rejected, outermost only; (b) `font-size: var(--dx-text-base)` on the root: 90/100 identical (noise only), leaf boxes moved 26 -> 26
(identical), `font-size` "follows host" classes 232 -> 83 but those are icons and parts with their own size -> not reset: it would
resize every caller label and paragraph inside a component in an app whose body is 14px, for no measured gain.

A browser scan of the 56 component pages that render a marked root found every `dx-<dir>` element marked except two hand-written
reuses of the class outside the component (the docs site's `CodeBlock` in `main.rs` and the generated props table in `build.rs`);
neither moved a pixel, and no browser spec was added (the static gate places the marker within 15 lines of the class literal in the
same function; the probe stays a `.txt`).

Secondary roots found by the probe (a part a caller can place without the namesake root): Field's `FieldSet`/`FieldGroup`, Trace's
collapsible, Input OTP's group and separator carry the marker too.

### Still open (follow-ups, not this row)

- `line-height` resets to `normal` (the site's body base, same as the popup rule). An app whose body sets another leading (Tailwind
  preflight: 1.5) sees caller text inside a component at `normal` unless its sheet declares one. A `--dx-body-line-height` slot
  read by both this rule and the popup rule would let an app say so; not done (the popup rule is another section).
- The 18 components with no namesake root (input_otp, context_menu, collapsible, the charts, ...) are not covered by the marker
  gate; the probe finds nothing broken in them beyond the secondary roots above.
