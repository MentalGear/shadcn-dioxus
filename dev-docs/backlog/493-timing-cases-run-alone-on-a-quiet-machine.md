---
id: 493
title: "Timing cases run alone, on a quiet machine: the @timing tag and the timing pass"
status: "The construction is MERGED in PR #160 (8edeb5f1) and DEPLOYED in 1ea333fa: the tag, the pass, the…"
size: "M"
area: playwright
updated: 2026-11-02
---

# Row 493: Timing cases run alone, on a quiet machine

## Item

A test that asserts how long the main thread took (a long task, an interaction, a per-frame cost) measures the machine as much as the code. The deploy's release suite ran them beside two other workers' browsers and the other chunks' files, and two consecutive runs on `c16027cc` were each red on one different main-thread case. Owner decision 2026-11-01: every timing case runs only when the machine is almost idle.

## Status / trigger

The construction is MERGED in PR #160 (`8edeb5f1`) and DEPLOYED in `1ea333fa`: the tag, the pass, the gate. Deactivated by owner decision 2026-11-02 until the Rust port is done: `playwright/tools/timing-switch.sh` is off and [row 494](./494-re-enable-the-timing-cases-after-the-rust-port.md) re-enables it. Owner decision 2026-11-01 (the general rule); the evidence below is from the two reds.

## Size

M

## Detail

**Construction.** One marker, one pass, one gate.
- `playwright/timing.ts` exports `TIMING_TAG` (`@timing`); every case that measures time is `test(title, { tag: TIMING_TAG }, ...)`: the three `main-thread.spec.ts` templates (the whole file, 63 cases), `virtual_list.spec.ts` and `message_scroller.spec.ts` (one per-frame-cost case each), `message-scroller-timing.spec.ts`.
- `playwright/tools/run-suite.sh` runs the chunks with `--grep-invert @timing`, then `tools/timing-pass.sh` once: `--grep @timing --workers=1`, under the same `pw.lock`, after `tools/wait-quiet.sh` (no build slot held or queued, CPUs at most 15 % busy for three 5 s windows; `TIMING_QUIET_TIMEOUT`, default 1800 s, then exit 1 with what it saw). The report is `OUT/chunk-timing.json`; `scripts/deploy-preview.sh`'s verdict counts a missing one as red, so a timing case is never skipped. `test:changed` does the same for the timing cases it selects. A debug build prints that its timing cases measure nothing and does not wait.
- `scripts/check-timing-tags.sh` (gate, with a self-test) fails a `test(` whose body names `assertResponsive`, `withGesture`, `perFrameCost`, `frameCostMeter` or `buildProfile()`, starts its own `requestAnimationFrame` loop in an `addInitScript`, drives a fling or flick with `page.mouse` and no clock of its own, or asserts an upper bound on a measured duration, without the tag (the last needs `timing-bound-ok: <reason>` to stand untagged).
- The rule's home is `dev-docs/dev-loop.md`, "Running tests".

