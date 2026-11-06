---
id: 495
title: "The two ~32 s-per-test specs wait on signals, not on fixed observation windows"
status: "Lane slow-specs-495: done, awaiting merge. Both specs wait on signals through…"
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

Lane `slow-specs-495`: done, awaiting merge. Both specs wait on signals through `playwright/stability-window.ts`.
Same debug build (`lane-ssg`), 2 workers, serial runs under `pw.lock`, summed test seconds (box shared with other lanes,
so only same-session pairs compare): `layout-stability` 766 s -> 627 s (wall 6.5 -> 5.3 min), `charts-hydration-stability`
273 s -> 191 s (wall 2.5 -> 1.7 min); per case e.g. `/` light 306 -> 246 s, `/docs/` light 72 -> 60 s, a charts tab
(7 widths) 21 -> 16 s, the 12 progressive-paint cases 4.4 -> 3.9 s each. The `durations.json` entries are the new numbers.

## Size

S-M

## Detail

**Where the numbers come from.** The suite's per-file durations (`playwright/durations.json`, measured 2026-10-21 on a
debug build with 2 workers; 13,255 s in all). These two files are 865 s, 6.5% of the suite, for 27 tests.

**Proof.** Per case: the old and the new duration on the same build, and the case still red on a build that has the
defect it guards (re-create the defect locally, or show the assertion fires on a forced shift), so the shorter wait
still observes what it must. Update their entries in `playwright/durations.json`.

**Where the time went (measured).** Each "test" is a group of 7-64 widths, each a fresh navigation (prerendered paint,
held wasm, debug hydration, recording). Per width the fixed costs were the 1.5 s `waitForTimeout` no-input window (about
1 s of the ~4.8 s a width took on `/`, ~1.1 s of ~3 s on a charts tab), the 200 ms + 50 ms polling of the paint-hold in
the wasm route, and in the progressive-paint cases a fixed 1 s after load. What remains is navigation, wasm download and
hydration of a debug build, which each width needs: the claim is about the prerendered-to-hydrated step at that width.

**The replacement.** `holdWasmUntilPainted`: a buffered `PerformanceObserver` on `paint` plus three animation frames
(no polling, no sleep). `settleUntilQuiet`: fonts loaded, images loaded, no finite animation or transition running, no
page timer pending that could fire inside the window (`trackPendingTimers` wraps `setTimeout`), and 30 consecutive
frames with no recorded box event and no DOM mutation; ceiling 1.5 s, so a page that never goes quiet is judged on the
window the specs used to observe. The pending-timer count is what keeps a timer-driven shift late in the window visible.

**Guard check.** A shift injected through `addInitScript` (9 px `padding-top` on `#main-content` set by a page timer
after `data-hydrated`) is red in both specs (`/docs/` light and `/charts/` light at 1280 px) at 100 ms, 400 ms and 1200 ms
after hydration. With the first version (frame count only) the 1200 ms shift passed, which is why the timer count was
added; the injected specs were temporary and are removed.
