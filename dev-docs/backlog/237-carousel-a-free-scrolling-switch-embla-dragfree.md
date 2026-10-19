---
id: 237
title: "Carousel: a free-scrolling switch (Embla dragFree parity)"
status: "LANDED 2026-10-19 (drag_free prop, carousel-drag-free demos, carousel-drag-free.spec.ts)"
size: "M"
area: carousel
updated: 2026-10-18
---

# Row 237: Carousel: a free-scrolling switch (Embla dragFree parity)

## Item

Carousel: a free-scrolling switch (Embla `dragFree` parity)

## Status / trigger

**LANDED 2026-10-19**

## Size

M

## Detail

shadcn's Carousel passes Embla options, and `opts={{ dragFree: true }}` gives free momentum scrolling with no snap and variable-width slides; ours is CSS scroll-snap with its own drag, paging and autoplay (`primitives/src/carousel.rs`). Construction: a `drag_free: bool` prop (default false) that drops `scroll-snap-type` on the track and lets a released drag coast instead of settling on a slide, while buttons and the keyboard still page by slide and `selected` tracks the nearest slide; variable-width items (`basis-auto`) supported; demo `carousel-drag-free`; spec: a fling coasts past a slide boundary without snapping back, Next still lands on a slide edge, RTL. The attachment strip stays on ScrollArea (owner Q&A 2026-10-18: list semantics, install weight).

## Update 2026-10-19

`drag_free: ReadSignal<bool>` on `Carousel` (default false; also on the themed wrapper). One switch: `snap_type_value(orientation, drag_free)` is the only place snap vs. no snap is decided; it feeds the track's inline `scroll-snap-type` and a `data-snap` attribute (plus `data-drag-free`), and every script that suspends and restores snapping (`CAROUSEL_SCROLL_TO_JS`, `CAROUSEL_DRAG_JS`) restores from `data-snap` instead of a hard-coded `x/y mandatory`, so the restore can never re-enable snap on a drag-free track. Mouse/pen release: `endDrag` skips the nearest-slide settle and coasts on the release velocity (last 100 ms of pointer samples, exponential decay, tau 325 ms, stops at an edge or below 0.03 px/ms) via a bounded `requestAnimationFrame` loop in the drag script, like the existing rubber-band return (the motion-gating script only audits Rust timers; the loop is finite and user-initiated). Reduced motion: no coast, the track rests where released. Touch/trackpad coast natively once the snap is gone. Prev/Next/keyboard are unchanged (scroll-to lands on a slide edge); `selected` is still the nearest slide. Variable widths: demo items use `flex: 0 0 auto` with differing widths. `CarouselVirtualContent` ignores the prop (documented). Demos `drag_free` and `drag_free_rtl`; spec `playwright/carousel-drag-free.spec.ts`; docs.md "Free scrolling". Evidence: see the lane report. Remains: a drag-free virtual carousel, and fling momentum for a touch-less keyboard-only user is N/A.
