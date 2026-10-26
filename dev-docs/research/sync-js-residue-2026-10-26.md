# The residue, re-attacked: W8, C5 and C1 to Rust (row 434, step 3), 2026-10-26

Follow-up to [`sync-js-root-2026-10-26.md`](./sync-js-root-2026-10-26.md) (the "root doc"). Read-only, on `origin/queue/batch-28` @ `d10d34b5`. Nothing was built or run. Evidence labels as in the root doc: **V** = read in code or crate source, **W** = web / web-features 3.44.0, **I** = inferred.

## Executive summary (for the owner)

**All three blocks can move to Rust, and none has to stay a bespoke JS module.** One of them carries a measured exit condition.

- **The policy inconsistency was real, and it is now fixed.** `overflow-anchor` reached Baseline newly available on 2026-09-14 (Chrome 56, Firefox 66, Safari 27; W). The root doc's own policy is "Baseline newly available, with a feature-detected fallback that stays correct", so it qualifies **today**. "Once the policy admits it" was wrong.
- **W8, the message scroller (1,065 lines):** every write it makes in the same frame becomes either CSS or a value Rust sets once.
  - "Follow the bottom" becomes native scroll anchoring on a 1 px end sentinel. While following, it is the only anchor the browser may pick.
  - Prepend preservation is native scroll anchoring on the rows. The current code already treats the browser's anchoring as the primary path (V).
  - The anchored-turn tail spacer becomes a CSS `min-height` from a variable Rust sets once per anchored turn.
  - Jumps use the B5 settle verb.
  - The mode machine, opening position, visibility, keep-rendered and scroll-intent logic move to Rust.
  - Engines without `overflow-anchor` (Safari ≤26) get Rust corrections from Dioxus's `onresize`. On web those are frame-exact, because the handler runs inside the ResizeObserver callback and `MountedData::scroll` is synchronous there (V).
- **C5, the carousel wheel band (477 lines):** the classifier becomes a pure Rust state machine.
  - Proof without a browser: `cargo test` replays the trackpad recordings already committed in `playwright/fixtures/carousel-wheel-recordings.json` against expected outputs captured once from the current JS.
  - Input is a **batched** per-frame stream: one message per frame carrying *every* wheel event with its own timestamp. It is **not coalesced**, because notch detection and the smooth-decay rule compare consecutive single events.
  - Output is one CSS variable per frame, and the spring home becomes a CSS `ease-out-cubic` transition.
  - It is passive, so on desktop a late answer means lag, never a wrong decision.
  - The **exit** is C2 (the same crate compiled to a tiny page-side wasm) if a desktop measurement shows more than one extra frame of lag at p95.
- **C1, the helper macros (65 lines):** confirmed to disappear.
  - `dxIsTrueEnd` becomes a Rust-published `data-dx-true-end-min/max` pair, and Rust uses it itself in C3/C5.
  - `dxTrace` becomes Rust tracing, plus the debug `window` records the specs read, written through the bridge.
  - The macros' four users are C2, C3, C4 and C5 (V). Once those move, the macro file and its SSR test go.
- **Result:** class (d) ends with **0 bespoke modules** (22 → A 3, B 9, G 5, R 5). The JS left in the page is the bridge plus the B interpreter.
- **Cost:** +22 agent-hours over the root plan, about 86 h in total (I). The message-scroller lane grows from M 5 h to L 16 h, and C5 becomes its own lane (10 h), next to the carousel lane in the same batch.
- **Biggest risks:**
  - the quality of Safari 27's new scroll anchoring;
  - Chromium's anchoring heuristics under token-rate streaming;
  - desktop latency for C5, which cannot be measured in this container (no gtk). It needs a machine with WebView2 or WKWebView.

---

## 1. Policy consistency (fixed)

Root doc §2.1 policy: *Baseline newly available across Chrome, Firefox and Safari; older engines must stay correct through a feature-detected fallback, not necessarily frame-perfect.* `overflow-anchor` (web-features `overflow-anchor`, W):
- Baseline low on 2026-09-14;
- Chrome 56, Firefox 66, Safari 27.

So it qualifies under the same rule as anchor positioning (Baseline low since 2026-01-13). The root doc's §4 wording ("once the policy admits it") contradicted §2.1. There was no hidden difference. The only real extra caveat is that Safari 27's implementation is weeks old, which the fallback design below absorbs. Root doc §3 (the W8 row), §4, §5.1 and §5.4 are now corrected.

