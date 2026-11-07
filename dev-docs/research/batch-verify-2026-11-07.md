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
