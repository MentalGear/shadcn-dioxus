# Board lens: root cause and platform timing (row 434 step 3), 2026-10-26

**Verdict: HOLDS WITH CHANGES.** The core rule holds: never ask Rust at commit time; publish the rule first, then decide locally. Dioxus's own handlers do run synchronously, and `prevent_default` reaches the native event on web and on desktop/mobile (V). Four supporting claims are wrong or incomplete. (1) The desktop timing model leaves out the renderer's own cadence. Desktop applies DOM edits inside a rAF callback, at most one render per frame, and polls no task or effect until the webview acknowledges the edits. That makes `commit()` redundant for render-to-render ordering on desktop, puts G latency at one frame or more, and lets an eval run before the edits of its own render. (2) The G gate as written fails: a running or `forwards`-filled animation outranks `opacity: 0`, and `animation-play-state: paused` also pauses the CSS fail-open. (3) The "one delegated capture-phase interpreter" changes what `defaultPrevented` means, and on window/document it either becomes passive by default or brings back the row 149/159 scroll jank. (4) The slow-channel harness delays only bridge messages, so it misses most of what makes desktop different. The root also misses more commit points: gesture arbitration at pointerdown, input routing by focus, activation windows, the compositor, and work that runs after paint. That moves `setPointerCapture` and `focus` out of the async `call` verb.

Labels: **V** = read in this repo (`/home/user/wt/b28`) or in the pinned crate sources (`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/*-0.7.9`). **W** = web source. **I** = inference.

---

## Findings

### 1. MAJOR: desktop's edit cadence is missing from the timing model. `commit()` is redundant on desktop, G costs at least a frame, and evals run before their render's edits

**Claims attacked.** Root doc 1.2, row "A Rust signal write → DOM: a later IPC batch … often a later frame". O3/V19: "On desktop two edit batches can land in one frame and lose the separation. The bridge needs a `commit` verb". 2.3: "Latency … under two frames".

**Evidence (V).**
- The webview applies edits in a rAF. `native.ts:444-458` `rafEdits` queues the bytes and runs them in `requestAnimationFrame`, then calls `markEditsFinished()` (`:495-499`), which sends an empty websocket frame back as an acknowledgement.
- Rust sends no new render until that acknowledgement. `dioxus-desktop-0.7.9/src/webview.rs:537-590` `poll_vdom` returns while `poll_edits_flushed` is pending (`:563-566`). Only after that does it poll `wait_for_work`, then `render_immediate` + `send_edits`.
- `wait_for_work` is also where tasks and effects run (`dioxus-core-0.7.9/src/virtual_dom.rs:445-536`, `poll_tasks`). So on desktop **no Rust task, eval receiver or effect runs while a render is in flight.**
- `evaluate_script` (eval, bridge `call`/`get`, MountedData queries via `scripted_getter!`, `element.rs:30-80`) runs as a webview task as soon as it arrives. It is not ordered with the rAF-deferred edits.

**Consequences (I, from the V chain above).**
- (a) Two Rust renders never land in one frame on desktop. Batch k+1 is sent only after batch k's rAF callback has acknowledged, and its `requestAnimationFrame` runs in the following frame. The row 421 "height, then style flush, then flag" order is therefore automatic on desktop. **It is web that needs the separation**: there, successive renders are successive microtasks inside one frame. On web, `commit()` (rAF plus flush) costs a whole frame when it is called from inside a rAF or ResizeObserver callback, so "zero frames on web" is false for any G chain that uses `commit()`.
- (b) Desktop G latency for a popover open: render(mount + pending) is applied in frame N's rAF. The acknowledgement arrives, an effect runs, and an eval calls `showPopover`; that eval can only run after frame N has painted. Then a measuring query, a render(done), and frame N+1 or N+2 applies it. That is at least one frame and typically two, plus one more for every *sequential* awaited read. That is below the roughly 100 ms that people notice on a discrete open (I), but it is a real cost on every gated open.
- (c) An eval issued from an event handler runs **before** the edits of the render that handler caused, on desktop (task before the rAF) and on web (the eval body runs synchronously; the render follows in a microtask). An eval issued from an effect runs **after** its render on both renderers. On desktop it runs at least one painted frame later.

