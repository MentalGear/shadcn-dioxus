# Batch verification, 2026-11-07

Session head `330a7254` (every lane merged), base main `d040554a`. Debug SSG build of HEAD (`verify-head`), chromium
only, `ssg.local.config.ts`. The HEAD pub dir was deleted afterwards to make room for the base build.

## Method notes

- `pnpm run test:changed -- --workers=2 --shard=i/N` does not pass its arguments to Playwright: pnpm hands
  `tools/test-changed.sh` a literal `--`, so everything after it becomes a spec filter. The `--workers`, `--shard`,
  `--project` and `--reporter` flags were ignored (4 workers, the whole set). Two "shards" were therefore two full runs of
  the 272 selected spec files (3902 tests each, about 1.3 h each). Call `tools/test-changed.sh --shard=i/N` directly.
- Two full runs of the same selection on the same build make a flake check: the same 15 tests were red in both.

## New or rewritten specs on HEAD

First run: 105 passed, 5 failed, 1 skipped. All five were stale spec expectations, not app defects; fixed in
`d818d2f7` and rerun green:

| Spec | Cause | Fix |
|---|---|---|
| combobox.spec.ts:21, :257 | `data-highlighted` is now a presence attribute (`""`), spec expected `"true"` | expect `""` |
| dismiss-parity.spec.ts:469, :480 (Combobox clear button) | listbox located by `[data-state='open']`, now `[data-open]` | locator |
| pagination.spec.ts:87 (disabled ends) | `tabIndex` of an `<a>` reads 0 with or without `href`; the frame locator `#component-preview-frame-disabled_ends` is correct | assert `focus()` cannot reach the link |

modality-restyle (4/4), controlled-docs, demo-modes, the rest of pagination: green.

## Full selection (`CHANGED_RANGE=d040554a...HEAD`, 272 spec files)

Run 1: 3867 passed, 15 failed, 20 skipped. Run 2: 3866 passed, 16 failed, 20 skipped (the extra red:
`menu-submenu.spec.ts:490`, green on rerun x3). Reds and dispositions (fix `a4c3f4e9`, rows 484/485 vocabulary):

| Test | Cause | Disposition |
|---|---|---|
| accordion.spec.ts (3), accordion-animation.spec.ts, keyboard-matrix Accordion Enter/Space (2) | `[data-open], [data-closed]` + has-button also matches the theme popover and the sort menu, so `.first()` is not an accordion item | locate `.dx-accordion-item` (also main-thread.spec.ts, same selector) |
| tabs.spec.ts (3), keyboard-matrix Tabs Right Arrow | active panel carries no attribute any more, inactive panels carry `data-hidden` | `:not([data-hidden])` |
| input_otp.spec.ts:228 | inactive slot has no `data-active` (was `"false"`) | `not.toHaveAttribute` |
| line_chart.spec.ts:168 | toggle `data-active` is `""` | expect `""` |
| keyboard-matrix Combobox Down Arrow | `data-highlighted` `""` | expect `""` |
| menu-submenu.spec.ts:322 (DropdownMenu.Sub hover intent) | hard red x3: the `hover()` wrapper of fixtures.ts (row 483) waits for panel animations, which a paused page clock never finishes, so the test times out in `hoverAndCheckIntentDelay` | helper moves the raw pointer |
| select-escape-race.spec.ts:32 | flaky: red in both full runs (round 3, round 7), green x3 alone, red 2/48 with 4 workers x12 | left for its lane; see base comparison below |
| menu-submenu.spec.ts:490 | run 2 only; green x3 | flake |

After the fixes, the reds rerun x3: 150 passed except menu-submenu:322 (then fixed: 48/48 over x3 of the whole file).

## all-routes (axe split, row 152), HEAD

94 tests passed in 8m11s wall (2 workers, debug build), 88 of 94 routes carry the annotation
`structure differs between light and dark, so dark ran every rule`, so the split falls back to the full dark scan on
almost every route. Base comparison and durations below.

## Red on base (`d040554a` debug build, HEAD specs overlaid; dismiss-parity and combobox from `f3a273d3`)

Red on base, green on HEAD (the batch's proof):

| Lane / row | Cases red on base |
|---|---|
| 490 modality | modality-restyle 4/4 |
| 498 pagination | `pagination.spec.ts` disabled Previous/Next |
| combobox Base UI | combobox.spec "dynamic option removal"; dismiss-parity "a mouse press on a control that keeps the input focused closes the list". The touch clear-button case was green on base (not red as listed). |
| 392/387 controlled docs | controlled-docs: the page case and 9 per-component cases |
| 478 item 4 navmenu | navigation_menu "hover opens the panel after the 50 ms intent delay" and the open-motion transform case; menu-submenu hover intent for DropdownMenu.Sub and ContextMenu.Sub |
| 487 dismiss | dismiss-parity touch Combobox scroll and Dialog drag-from-backdrop cases |

Caveat: `dropdown-menu.spec.ts:9` is also red on base, but through the `data-open` vocabulary of the overlaid spec, not
the delay case (the delay cases live in navigation_menu and menu-submenu). The skeleton reduced-motion case has no
spec of its own beyond `dx-class-migration.spec.ts` (green in the full selection).

`select-escape-race.spec.ts:32` is flaky on BASE too (4 of 48 with 4 workers x12; HEAD 2 of 48): not caused by the batch.
It belongs to the select/Escape-race work and is left for its owner.

## all-routes timing (row 152), debug, 2 workers, same machine

| Run | Tests | Wall | Sum of test seconds | axe "structure differs" annotations |
|---|---|---|---|---|
| base build, base spec (`d040554a` all-routes.spec.ts + axe.ts) | 93 | 6m38s | 788 | n/a |
| base build, HEAD spec | 93 | 7m24s | 880 | 87 |
| HEAD build, HEAD spec | 94 | 8m11s | 973 | 88 |

The axe split did not shorten the file here: 88 of 94 routes report that the light and dark structures differ, so the
dark pass runs every rule on nearly every route, and the HEAD spec does more per route. Left for lane `axe-split-152`:
find why the structure differs (a theme-dependent node, or the comparison key) before the split can save anything.
`playwright/durations.json` now carries measured entries for all-routes (973 s, 94 tests), controlled-docs, demo-modes,
modality-restyle and pagination.

## Left open

- Lane axe-split-152: the fallback above.
- select-escape-race flake (pre-existing).
- The full selection was run only on HEAD (twice, see notes); no base run of the full selection.
- Not run: the registry install smoke (no component source changed in the fixes).
