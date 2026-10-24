---
id: 91
title: "Carousel landed — CSS scroll-snap engine, tier-1 APG oracle vendored, three live-verification fixes"
status: "Landed 2026-09-20 (round 4; d64b014, 9a301b9, 01b7437, 2046136, 5b8c21a, 2ec1b70)"
size: "L"
area: tests
updated: 2026-09-20
---

# Row 91: Carousel landed — CSS scroll-snap engine, tier-1 APG oracle vendored, three live-verification fixes

## Item

Carousel landed — CSS scroll-snap engine, tier-1 APG oracle vendored, three live-verification fixes

## Status / trigger

**Landed 2026-09-20 (round 4; `d64b014`, `9a301b9`, `01b7437`, `2046136`, `5b8c21a`, `2ec1b70`)**

## Size

L

## Detail

The carousel is a CSS scroll-snap engine written in Rust (`primitives/src/carousel.rs`, themed package
`preview/src/components/carousel/`), landed in five waves. Owner decision (row 86): build it in-repo on scroll-snap, no new
dependency; root arrow keys RTL-swapped per Radix; vertical orientation in v1.

- **v1, 2026-09-20 (round 4; commits above):** `Carousel`/`CarouselContent`/`CarouselItem`/`CarouselPrevious`/`CarouselNext`
  plus `CarouselApi`/`use_carousel()`. Paging calls the target slide's own `scrollIntoView()` (later replaced, below); a
  second `document::eval` bridge keeps the selected index right after native drag/wheel by comparing
  `getBoundingClientRect()` positions, never `scrollLeft`. Region/group roles with `aria-roledescription`, SSR-stable "N of M"
  labels, native `disabled` at the ends. Oracle: the sixth vendored APG reference page (carousel, pinned `7e4034b`),
  `playwright/oracle/tier1-apg/carousel.spec.ts` plus `playwright/carousel.spec.ts`. Three defects only live verification
  caught: no keyboard path to the scroll region (`tabindex="0"`), the demo's own label containing the word "carousel", and
  a chevron that did not mirror under RTL (one scoped CSS rule).
- **Fast-follows, 2026-09-25 (round 8b; `bfcff32`, `4bd8816`, `59902a2`, `b557dfa`):** rewind-style `loop`, a behaviour-only
  `CarouselAutoplay` (pauses on hover and focus, off under `prefers-reduced-motion`), `CarouselRotationControl`, and a
  tablist variant. Same day (lane `carousel-no-page-scroll`): every paging path now calls `scroller.scrollBy` on the
  carousel's own scroller instead of `scrollIntoView`, which had scrolled the *page* on every autoplay tick; scroll-snap is
  suspended for the scroll's duration so a smooth multi-slide jump is not cut at the first snap point.
- **Inert slides, 2026-09-26 (lane `carousel-a11y`):** every slide outside the visible range renders `inert`, and focus is
  redirected to the scroller when its slide becomes inert. `inert` is typed `Option<&'static str>` because
  `dioxus-interpreter-js` 0.7.9 hardcodes its boolean-attribute list and `inert` is not on it (a raw `bool` rendered
  `inert="false"` as present). The visible range comes from a `ResizeObserver` inside the tracking bridge.
- **Virtualised content, 2026-09-26 (lane `carousel-virtual`):** `CarouselVirtualContent<T>` windows a data-driven list with a
  seamless loop (an `anchor` position that may run past `[0, N)`, the shortest signed delta mod N for paging).
  SSR renders the full list; the window starts after mount. Two feedback bugs found live (anchor drift at mount, a dropped
  step under back-to-back paging) were closed by construction.
- **shadcn parity, 2026-09-26 (lane `carousel-parity`, `d410269`..`c300a8f`, PR #70, merged in `78cff69`):** gap moved inside
  the slide (shadcn's padding plus negative margin), whole-slide default sizing with opt-in peek and per-view, an `align`
  option, `CarouselTabList`/`CarouselTab` renamed `CarouselIndicators`/`CarouselIndicator`, loop reshaped into
  `LoopMode { Seamless, Rewind }`. Two regressions only live runs caught (edge-clearance math reading the gap-widened box;
  `display: flow-root` for vertical margin collapse).

Evidence at the last wave: 98 Rust unit and SSR carousel tests, `carousel.spec.ts` plus the tier-1 oracle 150/151 on a live
server (the red is the documented parallel-worker flake), SSG release build 150/151 plus `hydration-parity` 40/41 (the
by-design `selftest:unfixed:id`), all gates clean. Left open or deferred: the bench's `MutationObserver` re-centre correction
was replaced by a `use_effect` validated empirically; `aria-controls` from an indicator to its slide is absent under
virtualised loop; touch-swipe and raw trackpad feel are not testable in the sandbox. The edge-overscroll mechanism is
rows 96 and 102 (mode D); research in `dev-docs/archive/research/carousel-*`.

*Trimmed in row 266 (repo cleanup). The full dated narrative of this row is in git: `git show 3c584cc2:dev-docs/backlog/done/091-carousel-landed-css-scroll-snap-engine-tier-1.md`.*
