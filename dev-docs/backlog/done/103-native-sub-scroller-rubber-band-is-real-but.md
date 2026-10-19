---
id: 103
title: "Native sub-scroller rubber-band is real but composited, so it is invisible to…"
status: "CLOSED 2026-10-19 (main-loop review: the fix or lesson is in place, see Size/Detail). Measured…"
size: "N/A — a measurement limit to know, not…"
area: inputs
updated: 2026-10-19
---

# Row 103: Native sub-scroller rubber-band is real but composited, so it is invisible to…

## Item

Native sub-scroller rubber-band is real but **composited**, so it is invisible to `getBoundingClientRect`/`scrollLeft` — an instrumented observer records nothing and reads as "no bounce"

## Status / trigger

**CLOSED 2026-10-19 (main-loop review: the fix or lesson is in place, see Size/Detail).** Measured 2026-09-23 with a script-free scroll-snap control added to the bench; the owner corrected two wrong conclusions I drew from its silence

## Size

N/A — a measurement limit to know, not work to schedule

## Detail

I reported first "leave the wheel to the platform, macOS does this natively" (wrong — macOS gives elasticity to the *document*, not to an arbitrary sub-scroller) and then "there is no native rubber-band here at all" (also wrong). The owner settled it: *"there is an end rubber-band if there is still momentum coming from slide 2 to slide 1 and over, even if there is none from slide 1 to end."* The bounce exists when momentum arrives at an edge; it just never reaches a JS-readable layer. **The consequence is constructive:** it cannot be *measured* from inside the page, but it can be *modelled*, because the WebKit curve is published — `b(x) = (1 - 1/(x*c/d + 1))*d` with `c ~= 0.55`, algebraically identical to `L*x/(x+L)` with `L = d/c`. Other platform numbers from the same control, all reusable: rest after the last wheel event **64-78ms** typical (outliers 210/526/609ms on deliberately gentle gestures), device delta quantum **1px**, momentum decay **~0.96x per event**. All from one engine on one machine — Chromium and Gecko have their own curves, and a wrong constant degrades gracefully (stiffer or looser band) rather than breaking.
