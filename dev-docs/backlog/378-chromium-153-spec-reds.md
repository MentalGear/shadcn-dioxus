---
id: 378
title: "Specs red on Chromium 153 (chromium-1243): two product bugs in the anchored-overlay path, three spec assumptions"
status: "LANDED in branch claude/chromium-153-specs, based on claude/b13fix (batch 13: it carries row 374's…"
size: "M"
area: primitives
updated: 2026-10-24
---

# Row 378: specs that fail on Chromium 153 and pass on Chromium 141

## Item

Fresh containers ship chromium-1243 (Chrome for Testing 153.0.8010.12) with Playwright 1.60; the browser path resolver
is row 374. On it these failed deterministically while passing on Chromium 141: top-layer Rule 8 (both cases), Rule
11(c) "Popover: no anchor support from the start", `date-picker.spec.ts:128` ("uses the CSS-anchor path"),
`markdown.spec.ts:74`, `carousel.spec.ts:1901`, `nav_link.spec.ts:91`. Root-cause each; fix by construction, never by
skipping; say which are product bugs.

## Status / trigger

LANDED in branch `claude/chromium-153-specs`, based on `claude/b13fix` (batch 13: it carries row 374's resolver and
edits the same specs). Verified on one debug SSG build of that branch, chromium-1243: `top-layer.spec.ts` +
`date-picker.spec.ts` 175 passed, `carousel.spec.ts` 167 passed (`--workers=2`), `markdown` "link policy" and
`nav_link.spec.ts` 3 repeats green. Before: 8 red on the batch-13 build (`nav_link` did not reproduce here; the
other seven did, plus `carousel.spec.ts:1635` and `:2252`, same cause as `:1901`).

## Size

M

## Detail

Product bugs (both in shared primitives, so every consumer is affected):

1. **The anchor outcome check measured the open animation** (`primitives/src/top_layer.rs`,
   `use_anchor_position_fallback`). The check compares the panel's `getBoundingClientRect()` with the side/align
   contract to decide whether CSS `anchor()` placed it. Chromium 153 applies an open animation's first keyframe at
   that synchronous first measurement, so the theme's 4px drop (`dx-drop-in`, `translate`) made a correct placement
   miss `matches()` by 4px and every animated anchored overlay (DatePicker, ColorPicker) took the JS fallback, losing
   native scroll tracking. Probe: panel at y=505 during the drop, `anchor()` target 509; with animations disabled the
   CSS path held. Fix: `restingRect` seeks the panel's own finite animations to their end for the one read and puts
   them back in the same task (no paint, no finish event). Fixed `date-picker:128`, `:162` and Rule 8 ColorPicker.
2. **Opening a panel let the browser's focusing step scroll the page** (`primitives/src/modality.rs`, `dxOpenFocus`,
   the `'browser'` plan for mouse and keyboard opens). That step has no `preventScroll`, and Chromium 153 focuses a
   `<dialog popover>` with no control inside and scrolls to it. On an engine without anchor positioning the panel is
   not placed yet at that moment, so the page jumped from scrollY 473 to 4710 and the panel, then placed next to its
   trigger, was 3,700px off screen (Rule 11(c) Popover). Fix: the page keeps its scroll across `show()`, as on the
   policy's other paths, which all pass `preventScroll`.

Spec races (no product change):

3. **Carousel end-edge wheel band** (`:1635`, `:1901`, `:2252`): Chromium 153's smooth scroll spends its last frames
   1px short of the snap point; `expectSnappedToBoundary`'s 2px tolerance accepted it, the test pushed while the
   scroller was still moving, and the band's own at-rest test (0.5px) rightly refused. The helper now also needs the
   slide to hold still for three frames.
4. **New tabs** (`markdown:74`, `nav_link:91`): a tab opened by a link can surface on its initial `about:blank`;
   `popup.evaluate` then died with the navigation, and `waitForLoadState` could resolve on the blank page. Both wait
   for the linked URL.
5. **Rule 8 JS-fallback case**: Chromium 153 places the fixture's stacking Popover natively, so "the fixture lands on
   the fallback" stopped holding. The case now forces the fallback with `stripAnchorSupportsBlock` before the open.
