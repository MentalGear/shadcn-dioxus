# The residue, re-attacked: W8, C5 and C1 to Rust (row 434, step 3), 2026-10-26

Follow-up to [`sync-js-root-2026-10-26.md`](./sync-js-root-2026-10-26.md) (the "root doc"). Read-only, on `origin/queue/batch-28` @ `d10d34b5`. Nothing was built or run. Evidence labels as in the root doc: **V** = read in code or crate source, **W** = web / web-features 3.44.0, **I** = inferred.

**Revised 2026-10-26 after the adversarial board** ([`SYNTHESIS.md`](./board-2026-10-26/SYNTHESIS.md)). W8 and C5 changed (below); C1 held. **Every desktop and mobile claim here is "web-verified; desktop inferred"** until a `desktop-pass/<tree>.pass` stamp exists ([row 442](../backlog/442-desktop-cadence-harness-and-desktop-pass.md)); Chromium is the only engine installed in the lane container, so Firefox and WebKit claims carry the same label.

## Executive summary (for the owner)

**All three blocks leave bespoke JavaScript. W8 and C5 are measured ports: the owner measures each, one at a time, with the old path switchable for one deploy.**

- **The policy inconsistency was real, and it is fixed.** `overflow-anchor` reached Baseline newly available on 2026-09-14 (Chrome 56, Firefox 66, Safari 27; W). The root doc's policy is "Baseline newly available, with a feature-detected fallback that stays correct", so it qualifies **today**.
- **W8, the message scroller (1,065 lines): a Rust invariant, with `overflow-anchor` as the accelerator.**
  - The invariant lives in Rust, always on and independent of engine support: while `mode == following-bottom`, **the end sentinel intersects the viewport within one frame, or Rust scrolls to the end**. Its source of truth is bridge `observe` (IntersectionObserver on a 1 px end sentinel, resize on the content).
  - CSS anchoring makes that frame-exact where it works. It cannot be the guarantee: the scroll-anchoring spec selects **no anchor at scroll offset 0**, skips fully clipped nodes (one slipped frame is permanent until a Rust scroll), and cancels a queued adjustment on a computed `height`/`margin`/`padding`/`transform`/`top` change on the way up to the scroller.
  - The mode machine, opening position, visibility and scroll-intent logic are Rust, fed by a **page-side reducer** over bridge `listen`/`observe`: never by vdom `onscroll`/`onresize`, which on desktop is one blocking request per event.
- **C5, the carousel wheel band (477 lines): a Rust attempt, measured.**
  - It is passive, so lateness is lag, never a wrong decision. A pure Rust classifier; input is a **batched** wheel stream flushed **per event task** (not per rAF: trackpad wheel already arrives about once per frame, so rAF batching only adds up to a frame of latency); output is one CSS variable.
  - The owner measures it against the old path (`?dx-impl`), including a "Rust busy 30 ms" scenario. **If Rust loses, C5 becomes an allowlisted TypeScript module**; page-side wasm (C2) is no longer the exit.
  - The proof without a browser is a `cargo test` replay plus a property test under jitter, batching and delay; the committed recordings alone are weak evidence (section 3.3).
- **C1, the helper macros (65 lines):** confirmed to dissolve. `dxIsTrueEnd` becomes a Rust-published `data-dx-true-end-min/max` pair; `dxTrace` becomes Rust tracing. While the legacy arms live, the macros live in `ts/legacy` with them; `js_macros.rs` is deleted at the legacy-switch expiry deploy.
- **Result:** class (d) ends with no per-component script. The page-side code is the bridge, the transport seam and the closed registry of named behaviours (root doc sections 2.2 and 4).
- **Cost:** 3.MS about 20 agent-hours and 3.WB about 12 (inferred; the first version said 16 and 10), each its own batch and deploy, in the owner's order: 3.MS, then 3.WB, then the legacy removal.
- **Biggest risks:**
  - the quality of Safari 27's new scroll anchoring and Chromium's heuristics under token-rate streaming (the Rust invariant is the answer);
  - desktop latency for C5, which cannot be measured in this container (no gtk) and needs a machine with WebView2 or WKWebView.

---

## 1. Policy consistency (fixed)

