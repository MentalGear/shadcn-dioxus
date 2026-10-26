# Board lens: ux-a11y-perf (row 434 step 3), 2026-10-26

**Verdict: HOLDS WITH CHANGES.**

- The plan's decisions survive on web: the screen-reader, focus, forced-colors and keyboard-repeat attacks mostly failed. What it does not survive is its desktop story and two "by construction" claims.
- **G is not a fallback.** The placed-side, arrow-offset and available-height publication runs on every open on every engine, so G is on the critical path of every popover, menu, select and combobox (F2). On desktop that is an estimated 2-4 frames per open.
- **A does not make W8 follow-bottom correct by construction.** The CSS Scroll Anchoring spec selects no anchor at scroll offset 0, and an anchor that slips out of view is never re-picked. Rust's corrective write must stay always-on, with A as an accelerator (F1).
- **Rust `onscroll`/`onresize`/`onpointermove` are blocking synchronous XHRs on desktop** (V, `native.ts`). Several lanes lean on them (F3).
- **Gesture loops in Rust couple feel to Rust-side load.** Batching C5's wheel samples from rAF adds avoidable latency, and `wheel-replay.ts` would sample the Rust arm one frame stale (F4).
- **The perf oracle cannot see any of the three last ports.** `main-thread.spec.ts` has no carousel, message-scroller or streaming subject, and its 50/200 ms bounds are about 20-100x too coarse. A per-port protocol and a feature-gated A/B toggle are in section 3.
- Eleven findings: 3 major plus 2 major-leaning on the engine paths, the rest minor. No blocker on web. Desktop and iOS-26 are where the unmeasured risk sits.

Evidence labels: V = verified in repo or crate source, W = web source, I = inference.

---

## 1. Findings

### F1. A does not make follow-bottom correct by construction (major)
**Attacks:** residue doc section 2.2 item 1 and section 2.5 ("the sentinel is the only anchor candidate ... frame-exact"; Rust fallback "only for engines without `overflow-anchor`").

