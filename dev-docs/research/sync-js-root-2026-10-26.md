# Can the "synchronous" JavaScript be bridged too? Root cause and construction (row 434, step 3), 2026-10-26

Read-only design analysis of `origin/queue/batch-28` @ `d10d34b5`, with Dioxus **0.7.9** sources from `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/` and browser support from **web-features 3.44.0** (npm, published 2026-10-22). Nothing was built, run or measured.

Evidence labels: **V** = read in this repo or in the pinned crate source. **W** = read on a web page or in the web-features data file. **I** = inferred from V/W facts, not run.

---

## Executive summary (for the owner)

**The question.** The current plan keeps two kinds of JavaScript as bespoke modules, because the bridge to Rust is asynchronous. The first kind is a decision made inside an event, such as "cancel this wheel". The second is a measure-then-place step that must finish within one frame, such as popup placement. You asked whether that is really true, or whether those two can also be bridged or abstracted.

**The root, in one sentence.** The browser commits twice per turn of its main thread: it applies an event's default action when dispatch ends, and it paints after layout. A value can affect either commit only if it is computed on the page's own thread before that point, from data that is already there. Everything else follows from this.

**What that means for Dioxus (verified in the 0.7.9 sources).**
- Dioxus's own event handlers already run synchronously, on web *and* on desktop/mobile. On desktop the webview blocks on a synchronous request to Rust for every event. So `prevent_default()` in a Rust handler works on both, and 133 handlers in this repo rely on that.
- What is **not** synchronous is the bridge. Messages from JS to Rust go over `dioxus.send`, and Rust's replies go back the same way. On desktop and mobile that crosses to another process. On web it only queues microtasks: these happen to finish before paint, but nothing guarantees it, so nothing should be built on it.
- So the real rule is not "async can't work". It is this: **never ask Rust at the moment of commit. Publish what the decision needs before the event, and decide locally.**

**Three ways that rule lets nearly everything leave bespoke JS:**
1. **(A) Let the browser decide.** CSS anchor positioning has been in all three engines since January 2026 (Chrome 125, Firefox 147, Safari 26). It flips, shifts, sizes the panel to the room it has, and follows scrolling with no script at all.
2. **(B) Rules as data, one interpreter.** Rust writes *which* rule applies, plus the state it needs, onto the DOM. One small generic interpreter evaluates it at event time. I listed every `preventDefault` in the repo: 13 sites, Escape included. Every one of them is a test on data that is already present when the event fires: the key, the modifiers, a regex on the typed text, where the event target sits, or whether a scroller can still scroll. None needs to ask Rust. Four small rule families cover them, along with the gestures and scroll-settle loops.
3. **(G) Make the in-between state safe to paint, then let Rust take its time.** A popup can stay invisible until Rust has placed it, and an old height can stay pinned until Rust sets the new one. Nothing wrong is ever painted, so a one-frame deadline becomes a small delay: zero frames on web, a round trip on desktop. The placement, toast-height and height-glide logic can then live entirely in Rust.

**Several premises fail when tested against the code.** The "wheel band" in the carousel never cancels anything: it is passive. The scroll lock's wheel test is a pure DOM test, which is case (B). Escape in nested layers already cancels unconditionally, and the "topmost layer" decision is already made in Rust, asynchronously. The drawer drag and the virtual scroll bridge already stream to Rust asynchronously. Only OTP filtering and the carousel's mouse drag make a real conditional cancel, and both are (B).

**Result for the 22 "JS on purpose" blocks:**

| construction | blocks | lines (of 3,894) |
|---|---:|---:|
| A: platform does it (plus a Rust fallback behind a gate) | 3 (T8, T11, W8) | 1,638 |
| B: rules as data, one interpreter | 9 (B1 event gates 1, B2 gestures 3, B3 publishers 1, B5 reveal/settle 4) | 1,038 |
| G: Rust behind a paint-safe gate | 5 | 211 |
| R: no real sync need, plain bridge verbs plus Rust | 5 (W1, C3, V11, C5, C1) | 1,007 |
| **Residue: stays a bespoke module** | **0** | **0** |

**Revised 2026-10-26 (see [`sync-js-residue-2026-10-26.md`](./sync-js-residue-2026-10-26.md)).** An earlier version of this table kept three blocks as a residue: the message scroller, the carousel wheel band and its helper macros. Re-attacked, all three move:
- **Message scroller (W8):** CSS `overflow-anchor` is Baseline newly available since 2026-09-14 (Safari 27), so it qualifies under this doc's own policy (section 2.1) *today*. Its anchoring becomes native scroll anchoring on a 1 px end sentinel and on the rows. Its mode machine moves to Rust, with a Rust `onresize` fallback for engines without anchoring.
- **Carousel wheel band (C5):** it is passive, so a late answer is lag, never a wrong decision. Its classifier becomes a pure Rust state machine, fed a batched per-frame event stream and proven by `cargo test` against the committed trackpad recordings. Its exit, if a desktop measurement shows more than one extra frame of lag at p95, is the same crate as page-side wasm (C2).
- **Helper macros (C1):** dissolve once their four users have moved.

**The gate that keeps it closed:** the rule vocabulary is one Rust enum that matches the interpreter's switch one to one. No `preventDefault`, rAF loop or observer may appear outside the bridge and the interpreter (the residue allowlist starts empty). Adding a rule kind needs a backlog row.

**Biggest risk:** desktop and mobile are not in CI. Every (G) mapping and every desktop claim needs either a real desktop measurement or the "slow channel" test harness proposed in section 5.6. The C5 lag measurement in particular needs a machine with WebView2 or WKWebView.

---

## 1. Root cause

### 1.1 The two commit points (W, platform semantics)

1. **Default actions are decided when dispatch ends.** `preventDefault()` only sets the event's *canceled flag*. The browser reads that flag after the last listener returns. A call after dispatch has finished does nothing. Inside a passive listener the call is ignored: the *in-passive-listener* flag is set around the callback, and the microtask checkpoint runs while it is still set. A wheel or touchmove listener that is not passive also makes the compositor wait for the main thread. This is the cost this repo measured in rows 149/159 (`scroll_lock.rs` module docs, V).
2. **Pixels are committed after style and layout in the "update the rendering" step.** Inside one rendering opportunity, the order is: run rAF callbacks → style and layout → ResizeObserver callbacks (which may loop back to layout) → paint. A microtask checkpoint follows each callback. A DOM write lands in the frame being drawn only if it happens before paint, which means inside an rAF or ResizeObserver callback, or in microtasks queued by one. A write that arrives in a later task misses that frame. If the pre-write state was painted, the user sees the flash.

**Invariant (the root):** *A decision can affect a commit only if it is computed on the page's main thread before that commit, from inputs present there at that time.* "Present" means event fields, the DOM, and anything Rust **published into the DOM beforehand**. Rust's own memory is not present, and neither is a message that is still in flight.

### 1.2 Where Dioxus 0.7.9 is synchronous, per renderer (V)

