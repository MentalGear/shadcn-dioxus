---
id: 104
title: "Instrumentation that keys off the wrong event, or records a threshold after it is reset, lies exactly where…"
status: "CLOSED 2026-10-19 (main-loop review: the fix or lesson is in place, see Size/Detail). Two telemetry…"
size: "Done for the bench; worth remembering…"
area: misc
updated: 2026-10-19
---

# Row 104: Instrumentation that keys off the wrong event, or records a threshold after it is reset, lies exactly where…

## Item

Instrumentation that keys off the wrong event, or records a threshold after it is reset, lies exactly where the bug is

## Status / trigger

**CLOSED 2026-10-19 (main-loop review: the fix or lesson is in place, see Size/Detail).** Two telemetry defects found 2026-09-22 (bench v22) while chasing a B2 glitch that the data appeared to show nothing about

## Size

Done for the bench; worth remembering as a class

## Detail

(a) Burst tracking keyed off **pad installation** rather than off the gesture, so a 49-second driving session recorded exactly **one** B2 burst — blind precisely in the region under investigation. (b) `releaseThreshold` was recorded **after** the peak it derives from had been zeroed, so every burst reported a flat `1.6` regardless of what was actually used. Both were fixed and the session re-driven. The general shape: instrumentation is code, it can be wrong, and a quiet dataset is evidence of nothing until the recorder itself has been checked against a case whose answer is already known. Related and separately instructive: the owner flagged *"1.6px floor might be wrong"* before any data did — it sat **below** the device's own 1px quantum, so it could never fire. It is now `deviceMinDelta * 2`, learned from the stream, with 1.6 as a fallback only until the device speaks.