Root doc §2.1 policy: *Baseline newly available across Chrome, Firefox and Safari; older engines must stay correct through a feature-detected fallback, not necessarily frame-perfect.* `overflow-anchor` (web-features `overflow-anchor`, W):
- Baseline low on 2026-09-14;
- Chrome 56, Firefox 66, Safari 27.

So it qualifies under the same rule as anchor positioning (Baseline low since 2026-01-13). The root doc's §4 wording ("once the policy admits it") contradicted §2.1. There was no hidden difference. The only real extra caveat is that Safari 27's implementation is weeks old, which the fallback design below absorbs. Root doc §3 (the W8 row), §4, §5.1 and §5.4 are now corrected.

The fallback is detected by `@supports (overflow-anchor: auto)` / `CSS.supports`. It cannot detect a *buggy* implementation, so the per-engine probe in §2.5 is part of the lane.

---

## 2. W8, the message scroller → a Rust invariant, `overflow-anchor` as the accelerator (+ `settle`, G)

### 2.1 What it does each frame today (V, `primitives/src/message_scroller.js`)

| function | trigger | same-frame write | line |
|---|---|---|---|
| follow bottom | content grows (`handleResize` / `handleContentChange`) while `mode === "following-bottom"` | `scrollToEnd("auto")` | `:840-848`, `:832` |
| prepend preservation | rows inserted above | `viewport.scrollTop += delta`, and *"a no-op where the browser already handled the prepend"* | `:756-776` |
| anchored turn hold | content below the anchored turn resizes | `reanchor()` resizes the tail spacer, so the browser does not clamp `scrollTop` and drop the turn | `:850-861`, `:579-588` |
| jump settle | jump to a message, re-aimed while neighbours render | scroll to element | `:621-710` |
| opening position | first content | poll per rAF until the viewport's sheet applies, transcript hidden meanwhile | `:711-755` |
| keep-rendered | live edge, focused row, selection ends | `data-keep-rendered` attributes | `:175-211` |
| visibility store | IntersectionObserver while Rust subscribes | `dioxus.send` only | `:213-311` |
| scroll intent | passive wheel, touch, keys (`:937-944`) | none: mode change only | `:863-905` |
| follow (re)arming | scroll position deltas, 180 ms autoscroll window | none: mode change and publish | `:510-577` |

Only the first four rows write layout-dependent scroll in the same frame. The rest is logic plus attribute writes.

### 2.2 Construction