| path | web (wasm, same thread) | desktop / mobile (wry webview, `mobile = ["dep:dioxus-desktop"]`, `dioxus-0.7.9/Cargo.toml:122-125`) |
|---|---|---|
| Dioxus element listener → Rust handler | **Synchronous.** The root listener calls `runtime.handle_event` inside the DOM event (`dioxus-web-0.7.9/src/dom.rs:88-110`). Bubbling through the vdom is a plain loop (`dioxus-core-0.7.9/src/runtime.rs:369-470`). | **Synchronous, by blocking.** `handleEvent` → `handleVirtualdomEventSync` does `xhr.open("POST", "__events", false)` (`dioxus-interpreter-js-0.7.9/src/ts/native.ts:353-395, 568-593`). Rust handles the request in `WebviewEdits::handle_html_event` (`dioxus-desktop-0.7.9/src/webview.rs:99-195`, routed at `protocol.rs:62-63`). |
| `Event::prevent_default()` | **Works.** After the handler returns, `if !event.default_action_enabled() { web_sys_event.prevent_default() }` (`dom.rs:112-116`). An async handler is polled once with `now_or_never` "to deal with things like prevent_default in the same tick" (`dioxus-core-0.7.9/src/events.rs:339-347`). | **Works.** The response is `SynchronousEventResponse { preventDefault }` (`webview.rs:191-195, 630-641`), and the JS applies it (`native.ts:380-384`). `stopPropagation` is read by the JS but **never sent by Rust** (the struct only has `prevent_default`), so on desktop `stop_propagation()` only affects vdom bubbling, not the DOM. |
| `MountedData` reads (`get_client_rect`, `get_scroll_*`) | **Synchronous values** wrapped in a ready future (`dioxus-web-0.7.9/src/events/mounted.rs:10-60`). | **Async webview queries** (`scripted_getter!`, `dioxus-desktop-0.7.9/src/element.rs:30-80`). The webview is blocked in the sync XHR while a handler runs, so a query issued in a handler can only be answered after it returns (I). |
| `document::eval(js)` Rust → JS | **Runs the JS body synchronously** at the call: `Function::new_with_args("dioxus", code).call1(..)` (`dioxus-web-0.7.9/src/document.rs:118-119, 208-220`), wrapped in an `async function` (`PROMISE_WRAPPER`, `:189`), so the body runs up to its first `await`. | **Async**: `webview.evaluate_script` (`dioxus-desktop-0.7.9/src/query.rs:60,169`). |
| `dioxus.send` JS → Rust | **Async, microtask latency.** The channel resolves a pending promise (`dioxus-document-0.7.9/src/ts/eval.ts:12-15`) → `JsFuture` → the wasm executor uses `queueMicrotask` (`js-sys-0.3.98/src/futures/queue.rs:63-69`, the version `Cargo.lock` pins) → Dioxus's run loop renders right away, with no idle or rAF gate (`dioxus-web-0.7.9/src/lib.rs:250-264`, where the idle/rAF scheduling is commented out as a TODO). | **Async, IPC**: `window.ipc.postMessage` (`dioxus-desktop-0.7.9/src/ts/native_eval.ts:36,75`). |
| A Rust signal write → DOM | Next microtask (I, from the run loop above). | A later IPC batch: a later task, often a later frame (I). |
| listener options, window/document targets | None. Bubbling events get one listener on the app root, non-bubbling ones a listener on the element, and `addEventListener` is called with no options (`dioxus-interpreter-js-0.7.9/src/ts/core.ts:109-130`). | Same interpreter. |

**What the table says (I).** The asynchronous hop is *the bridge*, not Dioxus's event path. On desktop and mobile it is a real cross-process hop. On web it is a microtask hop: for UA-dispatched events and rendering-step callbacks, that hop *happens* to finish before both commits. It is not a contract. It breaks for events dispatched from script, which get no checkpoint until `dispatchEvent` returns. It also breaks if dioxus-web re-enables its idle/rAF scheduling, or if any hop ever uses a macrotask. So: **on web the premise is mostly false in practice, on desktop it is true, and a design for all renderers must assume it is true.**

### 1.3 The premise, tested against the code (V)

The plan names four decisions made inside the listener and four read-then-write cases. The code says:

| named case | what the code actually does | sync need? |
|---|---|---|
| "wheel band" (carousel, `carousel/wheel.rs:110`) | `listen(el,'wheel',…,{passive:true})` (`:576`). Its doc says it "may **never** call … `preventDefault`" (`:13`). It writes a `transform` per frame. | No cancel. A per-frame feedback loop. |
| "wheel band" (scroll lock, `scroll_lock.rs:367`) | Cancels a wheel/touchmove/key unless an ancestor scroller can still scroll that way (`:422,432,450`). | Yes, and it is a **pure DOM test**: B1. |
| Escape in nested layers (`lib.rs:330-375`) | Each layer's JS calls `preventDefault()` **unconditionally** on Escape (`:367`). "Is this layer on top" is a Rust check, made after the event (`:350-356`). | Only a key test: B1. |
| drag (drawer `drawer.rs:525`, carousel `carousel/drag.rs:99`) | Drawer: no `preventDefault`, and it already sends `start/move/end` to Rust per rAF. Carousel: cancels `pointermove` once a drag has committed (`:388`), and suppresses the click after a drag in the capture phase (`:633`). | Drawer: no. Carousel: yes, on the recognizer's own state: B2. |
| OTP filtering (`input_otp.rs:367`) | `beforeinput`: cancel if `e.data` consists only of rejected characters (a regex from Rust). `input`: strip them, restore the caret. | Yes, a test on event data: B1. |
| popup/anchor placement (`anchor_fallback.rs:493`) | **Already a fallback.** CSS anchor positioning places the panel, and the JS measures the result and takes over only when it is wrong (doc `:7-60`). It also publishes `--dx-anchor-width`, `--dx-available-*` and the placed side on every engine. | One-shot placement: a gate (G). Continuous tracking: A. |
| carousel drag | Per move: `scrollBy(dx)` with snap suspended. The "read" is the previous pointer position. Geometry is read only for the rubber band at the true end. | Per-frame feedback: B2. |
| virtual scroll bridge (`virtual/bridge.rs:293`) | Reads the scroll position and **sends it to Rust**, with no DOM write. It is already an async bridge. | **None.** It is (b)-shaped. |
| toast heights (`toast.rs:617`) | Measures on insert and resize, publishes CSS variables, and commits the height before the `data-sized` flag (row 421). | Ordering plus "never paint unmeasured": G (or B3). |

The complete list of class-(1) sites (every `preventDefault()` in `primitives/src`, `preview/src` and `preview/index.html`, V by grep; comment hits removed):

| site | condition | rule |
|---|---|---|
| `lib.rs:367` L6 | `key === 'Escape'` | B1 key |
| `lib.rs:781` L1 | Enter/Space, no modifiers, not repeat, dialog closing or cancelled < 250 ms ago, opener connected | B1 state attribute plus a small time window (see section 4) |
| `lib.rs:848` L2 | every `cancel` | B1 always |
| `toast.rs:558` O1 | `key === 'F6'` and a live region exists | B1 key plus "exists" |
| `label.rs:69` L23 | `detail > 1`, not already cancelled, target not inside a form control | B1 field plus target |
| `context_menu.rs:40` L10 | target outside the menu root | B1 target |
| `scroll_lock.rs:422,432,450` L11 | no ancestor of the target can consume the delta (axis, RTL); scroll keys; not a form control or role | B1 scroll-capacity |
| `drag_and_drop_list.rs:767,773` L25 | every `dragover`/`drop`, plus set `dropEffect` | B1 always plus effect |
| `input_otp.rs:379` S11 | `e.data` matches the reject regex | B1 regex |
| `ts/focus-trap.ts:57` L4 | `key === 'Tab'` (then move focus in the container) | B1 focus-trap |
| `carousel/drag.rs:388,633` C4 | drag committed; the next click after a drag | B2 |
| `preview/.../sidebar/component.rs:229` V22 | key plus Cmd/Ctrl, not already cancelled, not composing, target not editable | B1 |
| `preview/.../photo_wall/mod.rs:190` V17 | the click after a pan | B2 |

