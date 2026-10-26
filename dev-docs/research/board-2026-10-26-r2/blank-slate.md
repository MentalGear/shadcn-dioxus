# Board round 2, lens `blank-slate` (row 434 step 3), 2026-10-26

**Verdict: HOLDS WITH CHANGES**

DESIGN findings: 1, DETAIL findings: 7

Summary (at most 8 lines):
- Method: I designed from the audit, the code and the Dioxus 0.7.9 sources first (section 1), and only then read the plan (section 2). The two designs converge on the root rule ("publish before, decide locally"), one bridge with one eval sink, scope-owned ref-counted page behaviours, keeping frame-rate streams off vdom handlers, rejecting web-sys and page-side wasm, and enforcing at the sinks. Two independent derivations reaching the same core is evidence that the core holds.
- Where the plan is better than my design: its R/B split (Rust logic over async verbs wherever there is no commit-time need) keeps more in Rust than my "anything outside rsx is a page behaviour" did. I adopt it.
- The one DESIGN gap: the plan applies "publish before" to *decisions* but not to *commands*. Imperatives that mirror Rust state (showPopover/hidePopover, showModal/close, `indeterminate`, `defaultChecked`) are still edge-triggered bridge calls from effects. They should be level-triggered: Rust publishes the desired state as an attribute and a page reconciler applies it. That closes the lost-command, early-command and stale-echo class by construction, and saves a frame per overlay open on desktop (finding 1). It is additive: no other part of the core changes.
- DETAIL majors: rung 1's "rsx `ontoggle` where Rust knows what it asked for" is unsound, because toggle events coalesce (finding 2). The hot vdom handlers already in the tree, and in-flight lane 2D's audit-era brief, have no owner under the new rule (finding 3). `dev-docs/js-interop.md`, which CLAUDE.md sends every agent to, still teaches the rejected interpreter design (finding 4).
## 1. Independent design (written before reading any plan document)

Inputs read for this section only: `dev-docs/research/js-audit-2026-10-25.md`, the code its blocks point to (`js_listener.rs`, `top_layer/popover_sync.rs`, `top_layer/mod.rs`, `modality.rs`), and the Dioxus 0.7.9 sources (`dioxus-web` `document.rs`, `events/{mod,mounted,toggle}.rs`; `dioxus-desktop` `document.rs`, `query.rs`, `webview.rs`, `protocol.rs`; `dioxus-interpreter-js` `ts/core.ts`, `ts/native.ts`).

### 1.1 The facts the design must respect (all V)

F1. **Where the DOM is.** On web, Rust runs in the page (wasm, same thread). On desktop and mobile (wry: WKWebView / Android WebView) Rust is a native process and the DOM is remote: every DOM read is an eval round trip (`dioxus-desktop/src/query.rs:60-80`, `element.rs` `scripted_getter!`).

F2. **A Dioxus event handler is synchronous with its DOM event on both renderers, but its writes are not.** Web: the handler runs inside the DOM event. Desktop: `handleEvent` sends the event with a *synchronous* XHR (`interpreter-js/src/ts/native.ts:379,569-590`) and only `preventDefault`/`stopPropagation` come back in the response (`desktop/src/webview.rs:101,194,630`); the DOM edits the handler caused are flushed later on the edits channel (`webview.rs:537-587`). So: *a decision* (prevent default, stop propagation) can be made in a Rust handler on every platform; *a frame-exact DOM write or any DOM read* cannot.

F3. **High-rate events in rsx are a construction hazard.** A bubbling rsx listener is one listener on the app root with no options (`core.ts:109-130`): `onwheel`/`ontouchmove`/`ontouchstart` become non-passive root listeners (the browser intervention that treats window/document/body listeners as passive does not cover a root `div`), and on desktop every delivered event is a blocking XHR into Rust. Any frame-rate stream (scroll, wheel, pointermove during a drag) handled in rsx costs one blocking IPC per event on desktop/mobile.