1. **Follow bottom → a Rust invariant, accelerated by A (scroll anchoring on an end sentinel).**
   - **The invariant (Rust, always on):** `mode == following-bottom` implies the sentinel intersects the viewport within one frame, or Rust writes `scroll(end)`. Source of truth: bridge `observe(IntersectionObserver, sentinel)` plus the content's resize through bridge `observe`. Both are idempotent with native anchoring, since both target the bottom.
   - **The accelerator:** Rust renders a 1 px `[data-message-scroller-end]` as the last child of the content, and CSS says `[data-mode="following-bottom"] [data-message-scroller-content] > * { overflow-anchor: none }` and `[data-message-scroller-end] { overflow-anchor: auto }`. While following, the sentinel is the only anchor candidate, so when the last row grows or a row is appended the browser shifts `scrollTop` inside layout, frame-exact (I; the well-known "pin to bottom" technique).
   - **Why A cannot be the guarantee (W, CSS Scroll Anchoring, <https://drafts.csswg.org/css-scroll-anchoring-1/>; [ux-a11y-perf F1](./board-2026-10-26/ux-a11y-perf.md)):** an anchor is selected only if the scroller is not scrolled away from the origin of its scrolling area (**no anchor at offset 0**: an empty or short chat, then a first long answer that grows past the fold, anchors nothing and leaves the sentinel below the viewport); fully clipped nodes are skipped (**one slipped frame is permanent**: a late arming, a fractional DPR or zoom where the 1 px sentinel sits half a pixel off, a tab returning from the background); and a change of computed `height`/`margin`/`padding`/`transform`/`top` from the anchor up to the scroller cancels the queued adjustment (the turn spacer's `min-height: calc(var(--dx-turn-top) + 100%)` is such a change, right after an anchored turn starts).
   - Arming and release may land a frame late on desktop; with the invariant in Rust that costs a frame, never a drift.
2. **Prepend preservation → A on the rows (`data-mode` not following), with a Rust correction that does not branch on `CSS.supports`.** The current code already relies on the browser and corrects only where it did not (`:762-776`, V). The Rust arm compares an anchor row's top before and after layout, as that code does, so Safari 27 plus the Rust arm never double-corrects.
3. **Anchored turn hold → A with a value Rust sets once.**
   - Instead of resizing a spacer whenever content below changes, Rust sets `--dx-turn-top` (the anchored row's offset in the content) once, when anchoring starts. CSS gives the content `min-height: calc(var(--dx-turn-top) + 100%)` (the viewport is a scroller with a definite height, so `100%` resolves; I). Content below can grow or collapse without a write, because rows above the turn do not change during a turn.
   - **Falsify:** if the viewport's height is not definite in a consumer layout, `100%` fails. The fallback is `100cqb` with `container-type: size` on the viewport, which needs an explicit height; otherwise keep a Rust-sized spacer on that path (I). The anchoring-suppression note of item 1 applies in the window after the turn starts: the invariant covers it.
4. **Jump settle → the `settle` behaviour** (root doc section 2.2), with explicit parameters.
5. **Opening position → G + R.** It already hides the transcript until placed. Rust waits for mount plus the first resize observation, computes `start` / `end` / `last-anchor` (pure Rust, from rects read in one bridge transaction), scrolls, then removes the gate. The pending state is opacity-only.
6. **Mode machine, follow arming, intent, publish → R.**
   - A pure Rust state machine (`following-bottom` / `free-scrolling` / `anchored-to-message` / `settling-jump`), unit-tested natively, a function-for-function port of the TypeScript, which is itself a port of shadcn's controller.
   - **Fed by bridge `listen`/`observe` with a page-side reducer** (emit only when a named derived key changes, the `key === last` dedupe `virtual/bridge.rs:293` already does; `message_scroller.js`'s own header says "Rust receives one message when the published { start, end } scrollability CHANGES, never per scroll event or token"). **Never by Dioxus `onscroll`/`onresize`:** on desktop every vdom event is a synchronous XHR that blocks the webview, and token streaming makes `onresize` fire about once per frame. Intent classification ("wheel toward the end, at the end") is a page-side transition emitter, not a per-event message.
   - The 180 ms window uses `crate::sleep`.
7. **Keep-rendered → R.** The live-edge rows are known to Rust, so they go in rsx. The focused row comes from `onfocusin` on the rows (low rate). Selection ends come from bridge `listen(document, 'selectionchange')` plus a `get` of the anchor/focus rows (I).
8. **Visibility → R** over bridge `observe(IntersectionObserver, {rootMargin})`.
9. **Consumer-composed rows.** Rust knows appends and prepends without a MutationObserver only for rows rendered by `MessageScrollerRows`. Rows passed as `children: Element` are unknown to Rust, so their growth reaches the controller only through bridge `observe` and native anchoring.

**Engines without `overflow-anchor` (Safari 26 and older, and the matching WKWebView/WebKitGTK)** run on the invariant alone: the observation callback runs Rust, which writes `scroll(end)` while following and corrects a prepend by the before/after comparison. On web that is frame-exact (the observer callback and the Rust handler share the frame through microtasks; `MountedData::scroll` is a synchronous `Element.scroll` on web, audit section 2, V). On desktop with an old WebKit it is a frame late (web-verified; desktop inferred), which the policy allows.

**Both arms ship for one deploy.** The legacy controller (`message_scroller.js`) is first frozen, byte for byte, into `primitives/src/ts/legacy/` behind the non-default `legacy-impl` feature that only the preview enables, and `?dx-impl=js|rust` (or a `localStorage` key) picks the arm once after hydration, never at render (root doc section 4). The legacy file is an allowlist entry with an expiry date; `message_scroller.js`'s baseline row in the volume gate carries the same expiry.

### 2.3 What it subsumes

- the 1,065-line controller;
- `message_scroller.js` as a standalone file;
- the `include_str!` (`message_scroller.rs:148`);
- the hand-rolled rAF polling (`OPEN_POLL_FRAMES`);
- the ResizeObserver and MutationObserver in JS (`:906-924`).

The content-change detection becomes Rust's own render knowledge: Rust renders the rows, so it knows appends and prepends without a MutationObserver (I; only for rows rendered by `MessageScrollerRows`, section 2.2 item 9).

### 2.4 Cost

Lane **3.MS is L, about 20 agent-hours** (I; the first version said 16: the added always-on invariant, specs and the legacy freeze and switch). It owns `message_scroller.rs`, `message_scroller.js` (frozen into `ts/legacy`, then deleted at the expiry deploy) and the message-scroller CSS in `preview/src/components/message_scroller/`. It runs **last and alone**, after 3.VS has built `settle`. Its order:
1. freeze the legacy controller into `ts/legacy` and add the `?dx-impl` switch (byte-parity, behaviour-neutral);
2. the CSS anchoring rules and the sentinel;
3. the Rust mode machine plus unit tests, and the always-on invariant;
4. G opening position;
5. `settle` adoption;
6. the Rust arm without anchoring (`overflow-anchor: none !important` forced run);
7. visibility and keep-rendered.

### 2.5 Falsification attempts and remaining risk

| attempt | result |
|---|---|
| Does native anchoring follow growth *inside* the last row (token streaming)? | Yes in principle: the sentinel after the row moves, and anchoring compensates (I). **Must be shown** by a streaming spec that goes through Rust on Chromium, and by a probe on Firefox and Safari 27. Today's `message-scroller-timing` clones DOM nodes from page JavaScript and never starts at offset 0 or at a fractional DPR. |
| The first overflow of an empty chat | The browser anchors nothing at offset 0; **the Rust invariant covers it** (the sentinel is observed, and Rust scrolls to the end when it is not intersecting). New spec. |
| One slipped frame (late arm, fractional DPR/zoom, tab return) | Clipped nodes are never re-picked by the browser; the invariant re-scrolls. New specs: DPR 1.25/1.5, zoom 110/125%, a background-tab return, rearm after a slip. |
| Chromium anchoring suppression (back-and-forth adjustments; a computed-size change on the path) | Streaming adjusts in one direction; the turn spacer is a suppression trigger in the window after a turn starts; the invariant covers both. Measure thousands of adjustments per minute at token rate. |
| content-visibility chunks | Skipped subtrees cannot be anchors (I). The sentinel is never skipped. In free mode the first visible row is rendered, so prepend anchoring works. |
| Safari 27's anchoring is new | `@supports` cannot see bugs. A `data-dx-anchoring="js"` override forces the Rust-only path per engine if a defect is found; a per-engine probe is part of the lane. |
| Upstream tracking (a port of shadcn's controller) | The Rust state machine mirrors theirs function by function: re-porting is TypeScript to Rust instead of TypeScript to TypeScript, a maintenance cost, not a blocker. |
| Desktop latency of mode flips | A late arm or release delays the start or stop of following by a frame; the invariant keeps the end visible. |

**Specs:**
- `message_scroller`, `message-scroller-api`, `message-scroller-timing` (rewritten to stream through Rust), `chat-*`;
- the same suite with `overflow-anchor: none !important` injected to force the Rust-only path;
- the new cases above, RTL, reduced-motion `smooth`;
- the bridge-delay setting at N = 0 and N = 2, and a Firefox and WebKit probe.

**Remaining risk:** high (chat UX). The exit if anchoring misbehaves in an engine is that engine's Rust-only path, never a revert to JavaScript.

---

## 3. C5, the carousel wheel band → R (classifier in Rust), measured, with a TypeScript fallback

### 3.1 What it reads and writes (V, `primitives/src/carousel/wheel.rs:110-590`)

- **Per wheel event:**
  - `deltaX`/`deltaY`/`deltaMode`/`shiftKey`;
  - `performance.now()` at handler time (note: not `e.timeStamp`);
  - `edgeGaps()`: the rects of every child against the container, read **on every event** (`atLimit`);
  - at gesture start, `dxIsTrueEnd` (the `data-index` of the visually extreme child plus `data-slide-count`/`data-loop`);
  - `axisSize()` (track `clientWidth`/`clientHeight`);
  - `matchMedia('(prefers-reduced-motion)')`.
- **Other inputs:** `window` `pointermove` (a cursor-moved release when ≥ 2 px); `pointerdown` on the element.
- **Timers:** three silence timers (`NOTCH_IDLE_MS` 140, `SLOW_IDLE_MS` 250, `IDLE_MS` 1500); gesture gap `GAP_MS` 100.
- **Writes:** `el.style.transform` per event, and an rAF cubic ease-out home (`from * (1 - t)^3` over 340 ms).
- **Never** `preventDefault`, `scrollBy` or a snap change (`:13`, `:576`).
- **Debug telemetry:** `window.__dxCarouselWheel`, which the specs read (`playwright/carousel-helpers.ts:540`, `carousel-virtual-wheel.spec.ts:378`, `carousel-edge-wheel.spec.ts:449`).

### 3.2 Construction

1. **Classifier → a pure Rust state machine** (`carousel/wheel_band.rs`): `step(state, WheelSample) -> (state, band_px, Option<Release>)` and `tick(state, now) -> …` for the silence rules. No DOM, and **`now` always comes from its input**, never from `std::time` or `web_time` (a gate bans those names in the file).
2. **Input → bridge `sample`, batched, not coalesced, flushed per event task, armed only during a wheel gesture.**
   - One message per event task carrying the array of that task's wheel events: `{dx, dy, mode, shift, t: e.timeStamp, gapMin, gapMax}` (a flush on `queueMicrotask`, merging only events of the same task). **Not per rAF:** trackpad wheel events already arrive about once per frame (60-120 per second), so waiting for a rAF saves no volume and adds up to one frame of latency before the hop.
   - The two gaps are a named per-event `sample` field ("children extent against the container on the axis"), read page-side per event exactly as today, because `atLimit` is decided per event; they are a metric the page must supply, so C5 is R plus a bespoke sampler, not "R only".
   - **Why not coalesced:** `isNotch` compares each step with the previous single step (`:316-333`), and the smooth-decay rule averages *consecutive pairs* (`:520-527`). Summing a frame's events would change the classifier's input and its decisions.
   - True end comes from Rust, which knows the rendered window and the direction (the Rust-published `data-dx-true-end-min/max`).
   - The `pointermove` displacement (a ≥ 2 px total is all that matters) is armed only while a wheel gesture holds, through the `until` argument. Today `listen(window, 'pointermove', ...)` lives for the carousel's whole mount (`wheel.rs:578`), which as a Rust `sample` would post 60-1000 messages per second on a desktop for the page's lifetime. `pointerdown` is a plain Dioxus `onpointerdown` on the element (human rate).
3. **Output.**
   - The band goes in one CSS variable (`--dx-wheel-band`) on an element whose `style` Rust does not otherwise own, written through the bridge `set` (`--dx-*` only; no vdom re-render per frame), mapped to `translate` by the sheet. **Until 3.WB lands, C4's bounce stays on inline `transform`**: the two share one `transform` and a pointerdown handoff (`wheel.rs:563`, `drag.rs:215`), unless both adopt one `--dx-band` channel with the pointerdown zeroing.
   - The home spring is a CSS `transition: translate 340ms cubic-bezier(0.33, 1, 0.68, 1)`, i.e. ease-out cubic, the curve `1-(1-t)^3` (A).
   - `prefers-reduced-motion` is CSS (`transition: none`), plus a bridge `match` for the classifier's "draw nothing" rule (read synchronously by the behaviour, default "reduce" until the first answer).
   - Re-engaging mid-home needs the band's current value. Rust computes it from the home's start time and the same curve (deterministic), with no read.
4. **Timers:** Rust `crate::sleep`. A release a frame late on desktop holds the band a few ms longer, which is harmless.

**Latency per renderer (I, from root doc section 1.2; web-verified; desktop inferred):**
- Web: the batch is flushed in the event task's microtask checkpoint. Rust runs in the following microtasks and the variable is written before that frame's style, so it paints in the **same** frame where today's per-event write paints. No added frame.
- Desktop/mobile: one IPC hop each way, so the band trails the fingers by about one to two frames, and a rubber band that tracks the fingers reads as sticky at 2-3 frames at 60 Hz (33-50 ms; 16-25 ms at 120 Hz). Because the effect is passive, no decision changes, only the drawing lags.
- **Isolation is lost.** Today a heavy Rust render during a carousel gesture cannot stall the band, because the JavaScript lives in the webview. With the classifier in Rust, a long app render delays the samples, then they all land at once and the band jumps. The measurement therefore includes a "Rust busy 30 ms during the gesture" scenario (a synthetic render injected).
- **Native bounce.** Safari and WKWebView rubber-band natively, so the band could stack on the platform's own (the open item in `wheel.rs`'s header); the ownership test (at rest at an edge at gesture start) is the only guard. The macOS pass asserts the band does not stack.

### 3.3 Verification without a browser (V for the data, I for the method)

`playwright/fixtures/carousel-wheel-recordings.json` holds MacBook trackpad bursts plus two labelled synthetic ones. The reference engine is `fixtures/carousel-overscroll-bench-rev40.html` (`carousel-wheel-replay.spec.ts:1-40`, V).
- **Step 1, once:** capture expected outputs from the current JS engine: band per event, release kind and release time. A tiny Playwright script runs the shipped `CAROUSEL_WHEEL_BAND_JS` over the recordings and writes `carousel-wheel-expected.json`. The capture script does not exist yet.
- **Step 2, in CI:** `cargo test` replays the recordings through `wheel_band.rs` and requires parity: band within 0.1 px, the same release kind, release time within one event.
- **Step 3, the property test (new):** replay each recording with (a) jittered inter-event gaps drawn from the spread of real 60 Hz and 120 Hz streams, (b) delivery in per-task batches, (c) batches delayed by 0, 1 and 2 frames. Assert the release kind is the same and the release time moves by no more than the injected delay plus one frame. It runs in milliseconds and exercises the classifier as a pure function of (events, tick schedule).
- **Step 4:** **keep the old JavaScript engine as a test oracle fixture** (not shipped) until the desktop measurement has run, so the golden can be regenerated and the live parity row of 3.5 re-run.

**The recordings are weak evidence (V, read):** 11 bursts (9 real, 2 synthetic), recorded on one MacBook trackpad in Firefox 156, with **no per-event timestamps** (events spaced uniformly at `intervalMs` or 17 ms, "real inter-event jitter is lost"), only the first 80 deltas logged and three bursts trimmed. The expected-output file is captured from the current JavaScript, so the replay is a golden against the old implementation on a synthetic uniform timeline: it proves the port computes what the JavaScript computed, and says nothing about real jitter, other devices (a Windows precision touchpad, libinput, Safari momentum) or batching. That is why steps 3 and 4 and the desktop measurement exist. The switch from `performance.now()` to `e.timeStamp` must be applied to the JavaScript when capturing, so both engines see the same clock.

**The in-browser replay's phase:** `playwright/wheel-replay.ts` dispatches the events and samples the band in the same rAF callback; the Rust arm's variable is written after the bridge's flush, so it would be sampled one frame stale and the phase of every assertion (`carousel-wheel-replay`, `carousel-edge-wheel`) would change. The sampler runs in a rAF callback registered after the bridge's, or one frame later, parameterised by the same `N` as the bridge-delay setting.

### 3.4 The exit: a TypeScript module, not page-side wasm

C5 is passive, so the Rust port adds per-event IPC on desktop, a desktop measurement nobody can run here, and removes no line of logic (about 477 lines of JavaScript heuristics become about 477 lines of Rust); the simpler-alternative member's position is that C5 stay a named TypeScript module ("passive stream heuristic, Rust only for policy"), with the replay proof run as `bun test` against it. That is a judgement call against the owner's "Rust first", flagged as simpler, not as wrong. The decision is therefore: **attempt Rust, measure, and fall back**. If the measurement in 3.5 fails on lag, parity or integrity, C5 becomes an allowlisted TypeScript module (with its reason and the measurement as the allowlist entry's row), and the batched `sample` mode is dropped if nothing else uses it.

**Page-side wasm (C2, the same crate compiled to a tiny module) is no longer the exit** (I): it adds a second wasm artefact with a committed binary and a reproducibility gate (like `check-js-bundle.sh`), an async compile at install (Chrome refuses synchronous `WebAssembly.Module` over 4 KB on the main thread, W web.dev), either two paths (a doubled test matrix) or a page kernel on every renderer, and it still needs JavaScript glue to marshal every event. It would be the one construction that is Rust-authored *and* frame-exact on desktop, so it is reconsidered only if a TypeScript module is itself rejected.

### 3.5 What the desktop measurement must show

Run on WebView2 (Windows) and WKWebView (macOS), and on WebKitGTK if Linux desktop is a target, with a 60 Hz and a 120 Hz trackpad, and with both engines (the old path behind `?dx-impl=js`, the new behind `?dx-impl=rust`, one commit, arms interleaved ABAB, at least 10 flicks per arm, release build):

| metric | how | pass |
|---|---|---|
| added input→paint lag | `e.timeStamp` vs the first rAF whose computed `translate` reflects that event (in-page, both engines) | p95 added ≤ 1 frame vs JS; p99 ≤ 2 frames |
| frame health during a band gesture | frame times while pushing at the edge | p95 ≤ 16.7 ms (60 Hz) or ≤ 8.3 ms (120 Hz); no long task > 50 ms in the webview |
| channel integrity | events counted page-side vs consumed in Rust per gesture | 0 lost, 0 reordered |
| decision parity live | release kind and time for scripted bursts (the same recordings replayed via `dispatchEvent`) | identical kind, time within one frame |
| Rust busy | the same gesture with a synthetic 30 ms render injected | the band does not jump on catch-up beyond the JS arm's tolerance |
| native bounce | macOS WKWebView at an edge with momentum | the band does not stack on the platform's own bounce |
| CPU cost | Rust process plus webview main thread during a 3 s band gesture | no more than the JS engine + 10% (I: an arbitrary but explicit bound) |

**Fail on lag, parity or integrity** → the TypeScript module (integrity or parity first look for a bridge bug). This cannot run in this container (no gtk/webkit2gtk); it needs a person with a desktop machine ([row 443](../backlog/443-owner-measurement-kit-perf-recorder-and-impl-switch.md) is the kit). Until it has run, the legacy arm ships next to the Rust arm behind the `legacy-impl` feature, and the Rust engine is the preview default.

### 3.6 Cost, falsification, risk

- **Lane 3.WB, about 12 agent-hours, size M** (I; the first version said 10):
  - classifier port, 3 h;
  - expected-output capture, `cargo test` parity and the property test, 3 h;
  - bridge `sample` wiring (flush per event task, armed `until`), CSS variable and home transition, 2 h;
  - legacy freeze and the `?dx-impl` arm, debug telemetry kept bit-compatible for the three specs, the "Rust busy" scenario and the harness phase fix, 3 h;
  - desktop measurement, 1 h of a person's time (the owner's).
- **Files:** `carousel/wheel.rs` and a new `carousel/wheel_band.rs`, plus the fixture. 3.WB runs after 3.CA (the true-end function and the transform channel) and after 3.MS.

| attempt | result |
|---|---|
| Does the band need a per-event decision the page commits? | No. Passive, never cancels (V). Lateness is lag only. |
| Is batching lossy? | Not if batched rather than coalesced (3.2). Coalescing **would** break notch and decay detection (V, `:316-333`, `:520-527`). |
| Is geometry per event available off-thread? | It is read page-side into the sample (a named field), at the same moment as today. |
| Debug telemetry the specs read? | Must be reproduced: Rust writes the same record shape to `window.__dxCarouselWheel` under the same flag, through the bridge (`carousel-helpers.ts:540`, `carousel-virtual-wheel.spec.ts:378`, `carousel-edge-wheel.spec.ts:449`). |
| Tuned feel | The parity test pins every number. The only intended change is `e.timeStamp` instead of handler time, applied to both engines in the capture. |
| Per-frame batching from rAF (the first version) | Replaced: it adds up to a frame before the hop and reads the Rust arm one frame stale in the replay harness. |

**Remaining risk:** high-feel and medium-probability. The desktop lag is unknown until measured, and the exit is defined.

---

## 4. C1, the helper macros → dissolve (confirmed)

**Users (V, grep):**
- `carousel_true_end_js!` in `wheel.rs:110` (C5), `drag.rs:99` (C4) and `scroll.rs:230` (C3);
- `carousel_trace_js!` in `drag.rs:99`, `scroll.rs:77` (C2) and `scroll.rs:230`;
- the test `carousel/ssr_tests.rs:412-415`, which asserts the macro is the rule's only home.

**Replacements:**
- **`dxIsTrueEnd`** → Rust. True end means "the visually extreme child is index 0 or count−1, not looping". Rust renders the window and knows the direction, so it publishes `data-dx-true-end-min` / `data-dx-true-end-max` per render (I).
  - C4's `pan` rubber-band reads the attribute.
  - C3 (R) and C5 (R) use the Rust value directly.
  - **Falsify:** for a frame during a window shift the attribute can be stale on desktop. The band only engages from rest at an edge, and window shifts happen on settle (`scroll.rs` settle, V), so the overlap is a settle frame. Low risk (I).
- **`dxTrace`** → Rust `tracing` in the R parts, plus the debug `window.__dxCarouselTrace` records written through the bridge under the same `dx-carousel-debug` flag, because `carousel-virtual-wheel.spec.ts:1195` reads `edgeIndex` from them (V).
- **Order:**
  - 3.CA removes the uses in `scroll.rs`/`drag.rs` from the Rust arm; the legacy copies of C2, C3, C4 and C5 keep the macros inside `ts/legacy` while the legacy arms live;
  - 3.WB removes the use in `wheel.rs` from the Rust arm;
  - **`js_macros.rs` is deleted at the legacy-switch expiry deploy**, with `ts/legacy/`, and `ssr_tests.rs:412` is rewritten as a Rust unit test of the true-end function (about 2 h with the other legacy removals). A main-loop deletion right after the ports would break the legacy arm.

**Subsumes:** the `concat!` construction the audit flagged as needing "its own plan".

---

## 5. Revised totals and lanes

**Class (d), 22 blocks, 3,894 lines (I; the classification is the root doc's, section 3):**

| construction | blocks | ids | lines |
|---|---:|---|---:|
| A, with a Rust invariant or verification behind it | 3 | T8, T11, W8 | 1,638 |
| Named behaviours (page-side TypeScript, closed registry) | 7 | S11; O5, C4; W3, W5, W7; C2 | 880 |
| R: rsx handlers or plain bridge verbs plus Rust | 6 | L30, W1, C3, C5 (measured, TypeScript fallback), C1 (dissolves), S14 | 1,153 |
| G: Rust behind a paint-safe gate | 3 | S1, O3, V19 | 172 |
| Spliced `const` text | 2 | T9, T10 | 39 |
| The transport seam | 1 | V11 | 12 |

**Lanes, in the owner's order (root doc section 5.4 holds the whole plan):** after the placement and behaviour lanes, **one at a time, each its own batch and deploy, and the next starts only after the owner's measurement**:
1. **3.MS** message scroller, about 20 h (section 2.4);
2. **3.WB** wheel band, about 12 h (section 3.6);
3. the **legacy removal** at the expiry deploy, about 2 h (`js_macros.rs`, `ts/legacy/`, the volume-gate expiry rows).

(The first version of this document and of the root doc's lane table showed 3.CA, 3.WB and 3.MS in one parallel batch; the order above is the decided one. 3.CA runs earlier, in the behaviour batches, and keeps C4's bounce on inline `transform` until 3.WB lands.)

**Bridge additions this needs:**
- `sample` must support *batched* event arrays plus per-event page metrics, flushed per event task and armed only while a gesture holds (the root doc's section 5.2);
- a `set` verb for `--dx-*` CSS variables on elements whose style Rust does not own.

**Gate (root doc section 5.3):**
- No JavaScript in a `.rs` file outside the transport seam and the class-(c) allowlist; `message_scroller.js` and every `concat!`-built carousel script leave the class-(d) list when their lanes land.
- The legacy arms (`ts/legacy/*`, behind `legacy-impl`) are allowlist entries with a row number and an `expires:` date that fails after it; the volume gate's `message_scroller.js` row carries the same expiry.

## 6. Not verified

- Scroll anchoring behaviour (sentinel follow, streaming rate, Safari 27 quality, the first-overflow and suppression cases) is inferred from the spec's model and the well-known technique. No engine was probed, and Chromium is the only engine installed here.
- The `min-height: calc(var(--dx-turn-top) + 100%)` turn hold assumes a definite viewport height, which was not checked against every consumer layout.
- The C5 parity method depends on capturing expected outputs from the shipped JavaScript first. The capture script does not exist yet, and the recordings carry no real timestamps (3.3).
- All hour figures are estimates. The desktop measurement needs hardware this container does not have; every desktop statement here is web-verified; desktop inferred.