**Finding (V):** none of the 13 sites needs Rust-only state at event time. Each condition reads event fields, the DOM around the target, or state Rust can mirror into an attribute before the event. That is exactly what B makes general.

---

## 2. Constructions, from the root

The invariant leaves four ways to satisfy it, plus one fallback:

| | how it satisfies "decide at the commit, from data present" | logic lives in |
|---|---|---|
| **A** platform-declarative | The browser owns the decision inside its own pipeline. | the browser (no code) |
| **B** rules as data plus one interpreter | Rust publishes rule plus state *before* the event, and a generic interpreter decides at the commit. | Rust picks the rule and parameters; one tested TS interpreter evaluates them |
| **G** paint-safe gate (new) | Removes the deadline: CSS keeps the unsettled state invisible or pinned, so a late answer paints nothing wrong. | Rust |
| **R** no real sync need | The block was never on a commit path (it only reads and reports, or runs a command). | Rust plus plain bridge verbs |
| **C** run Rust in the page | C1: the Dioxus sync event path. C2: a Rust kernel compiled to page-side wasm. C3: worklets. | Rust |
| residue | none of the above, within the support policy (empty after the 2026-10-26 re-attack) | a named TS module |

### 2.1 (A) Platform-declarative first

Support from web-features 3.44.0 (W, `status.by_compat_key`; "low" = Baseline newly available):

| feature | Chrome/Edge | Firefox | Safari | Baseline | use here |
|---|---|---|---|---|---|
| anchor positioning core (`anchor-name`, `position-area`, `position-try-fallbacks`, `anchor-size`, `@position-try`) | 125 (`position-area` 129, `position-try-fallbacks` 128) | 147 | 26 | **low since 2026-01-13** (the whole feature is not Baseline only because `position-visibility: anchor-valid/visible` is Safari 27) | T8, T11 |
| `position-try-order` | 125 | 148 | 26 | low | T8 side-by-room |
| anchored container queries (`@container anchored(fallback:…)`) | 143 | no | no | no | placed side / arrow flip in CSS: **not available** |
| popover | 116 | 125 | 17 (iOS 18.3) | low since 2025-01-27 | already used |
| `popover=hint` | 151 | 153 | no | no | — |
| `<dialog closedby>` | 134 | 141 | no | no | — |
| `dialog.requestClose()` | 134 | 139 | 18.4 | low 2025-05-27 | vetoable close |
| overscroll-behavior (current semantics: applies to non-scrollable `overflow:hidden` containers) | 144 | 150 | no (only the older semantics) | no | a CSS-only dialog scroll lock, later |
| touch-action, scroll-snap, inputmode, `beforeinput`, pointer capture | old | old | old | high | already used |
| `scrollend` | 114 | 109 | 26.2 | low 2025-12-12 | carousel feature detection can go |
| scroll-snap events (`scrollsnapchange`) | 129 | no | no | no | — |
| scroll-driven animations | 115 | no | 26 | no | S14 depth (already feature-detected) |
| `interpolate-size` / `calc-size()` | 129 | no | no | no | toast/questionnaire height: **not available** |
| `field-sizing` | 123 | 152 | 26.2 | low 2026-06-16 | — |
| `overflow-anchor` (scroll anchoring) | 56 | 66 | **27** | low 2026-09-14 | W8 anchoring (qualifies under the policy below; residue doc section 2) |
| `hidden=until-found` | 102 | 148 | no (26.2 partial per row 153) | no | already feature-degraded |
| view transitions | 111 | 144 | 18 | low 2025-10-14 | a size-glide alternative (section 5, V19) |
| `@starting-style`, `transition-behavior` | 117 | 129 | 17.5 | low 2024 | enter/exit without JS where unmounting allows |
| wasm | 57 | 52 | 11 | high | C2 |

Spec facts (W, `w3.org/TR/css-anchor-position-1`):
- `position-area` "makes that the box's containing block", so `max-height: 100%` is relative to that area: **size to the room**.
- By default the box "will 'shift' to stay within its original containing block": **cross-axis shift**.
- `position-try-order: most-height` sorts the options by available block size: **side by room**.
- The spec text read does not mention the visual viewport. Fixed positioning works against the layout viewport, so an iOS keyboard is not avoided (I).

**What A cannot cover (V/W):**
1. The visual-viewport (keyboard) exclusion of row 210.
2. Publishing the placed side and the anchor offsets for arrows and transform-origin (`@position-try` accepts only inset, margin, sizing, alignment and position properties, not custom properties; anchored container queries are Chrome-only).
3. Engines without anchor positioning.
4. Gestures and height animation within a Baseline policy (`interpolate-size` is Chrome-only). Scroll anchoring *is* covered: `overflow-anchor` is Baseline newly available since 2026-09-14.

**Support policy A needs (proposal).**
- **Web:** "Baseline newly available across Chrome, Firefox and Safari. Older engines must stay *correct* through a feature-detected fallback, but need not be frame-perfect."
- **Desktop and mobile:** WebView2 and Android WebView are evergreen Chromium. WKWebView follows the OS WebKit, so Safari ≥26 is needed for anchor positioning on macOS and iOS (I). **WebKitGTK is unverified:** the 2.50.0 release notes (W) do not mention anchor positioning, so the fallback must stay until a probe on Linux desktop shows support.

### 2.2 (B) Rules as data, one synchronous interpreter

**Shape.** Rust declares a rule from a closed enum, and it is serialized onto the element:
- either as `data-dx-rule-*` attributes, for state that changes with renders;
- or as a `bridge.rules.install(el, rule)` call, for static configuration.

State the rule reads is mirrored into attributes in the same render that changes it (for example `data-dx-closing`, `data-dx-layer`). The interpreter in `primitives/src/ts/rules/*.ts`, behind the step-3 bridge, owns the listener (with any option: capture, passive, window/document) and evaluates the rule at event or frame time. It emits only three things: the native effect (`preventDefault`, `stopPropagation`, `dropEffect`, a CSS-variable write, `scrollBy`), and/or a message to Rust (async, which is fine because it is after the commit).

**Vocabularies, sized by the blocks they absorb (I):**

