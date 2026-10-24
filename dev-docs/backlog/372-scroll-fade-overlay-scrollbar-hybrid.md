---
id: 372
title: "Scroll fade: transition the edge depths (landed) and paint the fade as gradients where the scrollbar is an overlay (deferred)"
status: "WAITING ON OWNER: part 1, the overlay-scrollbar hybrid, is deferred (\"leave mask for now\"); reopen…"
size: "M"
area: primitives
updated: 2026-10-24
---

# Row 372: scroll fade, edge transition and the overlay-scrollbar hybrid

## Item

The scroll fade is a `mask-image` on the scroller, so an overlay scrollbar (iOS, macOS; 0px gutter) fades with the content (seen on Firefox iOS). Two asks from the owner on 2026-10-23.

## Status / trigger

**WAITING ON OWNER: part 1, the overlay-scrollbar hybrid, is deferred ("leave mask for now"); reopen it to start.** Part 2 is landed (lane fadehybrid): the `--dx-scroll-fade-t/-b/-s/-e` depths transition, `transition: none` under reduced motion, `playwright/scroll-fade-transition.spec.ts`. Part 1 would paint the fade as sticky pseudo-element gradients from a `--dx-fade-surface` slot where the scrollbar is an overlay; the prototype notes below say what it needs.

## Size

M

## Detail

See Status. Spec: `playwright/scroll-fade-transition.spec.ts`; CSS: "Each edge's depth eases" in `preview/assets/dx-effects.css`.