F4. **Every eval compiles a new function.** Web: `Function::new_with_args("dioxus", code)` per call (`dioxus-web/src/document.rs:220`); desktop: `new AsyncFunction("dioxus", script)` per call (`query.rs:79-80`). A per-effect `format!`ed script is a parse per call, needs CSP `unsafe-eval`, and splices Rust values into JS text (`popover_sync.rs:145-152`: `getElementById('{id}')`), which is an injection/quoting hazard by construction.

F5. **Missing surface in 0.7.9:** no `target`/`relatedTarget`/`detail`, empty `ToggleData`/`FocusData`, no `onclose`/`onbeforematch`, no listener options, no window/document listeners, no imperative popover/modal/animation/media-query/visualViewport/pointer-capture APIs, `set_focus` without `preventScroll` (audit section 2). On web only, `WebEventExt::as_web_event()` and `MountedData::downcast::<web_sys::Element>()` expose all of it (`dioxus-web/src/events/mod.rs:190-210`, `events/toggle.rs:10-16`), but nothing equivalent exists on desktop.

F6. **Effects run after the DOM patch on both renderers** (web `queue_effect`; desktop does not run the vdom while an edit flush is pending, `edits.rs:117`), so anything sent from an effect finds its element.

### 1.2 The root rule

**State and policy live in Rust; the frame lives in the page. Rust publishes, ahead of time, everything a page-side decision needs, as attributes on the element the decision is about; the page decides synchronously, inside the DOM event or frame, from what is published plus what it measures; it reports outcomes back to Rust as typed events.** No DOM event and no frame may wait for Rust (F1, F2). Attributes, not messages, carry published state, because an attribute lands in the same edit batch as the DOM change that motivated it, so a page-side reader can never see new structure with old configuration (messages race the edits channel on desktop; F2).

### 1.3 Placement rule (a decision procedure, applied per behaviour)

1. Must it act before wasm (theme, modality before first input, pre-hydration buttons)? -> **pre-wasm inline script**, generated from the same typed source as the runtime (never a hand-kept copy).
2. Otherwise, it stays **pure Rust (rsx handler / attribute / `MountedData`)** only if ALL hold: the event and its data exist in Dioxus; the listener sits on an element the component renders; the event is not frame-rate or scroll-blocking (F3: no `onwheel`/`ontouch*`/drag `onpointermove`/per-frame `onscroll` in a primitive); its visible effect may land one render later (web) or one IPC later (desktop) without being seen; and any DOM read is one-shot, not a loop.
3. Otherwise it is a **named page behaviour** (section 1.4).

The rule has a direction of travel: when Dioxus grows a missing surface (F5), a behaviour moves from 3 to 2 by re-running the procedure; the architecture does not change.

### 1.4 Page behaviours

- **One runtime per document**, written in TypeScript, bundled and committed with a hash manifest (the existing `ts/` -> `js/` chain), installed once, idempotently, by the bridge (1.5). It owns the shared machinery: listener registration with options and derived removal (today's `listen`/`cleanup` PRELUDE), reference-counted window/document listeners, one IntersectionObserver/ResizeObserver/MutationObserver per configuration, rAF coalescing, `matchMedia`, the viewport probe, the geometry helpers.
- **Behaviours are named and element-scoped**: `attach(name, element, ...)` returns an instance whose teardown removes everything it registered. Lifetime is owned by the Rust scope that mounted the element (attach from `onmounted`/effect, detach on drop), with an `isConnected` sweep in the runtime as a backstop so a lost detach cannot leak. (DOM-driven attach through a document-wide `childList` MutationObserver was considered and rejected: it scans every added subtree, which is what a virtual list does on every scroll.)
- **"The attribute is the command" for state-like imperatives.** Where an imperative API mirrors Rust state (popover shown, modal open, `indeterminate`, `defaultChecked`, scroll lock active), the behaviour observes its own element's attribute (an attribute-filtered MutationObserver on that element, delivered before paint) and reconciles: Rust renders `data-state="open"`, the page calls `showPopover()`/`showModal()` in the same frame. This deletes the "effect -> eval per toggle" pattern and its races (row 115's stale `toggle` echo is a symptom of two writers).
- **One-shot imperatives** (focus this, scroll that item into view, measure now) are commands over the bridge, carrying the element's id as data, never spliced into code.
- **Outcomes** (native close, dismiss, measured size, visible range) are typed events back to Rust, coalesced to at most one per frame for streams.

