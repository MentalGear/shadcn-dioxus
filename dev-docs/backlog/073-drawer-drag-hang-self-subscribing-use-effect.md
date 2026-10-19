---
id: 73
title: "Drawer drag hang — self-subscribing use_effect wedged the main thread on drag"
status: "Found in integration round 2 (2026-09-18), fixed round 3 same day"
size: "S"
area: overlays
updated: 2026-09-18
---

# Row 73: Drawer drag hang — self-subscribing use_effect wedged the main thread on drag

## Item

Drawer drag hang — self-subscribing `use_effect` wedged the main thread on drag

## Status / trigger

Found in integration round 2 (2026-09-18), fixed round 3 same day

## Size

S

## Detail

Round 2's live integration run found 2 of 9 `drawer.spec.ts` tests hanging the 5-minute ceiling at the raw `page.mouse.down()` call, after the drawer opened and geometry was measured ("a long, slow drag past the dismiss threshold closes the drawer", "a short, slow drag snaps back without closing the drawer") — no panic, no console output; a follow-up `page.evaluate` hung too, proving the tab's whole main thread was wedged, not just the test. Root cause: `DrawerContent`'s drag-continuation `use_effect` read `raw_offset()` with the tracked call syntax (subscribing the running effect to that signal) and then, in the same execution, wrote a new value to that same signal via `.set()` — writing to a signal an effect just subscribed to reliably re-triggers it, and nothing on this path ever broke the cycle, so from the first `pointerdown` the effect re-invoked itself synchronously, forever. Fixed (commit `b7037cb`, cherry-picked from `896e603`): `.peek()` instead of the tracked read for both of the effect's self-referential reads of `raw_offset` (the release-decision read and the accumulation read), matching the read-through-`.peek()`-write-through-`.set()` discipline `lib.rs`'s `use_animated_open` already documents and uses for its own generation counter, for exactly this reason. Verified live, own server: before, `mouse.down()`/a follow-up `evaluate` both hung 5s+ with no recovery; after, both return immediately and the real `drawer.spec.ts` run is 7/7, including both drag tests (3.0s, 1.4s). **House-wide review rule this implies:** any `use_effect` that reads a signal with tracked syntax and then writes that same signal in the same run will re-trigger itself forever unless something in the same run breaks the cycle — worth a standing review checklist line, not just a fix on this one file. **Closed 2026-09-18/19, batch 2, row 79:** that standing checklist line is now `scripts/check-self-subscribing-effects.sh` (`f11b98b`), proven against this exact bug by failing on `cdb073b` (this row's own pre-fix commit) and passing once `b7037cb` (this row's own fix) is applied. **Family note (2026-09-19, batch 3):** a second, independent instance of the same defect *shape* (an unconditional reactive cycle wedging the main thread, found live in a Playwright run) recurred in `DateRangePicker` — see row 83. Different construction there (a value-equality guard on the write side, not a `.peek()` read), because that cycle spans multiple effects through a callback, not one effect's own tracked read+write
