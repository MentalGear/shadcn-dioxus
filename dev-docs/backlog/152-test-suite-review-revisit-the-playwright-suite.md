---
id: 152
title: "Test-suite review: revisit the Playwright suite (~1,786 tests, ~21 min on a release build with 4 workers) for…"
status: "IN PROGRESS 2026-10-24. Done: 152a (below); the shared-page fixture, through rows 327, 358 and 334…"
size: "M"
area: tests
updated: 2026-10-24
---

# Row 152: Test-suite review: revisit the Playwright suite (~1,786 tests, ~21 min on a release build with 4 workers) for…

## Item

Test-suite review: revisit the Playwright suite (~1,786 tests, ~21 min on a release build with 4 workers) for slimming and optimisation without losing still-relevant coverage

## Status / trigger

**IN PROGRESS 2026-10-24. Done: 152a (below); the shared-page fixture, through rows 327, 358 and 334 (`all-routes` loads each route once and runs every probe in both schemes on that load). 152b slice 1, branch `claude/axe-at-rest-fold`: the per-component at-rest axe scans are folded into `all-routes` (Detail (a); see "152b slice 1" below). 152b slice 2 (2026-11-06, lane prune-152): the same fold in `button_group`, `calendar`, `carousel*`, `date-picker`, `popover`, `spinner`, `toast`, and the `all-routes` scheme-switch wait (see "152b slice 2" below). Left for 152b: Detail (b)-(d): the route matrices that rows 331 and 334 did not already shrink, the light/dark and preset matrices, and the oracle-vs-component overlap.**

**NARROWED 2026-10-19: speed half 152a DONE (lane suite-speed badc56d7): measured (3,752 tests, 183 files, 14,168 s test time on debug; all-routes 16%, first-paint 7.7%; fixed sleeps < 2%, no external waits), duration-balanced shards + 3 workers about 1.4x faster on the same build, release about 1.47x faster than debug and runs the timing specs, changed-files spec selector playwright/tools/select.mjs, slow-spec budget gate check-slow-spec-budget on playwright/durations.json, serve-spa.py serves 404.html like Pages (four not-found specs no longer hang). Remaining: per-file page reuse via a shared-page fixture (all-routes loads each route three times), then 152b (prune, merge, re-tier specs)** -- owner note 2026-10-05; nothing was run for this row, the evidence below is read from the round-2 full-suite report

**Next target (owner 2026-11-05: item 4 of the build and test-suite review).** `all-routes.spec.ts` is the longest file, at 859 s of 13,255 s (`playwright/durations.json`, debug build, 2 workers); `first-paint-styles` (512 s) follows. Start the remaining 152b work there, after the 2026-11-05 lane batch has merged, since that batch edits `dismiss-parity` and `menu-family-parity`. `all-routes` is now profiled ("152b slice 2"): axe is 51 % of it and has no dominant rule to trim, so what is left of it is the two axe scans per route, the navigation and the 390px reading; `first-paint-styles` was not touched (its per-route tests already run on `SUBJECTS` only in a lane, row 325; the 512 s is the `FIRST_PAINT_ROUTES=all` deploy sweep).

## Size

M

## Detail