| family | predicates / parameters | absorbs |
|---|---|---|
| **B1 event gate** | `key ∈ set`, modifiers, `repeat`, `isComposing`, `defaultPrevented`, `detail > n`, `data =~ regex`; target `closest(sel)`, `inside(id)` / `outside(id)`; `can-scroll(axis, delta)` (the L11 walk, RTL-aware); `attr(el, name) = v`; `exists(sel)`; combined by AND/NOT only. Effects: `prevent`, `stop`, `dropEffect = v`, `focus-trap(container)`, `send(fields)`. A value-filter companion for `input`: strip regex, keep the caret. | S11; and from (b): L1, L2, L4, L6, L10, L11, L23, L25, O1, V22 |
| **B2 pan recognizer** | pointer types; button; ignore `closest(sel)`; owner `closest(sel) === el`; slop px; axis lock (fixed/auto); scroll guard (nearest scrollable along the axis must be at its edge, otherwise abandon); direction (sign, or toward the nearest viewport edge); resistance (none / rubber(c, d)); pointer capture; cancel `pointermove` once committed; suppress the next click; output to a CSS var, `scrollBy` (with snap suspended), or Rust per rAF; release: velocity window, threshold/flick → `send`. | L30, O5, C4, and from (b): L21, V17 |
| **B3 metrics publisher** | observe size/scroll of `el` (or of descendants matching `sel`); publish `--var = metric` (`gutter-inline/block`, `edge-distance t/b/s/e`, `border-box-block`); "commit before flag" (write, flush style, then set `data-sized`); rAF coalescing. | S14 (and the O3 fallback) |
| **B5 reveal/settle** | target `sel` inside `container` (wait up to N frames to mount); align start/center/end/nearest; cover insets; optional `focus({preventScroll})`; scroll to a known position when unmounted; settle until stable for k frames or N max; cancel on wheel/touch/pointer/key; `scroll-step(by, smooth, suspend-snap-until-scrollend)`. | W3, W5, W7, C2 |