**Change.**
1. Replace table 1.2 with a verified **desktop timing contract**:

   | step | desktop/mobile |
   |---|---|
   | Dioxus event | sync XHR |
   | render → DOM | in the next rAF, at most one per frame |
   | tasks and effects | only after the acknowledgement |
   | eval and queries | a task on arrival, unordered with pending edits |
   | `dioxus.send` | queued on the event loop (finding 4) |

2. Drop `commit()` as a desktop fix. Keep a web-only "flush between renders" helper that is a forced style read, never a rAF.
3. Make "show plus measure" **one bridge transaction that returns the rects**. That way desktop G costs exactly one round trip and one frame.
4. Lane rule: a bridge op that depends on a render's DOM is issued from an effect or `onmounted`, never from the handler that caused the render.

**Subsumes:** the "two batches in one frame" risk (5.5 item 5), the O3 "fall back to B3 if desktop ordering misbehaves" contingency, and the source of the repo's "separate evals have no guaranteed order" class (`lib.rs:590-602`).

### 2. MAJOR: the G gate's CSS is defeated by animations, and its fail-open contradicts `animation-play-state: paused`

**Claim attacked.** Root doc 2.3: `[popover][data-dx-place="pending"] { opacity: 0; pointer-events: none; animation-play-state: paused }`. Best-practice must #1: a CSS-only fail-open that is "a keyframe animation that reaches `opacity: 1` after a bounded delay".

