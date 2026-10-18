---
id: 237
title: "Carousel: a free-scrolling switch (Embla dragFree parity)"
status: "BACKLOG (owner 2026-10-18)"
size: "M"
area: carousel
updated: 2026-10-18
---

# Row 237: Carousel: a free-scrolling switch (Embla dragFree parity)

## Item

Carousel: a free-scrolling switch (Embla `dragFree` parity)

## Status / trigger

**BACKLOG (owner 2026-10-18)**

## Size

M

## Detail

shadcn's Carousel passes Embla options, and `opts={{ dragFree: true }}` gives free momentum scrolling with no snap and variable-width slides; ours is CSS scroll-snap with its own drag, paging and autoplay (`primitives/src/carousel.rs`). Construction: a `drag_free: bool` prop (default false) that drops `scroll-snap-type` on the track and lets a released drag coast instead of settling on a slide, while buttons and the keyboard still page by slide and `selected` tracks the nearest slide; variable-width items (`basis-auto`) supported; demo `carousel-drag-free`; spec: a fling coasts past a slide boundary without snapping back, Next still lands on a slide edge, RTL. The attachment strip stays on ScrollArea (owner Q&A 2026-10-18: list semantics, install weight).
