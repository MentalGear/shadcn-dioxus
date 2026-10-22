---
id: 334
title: "Make the four whole-site sweeps cheaper (all-routes, first-paint-styles, layout-stability, theme-picker-phone)"
status: "LANDED 2026-10-22 (lane sweeps): four sweeps 2,711 -> 2,054 worker-seconds on one build (-24 %)…"
size: "M"
area: tests
updated: 2026-10-22
---

# Row 334: Make the four whole-site sweeps cheaper (all-routes, first-paint-styles, layout-stability, theme-picker-phone)

## Item

Make the four whole-site sweeps cheaper (all-routes, first-paint-styles, layout-stability, theme-picker-phone).

## Status / trigger

**LANDED 2026-10-22 (lane sweeps): four sweeps 2,711 -> 2,054 worker-seconds on one build (-24 %); shared-load and dark-axe-subset ideas measured and NOT built (row 358)**

## Size

M

## Detail

Row 323's measurement (lane selector, 2026-10-21): any docs-shell-wide or token CSS change must run four sweeps on every page, and they are 4,659 of the 6,570 worker-seconds a stylesheet change selects -- `all-routes` 2,308 s (axe at ~4.3 s per test is the biggest line), `first-paint-styles` 1,086 s, `layout-stability` 819 s, `theme-picker-phone` 446 s. Row 327 (Round 4) already attacks `all-routes`' six loads per route; this row covers the rest: profile each sweep's per-test cost (navigation vs probe vs axe), share one navigation across probe families, run axe once per (route, scheme) with only the rules that can change, and check whether `layout-stability` and `theme-picker-phone` can run on a page sample when no shell CSS changed. Build only what a measurement shows holds up (owner condition, rows 320-331). Option (b) of row 323 -- a 10-page sample for shell-wide CSS in lanes (-35%) -- stays unbuilt: unsound in principle, revisit only if the lock queue still hurts after this row.

## Update 2026-10-22 (lane sweeps): profile first, then only what held up

One debug SSG build of `ac99f4e8` (origin/main), `ssg.local.config.ts`, 2 workers, the box shared with other lanes; same build
for every number. Per-sweep cost split taken with temporary `test.step`s and the JSON reporter (steps are flattened to
exclusive time; the analysis script is not committed). Row 323's table is stale: after row 327 `all-routes` is one load per
(route, scheme), and `theme-picker-phone` is 94 s, not 446 s.

### Profile (before, worker-seconds)

| Sweep | Total | Where it goes |
| --- | --- | --- |
| `all-routes` (183 tests) | 1,003 | axe analyze 379 (2.1 s/test), navigation (goto+hydrated) 321 (1.7 s), overflow probe 133 (two settles, a resize, three fresh phone loads), settle 86, forced colors 41 |
| `first-paint-styles` (174) | 827 | snapshots 219 (0.65 s each, 332 of them), hydration 215 (2 x 83 x 1.3 s), fixed observation windows 117 (500/500/400 ms), click + late-sheet poll 65, JS-off + JS-on page loads 37 |
| `layout-stability` (19) | 786 | the 1.5 s no-input window 359 (239 widths), wasm held until first paint + hydration 354, the rest 73 |
| `theme-picker-phone` (41) | 95 | 2-4 s a test, all navigation; nothing to remove |

### Built (each measured on the same build; none changes what a sweep asserts)

1. **`first-paint-styles`: one string per snapshot, and the two hard-load reads side by side.** `__dxSnap` returns
   `JSON.stringify(out)` and `snapOf` parses it: Playwright serialises an evaluate result value by value, and for a
   1,500-element x 38-property snapshot that cost 0.4 s of the 0.65 s. The same snapshots are compared, with the same
   properties and tolerance. The JS-off and JS-on contexts of test 2 now run through `Promise.all` (independent contexts).
   827 -> 512 s (-38 %); two full runs on the new code 552 and 527 s, 174/174 green both, JS-on layout shift still < 0.01.
2. **`layout-stability`: two pages per group on every page except the home page.** Each width is still a fresh navigation
   on its own page in its own context (same recorder, same wasm held until first paint + 200 ms, same 1.5 s window, same rule);
   `LS_POOL=1` restores the old order. Two pages in ONE context starved one page of animation frames (7-10 frames in 1.5 s
   against ~100), so each page has its own context. Sensitivity checked with an injected 7 px shift of `#main-content` 1.2 s
   after hydration (temporary hook, not committed): detected at every width, 1 page and 2 pages alike, ~100 frames each;
   injected at 1.7 s (outside the window) it is not seen, as before. The home groups stay at one page: with two pooled pages
   on the home page the first paint landed earlier in the progressive parse and a card below the fold counted as moved
   (833 px, 1 run in 3), so the pool is for the five other groups only: 3 of 3 runs green, 387 -> 243 s (-37 %). 786 -> 611 s (-22 %).
3. **`all-routes` and `expectNoAxeViolations`: `resultTypes: ["violations"]`.** Every rule still runs on every node and
   `violations` is complete; axe just stops building selectors and sources for the passing and incomplete nodes that the
   helper never reads (the home page has 14,700). Measured on 8 routes, min of 2 runs: scan 34.0 -> 27.1 s (-20 %), the
   violations identical on the real pages (none) and on a page with five injected defects (aria-roles, button-name,
   image-alt, label, link-name; plus color-contrast on the home page). Shared helper, so every spec's scan gets cheaper;
   no assertion changed. `all-routes`: 1,003 and 873 s in two before-runs (the box varies by 13 %), 840 s after: between -4 % and -16 %.

After, all four on the same build: `all-routes` 840, `first-paint-styles` 512, `layout-stability` 611, `theme-picker-phone` 91 (unchanged code)
= 2,054 s against 2,711 s (-24 %; -20 % if `all-routes`' second before-run is the reference). `durations.json` carries the after numbers.

### Measured and NOT built

- **One navigation for both schemes in `all-routes`** (saves ~160 s of the 321 navigation seconds): axe, the 390 px reading and the
  light forced-colors sweep come out identical to fresh loads on 20 routes when the page is switched with `__dxApplyMode`, but the
  dark forced-colors sweep cannot run on the same page without a reset of the focus start point; the best reset
  (blur, hash, scroll, a throw-away button prepended and removed) restores it on 3 routes and loses 2 of 83 controls on the
  home page. Needs an exception list of fresh-load routes: not clean enough; row 358.
- **Dark scheme runs only `color-contrast`/`link-in-text-block`** (would save ~130 s): non-contrast rules gave the same node counts in both
  schemes on 20 routes except `region` (8 fewer passing nodes in dark, from the two theme toggles), so the DOM is not provably scheme-independent
  to axe. It would stop proving that a dark-only structural regression is caught. Not built.
- **Settle without the 400 ms quiet window** (86 s): the window is the observation of late stylesheet insertion; measuring it from the last
  insertion time instead of from hydration weakens it. Not built.
- **A warm browser cache for navigations**: five loads of one route in one context took 1.09-1.24 s each, with or without the origin filter route; nothing is cached.
- **`theme-picker-phone`**: 91 s, all navigation, header-fit tests load at their width on purpose. Nothing taken.
