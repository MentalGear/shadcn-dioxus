---
id: 493
title: "Timing cases run alone, on a quiet machine: the @timing tag and the timing pass"
status: "MERGE-READY on lane/main-thread-timing. Owner decision 2026-11-01 (the general rule); the evidence…"
size: "M"
area: playwright
updated: 2026-11-01
---

# Row 493: Timing cases run alone, on a quiet machine

## Item

A test that asserts how long the main thread took (a long task, an interaction, a per-frame cost) measures the machine as much as the code. The deploy's release suite ran them beside two other workers' browsers and the other chunks' files, and two consecutive runs on `c16027cc` were each red on one different main-thread case. Owner decision 2026-11-01: every timing case runs only when the machine is almost idle.

## Status / trigger

MERGE-READY on `lane/main-thread-timing`. Owner decision 2026-11-01 (the general rule); the evidence below is from the two reds.

## Size

M

## Detail

**Construction.** One marker, one pass, one gate.
- `playwright/timing.ts` exports `TIMING_TAG` (`@timing`); every case that measures time is `test(title, { tag: TIMING_TAG }, ...)`: the three `main-thread.spec.ts` templates (the whole file, 63 cases), `virtual_list.spec.ts` and `message_scroller.spec.ts` (one per-frame-cost case each), `message-scroller-timing.spec.ts`.
- `playwright/tools/run-suite.sh` runs the chunks with `--grep-invert @timing`, then `tools/timing-pass.sh` once: `--grep @timing --workers=1`, under the same `pw.lock`, after `tools/wait-quiet.sh` (no build slot held or queued, CPUs at most 15 % busy for three 5 s windows; `TIMING_QUIET_TIMEOUT`, default 1800 s, then exit 1 with what it saw). The report is `OUT/chunk-timing.json`; `scripts/deploy-preview.sh`'s verdict counts a missing one as red, so a timing case is never skipped. `test:changed` does the same for the timing cases it selects. A debug build prints that its timing cases measure nothing and does not wait.
- `scripts/check-timing-tags.sh` (gate, with a self-test) fails a `test(` whose body names `assertResponsive`, `withGesture`, `perFrameCost`, `frameCostMeter` or `buildProfile()` without the tag.
- The rule's home is `dev-docs/dev-loop.md`, "Running tests".

**Evidence (release builds of `c16027cc` and its parent merge `2e171ed2`, 4-core box, nothing else running).**
- "dialog: close is responsive" is not a code regression and not (only) load. Run alone with the original spec it fails 6/40 on `c16027cc` and 7/40 on `2e171ed2` (before row 487's Escape change), the same rate. Cause: the spec presses Escape as soon as `toBeVisible()` holds, while the open's animation frames are still running; those frames are 50-90 ms each under headless software rendering, and the Long Tasks API charges a frame over 50 ms to the Escape window (`startTime` ~1.3-1.5 s of page age, no interaction, matching the deploy red). With 600 ms of rest before the gesture, or `settleAnimations` (every finite animation finished, two frames), the same flow is 0/40 and 0/60 on both builds. Alone with the fix: 0/40.
- Load makes it worse and moves it around. The whole `main-thread.spec.ts` at 3 workers (the original spec): 5, 1 and 2 reds of 60 in three runs, on different cases (alert_dialog close, drawer close, popover_modal open, combobox open, ...); the new spec at 3 workers: 3, 0 and 4 reds of 60, again different cases. Run through the timing pass (one worker, quiet machine) the 63 cases were green 1 of 3 times; the other two passes had one red each, both a first input restyling the page (below).
- Remaining solo reds (not this row's to fix): "combobox: open is responsive" 51 and 69 ms in two of three passes (keydown processing 56 and 72 ms), "date_picker: open is responsive" 52 ms once (the first `pointerover` 72 ms). This is row 490's cause: the first key or pointer input flips `html[data-dx-modality]` and restyles ~930 elements for 18-27 ms, on top of the open itself. A fix of that row, not a threshold change, turns them green.

**Follow-up.** The chunk balance (`playwright/durations.json`) still weights `main-thread.spec.ts`, `virtual_list.spec.ts` and `message_scroller.spec.ts` with their timing cases, which now run in the last pass; refresh the table after the next full run (`durations.mjs chunk-*.json --write`, the `chunk-timing.json` is part of the glob).