### 1.5 The bridge

- **One persistent eval per document** (installs the runtime, then loops on `dioxus.recv()` dispatching `{op, args}`), so after boot no code is compiled per call (F4) and no Rust value is ever spliced into JS.
- **Closed protocol**: a Rust enum of commands and a Rust enum of outcomes (serde), and a TypeScript declaration of the same shapes checked against it (generated, or a round-trip test); the only call site of `document::eval` in the library is the bridge.
- Rust API surface: `use_behaviour(name, element_id, attrs)` (hook: attach on mount, detach on drop, outcomes as a `Callback`) and `bridge().command(cmd)`.

### 1.6 Enforcement by construction

- Gate: no `document::eval` / `JsListeners::install` outside the bridge module; no JS text in `.rs` (ratchet down to the pre-wasm generated constants).
- Gate: no frame-rate/scroll-blocking rsx handler in `primitives/src` (F3) without a reasoned allowlist entry.
- Gate: every behaviour name used from Rust exists in the runtime, and vice versa (no orphan).
- Gate: the protocol types and the TS declarations agree.
- Existing gates (`check-js-listeners`, `check-js-bundle`) fold into the runtime.

### 1.7 Platforms

Web and desktop/mobile run the same runtime through the same bridge; the only platform difference is transport latency, which the root rule makes irrelevant to correctness. Blitz (dioxus-native) has no DOM and `eval` returns `Unsupported`: behaviours are absent there, and each hook must have a defined no-DOM arm (today's `feature = "web"` split).

### 1.8 Alternatives rejected

- **web-sys in Rust on web only** (F5 says it can do everything there): two implementations, one per platform, drift by construction. Rejected.
- **A Rust/wasm "page agent"** (same web-sys crate linked into the app on web, compiled to a second wasm loaded into the webview on desktop): the only design with almost zero hand-written JS. Rejected for now: desktop needs either a committed binary blob in a crate users install (unreviewable, against the repo's supply-chain stance) or a wasm32 + matching `wasm-bindgen-cli` toolchain on a desktop-only user's machine; the agent must load before first interaction in the webview; and `Closure` lifetimes in web-sys re-create the leak class `js_listener.rs` was built to close. Worth revisiting only if the hand-written TS outgrows review.
- **Custom elements** (`attributeChangedCallback` is the platform's own "attribute is the command"): needs wrapper elements (Safari has no customized built-ins), which breaks the many `>` selectors and changes structure. Rejected; the per-element attribute observer gives the same semantics without a wrapper.

## 2. My design compared with the hardened plan (read only after section 1 was written)

| element | my design (section 1) | hardened plan | verdict |
|---|---|---|---|
| root rule | state/policy in Rust, frame in the page; publish ahead as attributes; decide locally; report typed outcomes | "never ask Rust at commit time; publish beforehand (into the DOM), decide locally" (root doc 1.1) | **converges** (same F2 facts: sync XHR events, async edits, async getters) |
| hot events | F3: no frame-rate/scroll-blocking rsx handlers (root listener, no options; a blocking XHR per event on desktop) | root doc 1.4 item 6, 5.2 "hot events stay off the vdom", `check-hot-events.sh` | **converges** (the plan also has `onresize` per row; better) |
| placement | pre-wasm / rsx (all conditions) / page behaviour | rsx / named behaviour / bridge stream, plus the A, R, G layers | **converges on the shape; the plan is better**: R (async-tolerant logic in Rust over verbs) keeps more in Rust than my rung 3. My rung 2 condition "the event carries the data" is missing from the plan (finding 2). |
| behaviours | named, element-scoped, attached by the owning scope, ref-counted, `isConnected` backstop | closed registry, scope-owned, ref-counted, WeakMap, leak spec, pure `decide()` plus adapter, vectors, fail-open, epoch | **converges; the plan is richer**. Backstop: finding 8. |
| state-mirroring imperatives | **"the attribute is the command"**: a level-triggered page reconciler | bridge `call`/`transaction` from effects (T5, T7, L16, L1), with a lane rule "issue from an effect" and a ready handshake | **diverges** (finding 1, DESIGN) |
| bridge | one transport; a closed protocol of Rust enums and generated TS; no value spliced into code | one eval site in `bridge/transport.rs`, closed enums and tables, generation parity, constant plus JSON | **converges**. Per-op versus persistent eval is not a design point (attacks that failed). |
| pre-wasm | generated from the same typed source | out of scope; copies kept identical by gates | **diverges, DETAIL** (finding 6) |
| alternatives | web-sys on web only, a Rust page agent and custom elements rejected | the same three rejected (C2, simpler-alternative) | **converges** |
| enforcement | eval sink gate, JS ratchet, hot-handler gate, name parity | sinks plus clippy `disallowed_methods`, `git ls-files` universe, ratchet against the merge base | **converges; the plan is stronger** |
| G for fallback placement | (I had the fallback page-side) | Rust behind a CSS gate; A on evergreen engines | **the plan's choice is acceptable** (attacks that failed) |

## 3. Findings

### 1. DESIGN, major: state-mirroring imperatives must be level-triggered from a published attribute, not edge-triggered bridge calls from effects

**Claim attacked.** Placement and bridge design: T5, T7 and L16 go through a bridge `transaction` issued from an effect, and L1's `showModal`/`close` through bridge `call` (root doc 3.1 rows "T5, T7, L16" and "L1"). Lane rule 1.4 item 3: "a bridge op that depends on a render's DOM is issued from an effect or `onmounted`, never from the handler". The loading race is handled by "a bounded ready handshake (#5919)" (5.2).

**Evidence.**
- (V) On desktop, edits are applied inside a `requestAnimationFrame` callback (`dioxus-interpreter-js-0.7.9/src/ts/native.ts:444-458`). Effects run only after the webview acknowledges that flush (root doc 1.4, `webview.rs:537-590`). An eval travels on a different channel (`evaluate_script`) from the edits (a WebSocket, `native.ts:460-490`).
- (I) So the plan's open path on desktop paints `data-state=open` with the panel still `:not(:popover-open)` (display none) in frame N, and the show happens in a later webview task: frame N+1 at best, plus the G transaction.
- (W) A MutationObserver callback is a microtask, and a microtask checkpoint follows every rAF callback (HTML, "clean up after running script"). A page reconciler that observes the element's own published attribute therefore calls `showPopover()`/`showModal()` inside frame N's rendering opportunity, before style, layout and paint, on desktop and on web alike.
- (V) The plan's own sources show the edge-trigger failure class: #5919 (an eval runs before the `document::Script` of the same render, cited in `dioxus-best-practice-review-2026-10-26.md:13`), #5734 (a getter never completes), row 115 (a stale `toggle` echo of the hook's own `showPopover`, `popover_sync.rs:236-250`), and `lib.rs:590-606` (binding `<dialog open>` declaratively silently skips `showModal()`, so there are two writers of one state).
- (I) An edge-triggered command that is lost, arrives early, or arrives reordered is lost state. A level-triggered reconciler converges to the published value whenever it runs, including when the runtime loads late, so #5919 needs no queue for this class.

**Concrete change.**
- Add a rung between "rsx handler" and "R": **an imperative browser API that mirrors Rust state is reconciled page-side from a published `data-dx-*` attribute** (never the native `open` attribute, per `lib.rs:602`).
- The reconciler is one registry behaviour (`reconcile{kind: popover|popover-show-only|modal|indeterminate|default-checked}`), attached by the owning scope like every behaviour, with one attribute-filtered MutationObserver per element. It reconciles at attach (level) and on every change.
- It is the **only** caller of show/hide for its element. It suppresses the toggle events it caused itself (it reads `newState` synchronously), and reports to Rust only browser-initiated changes (light dismiss, Escape) as typed outcomes.
- `dxOpenFocus` runs inside the same reconcile step, so "same eval as `showModal()`" holds by construction.
- G's `beforetoggle` clear and generation move into the same reconciler (see finding 5).

**What it subsumes.**
- The lane rule "issue from an effect" for this class.
- The `transaction` ordering need for T5, T7 and L16.
- The ready-handshake queue for state-mirror ops.
- The row-115 stale-echo class and the two-writer `<dialog open>` trap.
- One desktop frame on every overlay open (16 driver call sites: `use_popover_sync`, `use_popover_shown_while_mounted`, `use_dialog_open_driver`).
- S8 (`indeterminate`) and S9 (`defaultChecked`) as attributes instead of verbs.

**Not subsumed.** One-shot commands (focus a cell, scroll an item into view, `scrollStep`) and streams. They stay R/B as planned.

**Why DESIGN and not DETAIL.** It changes the placement rule (a new rung) and the plan's assumption that commands are bridge calls. Every other element of the core is untouched. It is not a blocker: the plan as written works with its lane rule and handshake, at a frame per open on desktop and with a class of failure that is guarded rather than prevented.

### 2. DETAIL, major: rung 1 must require that Dioxus delivers every field the decision reads; "rsx `ontoggle` where Rust knows which state it asked for" (T4/T6) is unsound

**Claim attacked.** Root doc 2.2, placement table row 1 ("T4/T6 (`ontoggle` where Rust knows which state it asked for)"), and 3.1 row T4/T6.

**Evidence.**
- (V) `ToggleData` is empty (`dioxus-html-0.7.9/src/events/toggle.rs`).
- (W) WHATWG "queue a popover toggle event task" (https://html.spec.whatwg.org/multipage/popover.html) coalesces: if a toggle task is pending, it takes the pending task's `oldState`, cancels that task and queues one event. It never compares old with new, so show then hide before the task runs fires **one** `toggle` with old = new = closed.
- (I) Concretely, Rust asks open (effect: `showPopover`), and a light-dismiss pointerdown follows in the same window. Rust then receives one data-less `ontoggle`. Under "Rust knows what it asked" it reads that as the echo of its open and stays open while the DOM is closed: the row-115 symptom inverted. Counting echoes cannot work, because the coalescing makes the number of events differ from the number of transitions.

**Concrete change.**
- Rung 1 reads: "target inside a Dioxus element, an event Dioxus supports, no listener option, **and every field the decision reads is present in the Dioxus event data**". `ToggleData`, `FocusData`, `CancelData` and `SelectionData` are empty, and `MouseData` has no `detail`.
- T4 and T6 take `newState` page-side: the reconciler of finding 1, or failing that, bridge `listen(toggle, [newState])`.

**Subsumes.** Every future "infer it from what Rust asked" mapping on an empty event (L3's `detail`, focus-within from `onfocusin`/`onfocusout`, which lane 2B already rejected for L18).

### 3. DETAIL, major: the hot vdom handlers already in the tree, and in-flight lane 2D's brief, have no owner under the new rule

**Claim attacked.** 5.2 "Hot events stay off the vdom ... applies to W1, W8, O3, and to lane 2C's S2/S3", 5.4 (lane table), and W1's row ("**Not** Dioxus `onscroll`/`onresize` ... Lane 2D owns this file while it is in flight").

**Evidence (V).**
- Per-row `onresize`: `virtual_list.rs:486`, `table_rows.rs:289`, `virtual_grid.rs:729,1360,1526`. The root doc 1.4 item 6 itself counts this as "50 blocking round trips inside the rendering step".
- `select/components/list.rs:503,591` `onscroll` (S2, landed by step 2).
- `chart/components/chart.rs:701` `onpointermove` (hover hit test).
- `context_menu.rs:531` `onpointermove` (long-press tracking: a blocking XHR per move on mobile, where long press matters).
- No lane in 5.4 lists these sites. 3.VS must not touch `virtual_list.rs`. 3.PL owns `list.rs` only for S1.
- Lane 2D is in flight (`dev-docs/STATUS.md:24`, branch `lane/js-bridge-2d`, no commits yet). It was scoped by the audit (`js-audit-2026-10-25.md` section 5 "Virtual engine", section 7 row 2D) to replace W1 with Rust `onscroll`/`onresize`, which is exactly what the hardened W1 row forbids. Its exit criterion ("desktop cannot satisfy the ordering") cannot be evaluated here, because there is no desktop build. (I) whether 2D has been re-briefed: not visible in the tree.
- `scripts/check-hot-events.sh` does not exist yet (`ls scripts`).

**Concrete change.**
- Re-brief 2D now to bridge `listen`/`observe` with the page-side reducer (or stop it until 3.0 lands).
- Give the per-row `onresize` sites to 2D/3.VS as one observe-by-key stream.
- Add S2's `onscroll` to 3.PL's list (same file).
- Put chart and context-menu `onpointermove` into 3.GS.
- Land `check-hot-events.sh` with these sites as a shrink-only baseline in lane 3.0/3.0b, so the gate is not born red or born allowlisted forever.

**Subsumes.** The plan's own desktop hazard on the hottest path (every windowed list).

### 4. DETAIL, major: the agent-facing rule page teaches the rejected design

**Claim attacked.** 5.3 item 9 (one gate owner; the failure message names `dev-docs/js-interop.md`), and lane R "landed".

**Evidence (V).**
- `dev-docs/js-interop.md:12-13` says "generic verbs (`listen`, ..., `transaction`, `commit`, `sample`) and rules as data (a closed Rust `Rule` enum applied by one interpreter)".
- `:18` says the only new JS is "a new bridge verb or `Rule` variant".
- `:36-38` route conditional cancels to `Rule::Gate` ("one interpreter decides at event time"), drags to `Rule::Pan`, layout values to `Rule::Publish`.
- `:49` and `:63` say "one local interpreter decides. That is the bridge".
- The hardened plan dropped the interpreter (2.2: "four board members broke that form") and dropped `commit()` (1.4 item 1, 5.2).
- `git log`: js-interop.md landed at 09:49:55, and the hardened plan at 09:55:50.
- `CLAUDE.md:51-52` sends every agent to this page "before you write any interactive behaviour".

**Concrete change.**
- Rewrite js-interop.md to the three-way rule (plus finding 1's rung), the closed `Behaviour` registry and the verb list without `commit`.
- Replace the "I want to" rows that point to `Rule::*`.
- Point `check-js-volume.sh`'s header (it mentions "an interpreter") at the same terms.

**Subsumes.** Lanes building the rejected interpreter from the doc CLAUDE.md tells them to read first.

### 5. DETAIL, minor: G's `data-dx-placed` has two writers, and the vdom cannot see the page's clear

**Claim attacked.** 2.3 items 1 and 4: "The bridge removes `data-dx-placed` synchronously in `beforetoggle`" and "written only with the generation of the open that asked".

**Evidence.**
- (V) `set` accepts only `--dx-` custom properties (5.2), so Rust writes the attribute through rsx. Dioxus diffs against its own vdom, not the DOM.
- (I) After the page removes the attribute, a Rust render with the same value writes nothing. Every open path is gated, including native `popovertarget` (2.3 item 1; the fixtures at `preview/src/components/top_layer/component.rs:263,305,582`), which Rust did not initiate.
- (I) If the generation derives from Rust's `open` transitions, a native reopen produces no new value. The panel then stays pending until the CSS fail-open (up to about 100 ms) and paints unplaced, on exactly the engines G exists for.

**Concrete change.**
- The page owns the generation: `beforetoggle` increments it and reports it with the open. Rust echoes it in `data-dx-placed="<gen>"`.
- Alternatively, the finding-1 reconciler owns the attribute outright.
- Add a spec "native `popovertarget` reopen on the fallback engine" at N = 0 and N = 2.

### 6. DETAIL, minor: class (c) keeps hand-kept copies the row says it subsumes

**Claim attacked.** Root doc 4 ("class (c) ... out of scope") against the row file's Detail.

**Evidence (V).**
- `dev-docs/backlog/434-*.md` Detail: "Subsumes: ... the three-copy and two-copy identity checks".
- The audit, section 6: "`PREPAINT_JS` and `MODALITY_JS` are generated from one `.ts` and embedded by `include_str!`".
- `js-interop.md:79-80`: "`check-theme-mode-writer` and `check-open-focus` keep those copies identical".

**Concrete change.** Generate `MODALITY_JS`/`PREPAINT_JS` from `ts/prepaint.ts` through the bundle chain, `include_str!` them in `modality.rs` and `theme.rs`, and have one script write the inline copy into `preview/index.html`. The identity gates become a generated-file check.

**Subsumes.** Two copy-identity gates (detect becomes prevent).

### 7. DETAIL, minor: the volume ratchet taxes typing

**Claim attacked.** 5.3 item 5 ("a port that moves lines from a Rust string into a new `.ts` must still lower the sum") together with 2.2 (a pure `decide()` plus an adapter, typed).

**Evidence.**
- (V) `scripts/check-js-volume.sh` header: `js_file` counts non-blank, non-comment lines of the `.ts` source.
- (I) Interfaces, type annotations and the decide/adapter split add lines that emit no JavaScript, so a faithful typed port of a short block (S11, 25 lines; L25, 11 lines) raises the total.

**Concrete change.** Count `js_file` on the canonical emitted form that 5.3 item 4 already computes (`bun build --minify-syntax --no-bundle`, types stripped), and fall back to the source count when `bun` is absent, labelled as such.

### 8. DETAIL, minor: the lifetime backstop does not cover window listeners

**Claim attacked.** 2.2 "Lifetime is the owner's ... a `WeakMap` or on the element".

**Evidence.**
- (I) A WeakMap releases memory, not registrations. A behaviour's `window`/`document` listener closure keeps its element alive and stays registered if the teardown message never arrives (#4064, which the plan cites for teardown).
- (V) That is the leak class `js_listener.rs:1-20` was built against.

**Concrete change.** Each behaviour checks `owner.isConnected` at install and on every delivered event, and tears itself down when the owner is disconnected. The 200-mount leak spec adds an "owner removed without teardown" case.

## Attacks that failed

- **A Rust/wasm page agent or web-sys** (my section 1.8 and the plan's C2): both designs reject them for the same reasons (a second artifact or an unreviewable blob, async load, glue still needed). PR #5194 is kept behind the transport seam. This holds.
- **Custom elements** as the behaviour host: round 1 already found that there are no custom events in dioxus-html and that the wrappers break `>` selectors. Finding 1 takes their one real win (config ordered with the edit stream) without the wrapper.
- **The root rule:** re-derived independently from `native.ts:379,569-590` and `webview.rs:101,194,537-590`. Not broken.
- **My rung 3 ("everything else is a page behaviour") against the plan's R/B split:** the plan is better. Async-tolerant (b) logic belongs in Rust over verbs, which shrinks the page-side registry.
- **G (Rust placement under a CSS gate) on the fallback engines, instead of a page-side placement behaviour:** A covers the evergreen engines (Chrome 125, Firefox 147, Safari 26). The fallback costs one round trip and one frame per open (1.4 item 2), and tracking during scroll matters little, because T12 closes most overlays on scroll. An acceptable trade under "Rust first".
- **Per-op eval versus one persistent eval channel:** both reach the webview through `evaluate_script`, so a persistent channel buys no ordering against the edits WebSocket. Only the per-op `new Function` compile is saved (`dioxus-web document.rs:220`, `desktop query.rs:79-80`). That is lane 3.0's call, not a design point.
- **Behaviour config travelling in the install message rather than the DOM** (it lands after the edits on desktop): the window is 1-2 frames at install or release, and for the registered behaviours (`scrollLock`, `hotkey`, `otpFilter`) the consequence is harmless (a frame of page scroll, or a missed key in the first frames after mount).
- **Fail-open under a busy Rust** (over 100 ms, so an unplaced paint): a stated degradation, and preferable to an invisible panel (WCAG). Not a design flaw.
- **Hot rsx handlers on desktop, and the `onresize` per row:** already in the plan's root (1.4 item 6). Only the missing lane ownership is a finding (3).