The fallback is detected by `@supports (overflow-anchor: auto)` / `CSS.supports`. It cannot detect a *buggy* implementation, so the per-engine probe in §2.5 is part of the lane.

---

## 2. W8, the message scroller → A + R (+ B5), with a Rust fallback

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

1. **Follow bottom → A (scroll anchoring on an end sentinel).**
   - Rust renders a 1 px `[data-message-scroller-end]` as the last child of the content.
   - CSS: `[data-mode="following-bottom"] [data-message-scroller-content] > * { overflow-anchor: none }` and `[data-message-scroller-end] { overflow-anchor: auto }`.
   - While following, the sentinel is the only anchor candidate. When the last row grows or a row is appended before the sentinel, the browser shifts `scrollTop` to keep the sentinel in place, frame-exact, inside layout (I; the well-known "pin to bottom" technique).
   - Rust flips `data-mode` from its mode machine.
   - Arming and release may land a frame late on desktop. That is harmless: when the sentinel is off-screen nothing is adjusted, which is free-scrolling behaviour anyway (I).
2. **Prepend preservation → A** (native anchoring on rows, `data-mode` ≠ following). The current code already relies on it and only corrects where the browser did not (`:762-776`, V). With `overflow-anchor` in policy, the correction becomes the fallback.
3. **Anchored turn hold → A with a value Rust sets once.**
   - Instead of resizing a spacer whenever content below changes, Rust sets `--dx-turn-top` (the anchored row's offset in the content) once, when anchoring starts.
   - CSS gives the content `min-height: calc(var(--dx-turn-top) + 100%)` (the viewport is a scroller with a definite height, so `100%` resolves; I). The scroll range then always allows the turn at the top.
   - Content below can grow or collapse without a write, because rows above the turn do not change during a turn. A prepend during a turn re-measures, but it is rare and goes through the same G path.
   - **Falsify:** if the viewport's height is not definite in a consumer layout, `100%` fails. The fallback is `100cqb` with `container-type: size` on the viewport, which needs an explicit height. Otherwise keep a Rust-sized spacer on the fallback path (I).
4. **Jump settle → B5** (the root doc's reveal/settle verb, shared with W3/W5/W7).
5. **Opening position → G + R.** It already hides the transcript until placed. Rust waits for mount plus the first `onresize`, computes `start` / `end` / `last-anchor` (pure Rust, from `MountedData` rects), scrolls, then removes the gate.
6. **Mode machine, follow arming, intent, publish → R.**
   - A pure Rust state machine (`following-bottom` / `free-scrolling` / `anchored-to-message` / `settling-jump`), unit-tested natively. It is a function-for-function port of the TS, which is itself a port of shadcn's controller.
   - Fed by Dioxus `onscroll` (ScrollData has everything `scrollableState` needs, V audit §2) and bridge `listen` for wheel/touch/keys with a B1 filter (skip a "toward end at end" gesture: `can-scroll` predicate).
   - The 180 ms window uses `crate::sleep`.
7. **Keep-rendered → R.** The live-edge rows are known to Rust, so they go in rsx. The focused row comes from `onfocusin` on the rows. Selection ends come from bridge `listen(document, 'selectionchange')` plus a `get` of the anchor/focus rows (I).
8. **Visibility → R** over bridge `observe(IntersectionObserver, {rootMargin})`.

**Fallback (engines without `overflow-anchor`: Safari ≤26, and the matching WKWebView/WebKitGTK).** Rust performs rows 1-3 itself from Dioxus `onresize` on the content.
- On web this is **frame-exact**: the interpreter dispatches the `resize` `CustomEvent` synchronously inside the ResizeObserver callback (`dioxus-interpreter-js-0.7.9/src/ts/core.ts:51-72`, V). The web handler runs synchronously (`dioxus-web dom.rs:88-110`, V), and `MountedData::scroll` is a synchronous `Element.scroll` on web (audit §2, V). So read-then-write happens in Rust inside the RO callback.
- On desktop with an old WebKit it is a frame late (I), which the policy allows.

### 2.3 What it subsumes

- the 1,065-line controller;
- `message_scroller.js` as a standalone file;
- the `include_str!` (`message_scroller.rs:148`);
- the hand-rolled rAF polling (`OPEN_POLL_FRAMES`);
- the ResizeObserver and MutationObserver in JS (`:906-924`).

The content-change detection becomes Rust's own render knowledge: Rust renders the rows, so it knows appends and prepends without a MutationObserver (I).

### 2.4 Cost

Lane **3.MS** grows from M 5 h to **L, about 16 agent-hours** (I). It owns `message_scroller.rs`, `message_scroller.js` (deleted) and the message-scroller CSS in `preview/src/components/message_scroller/`. Its order:
1. CSS anchoring rules and the sentinel;
2. Rust mode machine plus unit tests;
3. G opening position;
4. B5 adoption;
5. fallback path;
6. visibility and keep-rendered.

### 2.5 Falsification attempts and remaining risk

| attempt | result |
|---|---|
| Does native anchoring follow growth *inside* the last row (token streaming)? | Yes in principle: the sentinel after the row moves, and anchoring compensates (I). **Must be shown** by `message-scroller-timing` on Chromium and by a probe on Firefox and Safari 27. |
| Chromium anchoring heuristics (suppression after back-and-forth adjustments; no anchoring at scroll offset 0) | Streaming only adjusts in one direction, and following is never at offset 0 unless the content fits, in which case nothing scrolls (I). Measure: thousands of adjustments per minute at token rate. |
| content-visibility chunks | Skipped subtrees cannot be anchors (I). The sentinel is never skipped. In free mode the first visible row is rendered, so prepend anchoring works. |
| Safari 27's anchoring is new | `@supports` cannot see bugs. Mitigation: a per-engine probe in the lane, and a `data-dx-anchoring="js"` override that forces the Rust fallback per engine if a defect is found. |
| Upstream tracking (a port of shadcn's controller) | The Rust state machine mirrors theirs function by function. Re-porting upstream changes is TS→Rust instead of TS→TS: a maintenance cost, not a blocker. |
| Desktop latency of mode flips | A late arm or release only delays the start or stop of following. The geometry itself is the browser's (A). |

**Specs:**
- `message_scroller`, `message-scroller-api`, `message-scroller-timing`, `chat-*`;
- the same suite with `overflow-anchor: none !important` injected to force the fallback;
- the slow-channel mode (root doc §5.6) at N = 0 and N = 2.

**Remaining risk:** high (chat UX). The exit if anchoring misbehaves in an engine is that engine's Rust fallback, not JS.

---

## 3. C5, the carousel wheel band → R (classifier in Rust), exit C2

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

1. **Classifier → a pure Rust state machine** (`carousel/wheel_band.rs`): `step(state, WheelSample) -> (state, band_px, Option<Release>)` and `tick(state, now) -> …` for the silence rules. No DOM.
2. **Input → bridge `sample`, batched, not coalesced.** One message per animation frame carrying the array of that frame's wheel events: `{dx, dy, mode, shift, t: e.timeStamp, gapMin, gapMax}`.
   - The two gaps are a page-side B3 metric ("children extent vs container on the axis"), read per event exactly as today, because `atLimit` is decided per event.
   - **Why not coalesced:** `isNotch` compares each step with the previous single step (`:316-333`), and the smooth-decay rule averages *consecutive pairs* (`:520-527`). Summing a frame's events would change the classifier's input and its decisions.
   - True end comes from Rust, which knows the rendered window and the direction, so it needs no geometry (I).
   - `pointermove` displacement can be coalesced per frame, since only a ≥ 2 px total matters. `pointerdown` is a plain Dioxus `onpointerdown` on the element.
3. **Output.**
   - The band goes in one CSS variable (`--dx-wheel-band`) on an element whose `style` Rust does not otherwise own, written through a bridge `set` (no vdom re-render per frame), mapped to `translate` by the sheet.
   - The home spring is a CSS `transition: translate 340ms cubic-bezier(0.33, 1, 0.68, 1)`, i.e. ease-out cubic, the curve `1-(1-t)^3` (A).
   - `prefers-reduced-motion` is CSS (`transition: none`), plus a bridge `match` for the classifier's "draw nothing" rule.
   - Re-engaging mid-home needs the band's current value. Rust computes it from the home's start time and the same curve (deterministic), with no read.
4. **Timers:** Rust `crate::sleep`. A release a frame late on desktop holds the band a few ms longer, which is harmless.

**Latency per renderer (I, from root doc §1.2):**
- Web: the batch is sent from the rAF callback. Rust runs in the following microtasks and the variable is written before that frame's style, so it paints in the **same** frame where today's per-event write paints. No added frame.
- Desktop/mobile: one IPC hop each way, so the band trails the fingers by about one to two frames. Because the effect is passive, no decision changes, only the drawing lags.

### 3.3 Verification without a browser (new; V for the data, I for the method)

`playwright/fixtures/carousel-wheel-recordings.json` holds real MacBook trackpad bursts plus two labelled synthetic ones. The reference engine is `fixtures/carousel-overscroll-bench-rev40.html` (`carousel-wheel-replay.spec.ts:1-40`, V).
- **Step 1, once:** capture expected outputs from the current JS engine: band per event, release kind and release time. A tiny Playwright script runs the shipped `CAROUSEL_WHEEL_BAND_JS` over the recordings and writes `carousel-wheel-expected.json`.
- **Step 2, in CI:** `cargo test` replays the recordings through `wheel_band.rs` and requires parity: band within 0.1 px, the same release kind, release time within one event.

This proves the classifier natively, in milliseconds, on every gate run, with no browser and no desktop. Timing note: the recordings' timestamps are reconstructed (spec header, V), so the parity is against the JS run *on the same reconstructed timing*. The switch from `performance.now()` to `e.timeStamp` must be applied to the JS when capturing, so both engines see the same clock.

### 3.4 Is C2 (the same crate as page-side wasm) the cleaner peak?

**Not as the default (I):**
- **Parts:** C2 adds a second wasm artefact, a committed binary plus a reproducibility gate (like `check-js-bundle.sh`), and an async compile at install (Chrome refuses synchronous `WebAssembly.Module` over 4 KB on the main thread, W web.dev).
- **Paths:** either two paths (app wasm on web, page wasm on desktop: a doubled test matrix), or the page kernel on every renderer (one path, but web then pays a second module for nothing, because its app wasm already runs on the page thread).
- **Glue:** it still needs JS to marshal every event into the kernel.
- CSP is *not* an extra cost on web: a Dioxus web app is wasm already (I).

**C2 is the peak only if desktop lag is a visible problem.** Then it is the one construction that is Rust-authored *and* frame-exact on desktop. It would also cover any later continuous loop with the same profile. So it is the **exit**, gated by the measurement below.

### 3.5 What the desktop measurement must show

Run on WebView2 (Windows) and WKWebView (macOS), and on WebKitGTK if Linux desktop is a target, with a 60 Hz and a 120 Hz trackpad, and with both the JS engine and the Rust one:

| metric | how | pass |
|---|---|---|
| added input→paint lag | `e.timeStamp` vs the first rAF whose computed `translate` reflects that event (in-page, both engines) | p95 added ≤ 1 frame vs JS; p99 ≤ 2 frames |
| frame health during a band gesture | frame times while pushing at the edge | p95 ≤ 16.7 ms (60 Hz) or ≤ 8.3 ms (120 Hz); no long task > 50 ms in the webview |
| channel integrity | events counted page-side vs consumed in Rust per gesture | 0 lost, 0 reordered |
| decision parity live | release kind and time for scripted bursts (the same recordings replayed via `dispatchEvent`) | identical kind, time within one frame |
| CPU cost | Rust process plus webview main thread during a 3 s band gesture | no more than the JS engine + 10% (I: an arbitrary but explicit bound) |

**Fail on lag** → C2. **Fail on integrity or parity** → a bug, fix it in the bridge.

This cannot run in this container (no gtk/webkit2gtk); it needs a person with a desktop machine. Until it has run, the lane keeps the JS behind a cfg (the 2D-style keep-JS exit), and the Rust engine ships on web first.

### 3.6 Cost, falsification, risk

- **Lane 3.WB, about 10 agent-hours, size M-L** (I):
  - classifier port, 3 h;
  - expected-output capture plus `cargo test` parity, 2 h;
  - bridge `sample` wiring, CSS variable and home transition, 2 h;
  - debug telemetry kept bit-compatible for the three specs, plus a slow-channel run, 2 h;
  - desktop measurement, 1 h of a person's time.
- **Files:** `carousel/wheel.rs` and a new `carousel/wheel_band.rs`, plus the fixture. Disjoint from 3.CA's files.

| attempt | result |
|---|---|
| Does the band need a per-event decision the page commits? | No. Passive, never cancels (V). Lateness is lag only. |
| Is per-frame batching lossy? | Not if batched rather than coalesced (§3.2). Coalescing **would** break notch and decay detection (V, `:316-333`, `:520-527`). |
| Is geometry per event available off-thread? | It is read page-side into the sample (B3 metric), at the same moment as today. |
| Debug telemetry the specs read? | Must be reproduced: Rust writes the same record shape to `window.__dxCarouselWheel` under the same flag, through the bridge. |
| Tuned feel | The parity test pins every number. The only intended change is `e.timeStamp` instead of handler time, applied to both engines in the capture. |

**Remaining risk:** high-feel and medium-probability. The desktop lag is unknown until measured, and the exit (C2) is defined.

---

## 4. C1, the helper macros → dissolve (confirmed)

**Users (V, grep):**
- `carousel_true_end_js!` in `wheel.rs:110` (C5), `drag.rs:99` (C4) and `scroll.rs:230` (C3);
- `carousel_trace_js!` in `drag.rs:99`, `scroll.rs:77` (C2) and `scroll.rs:230`;
- the test `carousel/ssr_tests.rs:412-415`, which asserts the macro is the rule's only home.

**Replacements:**
- **`dxIsTrueEnd`** → Rust. True end means "the visually extreme child is index 0 or count−1, not looping". Rust renders the window and knows the direction, so it publishes `data-dx-true-end-min` / `data-dx-true-end-max` per render (I).
  - C4's B2 rubber-band rule reads the attribute.
  - C3 (R) and C5 (R) use the Rust value directly.
  - **Falsify:** for a frame during a window shift the attribute can be stale on desktop. The band only engages from rest at an edge, and window shifts happen on settle (`scroll.rs` settle, V), so the overlap is a settle frame. Low risk (I).
- **`dxTrace`** → Rust `tracing` in the R parts, plus the debug `window.__dxCarouselTrace` records written through the bridge under the same `dx-carousel-debug` flag, because `carousel-virtual-wheel.spec.ts:1195` reads `edgeIndex` from them (V).
- **Order:**
  - 3.CA removes the uses in `scroll.rs`/`drag.rs`;
  - 3.WB removes the use in `wheel.rs`;
  - the main loop then deletes `js_macros.rs` and rewrites `ssr_tests.rs:412` as a Rust unit test of the true-end function, after both merge (about 1 h).

**Subsumes:** the `concat!` construction the audit flagged as needing "its own plan".

---

## 5. Revised totals and lanes

**Class (d), 22 blocks, 3,894 lines (I):**

| construction | blocks | ids | lines |
|---|---:|---|---:|
| A (+ G/R fallback) | 3 | T8, T11, W8 | 1,638 |
| B (B1 1, B2 3, B3 1, B5 4) | 9 | S11; L30, O5, C4; S14; W3, W5, W7, C2 | 1,038 |
| G: Rust behind a paint-safe gate | 5 | T9, T10, S1, O3, V19 | 211 |
| R: Rust over plain bridge verbs | 5 | W1, C3, V11, C5, C1 | 1,007 |
| **Residue** | **0** | — (C5 carries the measured exit to C2) | 0 |

**Lane changes against root doc §5.4:**
- 3.MS: M 5 h → **L 16 h** (Rust plus A).
- 3.CA: 12 → **13 h** (C1 uses removed).
- New **3.WB**: 10 h, batch 3 alongside 3.CA, disjoint files.
- Main-loop cleanup of `js_macros.rs` / `ssr_tests.rs`: about 1 h.
- **New total ≈ 86 agent-hours** (root plan ≈ 64).

**Bridge additions this needs:**
- `sample` must support *batched* event arrays plus per-event page metrics (root doc §5.2 said "coalesced"; now corrected to batched or coalesced per field).
- A `set` verb for CSS variables on elements whose style Rust does not own.

**Gate (root doc §5.3):**
- The residue allowlist starts **empty**.
- `message_scroller.js` and every `concat!`-built carousel script are removed from the class-(d) list.
- The C5 cfg keep-JS path, if it is kept until the desktop measurement, is listed with the row number and the measurement as its expiry.

## 6. Not verified

- Scroll anchoring behaviour (sentinel follow, streaming rate, Safari 27 quality) is inferred from the spec's model and the well-known technique. No engine was probed.
- The `min-height: calc(var(--dx-turn-top) + 100%)` turn hold assumes a definite viewport height, which was not checked against every consumer layout.
- The C5 parity method depends on capturing expected outputs from the shipped JS first. The capture script does not exist yet.
- All hour figures are estimates. The desktop measurement needs hardware this container does not have.
