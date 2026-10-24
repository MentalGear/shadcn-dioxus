---
id: 377
title: "Release-suite reds of batch 13: the detector's meta-proof, the baked build day, the spinner slot leak, the hidden code block's box"
status: "IN PROGRESS (lane b13fix)."
size: "M."
area: playwright
updated: 2026-10-24
---

# Row 377: the four reds of the first per-deploy release suite

## Item

`scripts/deploy-preview.sh --suite-only` (row 322) ran 3,791 tests on batch 13 and had four failures. The deploy refuses on any red, so each needed a fix by construction.

## Status / trigger

IN PROGRESS (lane b13fix).

## Size

M.

## Detail

1. **`hydration-parity` Rule 4c `selftest:unfixed:id`** is the detector's meta-proof (the synthesized fragment carries the collision on purpose), so it fails by construction. It is declared `test.fail` now: Playwright reports the failure as an expected outcome (`stats.expected`, which `deploy-preview.sh`'s `suite_verdict` counts as a pass) and the case turns red the day the detection stops seeing the collision. No change to `run-suite.sh` or `deploy-preview.sh` was needed.
2. **`date-picker` "starts as 'In 2 days'"**: not two clocks in the spec. `natural_language` read `UtcDateTime::now()` while rendering, so the prerendered page carries the BUILD day, and hydration keeps the server's text node: the sentence showed build day + 2 until something re-rendered it (the deploy built at 23:39 UTC, the suite ran after 00:00 UTC). The demo now reads the clock in an effect (and in the input handler, not at mount), the spec freezes the page clock with `page.clock.setFixedTime` at a day far from any build, and `ssr_parity` lists the demo and its page with the reason `CLOCK`. `unavailable_dates` had the same render-time read and got the same effect. `date-picker:128` ("uses the CSS-anchor path") reads no date; it was not in this run's failures.
3. **`variant-nesting` `/component/spinner/`**: the typed sizes of Spinner set `--dx-spinner-size` on the spinner, a custom property that inherits, so a default spinner nested in a small, large or extra-large one read the outer size (the default root never re-declared it). Two variables now: `--dx-spinner-size` stays the CONTAINER slot (Badge sets it) and no spinner ever sets it; `--dx-spinner-edge` is the spinner's own edge, re-declared by every root (default from the container slot, each typed size from its value).
4. **`layout-stability` `/component/avatar/` light**: the box was the `pre.dxc` of the "Manual installation files" code block, inside a closed `hidden="until-found"` collapsible (`content-visibility: hidden`). The recorder's `getBoundingClientRect` forced a layout of that skipped subtree from whatever the parser had delivered, so while the document was still `loading` the half-parsed block grew sideways and downwards (724x1029 -> 1001x6603); the spec's "still being filled" clause forgives a pure height change only, and the width grew too. Nothing was on screen. The recorder now skips an element that has client rects but fails `checkVisibility()` (a `display: none` box still counts), and the sweep takes `LS_CPU=N` (CDP CPU throttling) so the loaded-box timing is reproducible: red 3 of 5 widths before, green after.

## Same class, not fixed here

Render-time clock reads that bake the build day into the prerendered page: `calendar` and `date_picker` prop defaults (`OffsetDateTime::now_local_date()` in `primitives/src/calendar.rs`, `date_picker.rs`; `preview/src/components/calendar/component.rs`) and `calendar/variants/week_numbers` (`UtcDateTime::now().date().year()`). They only show on a page opened after the day or month changes, until the first re-render; the calendar and date-picker specs compare with the browser's clock, so they fail only across a month boundary.
