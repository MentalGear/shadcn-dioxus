---
id: 112
title: "Carousel tests that sample a live animation across an async gap -- both fixed; plus the B3 wheel release…"
status: "CLOSED 2026-10-19 (bookkeeping: status already said done). Found 2026-09-26 while verifying the…"
size: "done (instance 2 fixed d44a12d; the…"
area: theme
updated: 2026-10-19
---

# Row 112: Carousel tests that sample a live animation across an async gap -- both fixed; plus the B3 wheel release…

## Item

Carousel tests that sample a live animation across an async gap -- both fixed; plus the B3 wheel release treating a flat delta stream as spent

## Status / trigger

CLOSED 2026-10-19 (bookkeeping: status already said done). Found 2026-09-26 while verifying the carousel shadcn-parity merge (`22d4c95`). Instance 1, FIXED: `carousel.spec.ts` "a sustained constant wheel stream at the start boundary..." failed 15/15 at `22d4c95` (release SSG, `--workers=1 --repeat-each=15`) and passed 15/15 at `c2dd9d7` (debug SSG) -- an in-page MutationObserver trace showed the same release/spring-back sawtooth at both revisions; the test read the transform across CDP round trips while a spring-back was in flight. Fixed by dispatching both bursts and both reads in one synchronous in-page turn. Instance 2, FIXED 2026-09-26 (`d44a12d`; was OPEN): `carousel.spec.ts:687` "ArrowDown/ArrowUp page the vertical carousel..." fails ~1/5 under `--workers=2`, 5/5 green under `--workers=1`; not root-caused -- check it against row 110's smooth-scroll pre-scroll class first

## Size

done (instance 2 fixed `d44a12d`; the wheel-release note is moot since mode D, `863f8ac`)

## Detail

**2026-09-28:** the `wheelSpent()` product note is moot -- the B3 wheel path it describes was removed when mode D replaced it (`863f8ac`, row 102). **2026-09-26 (audit batch): instance 2 FIXED** in `d44a12d` -- not row 110's class (`.focus()` and raw key presses never pre-scroll); `expectSnappedToBoundary`'s `toPass` retry paid a CDP round trip per attempt inside a fixed 3s budget. The whole poll now runs in-page on rAF (all 25 call sites). 20/20 at `--repeat-each=20 --workers=2`; the original ~1/5 rate did not reproduce on demand in this session. The `wheelSpent()` product note stands, pending the owner's approach-E decision. [Superseded by the 2026-09-28 note above: the owner chose mode D, which removed that wheel path.] **Product note (not a test issue):** `CAROUSEL_WHEEL_BOUNCE_JS`'s `wheelSpent()` plateau branch accepts a delta merely *equal* to the previous one, so any flat stream (a notched wheel, or a perfectly steady push) is classified as spent after two events and `bounceHome()` starts mid-push. Rarely visible with a real hand's jittery deltas. Moot if the non-loop edges move to bench approach E (the browser's own snap releases at lift, no release heuristic at all), which is the owner's current preference; otherwise require a strict decline. **Class:** two carousel tests have now asserted on an animated value read across await gaps -- the construction is to sample in-page (one synchronous turn, or an in-page rAF/MutationObserver trace), never via separate round trips.
