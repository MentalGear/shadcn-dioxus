# Row 327: `all-routes` loads each page once, and what `networkidle` in `gotoHydrated` costs

Measured 2026-10-21 on one debug SSG build of main `0602f7f4` (`scripts/lane-ssg.sh`, served by `scripts/serve-spa.py`),
`ssg.local.config.ts`, 2 workers, Chromium 1194, 4-core box with another lane's work around it. Owner condition for
both parts: "do it only if a measurement shows it holds up".

## Verdict

| Part | Result | Decision |
| --- | --- | --- |
| (a) `all-routes.spec.ts`: one navigation per (route, scheme) | wall 907 s -> 541 s (-40 %), worker time 1,800 s -> 1,073 s (-727 s); 541 -> 183 tests, all green both ways | done |
| (b) `gotoHydrated`: `networkidle` -> `load` + `data-hydrated` | no measurable difference (three runs each, two spec files) | `hydration.ts` left unchanged |

## (a) One load per (route, scheme)

### What it was

`all-routes.spec.ts` ran three probe families as three tests per route and scheme (axe, overflow at 390 px, forced-colors
keyboard sweep), plus a fourth for `combobox` and `command`, so every page was navigated, hydrated and settled up to
three times per scheme. A navigation plus `gotoHydrated` plus the settle (iframes hydrated, stylesheet set unchanged for
400 ms, two animation frames) costs about 2.4 s on this debug build.

### What it is now

One test per (route, scheme), titled `<route>: axe + no horizontal overflow at 390px + skip link and visible focus in
forced colors (<scheme>)` (the forced-colors part is absent on `block-*` routes, the `the active option shows an outline
in forced colors` part is added on `combobox` and `command`). It loads once and runs the probes through `runProbes`:

- every probe runs even when an earlier one failed, each is a `test.step` named after the old test, and a failure is
  rethrown as thrown (one failing probe) or joined under the probe's name (several). Checked with an injected failure in a
  scratch copy: with the forced-colors assertion and the overflow assertion both broken, the report showed both
  messages, axe still ran and passed, and the matcher output was Playwright's own.
- probe bodies, assertions and messages are the old ones, moved into `probeAxe`, `probeOverflow`, `probeForcedColors`
  and `probeActiveDescendant`. `openRoute` is split into the navigation and `settle`, so the same settle also runs after
  a viewport change. `the route list parses from CATALOG`, the language-switcher test and the calibration test are
  untouched (they are not probe families; the last two load `button` and `sidebar` once more).
- no global context option was added. `emulateMedia` and `setViewportSize` are per page, as before.

### Timings (same build, same config, 2 workers)

| | tests | wall | worker time | result |
| --- | --- | --- | --- | --- |
| old spec | 541 | 907.2 s (15.1 min) | 1,799.8 s | 541 passed |
| new spec | 183 | 540.6 s (9.0 min) | 1,073.1 s | 183 passed |

Pass/fail sets: both runs are fully green, so there is no pre-existing failure to carry over. (`durations.json` still
listed 2 failures for the old file from an earlier build; the refresh writes 0.)

Old worker time by family: axe 879 s, overflow 438 s, forced colors 465 s, combobox/command 11 s. New, by probe step:
axe 430 s (2.39 s each), overflow 157 s (0.87 s each, two settles and a resize), forced colors 42 s (0.24 s each), active
option 1 s, and 443 s for the navigations (2.4 s each, 183 of them). The row's estimate of about 1,000 s did not hold:
the saving is 727 s of worker time, 367 s of wall, because the probes themselves are not free (axe is 2.4 s a page, the
resize costs two settles) and three routes need a second load (below).

### Probe isolation: how the order was chosen and checked

Three probes change the page. Measured on a debug build, one by one:

- **forced colors** (emulation, Tab/Enter, focus and `scrollIntoView` on up to 120 controls, `#main-content` in the
  URL) changes what everything after it sees, so nothing that judges normal rendering may follow it.
- **overflow** resizes to 390 px. A viewport round trip is box-for-box invisible on 87 of 90 routes, but it leaves the
  docs props table's scroller focusable (`tabindex=0`, `role=region`).
- **axe** is not read-only either: afterwards closed accordions report their content as laid out (`+4` boxes on
  `button`), and `content-visibility: auto` sections render (`site-home` moves 255 box edges, `+36` boxes).

The check, run as scratch Playwright specs (deleted, not committed) over the real routes: for each route record the
old-style reference on FRESH loads (overflow read on a page loaded at 390 px: the full list of right edges of every box in
`main`, the offenders and the page scroll; forced-colors sweep on a page loaded under forced colors: the signatures it
judged, how many it checked and its failures; axe on a fresh desktop load: node counts per rule for violations, passes and
incomplete), then record the same on the shared-load sequence, and diff.