**Starting evidence (all from `/home/user/tgt/r2/pw-full2.json` and `.log`, the 2026-10-04 round-2 full run: release no-base-path SSG build, Chromium only, 4 workers, `fullyParallel`; wall 1,255 s = 20.9 min, 1,761 passed / 12 failed / 13 skipped).** **Size:** 103 spec files; `grep -c "test("` finds 1,213 static `test(` calls but 1,786 tests run, because 54 spec files generate tests in loops (`main-thread` 3 -> 58, `dx-class-migration` 1 -> 52, `touch-double-tap` 12 -> 57, `touch-focus-zoom` 13 -> 58, `top-layer` 52 -> 99, `hydration-parity` 13 -> 42, `top-layer-ink` 1 -> 30, `carousel` 110 -> 144), so a slimming pass must count run tests, not source lines. **It is total work, not scheduling:** summed test time is 4,916 s; 4,916 / 4 workers = 1,229 s against a 1,255 s wall, so the 4 workers are ~98 % packed and only less work or more shards shortens the run. **Where the time is** (summed seconds, share of 4,916 s): carousel family 255 tests / 836 s (17 %; `carousel.spec.ts` 144 / 414 s, `carousel-virtual-wheel` 38 / 247 s, `carousel-wheel-replay` 22 / 80 s); chart family 289 tests / 938 s (19 %; radar 44 / 175 s, pie 36 / 151 s, `charts_gallery` 29 / 122 s, `chart_tooltip` 35 / 112 s, `chart_tooltip_follow` 33 / 83 s); `oracle/` 663 tests / 1,445 s (29 %; tier2-html 402 / 959 s, tier1-apg 173 / 309 s, tier3-radix 46 / 96 s, hydration-parity 42 / 81 s); `message_scroller.spec.ts` 25 / 171 s; theme-preset specs 51 / 257 s. **The cost is a per-test floor, not a few giants:** median test 1.9 s, 77 % of tests (1,366) take <= 3 s and make up 50 % of the time (2,469 s), while the 28 tests over 10 s are 525 s (11 %) and the slowest 50 are 722 s (15 %), so the lever is the ~1.5-2 s navigate-and-hydrate each test pays, multiplied by the number of tests, more than any single slow test. **Duplicated-coverage candidates (each needs a per-test check that the guard is really redundant before anything is cut):** (a) *per-component axe*: 181 tests titled "has no automatically detectable a11y issues" in 70 files = 956 s (19 % of all test time), 141 `expectNoAxeViolations`/`AxeBuilder` call sites in 72 files, about 55 of them the at-rest "loaded" scan; `preview.spec.ts` scans only the home page and `component/calendar`, so there is no global sweep today (hydration-parity already enumerates every prerendered route through `SSG_SITE_DIR`, the same list a sweep would use). Open-state scans (menus, dialogs, calendar focus) cannot be folded into an at-rest sweep. The saving is unmeasured: the slowest scans (`message_scroller` 23.6 s and 23.3 s, `preview.spec.ts` 19.7 s and 13.5 s, `calendar` 11.6 s) are dominated by axe's analysis, not navigation. (b) *repeated route matrices*: seven specs each walk (nearly) every component route and run one cheap probe per route, 312 tests / 614 s (12 %): `main-thread` (58), `dx-class-migration` (52), `touch-focus-zoom` (58), `touch-double-tap` (57), `top-layer-ink` (30), `hydration-parity` (42), `stylesheets-present` (15), so each component page is navigated ~7 times; a visit-each-route-once fixture running all the probes would keep every assertion. (c) *light/dark and preset matrices*: ~95 tests whose titles say light, dark, preset or theme = ~348 s; `theme-preset-contrast.spec.ts` (24 tests, 158 s, base x accent x light/dark swept in one in-page pass per probe) is already the efficient shape, the ones to compare it against are `carousel.spec.ts`'s 6-viewport x light/dark frame-escape matrix (12 tests, each a fresh navigation) and the `toggle` and `toggle_group` slate/gray x light/dark background tests (6 each). (d) *oracle vs component overlap*: the overlay family is exercised by its own spec plus up to ~9 oracle files (`dropdown_menu`, `dialog` and `popover` are named in 16, 24 and 30 spec files respectively: `top-layer` 99 tests / 239 s, `native-dialog` 18, `main-thread`, `top-layer-ink`, `auto-popover-opening-gesture`, `touch-*`, and five separate "Scrim" tests built on `assert-backdrop-fade.ts` for alert-dialog, dialog, sheet, drawer and popover); which of those guard different rules (every oracle header cites its source rule and backlog row) and which re-prove the same one has not been audited. **Long-timeout specs** (`test.setTimeout`): `charts_gallery.spec.ts:151` 240 s; `carousel-virtual-wheel.spec.ts` 11 calls of 120-180 s (lines 527-1191); `anchor-landing.spec.ts` 3 x `SLOW_TEST_MS`; `hydration-parity.spec.ts:931` 300 s; `computed-style-snapshot.spec.ts:312` 20 min (a tool, skipped unless `DX_SNAPSHOT`); config defaults are 120 s (`playwright.config.ts`) and 90 s (local configs). They are headroom, not cost: the slowest carousel-virtual-wheel test in the last run took 11.7 s; the only long real runs are `hydration-parity` Rule 2 extended (67.8 s) and `charts_gallery` Copy (24.4 s). **Known flaky or red specs:** the last run's 12 reds were 6 known (`calendar.spec.ts:364`, row 113, a 90 s timeout on every run = 1.8 % of all test time; `date-picker.spec.ts` x2 and `top-layer` Rule 8, rows 21 and 106, CSS-anchor geometry this sandbox's Chromium cannot pass; `drag_and_drop_list.spec.ts:300`, row 114; the by-design Rule 4c selftest) and 6 load flakes that were green on a `--workers=1` rerun (`alert-dialog` and `dialog` Scrim, `avatar`, `carousel.spec.ts:3389`, `message_scroller.spec.ts:169`, `main-thread` drawer drag); rows 112 (carousel sampling across async gaps, both instances fixed), 115 (tooltip, hover-card, popover, combobox:134, menu-submenu:307, carousel timing, `sampleCloseFade`) and 121 (`navigation_menu.spec.ts:128`, still marked OPEN, not among the last run's reds) are the rest of the family. **Consequence for any sharding or extra-worker plan:** the flakes are load-sensitivity, so more parallelism will raise them unless rows 112-115 are cleared first. Also: `main-thread` (58 tests / 138 s) and `scroll-main-thread` (8 / 56 s) are release-only timing specs, a natural separate tag. **Approach options (not decided):** tag a smoke subset (one at-rest axe, one behaviour spec, one hydration pass per component) against a full run; `--shard=i/n` across machines once the load flakes are fixed (4 shards x 4 workers would be ~5 min of wall time at today's total); collapse the seven route matrices (b) into one visit-each-route-once fixture and fold at-rest axe (a) into one sweep while keeping open-state scans; per-PR targeted runs through a changed-files map (a component directory -> its own spec plus the oracle subjects that name it; the oracles already hold explicit subject lists such as `OVERLAY_SUBJECTS` and `COMPONENTS`) with the full suite on demand, nightly or before a merge to `main` (all workflows are `workflow_dispatch`-only under the CI freeze, row 33, so today the full suite is the only run there is); a cheap first pass on the 90 s calendar test and the flakes. **Guard against losing coverage:** for each removal name the rule or regression it guarded (spec headers cite them), and keep the cheapest instance that still goes red against the bug it was written for (the repo's red-first rule, `plan.md`).

## 152b slice 1: at-rest axe lives in all-routes (2026-10-24)

`all-routes.spec.ts` scans every CATALOG component page, the site pages and every block demo at load, in light and in
dark, with `expectNoAxeViolations` and no rule disabled. 103 component-spec tests (57 test blocks in 55 files; five of
them a loop that loaded the same page once per variant) ran that same scan on the same page in one scheme, and are
removed. Kept: every scan of a state reached by interaction, and the at-load scans the sweep cannot reproduce
(`drawer` at phone width, `/charts/<tab>/`, the dashboard, `avatar`'s pending image, the oracle files). The convention is
written in `playwright/axe.ts` and `dev-docs/conformance-harness.md`.

A lane keeps the scan: `playwright/tools/select.mjs` adds the changed component's `all-routes` test (its page plus the
three page-less tests, through the existing `--grep-invert` narrowing) to `test:changed` for a component change and for
a primitive used by at most 30 components; `all-routes` is no longer selected whole for the five components its own
route-list check spells. Self-test 133 checks (2 new), the four mutations of `scripts/check-select-css.sh` still red.

Measured on one debug SSG build of main e5782224, 2 workers: the 103 removed tests passed and took 414 worker-seconds
(about 3.7% of the 11,273 s in `playwright/durations.json`).

## 152b slice 2: at-rest axe fold in the rest of the specs, all-routes scheme-switch wait (2026-11-06)

**Profile of `all-routes.spec.ts`** (debug SSG build of main d040554a, 2 workers, JSON report step times, 93 tests, 1,138.7 s on a
loaded box): the `axe` steps are 582.9 s (51 %, two scans per route, about 3.2 s each), the 390px overflow step 174.1 s, the
forced-colors walk 64.7 s, the rest (navigate, hydrate, first settle) about 317 s. Per-rule axe timing on `/component/calendar/` and
`/component/button/`: no rule dominates (`color-contrast` about 10 %, then `hidden-content`, `scrollable-region-focusable`,
`region`, each 3-6 %), so a rule-subset second scan would save little and would stop the dark pass from proving the structure
rules; not done. The one measurable fixed cost was `ensureScheme`: every in-place light/dark switch ran `settle` with its
400 ms quiet window for late `document::Link` sheets (the overflow step's dark switch and the axe step's light switch, two per
route). A scheme switch only rewrites `data-theme` and CSS variables, so no component renders and no sheet can arrive; it now
settles without the window (`watchNewSheets: false`; the viewport changes keep it). Same spec, same 93 tests, same probes:
1,138.7 s -> 991.9 s summed test time (axe step 582.9 -> 501.7 s, overflow 174.1 -> 131.6 s), wall 9 m 38 s -> 8 m 22 s, 93/93
green both times, the second run on a quieter box (load average 1.9 against 4-6 earlier), so the true saving is the removed
400 ms x 2 x 91 routes (about 73 s) plus whatever the load difference adds.

**Fold.** Removed the at-rest axe scans that `all-routes` already runs on the same page, in the same two schemes, with the same
tag set (each was a scan right after load, nothing clicked): `button_group` "loaded", `spinner` "loaded" (both specs keep their
behaviour tests), `calendar` "loaded", `date-picker` "loaded (popover closed)", `popover` "loaded (popover closed)", `toast`
"loaded (no toast)", `carousel.spec.ts` "carousel component page (every variant mounted)", `carousel-paging-inert` "axe: clean with
non-visible slides inert (main variant)", `carousel-rewind-autoplay` rewind, autoplay and indicators variant scans,
`carousel-virtual` "virtual_loop and virtual_many" (its own comment said the all-variants scan already covered both; that scan now
lives in `all-routes` `carousel`), and the first-slide half of `carousel-nav-opacity`'s hidden-arrow scan (the last-slide half is
a state reached by clicking and stays). 12 tests removed (the 293 that remain in these 11 files pass), plus one scan. Kept: every
open/focused/clicked state (`calendar` day focused, `date-picker` popover open, `popover` open and modal open, `toast` shown,
`carousel-virtual` `virtual_loop_rtl` after key presses, `carousel-nav-opacity` last slide). The scans the fold removed were scoped
`include` scans of one variant's frame; `all-routes` scans the whole `/component/carousel/` page (a superset of those frames).

