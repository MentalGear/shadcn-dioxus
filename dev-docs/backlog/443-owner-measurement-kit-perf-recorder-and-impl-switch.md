---
id: 443
title: "The owner's measurement kit for the row 434 ports: __dxPerf, bridge delay and spin, and the ?dx-impl legacy switch"
status: "BACKLOG. Filed 2026-10-26 from the row 434 step 3 adversarial board. The bridge-delay and spin…"
size: "M"
area: tooling
updated: 2026-10-26
---

# Row 443: the owner's measurement kit

## Item

The three feel-critical ports of row 434 (the message scroller W8, the carousel wheel band C5, the carousel macros and their users) depend on hardware CI does not have, and the release suite cannot see them: `oracle/tier2-html/main-thread.spec.ts` has no carousel, message-scroller or streaming subject, and its 50 ms long-task and 200 ms interaction bounds are about 20 to 100 times too coarse to see a per-frame regression or a two-frame lag ([ux-a11y-perf finding 11](../research/board-2026-10-26/ux-a11y-perf.md)). The owner measures each port, one at a time, on his own devices, **sized to the devices he has** (one person cannot run iOS 26 and 27, WebView2, Android and a 120 Hz display; the owner's device list is an input this row waits for). **Step 0 is one real desktop latency distribution**, taken before any port: it sets the next-frame budget of the deadline table, the desktop-cadence delay and the pass baseline, and none of it depends on a port ([root doc section 5.6](../research/sync-js-root-2026-10-26.md); round 3, coherence). This row builds the kit once so every port is measured the same way, and so the old path can run next to the new one for one deploy.

## Status / trigger

BACKLOG. Filed 2026-10-26 from the row 434 step 3 adversarial board. The bridge-delay and spin hooks need lane 3.0's transport seam; the `?dx-impl` switch plumbing and the freeze of every legacy path are lane 3.FZ's (phase 0 of the row 434 plan), needed before the first of 3.CA, 3.MS and 3.WB (under the default scope B of row 434, only when a measured port is scheduled). Everything is behind a debug flag (`?dx-perf=1` or `localStorage['dx-perf']`, as `dx-carousel-debug` already does) and ships to no user.

## Size

M

## Detail

Estimate: about 10 agent-hours, inferred, **counted once** and split: the **stats half** (the in-page recorder's statistics and the latency-distribution step, about 3 h) is phase 0 of the row 434 plan; the **rest** (about 7 h: the trace helper, the delay and spin hooks, the per-port relation specs) goes with the measured group. The legacy freeze and the `?dx-impl` plumbing are lane 3.FZ's (about 10-12 h), not part of this kit.

The full protocol, per-port scenarios and pass criteria are in [`../research/board-2026-10-26/ux-a11y-perf.md`](../research/board-2026-10-26/ux-a11y-perf.md) section 3; the summary table is in [`../research/sync-js-root-2026-10-26.md`](../research/sync-js-root-2026-10-26.md) section 5.6.

- **`window.__dxPerf`**, an in-page recorder with `start(label)`/`stop()`/`dump()` returning JSON: PerformanceObservers for `longtask`, `long-animation-frame` (Chromium, script attribution), `event` (interactionId, processing and presentation times) and `layout-shift` (`hadRecentInput` false, for anchoring jumps); a rAF loop recording the frame-delta histogram (a dropped frame is a delta over 1.5 times the display's refresh); the scroll position and a caller-supplied probe value per frame.
- **`window.__dxBridgeStats`**: messages and bytes each way, and a round-trip histogram (send stamp, Rust handler, applied stamp). That one number says "frames of lateness" on any device, including a phone, and is about 0 on web by construction.
- **Delay and cost emulation:** `?dx-bridge-delay=N` (the plan's bridge-latency mode, owned by the transport seam) and `?dx-bridge-spin=ms`, which burns 1 to 3 ms per bridge message and per vdom event (it models the blocking request per Dioxus event on desktop). With both, the owner can feel the desktop cost on a Mac trackpad in Chrome or Safari without a desktop build. It is a bridge-latency emulation, not a desktop model (row 442).
- **A Chrome trace helper:** a CDP `Tracing.start` wrapper with `devtools.timeline`, `disabled-by-default-devtools.timeline.frame`, `input.scrolling`, `latencyInfo` and `benchmark`, saving `.json.gz` into `research/measure/<port>/<arm>-<device>.json.gz`.
- **The legacy switch `?dx-impl=js|rust`** (also a `localStorage` key), for W8 and C5 only (C1 inherits its users' arm). Construction: freeze first, port second (lane 3.FZ moves the legacy JavaScript out of Rust strings into `primitives/src/ts/legacy/*.ts`, wrapped and behaviour-neutral, for the message scroller and the whole carousel, C2 to C5 and the macros); a non-default cargo feature `legacy-impl` on `dioxus-primitives`, enabled only by the preview crate (`check-primitives-features` lists it; installed apps never enable it and no `component.json` names the files); the arm is chosen once after hydration in a `use_effect`, never at render, with markup identical across arms so `hydration-parity` and SSR stay green, and changes on reload; Playwright runs the legacy arm only for the affected specs (`test:changed`), with the budget increase in `playwright/durations.json` and its reason; each legacy file is a frozen literal with its row, removed when the owner's `research/measure/<port>/verdict.json` (`{"verdict": "pass" | "fail", "devices": [...], "traces": [...]}`) says `pass` (planned: the gate fails when a `pass` verdict exists and the legacy file is still tracked; a `fail` deletes nothing and is C5's admission witness; no calendar date); `carousel/js_macros.rs` moves into `ts/legacy` and is deleted by the legacy removal lane. The library's default is Rust, and both arms are never shipped to users.
- **Method:** a release build, arms interleaved ABAB, at least 10 gestures per arm, median and p95 with `calibrationMs` (`playwright/frame-cost.ts`), the same commit serving both arms so build noise cancels.
- **In the existing suite:** carousel (wheel, drag), message-scroller (stream) and toast (stack of 5) subjects in `main-thread.spec.ts`; a relation spec per port using `frameCostMeter` on the two arms (Rust arm per-frame task time at most 1.1 times the legacy arm, interleaved); `scroll-main-thread` test 3 extended to `window`.
- **Per-port pass criteria** (from the board): W8 follow error at most 1 px over the JS arm, prepend drift 0.5 px, no "fight frames", no long tasks, per-frame task time at most 1.1 times; C5 p95 added lag at most 1 frame and p99 at most 2, zero lost or reordered events, identical release kind, no stacking on WKWebView's native bounce (a `fail` verdict admits C5's `wheelBand` named behaviour, row 434 and root doc section 2.2); carousel macros and users within one frame and 2 px of the JS arm.
