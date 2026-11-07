---
id: 497
title: "ScrollArea ScrollType::Hidden shows the scrollbar"
status: "FIXED 2026-11-07, landed in PR #168 (lane lane/scrollarea-497). Found by row 399's first-half lane…"
size: "S"
area: components
updated: 2026-11-07
---

# Row 497: ScrollArea ScrollType::Hidden shows the scrollbar

## Item

`ScrollType::Hidden` should hide the native scrollbar while the area still scrolls. `primitives/src/scroll_area.rs`
puts `scrollbar-width` among the element's owned attributes, so it reaches the DOM as an HTML attribute
(`scrollbar-width="none"`) instead of a CSS declaration: the Hidden scroller measured `overflow-y: scroll` with a
computed `scrollbar-width: auto`, so the bar shows. The `scroll_area` `scroll_type` demo (row 399) shows it side by
side with Auto and Always.

## Status / trigger

FIXED 2026-11-07, landed in PR #168 (lane `lane/scrollarea-497`). Found by row 399's first-half lane (2026-11-06); measured on a
debug SSG build in Chromium. `ScrollType::Hidden` now puts the presence attribute `data-scrollbar-hidden`
(`state_attrs::Flag::ScrollbarHidden`) on the scroller and `preview/src/components/scroll_area/style.css` maps it to
`scrollbar-width: none` plus `::-webkit-scrollbar { display: none }`. Spec `playwright/scroll-area-scroll-type.spec.ts`
(red on a base build: computed `scrollbar-width` is `auto`; Rust unit test in `scroll_area.rs`). The other `ScrollArea`
attributes (`dir`, `data-scroll-direction`, `data-direction`, `data-scroll-fade*`, `class`, `style`) are real attributes
or the style string; none was a CSS property.

## Size

S

## Detail

**Reproduce first:** a spec on the `scroll_type` demo reads the Hidden scroller's computed `scrollbar-width` (and
`::-webkit-scrollbar` width where it applies) and fails today. **Fix:** emit it as a style declaration (or a class in
the component sheet keyed on a presence attribute), and check the other `ScrollArea` attributes for the same
attribute-versus-style slip.
