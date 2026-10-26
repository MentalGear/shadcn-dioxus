# The residue, re-attacked: W8, C5 and C1 to Rust (row 434, step 3), 2026-10-26

Follow-up to [`sync-js-root-2026-10-26.md`](./sync-js-root-2026-10-26.md) (the "root doc"). Read-only, on `origin/queue/batch-28` @ `d10d34b5`. Nothing was built or run. Evidence labels as in the root doc: **V** = read in code or crate source, **W** = web / web-features 3.44.0, **I** = inferred.

**Revised 2026-10-26 after six adversarial boards** ([round 1](./board-2026-10-26/SYNTHESIS.md), [round 2](./board-2026-10-26-r2/SYNTHESIS.md), [round 3](./board-2026-10-26-r3/SYNTHESIS.md), [round 4](./board-2026-10-26-r4/SYNTHESIS.md), [round 5](./board-2026-10-26-r5/SYNTHESIS.md), [round 6](./board-2026-10-26-r6/SYNTHESIS.md)). Round 1 changed W8 and C5; round 2 added the `stickToEnd` named behaviour for W8's same-frame write, put the legacy freeze and the `?dx-impl` switch into lane 3.FZ (the first support lane of the owner-measured group), keyed the legacy removal to the owner's measurement and re-sized 3.MS and 3.WB; round 3 made `stickToEnd` own its page-side detach (the release is a page transition, not a Rust round trip) and put W8, C5 and C1 in the owner-measured group of the scope decision (root doc section 5.0); round 4 made `stickToEnd` run on every engine, idempotent with A (anchoring fails by state as well as by engine), and derived admission per effect from the placement table, so `wheelBand` is admitted only by a `lag` or `order` verdict. C1 held. Round 5 changed three things here: `stickToEnd`'s witness runs under fallback emulation (`overflow-anchor: none !important` and the A sheet removed), because on a current Chromium anchoring keeps the bottom natively and a delay witness would be green; the detach and every other Rust-written scroll position go through the page-side `scrollOwner` behaviour (an input classifier plus a memory of the owner's own writes), which 3.VS builds before 3.MS; and the plan is backlogged as rows 463 to 467 for this group ([row 434](../backlog/434-javascript-in-typescript-modules-not-rust-strings.md) lists them in order). Round 6 changed one thing here: `stickToEnd`'s detach is decided by the level (the distance from the end at each scroll, the `scrollOwner`'s level intent kind), not by the kind of input, and the `scrollOwner` classifies by the realized default action (one passive window bubble listener reading `defaultPrevented`), so a wheel-up, a find-in-page and a screen reader's scroll-up detach it while anchoring and re-snap do not; the classifier is built first as item 0.7's spike. **Every desktop and mobile claim here is "web-verified; desktop inferred"** until a `desktop-pass/<tree>.pass` stamp exists ([row 442](../backlog/442-desktop-cadence-harness-and-desktop-pass.md)); Chromium is the only engine installed in the lane container, so Firefox and WebKit claims carry the same label.

## Executive summary (for the owner)

**All three blocks leave bespoke JavaScript. W8 and C5 are measured ports: the owner measures each, one at a time, with the old path switchable for one deploy.**

