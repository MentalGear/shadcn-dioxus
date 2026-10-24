---
id: 96
title: "scroll-snap-stop: always caps a browser-driven scroll but NOT this crate's own pointer-drag release"
status: "Landed 2026-09-20 (round 5) after the user chose a CSS-only no-skip guarantee over matching embla's…"
size: "DONE as far as it goes; the residual…"
area: docs
updated: 2026-09-20
---

# Row 96: scroll-snap-stop: always caps a browser-driven scroll but NOT this crate's own pointer-drag release

## Item

`scroll-snap-stop: always` caps a browser-driven scroll but NOT this crate's own pointer-drag release

## Status / trigger

Landed 2026-09-20 (round 5) after the user chose a CSS-only no-skip guarantee over matching embla's velocity model

## Size

DONE as far as it goes; the residual gap is deliberate, not outstanding work

## Detail

The user chose a CSS-only no-skip guarantee over matching embla's velocity model (round 5, 2026-09-20): "the exact velocity is not
important, since we use native css ... we can only set no overscroll of slide css property if the browser supports it", then "lets not
go overboard with parity where it does not matter". So `scroll-snap-stop: always` is set on every slide and no JS velocity model was
built. The residual gaps below are deliberate, not outstanding work; reopen only if the user asks.

**What it covers (measured twice independently).** A browser-animated scroll is capped to one slide pitch: `scrollBy({left: 1000,
behavior: 'smooth'})` travels 1008 px (3.00 slides) without the property and exactly 336 px (1.00 slide) with it, at a 333 px pitch.

**What it does not cover.** Our own pointer-drag release: the drag path holds `scroll-snap-type: none` for the whole gesture, so there
is no scrolling operation for the property to constrain, and the settle lands on the numerically nearest slide (a 2.5-pitch drag
settles at 2, as if the property were absent). Nor the short-fast-flick symptom that motivated the lane (a quick flick settling back
to the origin): it is velocity-shaped and no CSS property can observe release speed. Closing either means a JS distance-or-speed
threshold, i.e. the velocity model under another name, which the user declined.

**Correction, same day: the first implementation was inert.** It set the property from a scoped `<style>` tag rendered by
`CarouselContent`; on a real release SSG build `getComputedStyle(slide).scrollSnapStop` read `normal` and the 1000 px scroll travelled
1008 px. The `<style>` had the right id but `styleElement.sheet.cssRules.length` was 0: Dioxus emits hydration marker comments
inside the element's text (`<!--node-id413-->@supports (...) {...}<!--#-->`), `<!--` and `-->` are CDO/CDC tokens in CSS, and the
ident between them starts a qualified rule that swallows the real `@supports` rule. It would have worked in pure CSR, which is why it
looked right in a dev loop. **Fixed** by declaring `scroll-snap-stop:always` inline on `CarouselItem` beside its
`scroll-snap-align:start`; the `@supports` guard was deleted (an unrecognised property in a `style` attribute is dropped by the
parser, which meets "if the browser supports it" with no stylesheet). Verified post-hydration: computed `always` on all five slides
and 1000 px, 2.5-pitch and 3-pitch scrolls all cap to 336 px. **The shipped test was a false green:** a 2.5-pitch scroll asserts slide
2, the nearest snap with or without the property (54/54 against an inert build); replaced by a 3-pitch scroll where capped (slide 2)
and uncapped (slide 4) diverge.

**Update 2026-09-21 (round 6).** The drag release now animates: it goes through the same `scrollIntoView` paging path as
`CarouselPrevious`/`CarouselNext`, and the `scroll-snap-type` restore is deferred until that scroll finishes (`scrollend`, timeout
fallback), a fresh `pointerdown` cancelling any pending restore. Measured: 11 distinct `scrollLeft` values across a release against 2
before. The cap-to-one-slide gap persists for a restated reason: the release picks the nearest slide by `getBoundingClientRect`
geometry, which has no "cap from the gesture's origin" concept. The release does one `getBoundingClientRect` read per gesture,
deliberately not a `scrollLeft` read (the RTL hazard), outside the move hot path.

**2026-09-26:** the owner re-evaluated the edge-overscroll framing in the overscroll bench
(`playwright/fixtures/carousel-overscroll-bench-rev40.html`; its copy under `dev-docs/research/` was deleted by row 266) and chose
mode D over B3 and E, see row 102.

*Trimmed in row 266 (repo cleanup). The full dated narrative of this row is in git: `git show 3c584cc2:dev-docs/backlog/done/096-scroll-snap-stop-always-caps-a-browser-driven.md`.*
