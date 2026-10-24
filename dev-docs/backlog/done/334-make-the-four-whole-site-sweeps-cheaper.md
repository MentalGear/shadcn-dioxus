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

Row 323's measurement (lane selector, 2026-10-21): a docs-shell-wide or token CSS change must run four sweeps on every page, and
they were 4,659 of the 6,570 worker-seconds a stylesheet change selects. This row (lane sweeps, 2026-10-22) profiled each sweep's
per-test cost on one debug SSG build of `ac99f4e8` (`ssg.local.config.ts`, 2 workers, one build for every number) and built only
what held up (owner condition, rows 320-331). Row 323's table was stale: after row 327 `all-routes` is one load per (route,
scheme), and `theme-picker-phone` is 94 s, not 446 s.

**Profile, before (worker-seconds).** `all-routes` 1,003 (axe analyze 379, navigation 321, overflow probe 133, settle 86, forced
colors 41); `first-paint-styles` 827 (snapshots 219, hydration 215, fixed observation windows 117, click and late-sheet poll 65,
JS-off and JS-on loads 37); `layout-stability` 786 (the 1.5 s no-input window 359, wasm held until first paint and hydration 354);
`theme-picker-phone` 95 (all navigation, nothing to remove).

**Built (none changes what a sweep asserts).**

1. *`first-paint-styles`: one string per snapshot, and the two hard-load reads side by side.* `__dxSnap` returns
   `JSON.stringify(out)` and `snapOf` parses it (Playwright serialises an evaluate result value by value; for a 1,500-element by
   38-property snapshot that was 0.4 s of 0.65 s); the JS-off and JS-on contexts of test 2 run through `Promise.all`.
   827 to 512 s (-38 %), 174/174 green in two full runs.
2. *`layout-stability`: two pages per group on every page except the home page.* Each width is still a fresh navigation on its
   own page in its own context (two pages in one context starved one of animation frames); `LS_POOL=1` restores the old order.
   Sensitivity checked with an injected 7 px shift 1.2 s after hydration (detected at every width) and 1.7 s (outside the window,
   not seen, as before). The home groups stay at one page: pooled, the first paint landed earlier in the progressive parse and a
   card below the fold counted as moved (833 px, 1 run in 3). 786 to 611 s (-22 %).
3. *`all-routes` and `expectNoAxeViolations`: `resultTypes: ["violations"]`.* Every rule still runs on every node; axe stops
   building selectors for the passing and incomplete nodes the helper never reads (14,700 on the home page). Violations identical
   on the real pages and on a page with five injected defects; scan 34.0 to 27.1 s on 8 routes. Shared helper, so every spec's
   scan gets cheaper. `all-routes` 1,003 and 873 s before (the box varies 13 %), 840 s after.

**After, same build:** `all-routes` 840, `first-paint-styles` 512, `layout-stability` 611, `theme-picker-phone` 91 = **2,054 s against
2,711 s (-24 %)**; `durations.json` carries the new numbers.

**Measured and NOT built** (row 358 holds the shared-load idea):

- One navigation for both schemes in `all-routes` (~160 s): the dark forced-colors sweep cannot run on a page switched with
  `__dxApplyMode` without a focus-start reset, and the best reset loses 2 of 83 controls on the home page.
- Dark scheme running only `color-contrast`/`link-in-text-block` (~130 s): axe's non-contrast results differed between schemes
  (`region`), so a dark-only structural regression would stop being caught.
- Settling without the 400 ms quiet window (86 s): it is the observation of late stylesheet insertion.
- A warm browser cache for navigations: five loads in one context took 1.09-1.24 s each; nothing is cached.
- `theme-picker-phone`: all navigation, header-fit tests load at their width on purpose.
Row 323's option (b), a 10-page sample for shell-wide CSS, stays unbuilt (unsound in principle).

*Trimmed in row 266 (repo cleanup). The full dated narrative of this row is in git: `git show 3c584cc2:dev-docs/backlog/done/334-make-the-four-whole-site-sweeps-cheaper.md`.*