- **The support policy is per-feature floors plus a boot probe**, not a Baseline date (root doc section 2.1). `overflow-anchor` reached Baseline newly available on 2026-09-14 (Chrome 56, Firefox 66, Safari 27; W); its floor is Safari 27, and an engine that fails the boot probe is a fallback engine.
- **W8, the message scroller (1,065 lines): a Rust invariant, with `overflow-anchor` as the accelerator and `stickToEnd` on every engine for the same-frame write.**
  - The invariant lives in Rust, always on and independent of engine support: while `mode == following-bottom`, **the end sentinel intersects the viewport within one frame, or Rust scrolls to the end**. Its source of truth is bridge `observe` (IntersectionObserver on a 1 px end sentinel, resize on the content).
  - CSS anchoring makes that frame-exact where it works. It cannot be the guarantee: the scroll-anchoring spec selects **no anchor at scroll offset 0**, skips fully clipped nodes (one slipped frame is permanent until a Rust scroll), and cancels a queued adjustment on a computed `height`/`margin`/`padding`/`transform`/`top` change on the way up to the scroller.
  - **Follow-bottom is a same-frame layout write**, and Rust over the bridge alone answers it 1-2 frames late on desktop (render, acknowledgement, eval back). Safari enabled scroll anchoring only in 27, so on the Apple webviews of macOS and iOS 26 (the majority for months after a September release) the CSS cannot do the write either. So the write has a named behaviour, **`stickToEnd`**: a ResizeObserver callback that sets `scrollTop = scrollHeight` while the Rust-published `data-mode` is `following-bottom` (about 40 lines, standard fail-open and epoch). **It runs on every engine, idempotent with A**: anchoring fails by state as well as by engine (no anchor at offset 0, a fractional DPR, a tab return), so choosing `stickToEnd` by engine would leave states where neither runs; where `overflow-anchor` is verified the CSS makes the same write natively and the two agree. **It owns its detach**: a user scroll away from the end leaves `following-bottom` in the page, decided by the level and the realized default action, so a wheel-up, touch, key, scrollbar drag, find-in-page or screen-reader scroll-up all detach it while anchoring does not (a page transition that mints a newer epoch, root doc section 2.2, order contract), so the release cannot land late and fight the user, and the boot probe selects nothing for W8; Rust learns the new mode afterwards and its mode machine accepts the page's transition rather than deciding it. The invariant's `scroll(end)` goes through `stickToEnd` carrying the epoch, so a write in flight cannot land after the wheel-up.
  - **Rust keeps the mode machine, scroll intent, opening position, visibility and the invariant**, fed by a **page-side reducer** over bridge `listen`/`observe`: never by vdom `onscroll`/`onresize`, which on desktop is one blocking request per event.
- **C5, the carousel wheel band (477 lines): a Rust attempt, measured.**
  - It is passive, so lateness is lag, never a wrong decision. A pure Rust classifier; input is a **batched** wheel stream flushed **per event task** (not per rAF: trackpad wheel already arrives about once per frame, so rAF batching only adds up to a frame of latency); output is one CSS variable.
  - It sits in the deadline table's **next-frame row** (root doc section 2.2): bridge stream plus Rust under the latency budget the owner sets from the row 442/443 measurements, a named behaviour only if the measured lag exceeds it. Section 3.5's pass column (p95 added lag at most 1 frame, p99 at most 2) is that budget for this port.
  - The owner measures it against the old path (`?dx-impl`, frozen by lane 3.FZ), including a "Rust busy 30 ms" scenario. **If Rust loses, C5's fallback is a named behaviour, `wheelBand`, admitted only by a verdict with the reason `lag` or `order`** (`research/measure/c5/verdict.json` is `{verdict, reason, devices, traces}`: the witness of the next-frame row of root doc section 2.2, not an allowlisted module and not an effect structural detection can see, because a passive stream heuristic has no commit-sensitive effect). A `parity` or `integrity` reason means the Rust arm is to be fixed and the legacy arm stays frozen. Page-side wasm (C2) is not the exit.
  - The proof without a browser is a `cargo test` replay plus a property test under jitter, batching and delay; the committed recordings alone are weak evidence (section 3.3).