**Evidence.**
- The theme's enter animation is `forwards` and runs `opacity 0 → 1` (`preview/assets/dx-components-theme.css:1775-1777`, `:1793-1800`, `:1813-1818`) (V).
- In the CSS cascade, animation declarations outrank normal author declarations (W: CSS Cascade 5 §6.1, <https://www.w3.org/TR/css-cascade-5/#cascade-origin>). Any re-entry into `pending` after the enter animation has finished therefore shows at opacity 1. This happens with S1's "one-shot placement plus window resize", or with T8's iOS-keyboard re-place. A one-line `opacity: 0` gate only hides while the animation is paused at t = 0 (I).
- `animation-play-state: paused` applies to every animation in the element's list. So a fail-open animation on the same element is paused as well, and the gate never opens (I, from CSS Animations 1 `animation-play-state`). A gate rule that sets `animation:` also replaces the component's enter animation, since only one `animation` declaration wins. Making the gate `!important` would beat animations, but then no animation can fail it open.
- The fail-open matters most where the fallback is needed most. WebKitGTK anchor-positioning support is unknown (WebKitGTK 2.52.0 notes are silent on it, W <https://www.webkitgtk.org/releases/webkitgtk-2.52.0.tar.xz.news>), and #5734 (`get_client_rect().await` never completing) is reported on Ubuntu desktop. On Linux desktop, the G fallback may therefore end up "late and unplaced" on every open.

**Change: one construction, with no pause and no `!important`.**
- **Invert the gate.** `:popover-open:not([data-dx-placed]), dialog[open]:not([data-dx-placed]) { opacity: 0; pointer-events: none; animation-delay: var(--dx-gate-timeout, 350ms) }`, with `data-dx-placed` written by Rust. Every open path is gated by default, including native `popovertarget`, light-dismiss restore and script opens. The interpreter removes `data-dx-placed` synchronously in `beforetoggle` (newState = open), so a fast reopen on desktop can never reuse a stale "placed".
- **Fail-open by delay.** With `forwards`-only fill, the enter animation does not apply during its delay. The gate's `opacity: 0` holds, and the geometry is untransformed, which is what S1's measurement needs. After the timeout the enter animation itself plays: that is the fail-open. When `placed` arrives earlier, the delay drops to 0, and the running animation is re-timed rather than restarted. That is a jump of at most the elapsed frames (I, Web Animations timing).
- **Never re-gate a visible panel.** Re-placement on resize or keyboard paints a frame late, as a stated degradation.
- **Gate the smallest dependent part.** Where A already places the panel, gate only what depends on Rust's answer: the arrow and transform-origin. Do not gate the panel. On conforming engines an open then costs zero frames.

**Safety review of `opacity: 0` (I).**
- Focus into the panel works, because a `visibility: hidden` or `inert` panel could not take it.
- Screen readers get the content at once, because opacity does not hide it from the accessibility tree.
- With `pointer-events: none`, a click in the panel's future area during the pending window falls through to the page. For `popover=auto` that light-dismisses the panel. This is negligible at 0-2 frames, but real during a 350 ms fail-open.
- There is no layout shift, because the panel is in the top layer.
- Keystrokes typed while pending go to the focused invisible panel, which is correct.

**Subsumes:** the four gate forms in 2.3, best-practice must #1, the 5.3 gate item 5 assertion ("`opacity:0` while pending" becomes "`:not([data-dx-placed])` matches until placed, and the panel is visible by the timeout"), and the S1 "untransformed while gated" premise. **Does not subsume:** the late-and-unplaced outcome on a platform where the Rust read never returns. Use a bridge `get` inside the show transaction (finding 1) rather than `MountedData`, so #5734's path is not on the critical path.

### 3. MAJOR: "one delegated capture-phase interpreter" breaks `defaultPrevented` predicates and the passive/blocking budget

**Claim attacked.** Step 3 adopted: "`data-dx-rule*` read by one delegated capture-phase interpreter". Best-practice 3.4: "That is how Dioxus itself delegates events".

**Evidence.**
- Dioxus delegates in the **bubble** phase on the app root, with no options (`dioxus-interpreter-js-0.7.9/src/ts/core.ts:109-130`) (V).
- B1 sites read `event.defaultPrevented` with "a handler inside already took it" semantics. V22 is a window keydown in the bubble phase (`preview/src/components/sidebar/component.rs:225-229`), and L23 is a target-phase mousedown (`primitives/src/label.rs:66-69`) (V). At document capture nothing has run yet, so `defaultPrevented` is always false. V22 would steal Cmd+B from an inner editor that handles it.
- In the bubble phase, by contrast, a document or window listener runs **after** the Dioxus root listener. On both web (`dioxus-web-0.7.9/src/dom.rs:110-116`) and desktop (`native.ts:380-384`) that listener has already applied Rust's `prevent_default`. **So a bubble-phase interpreter sees Rust's synchronous decision on every renderer**, which is a composition the plan does not use.
- Chrome treats `wheel`/`touch*` listeners on window/document/body as passive unless `{passive: false}` is given (W, <https://developer.chrome.com/blog/scrolling-intervention>). L11 must cancel, so a permanently installed non-passive document listener is exactly the regression that `scroll_lock.rs:94-103` documents (rows 149/159) (V).
- `stopPropagation` at document capture also cuts off every element-level listener and the Dioxus root. The carousel and dialog use element-capture stops deliberately (`carousel/drag.rs:620-635`, `lib.rs:775-783`) (V).

**Change.**
- Each `Rule` names its **phase**: `capture-at-el`, `bubble-at-doc`, or `window`. The interpreter registers one listener per (type, phase, passive), and **only while a live rule needs it**.
- Blocking types (`wheel`, `touchmove`, `touchstart` with prevent) are installed through `rules.install` / removal from Rust, never discovered from attributes. Attribute discovery stays for non-blocking types.
- `defaultPrevented` is a legal predicate only in `bubble-at-doc`/`window` bubble, and the gate parses this.
- The `stop` effect is legal only in `capture-at-el`.

**Subsumes:** the V22/L23 semantics, rows 149/159 by construction, and the carousel/dialog element-capture stops.

### 4. MAJOR: the slow-channel harness does not model desktop

**Claim attacked.** 5.6: `window.__dxBridgeDelayFrames = N`, which defers bridge messages by N rAFs, with G specs run at N = 0 and N = 2.

**What it misses (V for the mechanism, I for the effect).**
1. Edits applied in a rAF, at most one render per frame, with tasks and effects frozen until the acknowledgement (finding 1). On web, renders stay microtask-fast.
2. Eval running as a task before pending edits. Delaying the bridge makes evals *later* than edits, which is the opposite of desktop's handler-issued order, so it biases toward green.
3. `MountedData` stays synchronous on web. The plan admits this and relies on a lane rule nobody checks.
4. Delivering a delayed message inside a rAF callback is the best case. Desktop delivers it as a task at an arbitrary time.
5. **Cross-channel reorder.** `dioxus.send` goes to `window.ipc.postMessage`, then `proxy.send_event(UserWindowEvent::Ipc)` (queued: `webview.rs:308-317`, `app.rs:309-319`). A later Dioxus event is handled *inline* in the protocol handler (`protocol.rs:62-63`). A streamed `pointermove` can therefore be processed after the `pointerup` that followed it, so a release reads stale velocity. N > 0 catches some of this, but not under jitter.
6. The cost of one sync XHR per Dioxus event (finding 6).
7. Engine differences: WebKit anchor positioning, `overflow-anchor`, rAF throttling, and sync XHR to custom schemes.

**Change (strongest replacement).**
- **A "desktop-cadence" web build.** Use a harness-only workspace with `[patch.crates-io] dioxus-web` (about 30 lines against 0.7.9):
  - Flush edits in a rAF and await it before the next render. The hook for this is the commented-out `wait_for_raf` at `dioxus-web-0.7.9/src/lib.rs:256-262`.
  - Run the eval body in a task.
  - Resolve `MountedData` getters after a task hop.
  - Deliver `dioxus.send` after a task hop with N plus random jitter, in arrival order per channel and not across channels.

  Events stay synchronous, as on desktop. This models every rule in finding 1's table at the source level, with no lane rule needed.
- **Engine coverage.** Add Playwright WebKit (not installed here: only `chromium-1194` is under the browsers path, V) for the anchor and `overflow-anchor` probes.
- **One real run per batch.** Run a WebKitGTK pass per batch (`workflow_dispatch`, xvfb, WebKitWebDriver) on a runner that can install webkit2gtk.
- **Choose N from data.** Pick N and the jitter from one measured desktop latency distribution, not 2 by fiat.

**Subsumes:** the 5.6 lane rule ("reads through `get` or follow with `commit()`"), the N = never case (it becomes "the patched getter never resolves"), and part of the desktop measurement in 5.5 item 1.

### 5. MAJOR: the root has more commit points than two, and they break some bridge verbs

**Claim attacked.** 1.1: "The browser commits twice per turn". Lane 3.0 `call(el, method)` includes `setPointerCapture` and `focus({preventScroll})`.

**Missing commit points.** Each is a decision made from state present at that moment.
- **Gesture arbitration at pointerdown/touchstart.** `touch-action` and listener passiveness are read when the gesture starts. Pointer capture applies at the next pointer event. Once the pointer is released it is a no-op for mouse, and for touch it throws `NotFoundError`, because the pointer is gone (W, <https://www.w3.org/TR/pointerevents3/#setting-pointer-capture>). The repo calls `setPointerCapture` synchronously inside listeners today (`carousel/drag.rs:369`, `toast.rs:1490`) (V). As an async `call` on desktop it arrives after the first moves, or after the release.
- **Input routing.** Each key event goes to whatever has focus at dispatch. A `call(focus)` that arrives a frame late on desktop misroutes typeahead into the old focus (I).
- **Activation windows.** `DataTransfer` is writable only during `dragstart`, and `clipboardData` only during its event. Transient-activation APIs (clipboard write, `showPicker`, fullscreen) need activation. WebKit does not carry a gesture into a host-evaluated script (I, unverified).
- **The compositor.** Threaded scroll and compositor animations update pixels without the main thread. So **no** main-thread code, B included, tracks scroll frame-exactly, even on web. Only A (anchor positioning, sticky, scroll-driven animations) does. That sharpens 2.3's "continuous tracking goes to A or B" to "goes to A; B and R are a frame or more behind".
- **Post-paint deliveries.** IntersectionObserver callbacks (and so Dioxus `onvisible`), popover `toggle` and dialog `close` are queued **tasks**, not rendering-step callbacks (W, HTML "update the rendering" / "queue a popover toggle event task"). Work triggered from them always runs after a paint, on web too. G for a natively opened popover must therefore key on the synchronous `beforetoggle`, which finding 2's interpreter step does.
- **Scope of the script-dispatched case.** Script-dispatched events (1.2's caveat) break only the *cancel* commit. Their microtasks still run before paint.

**Change.**
- Restate the root: "irreversible decisions are taken at fixed points (end of dispatch, gesture start, input routing, activation window, paint, compositor frame) from state present there".
- Classify bridge verbs as **commit-sensitive**: `setPointerCapture`, `releasePointerCapture`, `focus` meant for the next keystroke, `preventDefault`, `DataTransfer`/clipboard writes, and activation-gated APIs. These exist **only as interpreter effects** (B1/B2/B5), never as `call`.
- `call` keeps `showPopover`, `hidePopover`, `showModal`, `close`, `scrollIntoView` and `setSelectionRange`.

**Subsumes:** a class of desktop-only "first moves lost" and "first keystroke lost" bugs before they exist, and it makes the B2 boundary exact.

### 6. MAJOR (desktop performance): continuous-rate Dioxus listeners cost one blocking sync XHR per event

**Claim attacked.** Root doc 2.4 C1 rejects the trampoline because desktop "pays one blocking XHR per event". Yet R and the best-practice review route streams into Dioxus listeners: W1 "or Dioxus `onscroll`/`onresize`", lane 2C's S2/S3 `onscroll`, and best-practice "reuse `onresize`, `onvisible`, `onscrollend`".

**Evidence (V).** Every Dioxus listener on desktop goes `handleEvent → handleVirtualdomEventSync → xhr.open(..., false)` (`native.ts:353-395, 568-593`). That includes `onresize`, which is a `CustomEvent` dispatched per ResizeObserver entry from inside the observer callback (`core.ts:54-72`).
- `virtual_list.rs:486`, `table_rows.rs:289` and `virtual_grid.rs:1526` attach `onresize` per row, so one frame with 50 resized rows makes 50 blocking round trips inside the rendering step.
- `select/components/list.rs:503,591` use `onscroll`, which blocks once per frame while scrolling.

**Change.** Lane rule plus gate. A Dioxus vdom listener is for events that need Rust's synchronous `prevent_default`, or that fire at human rate. Continuous-rate streams go through the bridge `listen`/`observe`, coalesced per rAF and async: `scroll`, `pointermove`, `wheel`, `touchmove`, `resize` on many elements. On web there is no difference. **Subsumes:** the trampoline rejection reason #3, applied consistently.

### 7. MINOR: `stop_propagation()` never reaches the DOM on web either

**Claim attacked.** 1.2: "on desktop `stop_propagation()` only affects vdom bubbling".

**Evidence (V).** `dioxus-web-0.7.9/src/dom.rs:110-116` applies only `prevent_default`. A grep for `stop_propagation`/`stopPropagation` across `dioxus-web` and `dioxus-desktop` finds only the desktop JS reader (`native.ts:391`), and `SynchronousEventResponse` has only `prevent_default` (`webview.rs:630-641`).

**Change.** State it for all renderers. Containment in B1 is a target predicate (`inside`/`outside`), never "Rust stopped it". The repo has 69 Rust `stop_propagation()` calls; none can hide an event from a bridge or interpreter listener.

### 8. MINOR: "synchronous on desktop/mobile" is verified for the code path, not the platforms

**Evidence.**
- Android sends the event in a base64 **header**, because `shouldInterceptRequest` gets no body (`native.ts:575-587`). The source itself warns of a 16k-per-header limit. Rust handles the event under `android_sync_lock` from the interception thread (`webview.rs:79-83`) (V).
- Edits are rAF-bound, so a throttled or hidden webview stalls every render. The source special-cases `headless` for exactly this (`native.ts:445-452`) (V). iOS Low Power Mode rAF throttling would double every frame cost in finding 1 (I).
- Sync XHR to a custom scheme on WKWebView and on WebKitGTK was not confirmed by any source found (I; the WebKit bugs searched cover headers and status codes only, W <https://bugs.webkit.org/show_bug.cgi?id=231880>).

**Change.** Label the claim "V (code path), I (platform)". Add a desktop/mobile smoke check of `prevent_default` (a Dioxus `onkeydown` that cancels Tab) to the WebKitGTK run in finding 4.

### 9. MINOR: "zero frames on web" has holes besides `commit()`

Finding 5 lists the post-paint triggers: `onvisible`/IntersectionObserver, `toggle`, `close`. A `commit()` called from a rAF or ResizeObserver callback defers to the next frame (finding 1a). Everything else holds on web: UA events, timers, rAF and ResizeObserver callbacks all drain their microtask chain before paint (V chain: `eval.ts:12-15` resolve → `js-sys-0.3.98/src/futures/queue.rs:63-69` `queueMicrotask` → `dioxus-web-0.7.9/src/lib.rs:250-264` immediate render; `mounted` fires synchronously in `flush_edits`, `mutations.rs:53-76`; effects run in the next `wait_for_work`, after the flush, `virtual_dom.rs:506-536`).

**Change.** List the post-paint triggers in 1.2, and require G chains to start from a pre-paint trigger.

---

## Attacks that failed (covered, not broken)

- **Dioxus handlers are synchronous on web.** The root listener calls `runtime.handle_event` inside the DOM event and applies `prevent_default` afterwards (`dom.rs:88-116`). V.
- **`prevent_default` works on desktop.** It goes through the sync XHR response (`webview.rs:191-195`, `native.ts:380-384`), and the 133 Rust `prevent_default()` calls count is exact (V, grep). Async handlers: only what runs before the first await counts, on both renderers (`events.rs:339-347`).
- **The web bridge is microtask-only**, end to end, for UA-dispatched events, timers, rAF and ResizeObserver: verified link by link (finding 9). wasm-bindgen-futures re-queues with `queueMicrotask`, never a macrotask.
- **Dioxus `onresize` on web runs inside the ResizeObserver callback**, and its render lands before paint: the microtask checkpoint runs after the callback returns, before the next depth pass. Caveat (I): a write that resizes a *shallower* observed element is delivered next frame under the ResizeObserver depth rule.
- **Choosing `opacity` over `visibility`/`inert` is right**, because focus must enter the panel. Screen readers are not harmed. No layout shift.
- **Microtasks in a non-passive listener can still cancel on web.** They run before the listener's invoke returns, which is before dispatch ends. The plan's passive-flag statement is right.
- **"Escape cancels unconditionally" and "the wheel band is passive"** hold: `carousel/wheel.rs:576-578` is all `passive: true` (V).
- **B1's sampled predicates (L1, L23, V22) need only local data.** Their phase is the problem (finding 3), not their data.
- **`MountedData` on desktop is an async eval query** that cannot resolve while the webview is blocked in the sync XHR (`element.rs:30-80`). V, as the plan says.
