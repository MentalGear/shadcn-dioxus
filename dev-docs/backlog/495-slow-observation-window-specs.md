---
id: 495
title: "The two ~32 s-per-test specs wait on signals, not on fixed observation windows"
status: "APPROVED (owner 2026-11-05: item 1 of the build and test-suite review). Lane slow-specs-495."
size: "S-M"
area: playwright
updated: 2026-11-05
---

# Row 495: The two ~32 s-per-test specs wait on signals, not on fixed observation windows

## Item

`playwright/layout-stability.spec.ts` (19 tests, 611 s) and `playwright/charts-hydration-stability.spec.ts` (8 tests,
254 s) spend about 32 s per test, against 1-3 s for the rest of the suite (`playwright/durations.json`, debug build, 2
workers). Find where the time goes (fixed observation windows, `waitForTimeout`, polling loops, repeated reloads) and
replace each wait with the signal it stands for (hydration done, fonts and images settled, a `PerformanceObserver`
buffer read, `page.clock`, a stable-frame count), without dropping any assertion or weakening what a case proves.

## Status / trigger

APPROVED (owner 2026-11-05: item 1 of the build and test-suite review). Lane `slow-specs-495`.

## Size

S-M

## Detail

**Where the numbers come from.** The suite's per-file durations (`playwright/durations.json`, measured 2026-10-21 on a
debug build with 2 workers; 13,255 s in all). These two files are 865 s, 6.5% of the suite, for 27 tests.

**Proof.** Per case: the old and the new duration on the same build, and the case still red on a build that has the
defect it guards (re-create the defect locally, or show the assertion fires on a forced shift), so the shorter wait
still observes what it must. Update their entries in `playwright/durations.json`.