Before/after on the same build, 2 workers (test seconds, tests): button_group 14.8 s (5) -> 12.1 s (4); calendar 111.7 (23) ->
90.1 (22); carousel 268.7 (59) -> 196.3 (58); carousel-nav-opacity 105.0 (18) -> 83.9 (18); carousel-paging-inert 87.4 (17) ->
63.0 (16); carousel-rewind-autoplay 146.6 (27) -> 99.0 (24); carousel-virtual 113.8 (16) -> 88.5 (15); date-picker 343.2 (83) ->
293.9 (82); popover 80.0 (24) -> 63.6 (23); spinner 9.5 (3) -> 4.6 (2); toast 27.8 (7) -> 19.8 (6): 1,308.5 s -> 1,014.7 s over the 11
files (the old run took 11 m 30 s wall, the new 9 m 0 s; the machine was not equally loaded, `oracle/tier1-apg/carousel.spec.ts`,
untouched, went 54.2 s -> 49.3 s in the same pair, so about 9 % of the gain is load). `playwright/durations.json` carries the
changed files' entries scaled by their measured before/after ratio (its seconds come from another, differently loaded run).

## 152b slice 3: the second colour scheme runs only the colour rules (2026-11-06)

The structure rules of axe (landmarks, names, roles, ARIA, headings, lists) read the DOM, and the colour scheme changes only
`data-theme` and CSS variables, so a second full scan in dark re-proves nothing. `all-routes.spec.ts` now runs every rule in
light and, in dark, only the rules of axe's `cat.color` category that the repo's tag set selects (`colorRuleIds()` in
`playwright/axe.ts`, read from axe-core's own metadata: `color-contrast` and `link-in-text-block`; `color-contrast-enhanced` is
outside the tag set). The premise is checked per route instead of assumed: `structureSignature` hashes, for every element of
every frame, tag, role, `aria-*` and name-bearing attributes, `hidden`/`inert`/`tabindex`, own text and the computed `display` and
`visibility`, once before the dark scan and once before the light scan; when the two differ the route runs every rule in dark too
and the test records an annotation saying so. A failing dark colour scan is reported under the same step as before.

Not measured in this lane: the build slot was withdrawn (disk), so the spec is syntax-checked (`playwright test --list`, 93 tests)
and the batch verifier's before/after of `all-routes` gives the seconds and `durations.json` entry. Expected from the slice 2
profile: the dark scan is about half the axe step (about 250 of 500 s), and the colour rules are roughly 10-15 % of a scan.

