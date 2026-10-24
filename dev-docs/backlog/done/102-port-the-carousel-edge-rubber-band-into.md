---
id: 102
title: "Port the carousel edge rubber-band into primitives/ as mode B3 (edge transform) — LANDED 2026-09-25 (round…"
status: "LANDED 2026-09-25 (round 8), SUPERSEDED 2026-09-28 by mode D (863f8ac) -- Owner reported twice that…"
size: "M (done: the port landed, then mode D…"
area: carousel
updated: 2026-09-28
---

# Row 102: Port the carousel edge rubber-band into primitives/ as mode B3 (edge transform) — LANDED 2026-09-25 (round…

## Item

Port the carousel edge rubber-band into `primitives/` as mode B3 (edge transform) — **LANDED 2026-09-25 (round 8), then SUPERSEDED 2026-09-28 by mode D**; was: the component still behaves as mode A (clamped)

## Status / trigger

**LANDED 2026-09-25 (round 8), SUPERSEDED 2026-09-28 by mode D (`863f8ac`)** -- Owner reported twice that you cannot drag past slide 1 or the last slide and get a rubber-band; five mechanisms benched side by side 2026-09-22/23. **LANDED 2026-09-25 (round 8, `188e50b`, `c59dc8b`, `51a96a6`)**

## Size

M (done: the port landed, then mode D replaced it)

## Detail

The owner reported twice that you cannot drag past slide 1 or the last slide and get a rubber-band; five mechanisms were benched
side by side 2026-09-22/23 (write-up: [`carousel-overscroll-2026-09-23.md`](../../archive/research/carousel-overscroll-2026-09-23.md)).
The bench HTML is `playwright/fixtures/carousel-overscroll-bench-rev40.html` (its byte-identical copy under `dev-docs/research/`
was deleted by row 266).

**The rule the research ended on.** Root cause of every glitch across ~20 bench revisions is one thing: during a wheel or trackpad
gesture the scroll position has three would-be writers (the compositor's momentum animation, where `preventDefault` is only
advisory; the snap engine on its own schedule; and our code), and two of the three are neither observable nor sequenceable from
JS. The mouse-drag path stayed clean for 20+ revisions because a drag has no native scrolling. So: where the browser is already
the scroller, never write the scroll position and never disable snapping, and express the overdrag as a transform only; where
there is no native scrolling, owning it is safe. Eight binding port invariants (never `scrollLeft`; edge test synchronous from
layout gaps; no smooth `scrollIntoView` for spring-back; never restart a spring-back in flight; no spacer children; suspend
`scroll-snap-type` once per gesture; never change measurement and timing in the same revision; no pixel threshold below the device
quantum) are in the research doc section 6, each the generalisation of a named bench regression.

**Landed 2026-09-25 (round 8; `188e50b`, `c59dc8b`, `51a96a6`) as mode B3.** The pointer-drag path keeps owning `scrollBy`; a
separate wheel/trackpad bridge, `use_carousel_wheel_bounce`, never calls `scrollBy`/`scrollLeft`/`preventDefault` or touches
`scroll-snap-type`, and expresses overdrag only as a transform on `CarouselContent`. Both write `style.transform` through `eval`
outside the vdom (hydration parity unaffected); reduced motion disables the bounce; invariant 4 uses a token counter. Tests were
red first (7 of 8 failed pre-port); afterwards `playwright/carousel.spec.ts` 65/65 and the APG oracle 11/11. Methodology note:
`page.mouse.wheel(0, deltaY)` does not scroll a horizontal scroller in headless Chromium, `deltaX` does.

**Two defects the owner's hardware found (addendum 2026-09-25, lane `carousel-rb-clip`).** (1) `CarouselContent` does not
self-clip: a `transform` moves the box and its clip region together, so an overdrag pushed the whole track ~750 px past the
carousel. Fixed with a stationary `div[data-slot="carousel-viewport"]` wrapper (`overflow: clip`, not `hidden`, which would make
the paging path's `scrollIntoView` scroll the wrapper). (2) The wheel curve was mis-transcribed: `L*x/(x+L)` with `L=d/c` is not
Apple's `x*c*d/(d+c*x)` (a factor of `c` apart), so it was about twice as sensitive and deep; fixed in
`CAROUSEL_WHEEL_BOUNCE_JS` only (the drag curve was untouched). Both are recorded in the research doc section 9.

**Superseded 2026-09-28 by mode D** (`863f8ac`). The owner benched approaches D and E on real hardware (bench revs 33-40, Firefox
156, MacBook trackpad) and chose D, the "platform-aware band", which replaces B3's wheel path in `primitives/src/carousel.rs`
(`CAROUSEL_WHEEL_BAND_JS`); the owner's tuned knob values became named constants. Approved differences from the bench: axis gating
(vertical page scroll over a horizontal carousel draws no band), notched wheels release after 140 ms of silence, vertical
orientation and RTL, no band on the seamless virtual loop or under reduced motion, telemetry only behind
`localStorage["dx-carousel-debug"]="1"`. Verified by 21 rewritten tests in `carousel.spec.ts` (13 fail on B3) and replays of 9 of
the owner's real recorded bursts plus 2 synthetic ones (`playwright/carousel-wheel-replay.spec.ts`). The pointer-drag rubber-band
is unchanged. **Open:** Safari (and possibly Firefox on some platforms) may bounce a scroll container at rest at its edge,
stacking with D's band (double bounce); it needs a real-device check (see the carousel's docs.md).

*Trimmed in row 266 (repo cleanup). The full dated narrative of this row is in git: `git show 3c584cc2:dev-docs/backlog/done/102-port-the-carousel-edge-rubber-band-into.md`.*