| Order tried | Overflow reading | axe | Forced-colors sweep |
| --- | --- | --- | --- |
| axe, overflow, forced (164 route x scheme) | offenders and page scroll equal everywhere, but all 164 differ in box list (axe's reveals) | not compared | 32 gain one control, none lost |
| axe, forced, overflow (180) | offenders and page scroll equal everywhere; 168 of 180 differ in box list, 10 of them with moved edges (`site-home`, `top_layer`, `chart_tooltip`, `message_scroller`, `progress`) | not compared | same 32 gains: the docs scroller becomes focusable after axe too |
| **overflow, axe, forced (90 routes, light)** | exact on 87; `virtual_grid`, `site-home`, `message_scroller` differ | identical on all 90 (every rule's pass, violation and incomplete node count) | 71 identical, 16 gain one control (the docs scroller), none lost or failing |

Two fresh phone loads of the three odd routes are identical to each other (0 differing boxes, three runs), so the
difference is history, not noise: `virtual_grid` virtualizes a different window after a round trip (2,955 boxes against
2,784), `site-home` renders its `content-visibility` sections (4,799 against 4,763), `message_scroller` moves 10 edges.
Those three therefore take their 390 px reading from a second, fresh page loaded at 390 px (`FRESH_PHONE_LOAD`, a
three-name set with the numbers in its comment), exactly like the old test. Everything else resizes in place.

The shipped order is **overflow, axe, forced colors, then the combobox/command probe**:

1. overflow reads the page nothing has touched, then restores the viewport (with a lighter settle: stylesheets loaded,
   no 400 ms quiet window, because going back to the loaded width can only re-render components whose sheets are in);
2. axe reads a pristine desktop page and, measured, returns the same result as on a fresh load;
3. forced colors may only gain (the one extra control is the docs scroller, `DIV|region|dx-docs-table-scroll`, which
   gets a visible outline; no control lost, none failed);
4. the combobox/command probe focuses an input and opens the palette, so it is last and turns forced colors on itself.

What the check does not prove: the final file's `FRESH_PHONE_LOAD` branch was not re-diffed (it is the old reading by
construction: a fresh page at 390 px through the same `openRoute`), the dark scheme was diffed only in the first two
orders (identical to light there), and a future route whose DOM depends on viewport history would be read through a
round trip unless added to the set. Its offenders are still computed on real geometry, so the probe stays valid; only
"same boxes as a fresh load" is not guaranteed for it.

### Not folded in

`the language switcher sets <html lang>` (changes the language and navigates) and the calibration test (strips an
iframe title) keep their own loads: they are not probe families and mutate the page by design.

## (b) `gotoHydrated`: `networkidle` against `load` + `html[data-hydrated="true"]`

`hydration.ts` navigates with `waitUntil: "networkidle"` (a 500 ms window with no request) and then waits for
`html[data-hydrated="true"]`. 940 `gotoHydrated(` calls in the suite; 31 pass their own `waitUntil` (27 `networkidle`, 4
`load`), so the default applies to about 909.

### Spec files, three runs each, alternating modes

`popover.spec.ts` was chosen: 18 `gotoHydrated` calls, 22 tests, overlay positioning, no `waitUntil` override and no
`waitForLoadState('networkidle')` of its own (`combobox.spec.ts` was rejected: it calls `waitForLoadState('networkidle')`
after most navigations, which would mask the change). `select` (the pattern also matched `native_select` and the other
`*select.spec.ts`: 69 tests) was run as a second sample. `hydration.ts` was edited for the `load` runs and restored after
each; the committed file is unchanged.

| Spec | Mode | Run 1 wall | Run 2 wall | Run 3 wall | Mean wall | Passed / failed / flaky |
| --- | --- | --- | --- | --- | --- | --- |
| popover (22 tests) | networkidle | 29.2 s | 29.7 s | 30.9 s | 29.9 s | 22 / 0 / 0 in all three |
| popover (22 tests) | load + hydrated | 28.9 s | 30.5 s | 29.2 s | 29.5 s | 22 / 0 / 0 in all three |
| select-pattern (69 tests) | networkidle | 78.5 s | 75.0 s | 72.2 s | 75.2 s | 69 / 0 / 0 in all three |
| select-pattern (69 tests) | load + hydrated | 84.3 s | 73.9 s | 70.7 s | 76.3 s | 69 / 0 / 0 in all three |

Differences are inside the run-to-run spread (popover -0.4 s, select +1.1 s): no speedup, no new failure, no flake in
12 runs.

### Why there is no gain: what is still loading after hydration

Direct timing of `goto` + `data-hydrated` from a fresh context (8 loads per cell, run twice, interleaved), plus a log of
every request that FINISHES after the hydrated flag appears (1.5 s watch):

| Route | networkidle, median (range) | load, median (range) | requests finishing after `data-hydrated` |
| --- | --- | --- | --- |
| `/component/popover/` | 1,331-1,359 ms (1,232-1,433) | 1,329-1,338 ms (1,217-1,392) | none |
| `/component/button/` | 1,331-1,352 ms | 1,311-1,339 ms | none |
| `/` | 3,322-3,441 ms | 3,370-3,371 ms | none |
| `/component/data_table/` | 1,337-1,349 ms | 1,357-1,414 ms | none |

On a debug build the wasm bundle's fetch, compile and hydrate take much longer than the 500 ms idle window, so the
window has already elapsed when the flag appears and `networkidle` adds nothing. No stylesheet, font or script request
arrives after hydration (the SSG delivers its stylesheet bundle and the preloaded self-hosted fonts in the head, gated by
`check-css-delivery.sh`), so no spec that measures after `gotoHydrated` is waiting on a late resource.

### Recommendation

Leave `hydration.ts` as it is. The change buys nothing measurable on a debug build and removes a real guarantee: a page
that is still fetching when hydration finishes (a lazy image, the deliberately pending avatar demo requests, a future
lazily loaded chunk) would no longer be waited for. Not measured: a release build, where hydration may finish less than
500 ms after `load` and `networkidle` would then cost up to the window per call. If the main loop's release run shows
that, repeat this table on it (two spec files, three alternating runs each, `hydration.ts` swapped inside the Playwright lock) before changing the
default; the specs with their own `waitUntil` (31 sites) and `waitForLoadState('networkidle')` (10 files) would need
their own look.
