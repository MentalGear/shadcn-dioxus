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

Audit section 4: of ~200 text- and icon-bearing parts 68 declared no `font-size` and 151 no `line-height`; a part that inherits its
size is right only by accident (Select's scroll buttons at 14px in a card footer, row 268). Two constructions landed in turn; the
second replaced the first.

**1. Per-root gate (lane typography, 2026-10-21).** A hostile-host probe
(`dev-docs/archive/research/typography-hostile-host-probe.spec.ts.txt`: copy to `playwright/`, run against an SSG) sets font-size
23px, line-height 3, weight 800 on every demo frame of all 83 component pages and re-reads every `dx-` element's computed style and
box. Result: the visible size exposures are few (7 leaf classes, six of them slots for caller content where shadcn also inherits);
line height and weight are the broad exposure (34 components with a leaf whose box moves with the host's line height, 29 with its
weight). A static root check predicts size with precision 33 % and recall 88 %, line-height 64 % and 100 %, so it can enforce
"declared or deliberately inherited" but not tell the inert from the exposed. `scripts/check-declared-typography.sh` landed with a
shrink-only baseline carrying an `evidence` column from the probe, and seven roots were burned down.

**2. Root reset, the construction that replaced it (lane root-reset, 2026-10-21).** Every namesake root carries
`"data-dx-root": ""` (57 roots; 4 wrapper roots opt out; the 4 `<dialog>`-panel roots stay with the popup-root rule; 18
components have no namesake root) and ONE theme rule, "Component root" in `dx-components-theme.css`,
`:where([data-dx-root]:not([data-dx-root] *, button, input, select, textarea))`, resets 51 inherited text properties on the
outermost root to the theme base. `scripts/check-component-root-reset.sh` (16-mutation self-test) replaced
`check-declared-typography.sh`; the 83-row baseline TSV is deleted. Measured with the probe, leaf elements whose box moved:
line-height 382 to 54, font-weight 395 to 53, font-size 26 to 26 (deliberately not reset: it would resize every caller label and
paragraph inside a component for no measured gain). What is left is the four opt-out wrappers, demo scaffolding classes and one
probe artifact. Screenshots of all 100 prerendered routes at 1280 and 390 px, before against after: 190/200 identical, the 10
differing are capture noise (the same routes differ between two captures of the base build).

- **The one real regression class the measurement found,** fixed by construction: the first build had no form-control exclusion, and
  an icon-only `<button class="dx-button">` lost the UA `text-align: center` (glyph moved 5 px); a UA-styled control is now
  excluded because its UA sheet already resets `font`, `text-transform`, `letter-spacing` and a textarea's `pre-wrap`.
- **Variants rejected on the same build (rule patched in the browser):** resetting nested roots too (82/100 identical, regressions
  on bubble, message, chart tooltip, field: a nested root loses its host component's declared typography), and
  `font-size: var(--dx-text-base)` on the root (no measured gain).
- **Not added:** a browser spec (one more spec over 83 pages costs ~100 s of suite time for no remaining defect class; rerun the
  probe when a root changes); a weight arm (shadcn inherits weight too).

**Owner decision 2026-10-21 (line height).** The reset's `line-height: normal` would have changed caller text inside components in
an app whose body sets another leading (Tailwind's 1.5), so both resets (component root and popup root) read one slot,
`--dx-root-line-height` (default `normal`, so this site is unchanged); an app sets it once on `:root`, and the README says so.
Left open: a `--dx-leading-*` token pairing for the `--dx-text-*` scale.

*Trimmed in row 266 (repo cleanup). The full dated narrative of this row is in git: `git show 3c584cc2:dev-docs/backlog/done/328-audit-2-declared-typography-every-text-or-icon.md`.*
