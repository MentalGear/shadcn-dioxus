---
id: 372
title: "Scroll fade: transition the edge depths (landed) and paint the fade as gradients where the scrollbar is an overlay (deferred)"
status: "Part 2 LANDED (lane fadehybrid): the registered --dx-scroll-fade-t/-b/-s/-e depths transition…"
size: "M"
area: primitives
updated: 2026-10-23
---

# Row 372: scroll fade, edge transition and the overlay-scrollbar hybrid

## Item

The scroll fade is a `mask-image` on the scroller, so an overlay scrollbar (iOS, macOS; 0px gutter) fades with the content (seen on Firefox iOS). Two asks from the owner on 2026-10-23.

## Status / trigger

**Part 2 LANDED (lane fadehybrid):** the registered `--dx-scroll-fade-t/-b/-s/-e` depths transition (`--dx-motion-duration-base`, `--dx-motion-ease-out`, fallbacks 150ms ease-out) on every `dx-scroll-fade*` class in `dx-effects.css`, `transition: none` under `prefers-reduced-motion: reduce`. Where `animation-timeline: scroll()` exists the depth is an animation of the scroll position, which outranks a transition, so the transition only smooths browsers without it (WebKit before 26, Firefox: the script there sets the depth between 0 and full). `playwright/scroll-fade-transition.spec.ts` (computed transition, CSSTransition objects sought to 75/150 ms, reduced motion, no transition under the scroll timeline). Documented in `scroll_area/docs.md` and `utilities_docs.rs`.

**Part 1 DEFERRED by the owner ("leave mask for now"):** keep the mask everywhere it works; where an axis overflows, the measured gutter is 0 and `scrollbar-width` is not `none`, have the scroller's script set one attribute and paint the fade as sticky pseudo-element gradients from a `--dx-fade-surface` theme slot instead. Prototype notes: sticky `::before`/`::after` need the scrollport size (percent heights are indefinite for `max-height` scrollers) and the host's padding (sticky insets are measured from the scrollport, the pseudo's natural position from the content box), so the script would also write those; grid hosts cannot take in-flow pseudo-elements; flex hosts need the footprint cancelled by negative margins; a bare `dx-scroll-fade` class with no script cannot detect an overlay at all.

## Size

M

## Detail

See Status. Spec: `playwright/scroll-fade-transition.spec.ts`; CSS: "Each edge's depth eases" in `preview/assets/dx-effects.css`.
