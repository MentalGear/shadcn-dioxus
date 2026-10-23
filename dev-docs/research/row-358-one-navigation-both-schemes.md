# Row 358: `all-routes` loads each route once for both schemes

Measured 2026-10-23 on one debug SSG build of main `52f28786` (`scripts/build-ssg.sh debug`, served by
`scripts/serve-spa.py`), `ssg.local.config.ts`, 2 workers, Chromium 1243, 4-core box with nothing else running. Owner
condition carried from rows 327/334: build only what a measurement shows holds up, every route checked against a fresh
load.

## Verdict

| | tests | worker seconds (same run, interleaved) | result |
| --- | --- | --- | --- |
| old spec (one load per route x scheme) | 183 | 1,083 | 183 passed |
| new spec (one load per route, both schemes) | 93 | 859 | 93 passed |

-224 worker-seconds (-21 %) for the file, measured in ONE run that held both files (a copy of main's spec beside the
new one), so the box load is the same for both. The saving is smaller than the 160 s of navigation row 334 expected
to free plus the halved settles, because five routes keep a fresh phone-width page per scheme and the scheme switch
costs a settle (~0.5 s) of its own.

## What it is now

One test per route, titled `<route>: axe + no horizontal overflow at 390px + skip link and visible focus in forced
colors (light, dark)`. It loads the route once, in light, and runs:

1. **overflow** for both schemes: resize to 390 px once, settle, read light, switch to dark in place (`ensureScheme`:
   the `dx_theme` cookie, `emulateMedia({ colorScheme })`, `__dxApplyMode` in every frame, settle), read dark, restore.
   `FRESH_PHONE_LOAD` routes read a fresh phone page per scheme instead.
2. **axe** dark, then light (switching back in place).
3. **forced colors** for both schemes on ONE keyboard walk: Tab to the skip link, read its outline in light and dark
   while it holds focus, Enter, Tab; then each distinct control is focused ONCE and its outline read in each scheme,
   the scheme flipped inside the page by `__dxApplyMode` (synchronous; `getComputedStyle` recalculates, no settle).
   `html[data-theme]` decides every theme token and outranks the `prefers-color-scheme` defaults on `:root`, so the
   emulated media query does not take part in what the sweep reads.
4. **active option** (combobox, command): highlight once, read the outline in each scheme.

Each probe reports both schemes at once (`{ light: ..., dark: ... }` against the clean value), so a failure names the
scheme.

## Two history effects found, and how each was handled

The first comparison round (the design row 334 measured: two complete probe sequences, one per scheme, on the same
page) was not exact:

- **A second focus sweep is not the first.** The home page judged 81 controls instead of 83 on the second sweep, the
  number row 334 saw. Cause: the first sweep's `focus()` of a roving-tabindex container (Menubar, Tag Group) leaves it
  handing focus to its item, so a second pick drops it, and a second `focus()` of the same element lands on the item
  (measured: `DIV|menubar|dx-menubar` "unfocusable" on the second pass). It is not the focus start point. Fixed by
  construction: each control is focused once and judged in both schemes while it holds focus.
- **Any second re-layout changes closed panels.** After the scheme switch the overflow walk saw a different box list
  on 81 of 180 (route, scheme) readings. A fresh dark page given a second viewport round trip produces exactly the
  switched list (0 differing boxes on `button` and `combobox`), so this is history, not the scheme. On 178 readings
  the difference is entirely under an `overflow: hidden` ancestor (closed Accordion content: its
  `.dx-accordion-content-inner` wrapper drops to zero size), which `readOverflow` never counts. On `collapsible` and
  `tabs` the closed find-in-page panels (`hidden="until-found"`) lay out 1-2 boxes that nothing clips; those two
  routes joined `FRESH_PHONE_LOAD`.

## The comparison (all 90 routes x 2 schemes)

A scratch spec (not committed) recorded, per route, the old-style reference on FRESH loads per scheme (load in the
scheme, overflow round trip, axe, forced-colors walk) and the new sequence on one load, then diffed:

| Reading | Equal to the fresh load |
| --- | --- |
| overflow verdict (page scroll and offenders) | 180 / 180 |
| boxes that can offend (not under a clipping ancestor, not fixed) | 178 / 180 (`collapsible` dark, `tabs` dark: now fresh loads) |
| full box list | 99 / 180 (closed accordion content, above) |
| axe, every rule's violation, pass and incomplete node counts | 180 / 180 |
| forced colors: skip link (class, href, outline) | 174 / 174 |
| forced colors: every judged signature and its verdict | 174 / 174, no control gained or lost |

## Not built

- A reset of the focus start point: not needed once the walk is shared.
- Running the dark scheme's axe with only the contrast rules (row 334 measured it and declined it; unchanged).