**Expressiveness limits (I).** B works while a decision is a *fixed-arity predicate over local data*. It does not fit, and the logic goes to Rust instead (R, or G where a paint is at stake), where the decision is:
- a stateful heuristic over a stream (the C5 trackpad-momentum classifier, W8's mode machine). Both are passive or paint-safe, so they can answer late (residue doc sections 2 and 3);
- open-ended arithmetic whose form changes per component.

The guard against vocabulary creep is that the DSL has no variables, loops or arithmetic expressions. Every predicate kind is a named enum variant with fixed parameters.

**Keeping it closed (gate, section 5.3).** The Rust enum and the TS switch must agree one to one. Every variant needs a `bun test` unit and at least one Playwright spec that names it. A new variant needs a backlog row (reasoned allowlist). No `preventDefault`, rAF loop or Resize/Mutation/IntersectionObserver may appear in `.ts` outside `ts/bridge.ts` and `ts/rules/` (the residue allowlist starts empty).

### 2.3 (G) Paint-safe gate: turn "same frame" into "bounded latency"

Same-frame is *sufficient* but not *necessary*. What is necessary is that **no frame paints a wrong state**. If the unsettled state is styled to be invisible or pinned, Rust can measure through `MountedData` and bridge `get`, compute, and write in one render whenever the answer arrives:

- **Popups:** `[popover][data-dx-place="pending"] { opacity: 0; pointer-events: none; animation-play-state: paused }` until Rust writes the placement plus `data-dx-place="done"` in one render. Use `opacity`, not `visibility`: the open-focus policy (`modality.rs`) focuses into the panel, and a `visibility:hidden` element cannot take focus (I). The enter animation starts on `done`, so the `restingRect` trick (seeking animations to their end, `anchor_fallback.rs:524-533`) disappears.
- **Height glide:** pin the old height in the same render that swaps the content. A forced style read (on web `get_client_rect` is synchronous and flushes style) commits the pin as its own style change. Then write the new height. This is exactly the row-421 ordering, done in Rust.
- **Latency:** zero frames on web (all microtasks, section 1.2). On desktop, one to three IPC round trips before the panel shows (unmeasured; I estimate under two frames).
- **Not for continuous loops:** a lagging write during scroll or drag *is* a wrong paint (the panel trails its anchor). G is for open-time, insert-time and swap-time work only. Continuous tracking goes to A or B.

### 2.4 (C) Running Rust in the page

- **C1, Dioxus's own sync event path (V).** It works on web and desktop/mobile, as section 1.2 shows. **Use it wherever the listener can be a vdom element listener** for a `dioxus-html` event that needs no listener option: this is already the house style (133 `prevent_default()` calls in `primitives/src`). As a *trampoline* for window/document/capture cases, where the bridge re-dispatches a cancelable synthetic event to a vdom "port" element and reads `dispatchEvent()`'s return value, it is **rejected as the primary path**:
  1. The synthetic event must be a type Dioxus converts (keydown, or a `CustomEvent` `resize` with a fake `detail`), which is an abuse.
  2. Other capture listeners see it, so the modality writer would mark keyboard modality.
  3. Desktop pays one blocking XHR per event, which is unacceptable for wheel or touchmove at 60-120 Hz.
  4. `stopPropagation` does not come back on desktop.
  5. Desktop is not in CI.

  B1 makes it unnecessary for all 13 sites. Note: Dioxus 0.8.0-alpha.1 adds `onbeforeinput` (audit section 3, W). With it, S11's cancel half becomes a plain Rust handler (C1) once fields and sync cancel are verified (I).
- **C2, a Rust kernel compiled to page-side wasm (I).** Every target webview runs wasm (Baseline high). Pure functions (placement maths, pan release, the C5 band heuristic) could be one `no_std` crate, unit-tested natively and called synchronously from the interpreter. **Against it:**
  1. Chrome refuses synchronous `new WebAssembly.Module` for buffers over 4 KB on the main thread (W, web.dev "Loading WebAssembly modules efficiently"), so loading is async, which reintroduces a first-use race (G can absorb that).
  2. It needs a second build artefact plus a reproducibility gate, like `check-js-bundle.sh`.
  3. Strict-CSP apps need `wasm-unsafe-eval`.
  4. Every input still has to be extracted by JS glue.

  **Verdict:** not now. It is the only route to "C5 logic in Rust *and* frame-exact on desktop", so it is kept as C5's exit, taken only if the desktop measurement in residue doc section 3.5 fails.
- **C3, worklets and workers.** AnimationWorklet and the Layout API have not shipped. A paint worklet is Chromium-only and cannot cancel events. Workers have no DOM. Rejected.
- **C4, "the bridge is microtask-synchronous on web" (V components, I end to end).** True today for UA-dispatched events and rendering-step callbacks. **Not a construction:** it fails on desktop/mobile and for script-dispatched events, and it would break if dioxus-web re-enabled its idle/rAF scheduling (the TODO at `lib.rs:250-262`). Its one use is to explain why G costs nothing on web, and why a Chromium-only CI cannot reveal desktop latency bugs (section 5.6).

---

## 3. Per-block mapping: the 22 class-(d) blocks

Locations re-verified on `d10d34b5`. Drift from the audit is noted. "Falsify" is the strongest attempt to break the mapping. Specs are under `playwright/` (names from the directory listing, not run).

| id | file:line (verified) | lines | construction | why, and the falsification attempt | risk | spec that would prove it |
|---|---|---:|---|---|---|---|
| T8 | `top_layer/anchor_fallback.rs:493` (audit 494) | 499 | **A**, plus a G (Rust) fallback and zone check | Conforming engines: `position-area` + `position-try-fallbacks: flip-block, flip-inline` + `position-try-order: most-height` + `max-height: 100%` + default shift + `min-width: anchor-size(width)` cover side-by-room, flip, shift, size clamp and scroll tracking with no script (W spec). **Falsify:** (1) The iOS keyboard (row 210) is not avoided by CSS: a Rust check after open and on `visualViewport` resize (bridge `listen`), which republishes `--dx-available-height` or moves to the fallback. Late by the keyboard animation, so harmless (I). (2) The placed side and anchor offsets for arrows have no CSS: Rust computes them from two rects after open (G). (3) Engines without support (WebKitGTK unverified, Safari <26, Firefox <147): Rust placement behind the gate. Tracking during scroll is then a frame late on desktop, a stated degradation that applies only on fallback engines. (4) The repo's css_module `@supports` scoping gap (`dev-docs/issues/css-module-supports-scoping.md`, quoted in the doc at `:330-345`) blocks `anchor-size` in component sheets: the rule must live in the theme/engine sheet (I). | Highest (every open) | `oracle/tier2-html/top-layer.spec.ts` rules 5-15 with the bridge's placement disabled (A alone) **and** with `stripAnchorSupportsBlock` (fallback); `anchor-landing`, `combobox-placement` (geometry sweep incl. the keyboard simulation), `select`, `close-on-scroll`; a new "no unplaced paint" first-frame check; release-build `main-thread` |
| T9 | `top_layer/scroll_clamp.rs:58` | 31 | **G** (Rust) | Dissolves under A. In the fallback, Rust renders the `position:fixed; inset:0` probe itself and reads its `MountedData` rect, plus `visualViewport` through bridge `get`. **Falsify:** `get` is async on desktop. Fine behind the gate. | Medium | `spec-viewport-bound` gate; `top-layer.spec` rule 12 |
| T10 | `top_layer/scroll_clamp.rs:261` | 8 | **G** (Rust) | Park, then measure: under the gate Rust writes the parked style, renders, and reads the rect. Nothing is visible, so the extra render costs no flash. | Low | `top-layer.spec` (wide multi-month date picker) |
| T11 | `top_layer/scroll_clamp.rs:348` (audit 349) | 74 | **A**, plus a G fallback | Rust renders a 0×0 fixed "point anchor" at the `contextmenu` client coordinates (`MouseData` has them), and the menu uses `position-area: bottom span-right` + flips + default shift + `max-height: 100%`. **Falsify:** the corner semantics of a point anchor with `position-area`: it should land at (x, y) and flip to up/left on overflow (I, needs the spec). | High (visible on open) | `context-menu.spec.ts`, `top-layer.spec` |
| L30 | `drawer.rs:525` | 127 | **B2** | No `preventDefault`. It already streams to Rust. Owner/ignore selectors, scroll guard, axis lock, rAF-coalesced `move` and release velocity are all B2 parameters. **Falsify:** "drag only from the top-most nested drawer" is the B2 `owner closest(sel) === el`. | High (feel) | `drawer-axis-lock`, `drawer-snap`, `drawer.spec` |
| S1 | `select/components/list.rs:102` | 77 | **G** (Rust) | One-shot placement plus window resize. Under the gate, Rust reads the trigger and option rects (untransformed: no animation before `done`), list scroll size and `clientTop`/visual viewport (bridge `get`), then computes the item-aligned top, left and scroll. **Falsify:** an option `offsetTop` walk equals `option.top - list.top + scrollTop` only with no transform, which the gate guarantees (I). CSS cannot align by a child's offset, so no A (`appearance: base-select` is Chromium-only and popper-style). | High (placement) | `select.spec.ts` (item-aligned cases), `top-layer.spec` |
| S11 | `input_otp.rs:367` | 25 | **B1** | `beforeinput` cancel by the Rust-supplied regex, plus the `input` value filter with caret restore. **Falsify:** `pattern`/`inputmode` do not filter (W), so no A. With Dioxus 0.8 `onbeforeinput` the cancel half could be C1. | High (typing) | `input_otp.spec.ts`, `input-otp-pattern.spec.ts` |
| S14 | `scroll_area.rs:150` | 31 | **B3** (+A) | Depth stays in CSS where scroll-driven animations exist (Chrome, Safari 26), as today. The publisher writes the gutters (all engines) and the depth only where they are missing (Firefox). **Falsify:** no CSS exposes the scrollbar width (I), so no pure A. Through Rust signals it would re-render per scroll frame (the block's own reason). | Medium | `scroll-fade`, `scroll-fade-scrollbar`, `scroll-fade-transition`, `scroll-area-always-show` |
| O3 | `toast.rs:617` (installed `:441`) | 59 | **G** (Rust), fallback B3 | Dioxus `onresize` *is* a ResizeObserver (`core.ts:62-72`). On web its handler runs inside the observer callback, so the write lands before paint. Rust keeps the heights and knows the front toast (it owns the list and the removed state). A new toast stays gated (`li:not([data-sized])` out of the pile) until measured. The row-421 order is height write, forced style read (`get_client_rect`), then flag. **Falsify:** (1) On desktop two edit batches can land in one frame and lose the separation. The bridge needs a `commit` verb (rAF plus a style flush) between them (I). (2) The natural height needs the body box plus the toast's padding and border: two observations, or one `get` (I). If desktop measurement fails, fall back to B3 (the same logic as one generic rule). | High (row 421) | `toast-height`, `toast-stack-motion`, `toast-stacking`, `toast-enter-exit`, `toast-update` |
| O5 | `toast.rs:1433` | 74 | **B2** | Direction toward the nearest viewport edge, slop, axis pick, CSS-var output `--toast-swipe-x/y`, ignore buttons and links, release threshold/flick, then `send(true)`. **Falsify:** no cancel anywhere. All parameters are B2. | High (gesture) | `toast-swipe`, `toast-swipe-vertical` |
| W1 | `virtual/bridge.rs:293` | 42 | **R** | Reads the scroll position and size and sends them, with no DOM write and no cancel: already an async bridge. Use the generic `listen`/`observe` verbs with RTL normalisation, or Dioxus `onscroll`/`onresize` (lane 2D's experiment). **Falsify:** "sent at once so Rust sees it before the next render" holds on web (microtasks) and was never true on desktop. No change. | Highest (engine) | `virtual-engine`, `virtual_list`, `virtual-grid*`, `table-rows`, `data_table-large`, release `main-thread`/`scroll-main-thread` |
| W3 | `virtual/scroll_to.rs:190` | 58 | **B5** (+A) | `scroll-padding` on the container for the pinned covers (A, Baseline high) lets the first `scrollIntoView` land clear of headers. The settle loop stays as B5, because content-visibility chunks change height after render. **Falsify:** `overflow-anchor` would hold the item only if the browser picked it as the anchor node, which is not guaranteed and is Safari 27 only. `scrollIntoView` scrolls ancestors too: B5's first step is optional per caller. | High | `virtual-scroll-to`, `virtual-grid-pinning` |
| W5 | `virtual/find.rs:135` | 26 | **B5** | The same settle verb, with a takeover token and cancel on input. | Medium | `virtual-list-find` |
| W7 | `virtual_grid.rs:2087` | 38 | **B5** (+A) | Wait-for-mount, scroll to a known position when unmounted, `focus({preventScroll})`, nearest alignment against covers. **Falsify:** a plain `focus()` would honour `scroll-padding`, but it can also scroll the page (the measured problem of lane 2B / L20). Keep `preventScroll` in B5. | High (keyboard grid) | `virtual-grid.spec`, `virtual-grid-pinning`, `data_table.spec` |
| W8 | `message_scroller.rs:148` (+ `message_scroller.js`, 1,065 lines) | 1,065 | **A + R** (+B5, G) | Every listener is passive (V), so there is no class-(1) work. Its sync work is scroll anchoring: correct `scrollTop` in the same frame as a ResizeObserver change (prepend preservation, follow-bottom). A: `overflow-anchor` is Baseline low since 2026-09-14 (Safari 27), so it qualifies under the section 2.1 policy now. Follow-bottom becomes native anchoring on a 1 px end sentinel (rows `overflow-anchor: none` while following). Prepend preservation becomes native anchoring on the rows. The anchored-turn spacer becomes a CSS `min-height` from a Rust-set variable. Jumps use B5, and the opening position uses G. The mode machine moves to Rust (R). Fallback without anchoring: Rust `onresize` plus `MountedData::scroll`, frame-exact on web (residue doc section 2). | High | `message_scroller`, `message-scroller-api`, `message-scroller-timing`, `chat-*` |
| C1 | `carousel/js_macros.rs:21,67` (audit 30) | 65 | **R** (dissolves) | Helper text spliced into C2, C3, C4 and C5 (V). `dxIsTrueEnd` becomes a Rust-published `data-dx-true-end-min/max` pair that Rust itself reads for C3/C5. `dxTrace` becomes Rust tracing, plus the debug `window` records the specs read, written through the bridge. Once its four users have moved, `js_macros.rs` and its SSR test (`ssr_tests.rs:412`) go (residue doc section 4). | Low | `carousel-edge-*` |
| C2 | `carousel/scroll.rs:77` | 114 | **B5** (`scroll-step`) | A command from Rust: `scrollBy(delta)` with snap suspended until `scrollend` or a timeout, `prefers-reduced-motion`. No commit path. **Falsify:** A via `scrollIntoView` on the target slide breaks `carousel-paging-inert` (it scrolls the page when the carousel is partly off-screen), so A is rejected. The procedure is generic enough for B5. | High | `carousel.spec`, `carousel-paging-inert`, `carousel-wheel-replay` |
| C3 | `carousel/scroll.rs:230` | 411 | **R** | Its own doc: "One read … followed by nothing but a `dioxus.send` -- no DOM write at all" (`:225-229`). The idle gate, settle and pin decision are logic over passive events, so they move to Rust. The bridge supplies `listen` (scroll, scrollend, wheel, touch; coalesced per rAF) and a per-frame `sample` verb for the pin tick. The `__dxWhenIdle` handoff with C2 disappears, because Rust orders its own writes. **Falsify:** desktop IPC volume while scrolling, which the bridge coalesces per frame. Pin re-centring decided a frame late on desktop delays a settle but paints nothing wrong (I). | High | `carousel*.spec`, `carousel-wheel-replay`, `carousel-virtual-wheel` |
| C4 | `carousel/drag.rs:99` | 545 | **B2** (+R) | Mouse/pen only, threshold, cancel committed `pointermove`, capture-phase click suppression, pointer capture, `scrollBy` output with snap suspended, rubber resistance at the true end. Spring-back becomes a CSS transition with a `linear()` spring (A). The release settle target moves to Rust (R). **Falsify:** the rAF spring was tuned on real hardware, and a CSS `linear()` approximation may feel different. If `carousel-edge-overdrag` or a hand check fails, the band physics stay a small residue inside B2's "resistance" variant. | High (feel) | `carousel-edge-overdrag`, `carousel-drag-free`, `carousel.spec` |
| C5 | `carousel/wheel.rs:110` | 477 | **R** (exit C2) | Passive (V): **no class-(1) work at all.** A per-gesture ownership decision plus a trackpad-momentum release classifier (smooth decay, notch detection, re-push, six release rules). That is a stateful heuristic over a stream (B limit), with no platform equivalent, so it becomes a pure Rust state machine (`carousel/wheel_band.rs`). Input: a *batched* per-frame `sample` with every wheel event and its own timestamp (not coalesced, because the notch and decay rules compare consecutive events). Output: one CSS variable per frame, with the spring home as a CSS transition. `cargo test` replays `playwright/fixtures/carousel-wheel-recordings.json` against outputs captured once from today's JS. On web it is frame-exact through microtasks. On desktop it is up to a round trip late, which is lag, never a wrong decision. Exit: C2 if a desktop measurement shows more than one extra frame of lag at p95 (residue doc section 3). | High | `carousel-edge-wheel`, `carousel-wheel-replay` |
| V11 | `preview/src/eager_head.rs:86,102,113` | 12 | **R** | A `Document` impl that inserts head elements through eval instead of `queue_effect`. This is ordering, not a commit decision. It becomes a bridge `head.insert` verb (or stays the trait default eval). | Medium | `oracle/tier2-html/global-stylesheet`, `stylesheets-present`, `first-paint-styles` |
| V19 | `preview/src/components/questionnaire/component.rs:66` | 36 | **G** (Rust) | Rust knows when the item swaps. Read the old height (sync on web), then in one render pin `height: old` and swap. Force a style read (`get_client_rect` of an inner wrapper gives the new natural height and commits the pin as its own style change). Then render `height: new`, and the CSS transition runs. **Falsify:** on desktop the pin and the new height can land in one frame: use the bridge `commit` verb between them. A via `interpolate-size` is Chrome-only. View transitions (Baseline low 2025-10) would animate a snapshot cross-fade, which looks different and is rejected. | Medium | `questionnaire-height.spec.ts` |

### 3.1 The (b) rows this affects

| id | where | maps to | note |
|---|---|---|---|
| L6 | `lib.rs:358-375` | **B1** `key=Escape → prevent, send` | The top-of-stack decision stays Rust. For uncontrolled layers, A (native popover/dialog close request) would need no listener, but the library's controlled `open` needs the veto, so keep B1. |
| L1 | `lib.rs:771-790` | **B1** + tiny residue | `data-dx-closing` mirrored by the close procedure in the same transaction. The "cancelled < 250 ms ago" window needs a `recent(event, ms)` predicate, or stays as 5 lines in the dialog procedure. |
| L2 | `lib.rs:843-852` | **B1** always-prevent `cancel` | `requestClose()` (Baseline low) does not remove the need: the veto is the Rust state. |
| L3, L7, L8, L9 | `lib.rs:886`, `:463`, `menu_sub.rs:429`, `navigation_menu.rs:888` | bridge `listen` with target-relation fields (`inside(id)`) | No cancel, so async is fine (a dismiss a microtask or IPC later). |
| L4 | `dialog.rs:508` etc., `ts/focus-trap.ts` | **B1** `focus-trap(container)` | Already TS; it becomes the B1 effect. |
| L10 | `context_menu.rs:37` | **B1** `outside(root) → prevent` | |
| L11 | `scroll_lock.rs:367` | **B1** `can-scroll` | A for modal dialogs (`overflow:hidden; overscroll-behavior: contain` with the current semantics) only on Chrome 144 / Firefox 150, and the root-scroller `scrollbar-gutter` constraint (module docs) still rules out `html{overflow:hidden}`. |
| L23 | `label.rs:63` | **B1** | |
| L25 | `drag_and_drop_list.rs:765` | **B1** + `dropEffect` effect | |
| O1 | `toast.rs:542` | **B1** `key=F6 ∧ exists(region)` + Rust focus move | The focus move can be async (I). |
| L21 | `pointer.rs:33` | **B2** (pointer capture, output to Rust) | |
| V17, V22 | preview photo wall, sidebar | **B2**, **B1** | demo / styled component |
| T5, T7, L16 | `popover_sync.rs:146,297`, `modality.rs:87` | bridge **transaction** (several ops in one JS turn) | "Same eval as `showModal()`" is an ordering need, not a commit decision. On desktop separate evals have no guaranteed relative order (the repo's own comments, `lib.rs:590-602`), so the bridge must offer an atomic batch. |
| L5 | `lib.rs:972` | bridge `call` (`getAnimations().finished`) | Async by nature. A (`transition-behavior: allow-discrete` + `overlay`) is blocked because `overlay` is Chromium-only (I). |
| T12, L27 | `scroll_clamp.rs:141`, `keyboard_inset.rs:55` | bridge `listen`/`observe` with options | Passive; a late close or inset is harmless. |

---

## 4. What genuinely stays bespoke, and why

**Revised 2026-10-26: nothing in class (d) stays bespoke.** An earlier version of this section kept W8, C5 and C1 as a residue. The re-attack is in [`sync-js-residue-2026-10-26.md`](./sync-js-residue-2026-10-26.md).

| block | was | now | what it would take to be wrong (falsification) | exit if it is |
|---|---|---|---|---|
| W8 message scroller (1,065) | residue, "A later" | **A + R** (+B5, G): native `overflow-anchor` on an end sentinel and on the rows; mode machine, opening position and scroll intent in Rust; a Rust `onresize` fallback | Chromium or Safari 27 anchoring picks a wrong anchor or drifts under token-rate streaming (`message-scroller-timing`, `chat-*` at N=0 and N=2) | keep the sentinel, and let the Rust `onresize` path make the correction on every engine (frame-exact on web; on desktop the bridge's `commit` ordering) |
| C5 carousel wheel band (477) | residue | **R**: a pure Rust classifier fed a batched per-frame event stream, output as one CSS variable | a desktop measurement shows more than one extra frame of lag at p95, a lost event, or replay divergence (residue doc section 3.5) | C2: the same crate compiled to page-side wasm, asynchronously compiled, called synchronously from the passive listener |
| C1 helper macros (65) | residue, "shrinks" | **dissolves**: Rust-published `data-dx-true-end-min/max`, Rust tracing | none: it is text spliced into its four users (V) | — |
| (in (b)) L1's 250 ms cancel window (~5) | tiny residue | unchanged | — | a `recent()` B1 predicate, if a second user appears |

Everything in class (d) leaves bespoke JS. The JS left in the page is the bridge and the B interpreter. Class (c) (pre-wasm `PREPAINT_JS`, the modality writer, the inline copy buttons) is unaffected: it must run before wasm and is out of scope.

---

## 5. The proposal

### 5.1 Layers (apply in this order per block)

1. **R first, where the "sync" label is false.** The block only reads and reports, runs a command, or drives a passive effect that may answer late: W1, C3, V11, C5, C1, W8's mode machine, plus the async (b) rows. Logic moves to Rust over the plain bridge verbs.
2. **A where the platform decides, within the policy** (section 2.1): anchor positioning for T8/T11, `overflow-anchor` scroll anchoring for W8, `scroll-padding` for covers (W3/W7), scroll-driven fade depth (S14, already in place), `linear()` springs (C4).
3. **G for one-shot layout work:** a gate attribute plus Rust: T8's fallback and zone check, T9, T10, T11's fallback, S1, O3, V19. A new bridge `commit` verb (rAF plus forced style) supports the two-style-change orderings on desktop.
4. **B for decisions at event or frame time and for continuous loops that fit a closed vocabulary:** B1 (S11 plus 10 (b) sites), B2 (L30, O5, C4, L21, V17), B3 (S14), B5 (W3, W5, W7, C2).
5. **No residue** (revised 2026-10-26). W8 is A + R (+B5, G), C5 is R with a measured exit to C2, and C1 dissolves (residue doc). The residue allowlist of the gate starts empty.

**What this subsumes (I):**
- All 22 (d) blocks and all 3,894 (d) lines (revised from 19 and 2,287).
- All 13 class-(1) `preventDefault` sites, through B1/B2 (one implementation per predicate).
- The geometry copies (the fixed probe pasted into `select/components/list.rs:102` and `scroll_clamp.rs:58`), which become one Rust helper.
- The `__dxWhenIdle` / `__dxCancelSnapRestore` cross-eval expandos.
- Most `listener-ok:` / `blocking-ok:` markers, because listeners live only in the interpreter.

**What it does not subsume:** class (c); the desktop latency of G and R, which is bounded but unmeasured (C5's lag above all, which carries the C2 exit); the quality of Safari 27's new scroll anchoring for W8, which keeps the Rust `onresize` fallback as its exit; the visual polish of C4's spring if `linear()` is not close enough.

### 5.2 The bridge additions this needs (on top of row 434's listen/call/get/observe/match)

- `transaction(ops[])`: several calls in one JS turn (T5, T7, L1, L16 ordering).
- `commit()`: resolve after rAF plus a forced style flush (row-421 ordering on desktop).
- `rules.install(el, rule)` / `rules.remove`: the B interpreter entry point. Rules can also be read from `data-dx-rule-*` attributes, for render-driven state.
- `sample(el, fields, until)`: per-frame sampling while a condition holds (C3's pin tick). It is coalesced or **batched** per field: C5 needs every wheel event of the frame with its own timestamp, because its notch and decay rules compare consecutive events (residue doc section 3.2).
- `set(el, var, value)`: write a CSS variable on an element whose style Rust does not own (C5's band offset, W8's turn top).

### 5.3 The gate (`scripts/check-bridge-rules.sh`, new; and the row's planned `check-js-in-rust.sh`)

1. The `Rule` enum in Rust (`primitives/src/bridge/rules.rs`) and the TS interpreter's discriminant switch (`primitives/src/ts/rules/*.ts`) list the same variants, with no default arm. The gate parses both.
2. Every variant has a `bun test` case and is named by at least one Playwright spec (grep for a `rule:` tag).
3. No `preventDefault`, `stopPropagation`, `requestAnimationFrame`, `ResizeObserver`, `MutationObserver` or `IntersectionObserver` in `primitives/src/**/*.ts` outside `ts/bridge.ts`, `ts/rules/` and the residue allowlist (`path<TAB>reason<TAB>row`, shrink-only, stale entries fail). The allowlist starts empty. If the C5 JS path is kept behind a cfg until the desktop measurement, it is listed with its row and the measurement as its expiry.
4. No such token in a Rust string at all: no JS in `.rs` except the class-(c) allowlist.
5. A G-gated component must declare its gate attribute, and a first-frame spec must assert `opacity: 0` while the gate is pending.
6. As CLAUDE.md requires, the new script goes into `run-gates.sh`'s list and into CLAUDE.md's gate sentence.

### 5.4 Lane plan (after lane 3.0 has built the bridge; disjoint files; sizes I)

| batch | lane | blocks | files it owns | must not touch | size | agent-h |
|---|---|---|---|---|:-:|---:|
| 1 | **3.B1 event gates** | S11, L1, L2, L4, L6, L10, L11, L23, L25, V22; the B1 interpreter plus the gate script | `ts/rules/gate.ts`, `bridge/rules.rs` (B1 variants), `scripts/check-bridge-rules.sh`, `lib.rs` (the L1/L2/L6 blocks only), `label.rs`, `scroll_lock.rs`, `context_menu.rs` (L10 only), `drag_and_drop_list.rs`, `input_otp.rs`, `ts/focus-trap.ts`, `dialog.rs`/`alert_dialog.rs`/`popover.rs` (focus-trap glue only), `preview/src/components/sidebar/component.rs` | `top_layer/`, `toast.rs`, `select/`, `carousel/` | L | 10 |
| 1 | **3.PL placement A+G** | T8, T9, T10, S1 | `top_layer/{anchor_fallback,scroll_clamp,styles,mod}.rs`, `select/components/list.rs`, the theme/engine CSS for anchoring, the popover/select/combobox component sheets (anchor-size rule) | `context_menu.rs` (T11 is batch 2) | L | 12 |
| 1 | **3.B5 reveal/settle** | W3, W5, W7 (+ W1 as R if lane 2D has not already) | `ts/rules/reveal.ts`, `bridge/rules.rs` (B5 variants; append-only region), `virtual/{scroll_to,find}.rs`, `virtual_grid.rs` (FOCUS_CELL only), `virtual/bridge.rs` | `virtual_list.rs` and the rest of `virtual_grid.rs` (lane 2D) | M | 6 |
| 1 | **3.QH height glide G** | V19 | `preview/src/components/questionnaire/{component.rs,style.css}` | — | S | 2 |
| 2 | **3.GS gestures + toast** | L30, O5, O3, O1 (B1 reuse), L21, V17 | `ts/rules/pan.ts`, `ts/rules/publish.ts` (if the O3 fallback is needed), `bridge/rules.rs` (B2 region), `drawer.rs`, `toast.rs`, `pointer.rs`, photo-wall demo | `carousel/` | L | 10 |
| 2 | **3.CM context menu A+G** | T11 (+ point-anchor element) | `context_menu.rs`, `top_layer/scroll_clamp.rs` | — | M | 4 |
| 2 | **3.SF scroll fade** | S14 | `scroll_area.rs`, `ts/rules/publish.ts` (owned here if 3.GS does not need it; otherwise sequence after it) | — | S | 3 |
| 3 | **3.CA carousel** | C2 (B5), C3 (R), C4 (B2 + R), and their C1 uses removed (`dxIsTrueEnd` to `data-dx-true-end-min/max`, `dxTrace` to Rust tracing) | `carousel/*` except the 3.WB files (`scroll.rs`, `drag.rs`, `root.rs`, `content.rs`, `virtual_content.rs`, `helpers.rs`, …) | `carousel/wheel.rs`, `carousel/wheel_band.rs` (3.WB), `js_macros.rs` | L | 13 |
| 3 | **3.WB carousel wheel band** | C5 to a pure Rust classifier (`wheel_band.rs`), batched `sample` input, CSS-variable output, `cargo test` replay of the committed recordings, the desktop-measurement protocol written down | `carousel/wheel.rs`, `carousel/wheel_band.rs` (new), `playwright/fixtures/carousel-wheel-expected.json` (new) | the other `carousel/*` files (3.CA) | M | 10 |
| 3 | **3.MS message scroller** | W8 to A + R (+B5, G): end sentinel and row anchoring in CSS, mode machine in Rust, `onresize` fallback; `message_scroller.js` deleted | `message_scroller.{rs,js}`, its `style.css` and component | — | L | 16 |
| after 3 | **main loop** | delete `carousel/js_macros.rs`, rewrite `ssr_tests.rs:412` | those two files | — | S | 1 |

The `bridge/rules.rs` enum is shared. Lanes append in separate marked regions (B1 / B2 / B5 / B3), and the main loop merges them. If that proves uncomfortable, 3.B1 creates the file with all four region headers first.

**Total ≈ 86 agent-hours** (revised from ≈ 64; the audit's step 3 was ≈ 66), with **no residue module** instead of 22 (I). The extra 22 h buy W8 and C5 in Rust instead of typed TS.

### 5.5 Risks (in order)

1. **Desktop/mobile are unmeasured and not in CI** (no gtk/webkit2gtk here). G latency, R event volume and B interpreter behaviour on WebView2/WKWebView/WebKitGTK are all inferred. Mitigation: the slow-channel harness (5.6) plus one manual desktop pass per batch on a machine with a webview.
2. **Placement (3.PL)** is visible on every open. A-only must be proven against the existing geometry sweeps, the fallback under `stripAnchorSupportsBlock`, and the gate under a first-frame check. Focus into an `opacity:0` panel must keep `touch-open-focus`/`modal-initial-focus` green.
3. **B vocabulary creep:** any request for "one more predicate" goes through a backlog row. If a predicate needs arithmetic or state over a stream, the logic goes to Rust (R, or G where a paint is at stake), not into a B variant.
4. **Gesture feel (B2, C4's spring):** hand checks on a trackpad and touch device, not only specs.
5. **Row 421 ordering on desktop (O3, V19):** depends on the `commit` verb. If it misbehaves, O3 falls back to B3.
6. **WebKitGTK anchor-positioning support** is unknown, so the G fallback cannot be dropped for Linux desktop until a probe confirms it.

### 5.6 Verification construction: the slow-channel harness (new; I)

All specs run on Chromium web, where the bridge is microtask-fast (section 1.2). That **hides** every desktop-only latency bug. Proposal: a test-only bridge mode, `window.__dxBridgeDelayFrames = N`, that defers every JS→Rust and Rust→JS bridge message by N rAFs. G-mapped specs then run with N = 0 and N = 2. **Limit:** `MountedData` reads stay synchronous on web, so code under G must take its reads through `get` or follow them with an explicit `commit()` for the harness to model desktop. A lane rule must say so.

---

## 6. Not verified / caveats

- Nothing was built or run. All latency figures (G on desktop, IPC cost) are estimates.
- The "microtask-synchronous on web" chain (section 1.2) is verified piece by piece (eval channel resolve, `queueMicrotask` executor, run loop without idle/rAF), not end to end.
- The spec quotes come from a WebFetch summary of the TR page (W). Engine behaviour for `position-area` shift and `max-height: 100%` should be confirmed in the 3.PL spec on Chromium, and by a probe on Firefox and Safari.
- The web-features `overscroll-behavior` entry now records the post-Chrome-144 semantics, so Safari is listed as unsupported even though it implements the older semantics (W, `lists.w3.org/Archives/Public/public-css-archive/2025Dec/0150.html`, MDN page).
- WebKitGTK: only the 2.50.0 release notes were read (W). Anchor positioning is not mentioned there, which neither confirms nor rules it out.
- The trampoline (C1 for window-level listeners) was evaluated on paper. The modality-writer interaction and the desktop `stopPropagation` gap are V, but the trampoline was not prototyped.
- Line counts are the audit's (string extents, comments included), and are meant for ranking only.

Sources (W): web-features 3.44.0 `data.json` (npm registry, published 2026-10-22); CSS Anchor Positioning Level 1, <https://www.w3.org/TR/css-anchor-position-1/>; web.dev, "Loading WebAssembly modules efficiently", <https://web.dev/articles/loading-wasm>; CSSWG issue thread on overscroll-behavior's target, <https://lists.w3.org/Archives/Public/public-css-archive/2025Dec/0150.html>; MDN `overscroll-behavior`, <https://developer.mozilla.org/en-US/docs/Web/CSS/overscroll-behavior>; WebKitGTK 2.50.0 release notes, <https://webkitgtk.org/2025/09/17/webkitgtk2.50.0-released.html>.