- **C1, the helper macros (65 lines):** confirmed to dissolve. `dxIsTrueEnd` becomes a Rust-published `data-dx-true-end-min/max` pair; `dxTrace` becomes Rust tracing. While the legacy arms live, the macros live in `ts/legacy` with them (lane 3.FZ froze them); `js_macros.rs` is deleted by the legacy removal lane, keyed to the owner's `pass` verdict.
- **Result:** class (d) ends with no per-component script. The page-side code is the bridge, the transport seam and the closed registry of named behaviours (root doc sections 2.2 and 4).
- **Cost:** 3.MS about 30-45 agent-hours and 3.WB about 12-16 (inferred, from the execution member's resizing; the first version said 16 and 10, the second 20 and 12), each its own deploy in the owner-measured group of the lane plan (root doc section 5.4), in the owner's order: 3.MS, then 3.WB, then the legacy removal. The freeze of the legacy arms and the `?dx-impl` switch are **lane 3.FZ**, which comes first in that group and is not part of either.
- **Biggest risks:**
  - the quality of Safari 27's new scroll anchoring and Chromium's heuristics under token-rate streaming (the Rust invariant plus `stickToEnd` is the answer);
  - desktop latency for C5, which cannot be measured in this container (no gtk) and needs a machine with WebView2 or WKWebView.

---

## 1. Support policy: per-feature floors and a boot probe

The root doc's section 2.1 no longer says "Baseline newly available". Baseline dates the first release of each engine, and those releases carry placement bugs, so A is used behind a floor per feature and a boot probe decides at run time; an engine that fails the probe is a fallback engine. `overflow-anchor` (web-features `overflow-anchor`, W): Chrome 56, Firefox 66, **Safari 27** (Baseline low on 2026-09-14). Its floor is Safari 27, so Safari 26 and older WKWebView run the Rust invariant plus `stickToEnd`.

The fallback is detected by `@supports (overflow-anchor: auto)` / `CSS.supports`, which cannot detect a *buggy* implementation. The boot probe (a growth step while following, a prepend in free mode, a first overflow from offset 0) and the per-engine run in section 2.5 are part of the lane.

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
   - Arming may land a frame late on desktop; with the invariant in Rust that costs a frame, never a drift. **Release is not late**: the detach is page-owned (`stickToEnd`), so a user who scrolls away from the end is never pulled back by a pending Rust mode.
   - **The same-frame write is the `stickToEnd` behaviour on every engine** (root doc section 2.2, registry): while the Rust-published `data-mode` is `following-bottom`, a ResizeObserver callback sets `scrollTop = scrollHeight` in the frame the content grew and sends the mode transitions to Rust. Rust keeps the mode machine, the intent, the opening position and the invariant above, which stays the backstop for the cases that neither A nor `stickToEnd` covers (a tab return). Its effect (a `scrollTop` write, the `scrollTo` effect; `sameFrameWrite` is a deadline class, not an effect) is admitted by its witness (a write delayed by R's modelled latency must paint a wrong frame, read by `wrongPaint`, run under fallback emulation so that native anchoring does not mask it), and it is the deadline table's "before paint" row, not R: a lagging write here is a wrong paint. Its detach is a transition of the scroller's `scrollOwner`.
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

**Every engine runs the invariant plus `stickToEnd`; on an engine without verified `overflow-anchor` (Safari 26 and older, the matching WKWebView, WebKitGTK unverified, an engine failing the boot probe) `stickToEnd` is the only same-frame write:** the ResizeObserver callback writes the follow in the same frame, and Rust writes `scroll(end)` while following (through `stickToEnd`, carrying the epoch) and corrects a prepend by the before/after comparison as the backstop. On web the Rust path alone would also be frame-exact (the observer callback and the Rust handler share the frame through microtasks; `MountedData::scroll` is a synchronous `Element.scroll` on web, audit section 2, V); on desktop it is IPC out, a Rust task that cannot run while a render is unacknowledged, and an eval back, at least 1-2 frames per growth step (I), so at token rate the newest line paints below the fold first and the acceptance row "follow error at most 1 px over the JS arm" cannot be met by Rust alone. That is why the write has a behaviour (web-verified; desktop inferred).

**Both arms ship until the owner has measured.** **Lane 3.FZ** (the first support lane of the owner-measured group) freezes the legacy controller (`message_scroller.js`) into `primitives/src/ts/legacy/`, wrapped and behaviour-neutral (not literally byte for byte: it relies on injected globals and a top-level `return`), behind the non-default `legacy-impl` feature that only the preview enables, and `?dx-impl=js|rust` (or a `localStorage` key) picks the arm once after hydration, never at render (root doc section 4). The legacy file is a frozen literal with its row, removed when the owner's measurement verdict for this port is `pass` (a `fail` deletes nothing); `message_scroller.js`'s baseline row in the volume gate carries the same condition. No calendar date.

### 2.3 What it subsumes

- the 1,065-line controller;
- `message_scroller.js` as a standalone file;
- the `include_str!` (`message_scroller.rs:148`);
- the hand-rolled rAF polling (`OPEN_POLL_FRAMES`);
- the ResizeObserver and MutationObserver in JS (`:906-924`).

The content-change detection becomes Rust's own render knowledge: Rust renders the rows, so it knows appends and prepends without a MutationObserver (I; only for rows rendered by `MessageScrollerRows`, section 2.2 item 9).

### 2.4 Cost

Lane **3.MS is L, about 30-45 agent-hours** (I; the first version said 16, the second 20; the execution member's resizing: a 1,065-line controller with a mode machine, seven new scenarios and a 26 min spec pass, the most uncertain lane, so it is time-boxed with the TypeScript-module exit). The freeze and the switch are **not** in it: lane 3.FZ does them first. It owns `message_scroller.rs`, the message-scroller CSS in `preview/src/components/message_scroller/` and `ts/behaviours/stick_to_end.ts`; the frozen `message_scroller.js` is deleted by the legacy removal. It runs **last and alone** (batch 4 under scope 1, the measured group otherwise), after 3.VS has built `settle`. Its order:
1. the CSS anchoring rules and the sentinel;
2. the Rust mode machine plus unit tests, and the always-on invariant;
3. `stickToEnd` (the module, its vectors and its witness; no probe selects it, it runs on every engine);
4. G opening position;
5. `settle` adoption;
6. the Rust-plus-`stickToEnd` arm without anchoring (`overflow-anchor: none !important` forced run, where `stickToEnd` is the only same-frame write);
7. visibility and keep-rendered.

### 2.5 Falsification attempts and remaining risk

| attempt | result |
|---|---|
| Does native anchoring follow growth *inside* the last row (token streaming)? | Yes in principle: the sentinel after the row moves, and anchoring compensates (I). **Must be shown** by a streaming spec that goes through Rust on Chromium, and by a probe on Firefox and Safari 27. Today's `message-scroller-timing` clones DOM nodes from page JavaScript and never starts at offset 0 or at a fractional DPR. |
| The first overflow of an empty chat | The browser anchors nothing at offset 0; **the Rust invariant covers it** (the sentinel is observed, and Rust scrolls to the end when it is not intersecting). New spec. |
| One slipped frame (late arm, fractional DPR/zoom, tab return) | Clipped nodes are never re-picked by the browser; the invariant re-scrolls. New specs: DPR 1.25/1.5, zoom 110/125%, a background-tab return, rearm after a slip. |
| Chromium anchoring suppression (back-and-forth adjustments; a computed-size change on the path) | Streaming adjusts in one direction; the turn spacer is a suppression trigger in the window after a turn starts; the invariant covers both. Measure thousands of adjustments per minute at token rate. |
| content-visibility chunks | Skipped subtrees cannot be anchors (I). The sentinel is never skipped. In free mode the first visible row is rendered, so prepend anchoring works. |
| Safari 27's anchoring is new | `@supports` cannot see bugs. A `data-dx-anchoring="js"` override turns the CSS accelerator off per engine if a defect is found, leaving the invariant plus `stickToEnd`, which run on every engine; the boot probe and a per-engine run are part of the lane. |
| Upstream tracking (a port of shadcn's controller) | The Rust state machine mirrors theirs function by function: re-porting is TypeScript to Rust instead of TypeScript to TypeScript, a maintenance cost, not a blocker. |
| Desktop latency of mode flips | A late arm delays the start of following by a frame; the invariant keeps the end visible. The release is page-owned, and the follow itself is the same-frame write of `stickToEnd` or of A, not a Rust round trip. |
| Wheel-up while Rust is stalled (a 30 ms render, a busy main loop) | With a Rust-owned release, the page keeps writing `scrollTop = scrollHeight` for the frames until Rust answers and the user is pulled back down. With the page-owned, level-decided detach the first user scroll away from the end (wheel-up, touch move, key, scrollbar drag, find-in-page) leaves following in that frame. Vector: a spec that stalls Rust (bridge delay N = 2, plus a 30 ms synthetic render) and sends a wheel-up while the content grows at token rate; the end sentinel must stay out of view and `scrollTop` must not move down again. |

**Specs:**
- `message_scroller`, `message-scroller-api`, `message-scroller-timing` (rewritten to stream through Rust), `chat-*`;
- the same suite with `overflow-anchor: none !important` injected to force the invariant-plus-`stickToEnd` path (scripted at token rate through Rust);
- the new cases above, RTL, reduced-motion `smooth`;
- the bridge-delay setting at N = 0 and N = 2, and a Firefox and WebKit probe.

**Remaining risk:** high (chat UX). The exit if anchoring misbehaves in an engine is that engine's invariant-plus-`stickToEnd` path with A off (about 40 lines of page-side code on top of the Rust invariant, already running everywhere), never a revert to the 1,065-line controller.

---

## 3. C5, the carousel wheel band → R (classifier in Rust), measured, with a named-behaviour fallback

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

### 3.4 The exit: a named behaviour admitted by a `lag` or `order` verdict, not an allowlisted module and not page-side wasm

C5 is passive, so the Rust port adds per-event IPC on desktop, a desktop measurement nobody can run here, and removes no line of logic (about 477 lines of JavaScript heuristics become about 477 lines of Rust); the simpler-alternative member's position is that C5 stay page-side ("passive stream heuristic, Rust only for policy"), with the replay proof run as `bun test` against it. That is a judgement call against the owner's "Rust first", flagged as simpler, not as wrong. The decision is therefore: **attempt Rust, measure, and fall back**. If the measurement in 3.5 fails on lag, parity or integrity, the fallback is the `wheelBand` named behaviour: it is admitted only by the owner's `fail` verdict with reason `lag` or `order` (`research/measure/c5/verdict.json`, the next-frame row's counterfactual witness measured on hardware, root doc section 2.2), it is **promoted, not removed**: the Rust arm stays as the reference and the recorded replay is its parity test. There is no allowlisted-module category (round 3). The batched `sample` mode is dropped if nothing else uses it.

**Page-side wasm (C2, the same crate compiled to a tiny module) is no longer the exit** (I): it adds a second wasm artefact with a committed binary and a reproducibility gate (like `check-js-bundle.sh`), an async compile at install (Chrome refuses synchronous `WebAssembly.Module` over 4 KB on the main thread, W web.dev), either two paths (a doubled test matrix) or a page kernel on every renderer, and it still needs JavaScript glue to marshal every event. It would be the one construction that is Rust-authored *and* frame-exact on desktop, so it is reconsidered only if a named behaviour is itself rejected.

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

**Fail on lag, parity or integrity** → `verdict.json` records the reason. Only `lag` or `order` admits `wheelBand`; `parity` or `integrity` means the Rust arm is fixed first (a bridge bug is the first suspect) and the legacy arm stays frozen. This cannot run in this container (no gtk/webkit2gtk); it needs a person with a desktop machine ([row 443](../backlog/443-owner-measurement-kit-perf-recorder-and-impl-switch.md) is the kit). Until it has run, the legacy arm ships next to the Rust arm behind the `legacy-impl` feature, and the Rust engine is the preview default.

### 3.6 Cost, falsification, risk

- **Lane 3.WB, about 12-16 agent-hours, size M** (I; the first version said 10, the second 12; the execution member: the best-specified lane, because an oracle already exists in `wheel-replay.ts`, `carousel-wheel-replay.spec.ts` and the recordings, and only the property test and the phase fix are new):
  - classifier port, 3 h;
  - expected-output capture, `cargo test` parity and the property test, 3 h;
  - bridge `sample` wiring (flush per event task, armed `until`), CSS variable and home transition, 2 h;
  - debug telemetry kept bit-compatible for the three specs, the "Rust busy" scenario, the harness phase fix and C4's bounce lines in `drag.rs`, 3-4 h (the legacy freeze and the `?dx-impl` arm are lane 3.FZ's);
  - slack for a red, 1-4 h;
  - desktop measurement, 1 h of a person's time (the owner's).
- **Files:** `carousel/wheel.rs`, a new `carousel/wheel_band.rs`, the bounce lines of `carousel/drag.rs` (the handoff with C4 needs both sides edited, because 3.CA has already merged), plus the fixture. 3.WB runs after 3.CA (the true-end function and the transform channel) and after 3.MS (the measured group's third item).

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
  - **`js_macros.rs` is deleted by the legacy removal lane**, with `ts/legacy/`, once the owner's `pass` verdict for the port exists, and `ssr_tests.rs:412` is rewritten as a Rust unit test of the true-end function (about 2 h with the other legacy removals). A main-loop deletion right after the ports would break the legacy arm.

**Subsumes:** the `concat!` construction the audit flagged as needing "its own plan".

---

## 5. Revised totals and lanes

**Class (d), 22 blocks, 3,894 lines (I; the classification is the root doc's, section 3):**

| construction | blocks | ids | lines |
|---|---:|---|---:|
| A, with a Rust invariant or verification behind it (W8 also takes `stickToEnd`) | 3 | T8, T11, W8 | 1,638 |
| Named behaviours (page-side TypeScript, closed registry) | 7 | S11; O5, C4; W3, W5, W7; C2 | 880 (+ about 40 for `stickToEnd`) |
| R: rsx handlers or plain bridge verbs plus Rust | 6 | L30, W1, C3, C5 (measured, TypeScript fallback), C1 (dissolves), S14 | 1,153 |
| G: Rust behind a paint-safe gate | 3 | S1, O3, V19 | 172 |
| Spliced `const` text | 2 | T9, T10 | 39 |
| The transport seam | 1 | V11 | 12 |

**Lanes (root doc sections 5.0 and 5.4 hold the plan).** W8, C5 and C1 are in the **owner-measured group** of the scope decision (section 5.0): under the default scope (scope 2) they are ported last and one at a time, each after its own owner measurement, and as backlog rows by yield rather than as a fixed phase. Phase 0 constructs the bridge seam and the two proof ports; lane 3.FZ, the first support lane of the measured group, freezes the legacy arms. Where the full port is chosen, 3.CA adds the Rust arm of C2 to C4 next to the frozen legacy arm, and then, **one at a time, each its own deploy, the next starting only after the owner's measurement**:
1. **3.MS** message scroller ([row 465](../backlog/465-port-3-ms-message-scroller-sticktoend-and-native-anchoring.md)), about 30-45 h (section 2.4), time-boxed with the exit that keeps the invariant plus `stickToEnd`, after 3.VS has built `scrollOwner`;
2. **3.WB** wheel band ([row 466](../backlog/466-port-3-wb-wheel-band-pure-rust-classifier.md)), about 12-16 h (section 3.6);
3. the **legacy removal** ([row 467](../backlog/467-port-legacy-removal-keyed-to-the-verdict.md)), about 2 h (`js_macros.rs`, `ts/legacy/`, the volume-gate rows), keyed to the owner's `pass` verdict.

(The first version of this document and of the root doc's lane table showed 3.CA, 3.WB and 3.MS in one parallel batch; the order above is the decided one. The execution member's alternative, building the three Rust arms in parallel behind the switch and deploying once, stays an option for the owner: it changes calendar time, not the design. 3.CA runs earlier, in the same group, and keeps C4's bounce on inline `transform` until 3.WB lands.)

**Bridge additions this needs:**
- `sample` must support *batched* event arrays plus per-event page metrics, flushed per event task and armed only while a gesture holds (the root doc's section 5.2);
- a `set` verb for `--dx-*` CSS variables on elements whose style Rust does not own;
- the `stickToEnd` behaviour (with its vectors and its witness) and the closed-table rows `location.search` and `localStorage` for the `?dx-impl` read, each added by the first row that needs it with its grant; `sample` and `subscribe` are latest-wins per frame, and a batched mode carries a cap and a drop-oldest rule that records the loss.

**Gate (root doc section 5.3):**
- No JavaScript in a `.rs` file outside the transport seam and the class-(c) allowlist; `message_scroller.js` and every `concat!`-built carousel script leave the class-(d) list when their lanes land.
- The legacy arms (`ts/legacy/*`, behind `legacy-impl`) are frozen literals with a row number, removed when the owner's `verdict.json` for the port says `pass` (planned: the gate fails when a `pass` verdict exists and the legacy file is still tracked; a `fail` deletes nothing; no calendar date); the volume gate's `message_scroller.js` row carries the same condition. The shipped (feature-off) arm gets its own clippy and test arm, because `legacy-impl` unifies on under `--workspace`.

## 6. Not verified

- Scroll anchoring behaviour (sentinel follow, streaming rate, Safari 27 quality, the first-overflow and suppression cases) is inferred from the spec's model and the well-known technique. No engine was probed, and Chromium is the only engine installed here.
- The `min-height: calc(var(--dx-turn-top) + 100%)` turn hold assumes a definite viewport height, which was not checked against every consumer layout.
- The C5 parity method depends on capturing expected outputs from the shipped JavaScript first. The capture script does not exist yet, and the recordings carry no real timestamps (3.3).
- All hour figures are estimates. The desktop measurement needs hardware this container does not have; every desktop statement here is web-verified; desktop inferred.