**Evidence (release builds of `c16027cc` and its parent merge `2e171ed2`, 4-core box, nothing else running).**
- "dialog: close is responsive" is not a code regression and not (only) load. Run alone with the original spec it fails 6/40 on `c16027cc` and 7/40 on `2e171ed2` (before row 487's Escape change), the same rate. Cause: the spec presses Escape as soon as `toBeVisible()` holds, while the open's animation frames are still running; those frames are 50-90 ms each under headless software rendering, and the Long Tasks API charges a frame over 50 ms to the Escape window (`startTime` ~1.3-1.5 s of page age, no interaction, matching the deploy red). With 600 ms of rest before the gesture, or `settleAnimations` (every finite animation finished, two frames), the same flow is 0/40 and 0/60 on both builds. Alone with the fix: 0/40.
- Load makes it worse and moves it around. The whole `main-thread.spec.ts` at 3 workers (the original spec): 5, 1 and 2 reds of 60 in three runs, on different cases (alert_dialog close, drawer close, popover_modal open, combobox open, ...); the new spec at 3 workers: 3, 0 and 4 reds of 60, again different cases. Run through the timing pass (one worker, quiet machine) the 63 cases were green 1 of 3 times; the other two passes had one red each, both a first input restyling the page (below).
- Remaining solo reds (not this row's to fix): "combobox: open is responsive" 51 and 69 ms in two of three passes (keydown processing 56 and 72 ms), "date_picker: open is responsive" 52 ms once (the first `pointerover` 72 ms). This is row 490's cause: the first key or pointer input flips `html[data-dx-modality]` and restyles ~930 elements for 18-27 ms, on top of the open itself. A fix of that row, not a threshold change, turns them green.

**Verdicts that read the clock without a harness (the deploy of `8edeb5f1`, 2026-11-02).** Two untagged specs went red once in 4,100 tests of the release suite and were green on the deploy before it (`7d50e068`); neither product file nor spec changed between the two trees (`avatar.rs`, `carousel/`, the avatar and carousel specs are byte-identical; the diff is the menu/dismiss/date-segment work). Both are the same class: a verdict that depends on the machine's pacing, in a spec that does not say so.
- `carousel-drag-free.spec.ts` "a fling coasts past a slide boundary": the carousel reads the release velocity from the `timeStamp`s of the pointer events (`primitives/src/carousel/drag.rs`, over the last 100 ms, none if the release is 80 ms after the last move), and the browser stamps a CDP-dispatched event with the moment it received it, so the velocity was the spacing of Playwright's round trips. Measured on the release build: with 120 ms of stall between the calls the original fling fails 6/6 and the same fling on a controlled timeline passes 6/6. The deploy's `received 180` is a drag of 140 px plus a short coast. Fix: `playwright/pointer-timeline.ts` (`dragOnTimeline`) sends `Input.dispatchMouseEvent` with a `timestamp` per event, so `event.timeStamp` spacing is the test's; the slow drag holds still for a virtual 250 ms instead of waiting; the coast is judged against the scroll position recorded at `pointerup`.
- `avatar-loading.spec.ts` "no frame is blank": the sampler started in an `addInitScript` and read the avatar while the HTML parser was still inside it (the `<img>` is in the document, its fallback sibling is not parsed yet): a state no hydrated page has. A probe on the release build caught it (`t` 227 ms, `data-hydrated` unset, `fb: false`, next state 18 ms later `fb: true`). Rate on `8edeb5f1`: 1 in 20 (original spec) and 1 in 140 (probe) at 3 workers of the same spec, 0 in 60 on another pass, 0 in 20 alone, 0 in 20 beside a 3-worker suite load; on `7d50e068` 0 in 20, 0 in 20 and 0 in 60 (the same distribution, a rare event on both trees). Fix: `playwright/frame-recorder.ts` (`installFrameRecorder`), whose sampler gets `parsed(el)` and returns `null` for a subtree the parser has not finished; the avatar samples after every DOM-changing task too (`perTask`), so the verdict does not depend on which states the frames land on. Controls in the spec: a hand-removed fallback is flagged, a half-parsed subtree is not judged. The same raw loop is gone from `docs-nav-sort`, `first-pose` and `message_scroller`; `avatar-loading`'s `waited < 4000` upper bound is dropped (the 10 s wait already proves the fallback appears).
- Also found by the widened gate: `toast-swipe` "a short fast flick" (the same pacing; now on the timeline), `drawer` "a real-mouse flick" (a 35 % drag, not a flick; renamed), `hover-card` and `tooltip` (`elapsed(...) < 600/300`: bounds against the configured delay, each carries `timing-bound-ok`), and four `carousel-virtual-wheel` cases that bound a frame-sampled stall at 200 ms (tagged `@timing`, so dormant until row 494 re-enables the pass).
- Not covered, and open: the trackpad-shaped wheel replays (`carousel-virtual-wheel`, `carousel-edge-wheel`) pace `page.mouse.wheel` calls with `waitForTimeout`; `Input.dispatchMouseEvent` takes a `timestamp` for `mouseWheel` too, so they can move to the same timeline when one of them flakes.

**Follow-up.** The chunk balance (`playwright/durations.json`) still weights `main-thread.spec.ts`, `virtual_list.spec.ts` and `message_scroller.spec.ts` with their timing cases, which now run in the last pass; refresh the table after the next full run (`durations.mjs chunk-*.json --write`, the `chunk-timing.json` is part of the glob).
