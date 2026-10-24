---
id: 377
title: "Release-suite reds of batch 13: the detector's meta-proof, the baked build day, the spinner slot leak, the hidden code block's box"
status: "LANDED in lane b13fix. Verified on one debug SSG build: variant-nesting…"
size: "M."
area: playwright
updated: 2026-10-24
---

# Row 377: the four reds of the first per-deploy release suite

## Item

`scripts/deploy-preview.sh --suite-only` (row 322) ran 3,791 tests on batch 13 and had four failures. The deploy refuses on any red, so each needed a fix by construction.

## Status / trigger

LANDED in lane b13fix. Verified on one debug SSG build: variant-nesting, demos-field-input-group-badge, date-picker (natural language, `:128`), hydration-parity Rule 4c each 3 repeats green (183 tests), layout-stability avatar light and dark at `LS_STEP=32` 3 repeats green (64 widths each); `LS_CPU=4` at 5 widths was red (4 of 5 widths) before the recorder change and green after; the natural-language test was red on the build-day markup (expected April 02, 2031, page showed October 25, 2026). `pnpm run test:changed` with `CHANGED_RANGE=queue/batch-13..HEAD`: 26 spec files, 657 passed. The cargo arms (clippy x3, test, test-server, `ssr_parity` with the new `CLOCK` entries) were not run in the lane (disk).

## Size

M.

## Detail

1. **`hydration-parity` Rule 4c `selftest:unfixed:id`** is the detector's meta-proof (the synthesized fragment carries the collision on purpose), so it fails by construction. It is declared `test.fail` now: Playwright reports the failure as an expected outcome (`stats.expected`, which `deploy-preview.sh`'s `suite_verdict` counts as a pass) and the case turns red the day the detection stops seeing the collision. No change to `run-suite.sh` or `deploy-preview.sh` was needed.
2. **`date-picker` "starts as 'In 2 days'"**: not two clocks in the spec. `natural_language` read `UtcDateTime::now()` while rendering, so the prerendered page carries the BUILD day, and hydration keeps the server's text node: the sentence showed build day + 2 until something re-rendered it (the deploy built at 23:39 UTC, the suite ran after 00:00 UTC). The demo now reads the clock in an effect (and in the input handler, not at mount), the spec freezes the page clock with `page.clock.setFixedTime` at a day far from any build, and `ssr_parity` lists the demo and its page with the reason `CLOCK`. `unavailable_dates` had the same render-time read and got the same effect. `date-picker:128` ("uses the CSS-anchor path") reads no date; it was not in this run's failures.
3. **`variant-nesting` `/component/spinner/`**: the typed sizes of Spinner set `--dx-spinner-size` on the spinner, a custom property that inherits, so a default spinner nested in a small, large or extra-large one read the outer size (the default root never re-declared it). Two variables now: `--dx-spinner-size` stays the CONTAINER slot (Badge sets it) and no spinner ever sets it; `--dx-spinner-edge` is the spinner's own edge, re-declared by every root (default from the container slot, each typed size from its value).
4. **`layout-stability` `/component/avatar/` light**: the box was the `pre.dxc` of the "Manual installation files" code block, inside a closed `hidden="until-found"` collapsible (`content-visibility: hidden`). The recorder's `getBoundingClientRect` forced a layout of that skipped subtree from whatever the parser had delivered, so while the document was still `loading` the half-parsed block grew sideways and downwards (724x1029 -> 1001x6603); the spec's "still being filled" clause forgives a pure height change only, and the width grew too. Nothing was on screen. The recorder now skips an element that has client rects but fails `checkVisibility()` (a `display: none` box still counts), and the sweep takes `LS_CPU=N` (CDP CPU throttling) so the loaded-box timing is reproducible: red 3 of 5 widths before, green after.

## Follow-up (lane b13fix2): the date's span reserves its width

Moving the clock read into an effect left the `natural_language` date span empty at first paint, so the sentence was one line with JavaScript off (84.25 px field) and wrapped to two once the date filled in (101.25 px): `first-paint-styles` `/component/date_picker/` was the one red left of the batch-13 release suite. The span now shares one grid cell with a hidden `::before` copy of the widest date text (`data-widest-date="September 30, 2026"`, a demo-local `style {}` rule, `visibility: hidden` so it is out of the accessibility tree and `textContent`), so the span is as wide as that text at first paint, hydrated and for every date, and the clock stays read after mount. `date-picker.spec.ts` gained "the sentence takes the same box for every month's date" (all twelve months, same sentence height and span width). `unavailable_dates` needed nothing: its effect fills `disabled_ranges`, which paints no box in the closed picker. `ssr_parity` still lists `date_picker/natural_language` (the text still goes empty -> filled, so the entry is not stale); its `CLOCK` reason now says the reserved width is why no box moves. Verified on one debug SSG build: `first-paint-styles` for date_picker 3/3 and 3 repeats green, `date-picker.spec.ts` 81/81, `ssr_parity` green in both arms, clippy `-p preview --features web` clean.

## Same class, not fixed here

Render-time clock reads that bake the build day into the prerendered page: `calendar` and `date_picker` prop defaults (`OffsetDateTime::now_local_date()` in `primitives/src/calendar.rs`, `date_picker.rs`; `preview/src/components/calendar/component.rs`) and `calendar/variants/week_numbers` (`UtcDateTime::now().date().year()`). They only show on a page opened after the day or month changes, until the first re-render; the calendar and date-picker specs compare with the browser's clock, so they fail only across a month boundary.
