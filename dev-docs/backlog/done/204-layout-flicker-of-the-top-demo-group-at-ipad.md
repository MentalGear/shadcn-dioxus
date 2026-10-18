---
id: 204
title: "Layout flicker of the top demo group at iPad-landscape widths"
status: "LANDED 2026-10-14 (layout lane e9c53b1, 0ccc680, 6ca15cf; main-loop 7dda14b) -- the reported…"
size: "M"
area: process
updated: 2026-10-14
---

# Row 204: Layout flicker of the top demo group at iPad-landscape widths

## Item

Layout flicker of the top demo group at iPad-landscape widths

## Status / trigger

**LANDED 2026-10-14 (layout lane e9c53b1, 0ccc680, 6ca15cf; main-loop 7dda14b) -- the reported oscillation was NOT reproduced; two real re-layouts fixed; follow-ups 214, 215**

## Size

M

## Detail

Owner report; cause unknown. Debug lane records boxes/ResizeObserver/matchMedia at 1024-1366px, bisects; construction: a width-sweep layout-stability spec (320-1440 step 8) failing on any box change after load without input. Item 1. Plan: `dev-docs/research/next-round-plan-2026-10-14.md`. **Result:** no post-load oscillation at any iPad size or in a 320-1440 sweep. Two real top-group re-layouts fixed: (1) Avatar SSR != hydrated (the prerendered `data-state=empty` rendered the image and the fallback side by side, so the image jumped ~8.6px on hydration; 12 avatars in the home masonry): image and fallback now share one grid cell, image on top (`--dx-z-raise`); (2) `content-visibility: auto` on the multicol masonry widget cards made the columns re-balance 3-6 times after load: `contain: layout paint style` instead (14 cards). New `playwright/layout-stability.spec.ts` records boxes from the first frame and fails on any change >0.5px after max(FCP, DOMContentLoaded)+20ms (`/` light 64 widths + dark 34, `/docs`, `/demos`, `/component/avatar`, `/component/button`; `LS_STEP=8` for the full sweep; ~8 min on 3 workers): red on main, green on the fix. **Open:** row 214 (masonry column assignment depends on parse progress; the most plausible remaining cause of the owner's iPad flicker, not covered by a spec that starts at DOMContentLoaded) and row 215 (`/charts/`). A slider range grows 1-1.8px at hydration in some runs (not deterministic).
