---
id: 402
title: "Release timing specs compare same-run measurements, not absolute milliseconds"
status: "Done on lane/timing-relative (numbers below). Trigger for reopening: a release timing spec that…"
size: "S"
area: tests
updated: 2026-10-24
---

# Row 402: Release timing specs compare same-run measurements, not absolute milliseconds

## Item

`message-scroller-timing.spec.ts` asserted `msPerFrame < 12` and measured 13.24 ms in the batch-16 release suite (another lane was building), and the same test failed in an earlier batch's suite; it passes on a quiet box. The class: a release-only timing spec that compares a measurement with a constant number of milliseconds goes red when the shared 4-core box is loaded, because load scales the measurement and not the constant.

## Status / trigger

Done on `lane/timing-relative` (numbers below). Trigger for reopening: a release timing spec that goes red on load although the component is unchanged.

## Size

S

## Detail

Construction: a timing spec measures the component's path AND a reference path on the same page in the same run, interleaved, and asserts the ratio (`playwright/frame-cost.ts`: `frameCostMeter().perFrameCost`, the CDP `TaskDuration` per frame; the least disturbed of several rounds per path; `calibrationMs` reports the box's speed next to each result).

- `message-scroller-timing.spec.ts`: chunked append (what `MessageScrollerRows` renders) against the same ten rows per frame appended as direct children (one skippable element per row); asserts chunked / per-row < 0.85. Every appended node is removed between measurements.
- `message_scroller.spec.ts` scroll test: the 2,000-row scroll with the chunk's `content-visibility` against the same page with a stylesheet that moves the skipping to each row; asserts < 0.7.
- `virtual_list.spec.ts` content-visibility scroll: the same construction on the 1,000-row list; asserts < 0.7.
- `oracle/tier2-html/main-thread.spec.ts` (through `main-thread.ts`) keeps its absolute 50 ms (Long Tasks definition) and 200 ms (interaction) budgets: the promise is the platform's number, so it carries `timing-absolute-ok` and now reports `calibrationMs` in every annotation. `scroll-main-thread.spec.ts` asserts counts (timer fires, layouts), not durations.

Gate: `scripts/check-timing-relative.sh` (in `run-gates.sh`, with a 13-case mutation self-test) fails a `expect(<duration identifier>).toBeLessThan(<number>)` or `toBeLessThanOrEqual(<number>)` in a spec that gates itself on `buildProfile() !== "release"` or drives `main-thread.ts`, unless `timing-absolute-ok: <reason>` sits on the line or the line above. It reads identifier names (`ms`, `msPerFrame`, `maxTaskMs`, `elapsed`, `duration`, `latency`, `cost`, `PerFrame`), so a duration in a neutrally named variable or a constant defined elsewhere is not seen; the old three assertions all turn it red.

Evidence (release build of the batch-16 tree, headless Chromium, `PW_BUILD_PROFILE=release`; ms of main-thread task time per frame):

| spec | old absolute | new chunked / per row, quiet | mutant (chunked path made per-row) |
|---|---|---|---|
| message-scroller-timing | 9.5-10.2 vs `< 12` | 8.9-9.6 / 15.7-15.9, ratio 0.56-0.65 (limit 0.85) | ratio 1.03, 0.98 |
| message_scroller scroll | 2.5-3.5 vs `< 8` | 2.3-3.4 / 8.6-10.5, ratio 0.22-0.39 (limit 0.7) | ratio 0.99, 0.94 |
| virtual_list content-visibility | 0.5-0.8 vs `< 4` | 0.6-0.7 / 2.4-2.7, ratio 0.23-0.25 (limit 0.7) | ratio 1.12, 0.98 |

A slower machine is simulated with CDP `Emulation.setCPUThrottlingRate` (the page's calibration time doubles and triples): at 2x the old message-scroller-timing measured 18.6 ms and failed `< 12`, at 3x it measured 30.1 ms and the old scroll spec measured 8.3 ms and failed `< 8`; the new specs passed at 2x and 3x with the same ratios (0.58 / 0.27 / 0.19 at 2x, 0.55 / 0.35 / 0.21 at 3x). Plain CPU hogs (four `nice -20` busy loops or a cache-thrashing loop) do not move `TaskDuration` and neither assertion style notices them; the failures seen on the shared box came from other lanes' memory-bound builds, which the throttle reproduces as a uniform slowdown. The ratio specs run with `test.setTimeout(5 * 60_000)` because a loaded box stretches the interleaved rounds' wall time (the 90 s default timed one out).
