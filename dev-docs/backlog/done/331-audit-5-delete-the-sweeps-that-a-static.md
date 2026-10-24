---
id: 331
title: "Audit 5: delete the sweeps that a static gate or another spec already covers"
status: "DONE 2026-10-21 (lane b-sweeps: 5 sweeps shrunk by measurement, golden half kept; 435 s of 721 s…"
size: "S"
area: tests
updated: 2026-10-21
---

# Row 331: Audit 5: delete the sweeps that a static gate or another spec already covers

## Item

Audit 5: delete the sweeps that a static gate or another spec already covers.

## Status / trigger

**DONE 2026-10-21 (lane b-sweeps: 5 sweeps shrunk by measurement, golden half kept; 435 s of 721 s saved on the six files; branch claude/sweep-shrink)**

## Size

S

## Detail

Owner condition (2026-10-21): change only what a measurement shows holds up. Result of the audit and the lane (branch
`claude/sweep-shrink`): **no sweep was deletable by a gate** (each gate scans sources or one namespace, each spec reads a
browser-computed or served result), so nothing was deleted; five sweeps were shrunk to a sample that provably carries every case
the old sweep carried, and the golden half of `theme-all-reset` was kept.

**Stale claims in the original text, corrected.** `stylesheets-present` is already a sample, not a sweep, and stays;
`demo-forms-no-navigation` is not covered by `check-demo-forms.sh` (that gate scans `preview/src` only and cannot see the
rendered `method` attribute or a form from a dependency); the `rtl-toggle` overflow probe is not vacuous (it catches a positioned
element escaping `body`, which `overflow-x: clip` otherwise hides, in LTR and RTL alike); `user-select`, `touch-double-tap` and
`theme-all-reset` read browser-computed values no row 330 gate can see, so they shrink rather than go; `check-touch-action.sh`
never existed (`touch-action` is arm 1 of `check-theme-baseline.sh`); the audit's mutation probe 4 could never go red because
`main.css` carries an `!important` backstop, so the backstop list losing a family was probed instead.

**The shrinks, each with its sampling rule and a red-first probe** (debug SSG builds of `dd14365e`; CSS defects applied to a copy
of the served build, Rust defects to a second build):

| Spec | Tests before to after | Sample | Probe, and result |
|---|---|---|---|
| `demo-forms-no-navigation` hydrated test | all ~190 routes to the 12 whose served HTML has a `<form` (plus the dashboard compose modal) | measured lossless: the unshrunk walk found the same 12 routes, 30 forms, 18 submit controls; falls back to every route when nothing is served with a form; also fixed `beforeAll` appending to the shared list once per worker | `DemoForm` without `method=dialog` or `prevent_default()`: 3 of 3 forms tests red |
| `rtl-toggle` | 82 pages to 22 | one catalog page per primitive that reads the direction (the set `check-direction-keys.sh` polices), plus a guard that the sample names only catalog pages | positioned element escaping `body` only under `[dir=rtl]`: red on the sampled pages (the `pageerror` half was not probed) |
| `user-select` | 36 to 12 | greedy set cover on (CONTROLS clause, first `dx-` class) pairs: 12 pages carry all 38 | `.dx-button { user-select: text }`: all 12 pages red |
| `touch-double-tap` | 46 to 23 | greedy set cover on (tag, role, first `dx-` class or raw) families: 23 pages carry all 75 (the audit's 5 pages would have dropped 63) | `input, select, textarea, summary` removed from the backstop: top_layer, form, combobox, home and two overlay tests red |
| `dx-class-migration` | 52 to 31 | greedy cover of the 257 distinct non-chrome `dx-` classes; all three assertions still run on each page | avatar root rendering `dx-avatar-7ca1a710`, and every `.dx-slider` rule renamed in the bundle: both caught |

**`theme-all-reset` golden half kept, row 258 fixed.** The test was whether a re-baselined golden catches something
`check-theme-baseline.sh`, the per-control computed-style tests, `check-popup-root-reset.sh` and `popup-style-isolation` do not.
Appending `cursor`, `text-indent`, `appearance`, `transition-property`, `letter-spacing`, `border-width` and similar to the shipped
sheet went red in 4 of 4 golden tests and in none of the per-control specs (the gates read no computed value, and
`popup-style-isolation` compares a popup with its own clone). It uniquely catches a dropped reset line on the four controls,
which is what row 239's `all: unset` removal risked; cost 8.5 s of 721 s. Row 258 (the cp-area-thumb `20x19.99999` sub-pixel
flake) is fixed in the same file by rounding `rect` to 2 decimals.

**Measured saving** (six files, one debug SSG build, 2 workers, +/- 10 %): 260 to 133 tests, **721 s to 286 s of worker time
(-435 s, -60 %)**, wall 367 s to 146 s. `playwright/durations.json` merged (total 14,924 to 14,559 s, budget 17,000 s unchanged,
`check-slow-spec-budget.sh` OK). Not done on purpose: the audit's near-misses (`popup-style-isolation`, `all-routes` forced-colors,
`listener-inventory`, `hydration-parity`, `press-feedback`, `touch-focus-zoom`, `layout-stability`) stay; the listener-inventory
5 to 2 modal scenarios needs its own mutation probe first.

*Trimmed in row 266 (repo cleanup). The full dated narrative of this row is in git: `git show 3c584cc2:dev-docs/backlog/done/331-audit-5-delete-the-sweeps-that-a-static.md`.*
