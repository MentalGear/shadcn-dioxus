---
id: 323
title: "Finer spec selection: map changed CSS rules to the classes and routes that use them"
status: "LANDED 2026-10-21 (lane selector): rule-level CSS selection in playwright/tools/select.mjs…"
size: "M"
area: tests
updated: 2026-10-21
---

# Row 323: Finer spec selection: map changed CSS rules to the classes and routes that use them

## Item

Finer spec selection: map changed CSS rules to the classes and routes that use them.

## Status / trigger

**LANDED 2026-10-21 (lane selector): rule-level CSS selection in `playwright/tools/select.mjs`; measured gain is real only for page-local CSS, see Update**

## Size

M

## Detail

Before this row any change to `main.css`, `main.rs` or a theme sheet made `playwright/tools/select.mjs` select ~1,800 tests.
Splitting `main.css` per page was rejected (it breaks the one render-blocking bundle of `check-css-delivery.sh` and changes
cascade order for equal-specificity rules). Instead `select.mjs` maps changed CSS rules to the components and routes that render
them. The owner's condition was to build it only if a measurement showed it held up; it was measured first (lane selector,
2026-10-21).

**Design (no browser, no build; the soundness argument is the header of `playwright/tools/css-impact.mjs`).**

1. The diff is taken rule by rule against the range base (an edited rule is one change, a `:root` edit traces only the declaration
   that changed, a moved rule is a change, a comment is none).
2. The classes and ids a changed selector REQUIRES (the constraining compound; `:not()`, `:has()`, attribute-only and element
   selectors require nothing) are placed on the pages of the SSG build under test. The class index is read from the SSG HTML on
   every run (`--ssg-dir` / `SELECT_SSG_DIR`, or a crawl of `PLAYWRIGHT_BASE_URL`), never derived from rsx, so it cannot be stale
   against the build being tested; an empty-shell build (a `dx serve` server) is refused.
3. Custom properties and keyframes are traced to the rules that read them, transitively.
4. The 18 matrix specs whose test titles name the page they visit drop the tests of unreached pages with `--grep-invert`; every
   test whose title names no page is kept.
5. Added on top: the specs about the component that owns a changed class, and every spec that spells a changed class or a rarely
   spelled property.

Anything outside that argument (a selector with no class or id, `@font-face`, `@import`, a nested rule, a token spelled in script
or inline style, a class that cannot be placed, no prerendered build) falls back to the whole BROAD set.

**Measurement (the last 30 commits touching `preview/assets`, `main.rs` or `theme.rs`; 22 touch a stylesheet).** The old mapping
selected the BROAD set for any stylesheet edit: 32 specs, 1,442 tests, 6,570 s of worker time (4,659 s of it four route sweeps:
`all-routes`, `first-paint-styles`, `layout-stability`, `theme-picker-phone`).

| class of commit | commits | old | new | new, core only |
|---|---|---|---|---|
| broad: no class to narrow by | 7 | 45,990 s | 46,308 s (+1 %) | +0 % |
| shell-wide: a docs-shell class on 80+ pages | 11 | 72,270 s | 75,616 s (+5 %) | -5 % |
| page-local: one to seven pages | 4 | 26,280 s | 14,542 s (-45 %) | -57 % |
| all 22 | 22 | 144,540 s | 136,466 s (-5.6 %) | 126,386 s (-12.6 %) |

The honest reading: the finer mapping is sound and pays only for page-local CSS; it does not touch the cost the owner saw,
because seven of 22 commits are broad by construction (`:where([popover][class^="dx-"], ...)`, the press wash on
`button[class^="dx-"]:active`, `--dx-foreground` read by inline style) and eleven change a docs-shell class that exists on every
page, which a sweep must visit on every page. The shell-wide row grows because the new selection adds coverage.

**Soundness.** Across the 30 commits the new selection contains every spec the old one did except `focus-ring.spec` and
`keyboard-matrix.spec` for one hover-mix commit (neither has a test on the affected pages). Run on the trees of eight culprit
commits whose regressions later reddened a spec, the old mapping caught 3 of 11 specs and the new one 8 of 11; the three misses
(a focus-ring token zeroed outside keyboard modality, the row-295 press wash) are the broad fallback, which only the merge-batch
full run can cover.

**Follow-up put to the owner.** The remaining cost is the four sweeps on shell-wide CSS: (a) leave it, the full suite per merge
batch covers it; (b) a lane-only page sample (4,246 s instead of 6,570), not sound in principle and not built; (c) make the sweeps
cheaper (`all-routes` axe at 4.3 s a test is the biggest line). Recommendation: (c) as its own row (it became row 334), then (b)
only if the lock queue still hurts.

Files: `playwright/tools/{css-rules,css-impact,site-index,select-titles,select-selftest}.mjs`, `select.mjs`, `test-changed.sh`,
`scripts/check-select-css.sh` (gate, in `run-gates.sh`). Escape hatches: `--no-css-narrow` / `CHANGED_FULL_CSS=1`, `--explain`,
`--pages c:button,site:home`.

*Trimmed in row 266 (repo cleanup). The full dated narrative of this row is in git: `git show 3c584cc2:dev-docs/backlog/done/323-finer-spec-selection-map-changed-css-rules-to.md`.*