**Evidence.**
- W (CSS Scroll Anchoring, <https://drafts.csswg.org/css-scroll-anchoring-1/>): an anchor is selected only if the scroller is "not scrolled away from the origin of its scrolling area"; "scroll offset of the scrollable element being zero" is also a suppression trigger. Fully clipped nodes are skipped.
- I, three consequences:
  - **First overflow.** An empty or short chat is at offset 0, and the first long answer grows past the fold. The browser anchors nothing, the sentinel falls below the viewport, and the mode stays `following-bottom` while the stream runs off-screen.
  - **One slipped frame is permanent.** Any frame where the sentinel is not intersecting the viewport (late arming on desktop, fractional DPR/zoom where the 1 px sentinel sits half a pixel off, a tab returning from background) makes it a non-candidate, because fully clipped nodes are skipped. Nothing re-anchors until a Rust `scroll` to end.
  - **Suppression triggers.** A change of computed `height`/`margin`/`padding`/`transform`/`top` on any element from the anchor up to the scroller cancels the queued adjustment. The plan's `min-height: calc(var(--dx-turn-top) + 100%)` on the content is exactly such a change, in the window right after an anchored turn starts.
- V (`message-scroller-timing.spec.ts`): the only streaming spec clones DOM nodes from page JS and watches `distance <= 1`. It never starts at offset 0 and never uses fractional DPR.

**Change.** State the invariant and enforce it in Rust, always on, independent of engine support:
- `mode == following-bottom` implies the sentinel intersects the viewport within one frame, or Rust writes `scroll(end)`.
- Source of truth: `observe(IntersectionObserver, sentinel)` through the bridge, plus the content `onresize`. Both are idempotent with native anchoring, since both target the bottom.
- Rust's prepend correction must compare an anchor row's top before and after layout, as the current JS does (`:762-776`), not branch on `CSS.supports`. Otherwise Safari 27 plus the Rust fallback double-corrects.
- Add specs: empty-then-first-overflow; DPR 1.25/1.5 and zoom 110/125%; background-tab return; rearm after a slip.

**Subsumes:** the whole "native anchoring slipped" class: Safari 27's unknown bugs (weeks old), Chromium suppression heuristics, fractional pixels. A becomes a pure accelerator with a measured, always-on safety net, which is what "frame-exact on web" needs anyway.

### F2. G is on every popup open's critical path on every engine (major)
**Attacks:** root doc section 2.3 ("Latency: zero frames on web ... one to three IPC round trips on desktop") and the T8 row ("A ... plus a G fallback"); the "A is primary, G is the fallback" framing.

**Evidence.**
- V (`anchor_fallback.rs:693-709`): the engine publishes `data-placed-side`, `--dx-anchor-offset-x/-y` and `--dx-available-*` at every exit, "CSS-anchor path and JS fallback alike". The tooltip arrow and the Select's `transform-origin` read them (`tooltip/style.css:111-155`, `select/style.css:197-201`).
- V (root doc section 2.1 limit 2, and the T8 falsification (2)): CSS cannot publish the placed side. Anchored container queries are Chrome-only. So Rust must measure two rects after every open, even when A placed the panel perfectly, and it gates the enter animation until it has (section 2.3).
- I: on desktop the chain after a click is vdom handler (blocking XHR), then edits to the webview, then `showPopover`, then an async rect query, then compute, then an edit batch back. That is about 2-4 frames (33-66 ms at 60 Hz) added to every select, combobox, dropdown, menubar and context menu open, on the engine where A already did the work. Tooltips and hover cards hide it behind their delay. Context menus and selects will feel it: a native menu is instant.
- I: the plan leaves the fail-open timeout unspecified, and that decides the trade. A timeout shorter than the IPC makes the gate meaningless; a long one is a visible hang.

**Change.** Split G into two tiers by what a wrong paint costs.
1. **Placement-critical gate.** Only where Rust owns the position: engines without anchor positioning (the current fallback), and the iOS-keyboard re-clamp. Keep `opacity: 0` until placed. Fail-open at about 100 ms.
2. **Decoration tier, never gated.** Side, arrow offset, `transform-origin` and available-height. The panel opens at once with the existing neutral defaults and updates when Rust answers. The precedent is already in the repo: the tooltip's `:not([data-placed-side])[data-side=...]` fallback.
   - Make the enter animation origin-independent for the first frames (opacity-first, or `transform-origin` from `anchor-center` where supported), so a late origin never shows as a wrong-corner pop.
   - If a late arrow or origin flicker remains, accept one visible correction over a 66 ms stall.
3. **Budget in the gate.** `check-bridge-rules` should require every G gate to declare a timeout of at most 100 ms and a tier. A first-frame spec asserts the gate is `opacity: 0` while pending, and that the panel is visible by the timeout with the slow channel stuck at `N = never`.

**Subsumes:** every "gate is on the hot path" regression (S1, T11, O3, V19 reuse the tiering) and the unspecified timeout.

### F3. Vdom `onscroll`/`onresize`/`onpointermove` is a blocking synchronous XHR on desktop (major)
**Attacks:** residue doc section 2.2 item 6 ("Fed by Dioxus `onscroll`"), the W8 fallback ("frame-exact ... from Dioxus `onresize`"), W1/lane 2D, S2, O3 ("Dioxus `onresize` is a ResizeObserver").

**Evidence.**
- V (`dioxus-interpreter-js-0.7.9/src/ts/native.ts:353-395, 568-593`): on desktop and mobile every Dioxus event runs `xhr.open("POST", endpoint, false)`, a blocking sync XHR on the webview main thread, with a base64 `dioxus-data` header (Android: 16 KB per header, 32 KB max).
- I: scroll and resize events arrive at 60-120 per second per active scroller. During token streaming `onresize` fires about once per frame. At an estimated 1-5 ms per round trip (unmeasured) that is 6-60% of the webview main thread, and every blocked millisecond also delays rAF and layout.
- V (root doc section 1.2): the plan knows this table, but its per-block mapping still assigns high-rate events to vdom handlers. Lane 2C's measurement was web only ("Desktop ... not measured", row 434 progress).

**Change.**
- **Rule.** Events that fire at frame rate (`scroll`, `resize` observers, `pointermove`, `wheel`, `touchmove`, `mousemove`, drag events) reach Rust only through bridge `listen`/`observe`/`sample` (async, batched, armed only while a gesture or condition holds), never through a vdom `on*` handler in `primitives/src`.
- **Gate.** Add `scripts/check-hot-events.sh`: a grep for those `on*` handlers in `primitives/src/**/*.rs` with a reasoned allowlist. It is a Rust-side sibling of `check-blocking-scroll-listeners.sh`.
- **Why one path is enough.** The bridge's `observe` on web still hands Rust the data in a microtask after the RO callback, so the "frame-exact on web" claim survives, and desktop gets the async path. This also removes the cfg split between the web and desktop arms of W8's fallback.
- **C5 in particular.** The current `listen(window, 'pointermove', ...)` (V, `carousel/wheel.rs:578`) lives for the carousel's whole mount. As a Rust `sample` it must be armed only during a wheel gesture ("until" argument), or the desktop sees 60-1000 messages per second for the page's lifetime.

**Subsumes:** the class "hot event reaches Rust synchronously on desktop", including S2/S3/W1/O3/W8 together.

### F4. Gesture-feedback loops in Rust: avoidable latency, lost isolation, and a harness that would read the Rust arm a frame stale (major for C5, minor for the rest)
**Attacks:** residue doc section 3.2 (batched per-frame `sample` sent from rAF), 3.4-3.5, and C3.

**Evidence.**
- V (`carousel/wheel.rs:404,576-578`): C5 is passive, writes a `transform` per event, and today runs entirely in the page.
- I (C5 on desktop):
  - **Waiting for rAF buys nothing.** Trackpad wheel events already arrive at about one per frame, 60-120 per second, so batching from rAF saves no IPC volume. It does add up to one frame of latency before the hop (and the hop itself is 1-2 frames).
  - **Rubber-band feedback tracks the fingers directly**, so 2-3 frames at 60 Hz (33-50 ms) is the range where a rubber band reads as sticky. At 120 Hz ProMotion it is 16-25 ms.
  - **Isolation is lost.** Today a heavy Rust render during a carousel gesture cannot stall the band, because the JS lives in the webview. With the classifier in Rust, a long app render delays the band's samples, then they all land at once and the band jumps.
- V (`playwright/wheel-replay.ts` header): the replay dispatches the events and samples the band in the same rAF callback. The Rust arm's variable is written after the bridge's rAF flush, so the replay samples one frame stale. The phase of every existing assertion (`carousel-wheel-replay`, `carousel-edge-wheel`) changes.
- V (`ts/focus-trap.ts`, scroll-lock): nothing here is affected; this is C5/C3 only.

**Change.**
- Send a wheel batch per event task (flush on a `queueMicrotask`, merging only same-task events), not per rAF. The classifier's notch and decay rules are unchanged, since every event keeps its own timestamp.
- Keep the 120 Hz and 60 Hz cases in the desktop measurement (residue doc section 3.5), and add "Rust busy for 30 ms during the gesture" as a scenario (inject a synthetic render).
- Make the harness phase explicit: the replay's sampler runs in a rAF callback registered after the bridge's, or one frame later, parameterised by an `N` that also offsets `__dxBridgeDelayFrames`.

**Subsumes:** C5's decision rule (the exit is a measured p95, not "if it feels bad") and the same class for C3's pin tick and toast/drawer gesture streams.

### F5. The delegated B1 interpreter must not become a permanent non-passive wheel/touchmove listener (major)
**Attacks:** "`data-dx-rule*` read by one delegated capture-phase interpreter" (row 434 Step 3 adopted, best-practice fold-in).

**Evidence.**
- V (`scroll_lock.rs:367-453`): non-passive `wheel`/`touchmove` exist only while a lock is held, removed with it (rows 149/159). Any non-passive wheel/touch listener makes every scroll tick wait for the main thread.
- V (`scroll-main-thread.spec.ts` test 3): only `#main`'s listeners are asked. The `window`/`document`/`<html>`/`<body>` half is `listener-inventory.spec.ts` (V, header): the multiset before the first open must equal the multiset after, with no warm-up, so a lazily installed and then kept listener fails.
- I: a "delegated capture interpreter" invites registering one window listener per rule type at load, including `wheel` and `touchmove` with `passive: false`.

**Change.** Interpreter contract, enforced by `check-bridge-rules`:
- A cancelling listener type (`wheel`, `touchmove`, `touchstart`, `mousewheel`) is registered only while at least one installed rule of that type needs `prevent`, and removed with the last one. Other types (`keydown`, `click`, `pointerdown`) may stay for the page.
- Passive-capable rules register passive.
- The inventory spec runs on the interpreter-owned lock path; `scroll-main-thread` test 3 extends to window.
- No `stop` effect on window-capture rules. Desktop never forwards `stopPropagation` (V, root doc section 1.2), and a window-capture `stop` also starves Rust's own handlers on web.

**Subsumes:** the class "blocking scroll listener" for every future B1 rule.

### F6. Fail-open exists for G only; B1 "prevent, then Rust acts" can trap keyboard users (minor, WCAG 2.1.2)
**Attacks:** "G gates fail open"; B1 `always` rules for `cancel` (L2), Escape (L6), F6 (O1).

**Evidence.**
- V (`lib.rs:848`, root doc B1 table): `cancel` is prevented always, so Escape closes only through Rust. If the Rust runtime is wedged (wasm trap, row 73/74 shape) or its channel is late, Escape does nothing and the dialog stays. Today's JS has the same trap, so this is not a regression, but the plan makes Rust the sole closer on more paths.
- I: the interpreter has no liveness signal.

**Change.** B1 rules with a `prevent` effect and a Rust-acts `send` carry an ack: `send` returns an id, Rust calls `ack(id)`. If an ack is outstanding more than about 1 s, the rule stops preventing until the next ack. Add one spec at `N = never`: Escape on an open dialog still closes it natively.

**Subsumes:** keyboard-trap risk for every B1 rule, present and future.

### F7. Pending states must be opacity-only; stale answers need an epoch (minor-to-major, a11y and correctness)
**Attacks:** G for popups, toast (O3), questionnaire (V19), S1.

**Evidence.**
- V (`toast.rs:1517-1535`): a `role="alert"` region inserted with its content is announced; flipping a live region is not reliable. A gate that hides an unsized toast with `visibility`, `display`, `hidden`, `inert` or `content-visibility` would drop it from the accessibility tree and lose the announcement.
- V (`modality.rs:108-165`): focus goes into the panel at show time, before placement. `opacity: 0` does not block focus; `visibility: hidden` would. The plan correctly says opacity here (root doc section 2.3). The same must hold for toast and V19.
- I (desktop): a late answer to open #1 arrives after close and reopen #2, and writes the wrong placement. `modal-quick-reopen.spec.ts` covers dialogs only.

**Change.**
- A rule in `check-bridge-rules`: a pending-state selector may declare `opacity`, `translate`, `pointer-events` and `animation-play-state` only.
- Every G answer carries an epoch (the open's generation); Rust drops an answer for an older epoch.
- Extend the quick-reopen spec to popups and select at `N = 2`.
- Panels exposed while pending: VoiceOver on iOS/macOS reads the panel from its frame when it receives focus, and a frame that changes a few frames later may leave the VO cursor box stale (I, one frame on web, up to the gate on desktop). Add it to the manual checklist, nothing else.

**Subsumes:** the class "unannounced or double-announced pending content" and "stale async answer".

### F8. Reduced motion: only the duration is reduced; JS-driven animation must read the preference itself (minor)
**Attacks:** B5 `scroll-step(smooth)`, C2/C5 `matchMedia` verb as a subscription, `linear()` spring (C4), G timeout.

**Evidence.**
- V (`dx-components-theme.css:1785-1790`): the floating recipe reduces `animation-duration`, it does not remove the animation. A pending state that sets `animation: none` or a fail-open built on a long `animation-delay` interacts with that.
- I: `use_media_query` as an async subscription has a first-answer gap (one IPC on desktop). If its default is "no preference", an autoplay carousel or a smooth `scrollBy` animates once for a user who asked for none.

**Change.**
- The interpreter reads `matchMedia('(prefers-reduced-motion: reduce)')` synchronously at execution time for every animated verb (B5 smooth, C2, spring).
- `use_media_query` defaults to the safe value (reduce) until the first answer.
- The gate's fail-open timeout is a CSS animation whose `animation-delay` the reduced-motion rule does not touch.
- Add a reduced-motion pass to each port's checklist.

### F9. Zoom, RTL and iOS keyboard: three concrete holes in A (minor)
**Attacks:** T8/T11 under A; sentinel; row 210.

**Evidence.**
- I (zoom): the sentinel is 1 px; at browser zoom 110/125% and Windows DPR 1.25/1.5 `scrollTop` is fractional (F1). For popups at 400% zoom (a 320 x 256 CSS-px viewport, WCAG 1.4.10), `max-height: 100%` of a `position-area` region beside the anchor can be about 100 px. The JS path today clamps to the whole visual viewport (V, `anchor_fallback.rs` size clamp), so the menu is smaller under A than today.
- V (root doc T11 row): the context-menu mapping is `position-area: bottom span-right`. That is physical, so an RTL page opens rightward from the pointer where a native RTL menu opens leftward. Logical `span-inline-end` resolves against the containing block's writing mode, so a subtree-only `dir="rtl"` (the preview's RTL toggle may set it below `<html>`) would still differ. I.
- W/I (iOS, row 210): V (`anchor_fallback.rs:716-735`) hands any placement that is outside the visual viewport or overlaps the anchor to the JS path synchronously. Under A the first frame is placed against the layout viewport, which the keyboard does not shrink, so the list can sit under the keyboard until the `visualViewport` `resize` plus a Rust republish arrive. It cannot cover the input (the position-area region excludes the anchor), so row 210's invariant holds, but "list visible" does not for that window.

**Change.**
- Add three geometry cases to the A-only run: 320 x 256 at 400% zoom, an RTL subtree, and a keyboard-reduced `visualViewport` with an open combobox.
- Use `span-inline-end` for T11 and drive RTL from the same `use_direction` signal the carousel uses.
- Keep the placement-critical gate (F2 tier 1) for the keyboard-clamp case, since that is the only A case where Rust owns the position.

### F10. B2 generic recogniser: touch-action and pointercancel are part of the rule (minor)
**Attacks:** B2 for L30/O5/C4/L21/V17; the "scroll guard" parameter.

**Evidence.**
- I: for touch, once the browser has started a native pan it fires `pointercancel`, and no later decision can take the gesture back. Whether the recogniser can own a gesture is decided by `touch-action` in CSS before touchstart (V, `scroll-main-thread` test 4: the slider/handle/color thumbs rely on `touch-action: none`). A recogniser described only by slop, axis lock and a scroll guard has no `touch-action` input.
- I: the drawer, toast swipe and photo-wall pan were each tuned (slop, velocity window, flick threshold). A generic parameter set tends to flatten those differences; the plan defends the numbers only for C5 (parity test).
- I (feel): a wrong touch-action on iOS shows as "page scrolls under my drawer drag"; a velocity window off by one sample shows as a drawer that snaps shut after a slow release.

**Change.** Make the rule publish its `touch-action` (the interpreter checks the computed value and the rule's axis agree, and fails the gate's spec otherwise). Record per-component baselines before the port: slop, release velocity and snap decision for a fixed set of scripted swipes (F11), so the port is a parity proof and not a hand-check.

### F11. The release suite cannot see these ports (major for the owner's plan)
**Attacks:** "timing-sensitive paths compare long tasks before and after" (row 434 item 2); the slow-channel harness.

**Evidence.**
- V (`main-thread.spec.ts`, subject list, lines 204-799): no carousel, message-scroller, toast-stream, drawer-fling or context-menu-latency subject. The long-task bound is 50 ms and the interaction bound 200 ms (`main-thread.ts:99-105`). A per-frame 1-3 ms regression, or a 2-frame lag, passes silently. Chromium only.
- V (`frame-cost.ts`, row 402): the repo already has the right design, a relation between two paths measured in one run and one page, which is exactly what an A/B toggle enables.
- V (root doc section 5.6): the slow-channel harness defers messages by `N` rAFs. It models neither the blocking XHR of F3 nor Rust-side busy time (F4).

**Change.** See section 3. The short version: relation specs per port using `frameCostMeter` on the toggle's two arms, a harness that can also spin 1-3 ms per vdom event to emulate F3, and a bridge round-trip histogram.

---

## 2. Per-construction answers to the brief

| | focus order and initial focus | SR announcements while gated | aria-live | keyboard repeat | reduced motion | forced colors | zoom / RTL / iOS keyboard |
|---|---|---|---|---|---|---|---|
| **A** | unchanged, CSS only | none | none | none | none | none | F9: 400% zoom room, RTL `span-right`, keyboard window |
| **G** | focus into an `opacity: 0` panel works (V `modality.rs`, no visibility check); touch policy unchanged; F7 for VO frames | panel exposed in the tree while pending, benign; toast alert must stay opacity-only (F7) | toast region insertion announced once if the region node is unchanged | none | F8: reduced rule only shortens duration | opacity unaffected; the forced `:focus-visible` outline (V theme 2464-2484) is invisible at opacity 0, fine | keyboard-clamp gate stays (F9) |
| **B** | focus-trap becomes an effect; capture ordering with the modality writer (registered first) holds unless a rule uses `stop` (F5) | none | none | `repeat` predicate exists; L1's 250 ms window is state the DSL lacks (small `recent()` predicate) | none | none | RTL handled in the L11 walk (V) |
| **R** | none | token streaming: only if rows keep their DOM identity inside `role="log"` (F1/F7: sentinel `aria-hidden`; no re-parenting) | `role="log"` `aria-relevant="additions"` (V `message_scroller.rs:745-766`) must not see re-added rows | none | F8 | none | F3 desktop sync XHR; iOS 26 runs the Rust fallback, not A |

Two items the table hides:
- **Consumer-composed rows.** The residue doc claims Rust knows appends and prepends without a MutationObserver (section 2.3). That holds for `MessageScrollerRows` only. Rows passed as `children: Element` are unknown to Rust, so their growth reaches the controller only through `onresize` (F3's bridge `observe`) and native anchoring.
- **The timing spec measures something else after the port.** `message-scroller-timing.spec.ts` appends cloned DOM nodes from page JS (V). Rust never sees them, so it stops exercising the new controller. It must stream through Rust.

### Perceived performance, counted

**Events per second, upper bounds (I, from input hardware and the frame-aligned dispatch of Chromium; not measured here):**

| source | rate | if bridged | cost note |
|---|---|---|---|
| trackpad wheel (macOS, momentum included) | 60-120/s for 1-2 s per flick | one message per event (F4), 150-400 B | under 50 kB/s; fine |
| notched mouse wheel | up to about 30/s | same | trivial |
| `pointermove` | frame-rate in Chromium, can be higher in others | only while a gesture is armed | window listener for the carousel's lifetime (V `wheel.rs:578`) is the hazard |
| `scroll` | 60-120/s per active scroller | `observe`/`listen` coalesced per frame | vdom `onscroll` on desktop is a blocking XHR each (F3) |
| ResizeObserver | at most once per frame | same | token streaming makes it about once per frame |
| `keydown` repeat | about 12-33/s after a 250-500 ms delay | B1 evaluates locally; `send` only when a rule says so | trivial |

- **Interpreter cost per event.** A capture listener on `window` is about 5-20 µs plus predicate work. It is cheap if rules are indexed by `(type, key|selector)` and `closest(sel)` is the only DOM read. The L11 `can-scroll` walk calls `getComputedStyle` per ancestor per wheel event, forcing a style recalc if the DOM is dirty. That is today's cost, but only while a lock is held, so keep it that way (F5).
- **IPC volume.** Not the problem. Latency and blocking are (F2, F3, F4).
- **Net perf upside the plan under-sells.** V (root doc section 1.2): web `document::eval` builds a `new Function` from the whole script text on every mount, including the 400-line carousel scripts and the 1,065-line message scroller. One resident bridge module removes that compile from every mount. Count it in the "after" numbers.

### Interaction feel, per port

- **Message scroller under token streaming.** On Chromium and Firefox (anchoring since 56/66) following is native and frame-exact once the sentinel intersects. It feels better than JS correction if F1's invariant holds, and drifts invisibly off the bottom if it does not. On Safari 27 it is a weeks-old implementation (W: Baseline newly available 2026-09-14); no data on streaming at 50 tokens per second, so budget a probe lane. iPhone users on iOS 26 (most, for months) run the Rust fallback: frame-exact on web via the RO microtask, so they should feel today's behaviour. Wheel up during a stream: the sentinel leaves the viewport and anchoring releases by itself, which is good; the late part is rearm (F1).
- **Carousel trackpad momentum through a Rust classifier.** On web the band is drawn in the same frame as today (Chromium aligns wheel dispatch to the frame; elsewhere rAF-batching lands in the same vsync that today's write would). On desktop it trails the fingers by 1-3 frames and stalls under Rust load (F4). What can go wrong: a sticky or laggy edge pull, a band that snaps catch-up after a Rust render, a release a frame late (harmless), and a replay harness that reads a stale frame (F4).
- **Drawer and swipe through a generic recogniser.** Same-frame on web. Risks are tuned-parameter drift and `touch-action` (F10). A drawer fling on a phone is the most sensitive: the snap decision depends on the velocity window.
- **Popups on desktop.** F2: a 2-4 frame open delay that tooltips hide and menus do not.

---

## 3. How the owner should measure it

### 3.1 Common instrumentation (build once, before port 1)

Prerequisites in lane 3.0 (all behind a debug flag, `localStorage['dx-perf']`/`?dx-perf=1`, as `dx-carousel-debug` already does):
- **`window.__dxPerf`**, an in-page recorder with `start(label)`/`stop()`/`dump()` returning JSON:
  - PerformanceObserver for `longtask`, `long-animation-frame` (Chromium; script attribution), `event` (interactionId, processingStart/End, presentationTime) and `layout-shift` (hadRecentInput false, for anchoring jumps);
  - a rAF loop recording the frame-delta histogram (dropped frame = delta over 1.5x the display's refresh);
  - scroll position and a caller-supplied probe value per frame.
- **Bridge stats**, `window.__dxBridgeStats`: messages and bytes each way, and a round-trip histogram (send stamp, Rust handler, applied stamp). That one number says "frames of lateness" on any device, including the iPhone, and is about 0 on web by construction.
- **Delay emulation**: `?dx-bridge-delay=N` (the plan's `__dxBridgeDelayFrames`) plus a `?dx-bridge-spin=ms` that burns 1-3 ms per bridge message and per vdom event (models F3). With these the owner can feel the desktop cost on his own Mac trackpad in Chrome or Safari, without a desktop build.
- **Chrome trace**: a CDP `Tracing.start` helper with `devtools.timeline`, `disabled-by-default-devtools.timeline.frame`, `input.scrolling`, `latencyInfo` and `benchmark`, saving the `.json.gz` into `research/measure/<port>/<arm>-<device>.json.gz`.
- **Method**: release build only (CLAUDE.md), arms interleaved ABAB, at least 10 gestures per arm, report median and p95 with `calibrationMs` (V `frame-cost.ts`). The same commit serves both arms, switched by the toggle (3.4), so build noise cancels.

### 3.2 Per-port protocol (owner's order)

**Port 1: message scroller (W8).**
- *Scripted (Chromium release, both arms, `N = 0` and `N = 2`):*
  - S1 empty chat, first answer grows past the fold (F1 case);
  - S2 follow a 60 s stream at 50 tokens/s, with a growing code block and a late-loading image;
  - S3 wheel up mid-stream, wait 2 s, wheel back to the bottom (rearm);
  - S4 prepend 50 rows while scrolled to row 10;
  - S5 jump to a message while neighbours render;
  - S6 zoom 125% and DPR 1.25;
  - S7 tab hidden 30 s then back.
- *Record:* distance from bottom per frame while following (max, frames over 1 px); anchor-row top drift on prepend (should be 0 +/- 0.5 px); layout-shift entries during the stream; "fight frames" after a wheel-up (frames until scrollTop is yanked back, should be 0); token-append to bottom-aligned latency; LoAF and `TaskDuration` per frame (the existing relation spec, now streaming through Rust); count of added nodes inside the `role="log"` (no re-announce).
- *Manual:* iPhone Safari on iOS 26 (fallback path) and, when available, iOS 27; Mac Safari 27, Chrome and Firefox with the trackpad: flick up during a stream and release; overscroll at the bottom; keyboard rise in a composer; rotate; VoiceOver reads a new message once, not per token; reduced motion on.
- *Pass:* follow error no worse than the JS arm (max 1 px); drift 0.5 px; no fight frames; zero long tasks; per-frame task time at most 1.1x the JS arm. Fail on any of the F1 cases is a Rust-always-on fix, never a revert of A.

**Port 2: carousel wheel band (C5).**
- *Scripted:* `cargo test` parity (already planned); in-browser `wheel-replay.ts` on both arms with the F4 phase fix, over `carousel-wheel-recordings.json`. Record: in-page input-to-paint lag (`e.timeStamp` to the first rAF whose computed band reflects it) as p50/p95/p99 in frames; release kind and time parity; peak band; lost or reordered messages (bridge stats counters).
- *Real hardware (cannot be CI):* Mac trackpad in Chrome, Safari and Firefox 156 (the tuning browser, V `wheel.rs` doc), plus one desktop-app build on macOS (WKWebView) and Windows (WebView2, precision touchpad), at 60 and 120 Hz. Capture 10 flicks per arm at an edge with the in-page logger, and for feel a 240 fps slow-motion video of the screen. Add the "Rust busy 30 ms" scenario (F4).
- *Manual checklist:* slide 1 push and release; slide N; a flick arriving at the edge with momentum (no double bounce); re-push during spring-back; notched mouse wheel; Shift+wheel; vertical orientation; RTL; looping carousel (no band); virtual carousel mid-list (no band); reduced motion (no band).
- *Pass (the plan's own, section 3.5):* p95 added lag at most 1 frame, p99 at most 2; zero lost or reordered; identical release kind. Fail on lag exits to C2. Touch devices are not affected (C5 is wheel only), so the iPhone and Android pass here is a smoke check that nothing regressed.

**Port 3: carousel macros (C1) and its users (C2/C3/C4).**
- C1 itself has no behaviour, so measure what its removal rides on:
  - *Scripted:* `carousel-edge-*`, `carousel-virtual-wheel` (reads `edgeIndex` from trace records, V), `carousel-drag-free`, `carousel-edge-overdrag`.
  - *True-end freshness:* push at the edge while a virtual window shifts, and record the frames between the shift and `data-dx-true-end-*` updating.
  - *Drag and settle:* time-to-settle and pin re-centre frames after a trackpad pan, a touch swipe (iPhone, Android Chrome) and a mouse drag release. Record the spring-back curve (`getAnimations()` keyframes for the `linear()` version against the JS rAF samples, same start and release velocity).
  - *Manual:* touch swipe snap on iPhone and Android, autoplay with `aria-live` (VoiceOver and TalkBack announce the slide once), mouse drag with rubber band at the end, trackpad pan.
- *Pass:* settle time and spring curve within one frame and 2 px of the JS arm; no new long tasks.

### 3.3 What to run in the existing suite

- Add carousel (wheel, drag), message scroller (stream) and toast (stack of 5) subjects to `main-thread.spec.ts`. They are cheap and nothing in the list covers them (F11).
- Add a relation spec per port with `frameCostMeter` on the two toggle arms: Rust arm per-frame task time at most 1.1x the legacy arm, interleaved, with `calibrationMs` reported. This is row 402's design, applied to a path comparison.
- Extend `scroll-main-thread` test 3 to `window` (F5). Budget the new specs in `playwright/durations.json` (`check-slow-spec-budget` fails without it).

### 3.4 Should the old path stay switchable for one deploy? Yes, for W8 and C5. For C1, only as a by-product.

Reason: both are feel-critical, depend on hardware CI lacks, and the plan's own exit (C2) is "if a desktop measurement fails". Without the toggle the owner cannot A/B on his own devices, and the relation specs in 3.3 need two arms on one page. C1 dissolves with its users, so it inherits whichever arm its users run.

**How to build it without breaking the gates:**
1. **Freeze first, port second.** Move the legacy JS (message scroller, C5, and the C2/C3/C4 scripts with their macros) out of Rust strings into `primitives/src/ts/legacy/*.ts` through the existing bundle chain (`bundle.manifest`, `check-js-bundle.sh`). That is a behaviour-neutral, byte-parity step and it also satisfies `check-js-in-rust.sh` (no JS in `.rs`).
2. **Feature-gate it.** A non-default cargo feature (`legacy-impl`) on `dioxus-primitives`, enabled only by the preview crate. `check-primitives-features.sh` gets the feature added to its list; `check-cargo-dep-versions` and `check-installed-paths` are unaffected because an installed app never enables it and the file is not in any `component.json`. Users of `dx components add` never ship the legacy arm.
3. **Runtime choice, made once, after hydration.** `?dx-impl=js|rust` and a `localStorage` key, read in a `use_effect` at mount, never at render. Markup is identical across arms (the sentinel and the attribute are present in both; the legacy JS ignores them), so `hydration-parity` and SSR stay green. No mid-gesture switching: the arm changes on reload.
4. **Playwright.** One project (or `addInitScript` setting the localStorage key) per arm. Run the legacy arm only for the affected specs (`test:changed` already selects by diff), so the slow-spec budget grows by those specs only. Put the budget increase in `durations.json` with the reason.
5. **Gates and expiry.** List each legacy file in `check-bridge-rules`'s allowlist (`path<TAB>reason<TAB>row`) with an `expires: YYYY-MM-DD` that fails after the date. Removal is then enforced, not remembered.
6. **Sequencing fix.** The plan deletes `carousel/js_macros.rs` and rewrites `ssr_tests.rs:412` "after both merge". With legacy arms alive for one deploy, the macros live in `ts/legacy` and the deletion moves to the expiry deploy. Otherwise the main-loop cleanup breaks the legacy arm.
7. **Never ship both to users.** The preview toggle is a preview concern only; the feature flag keeps the library's default Rust.

---

## 4. Attacks that failed

- **Focus into an `opacity: 0` pending panel** (touch-open-focus, modal-initial-focus): `dxOpenFocus` has no visibility or visible-rect test (V `modality.rs:108-165`), the touch policy focuses the panel with children inert for one call, and `preventScroll` is already set. Opacity does not block focus. The plan's choice of `opacity` over `visibility` holds. The iOS-like capture `mousedown` `preventDefault` in the spec (V `touch-open-focus.spec.ts:90-110`) does not interact with an interpreter listener unless it uses `stop` (F5).
- **Forced colors:** the theme's one `:focus-visible` outline plus `data-highlighted` rule (V `dx-components-theme.css:2464-2484`) is global, and `all-routes.spec.ts` proves it per page. Nothing in A, G, B or R paints with `box-shadow`, so no hole was found.
- **Toast live region** (`aria-live`, `role="alert"` insertion): safe if the pending state never removes the node from the accessibility tree (rule in F7). The keyed `for r in [role]` re-insertion is untouched by O3.
- **C5 "passive, so no decision is lost":** V (`wheel.rs:404,576-578`) confirms `{ passive: true }` and a doc rule that the script may never cancel. The plan's reading is right; my objections are latency and isolation, not correctness.
- **IPC volume** at 120 messages per second of 150-400 B: small. I could not make it a bandwidth or CPU problem. It is a blocking and latency problem (F3, F4).
- **Web latency of G**: the microtask chain (V root doc section 1.2: `queueMicrotask` executor, no rAF gate) was verified piece by piece by the plan. I could not break it without running code; it holds for UA-dispatched events and rendering-step callbacks. Script-dispatched events (including `wheel-replay`, F4) are the exception.
- **Interpreter overhead per event:** with rules indexed by `(type, key|selector)` it is microseconds; the only real cost is the `can-scroll` walk, which exists today and only while a lock is held.
- **Keyboard repeat:** `repeat` is a B1 predicate; Escape and Enter/Space rules can use it. The only stateful item is L1's 250 ms "cancelled recently" window, which the plan already names.
- **Drawer drag already streams to Rust** (V, root doc section 1.3): B2 adds no new IPC for the drawer.
- **Unspecified items I did not test:** WebKitGTK anchor-positioning support, and the true cost of `evaluate_script`/`ipc.postMessage` per renderer. Both need a real desktop and are in the owner's measurement list above.
