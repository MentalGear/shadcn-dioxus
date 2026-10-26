# Dioxus best-practice review of the row 434 step 3 plan (2026-10-26)

**Status.** This review reads the plan as it stood before boards 2 to 4; its findings were folded into the plan, which is now [`sync-js-root-2026-10-26.md`](./sync-js-root-2026-10-26.md) as amended by [round 4](./board-2026-10-26-r4/SYNTHESIS.md). Where a section here names an option as A, B or C (now scope 1, 2 and 3), lane R (now lane VR) or "step 0" (now the latency session), read the plan's terms block. The review is kept as written.

Independent review of `sync-js-root-2026-10-26.md` and the "Step 3" sections of backlog row 434, against official Dioxus guidance, dioxus-sdk, upstream DioxusLabs/components, the DioxusLabs/dioxus issue tracker, and comparable systems. Read-only: nothing was built or run, and the repo was only read (`/home/user/wt/b28`, branch `queue/batch-28`).

Evidence labels: **[D]** official docs page, **[S]** source file or crate page, **[G]** GitHub issue or PR page, **[X]** other project's docs, **[I]** my inference, not verified by running anything. Most GitHub and docs.rs facts came through WebFetch, which summarises pages with a small model, so field-level claims are only as good as that summary. Where a fetch failed, I say so in section 8.

---

## Executive summary (for the owner)

The direction is right, but the plan is not yet the peak construction in Dioxus terms. Using `document::eval` and `dioxus.send` as the one portable channel is exactly what the Dioxus docs prescribe, because web-sys "is currently not portable" to desktop and mobile [D1]. Upstream dioxus-primitives does the same thing: eval strings with `dioxus.send`, plus one TypeScript file compiled to a committed JS file and loaded with `asset!` and `document::Script` [S5-S8]. dioxus-sdk's own scroll hook is an eval listener [S3]. So the bridge, the no-web-sys rule and the committed-bundle chain are all idiomatic, and no upcoming 0.8 API makes them redundant soon. Rules-as-data is not a Dioxus idiom, but it is well precedented elsewhere (Phoenix LiveView JS commands, Elm decoders, React Native Gesture Handler's native-thread recognisers), and its closed-vocabulary guard is the right one.

What keeps it short of "peak" is four gaps that Dioxus's own tracker exposes. First, a paint-safe gate (G) that waits on a Rust answer can stay invisible forever, because `get_client_rect().await` is reported never to complete on Ubuntu desktop (#5734) and `eval` can run before a `document::Script` from the same render (#5919): G needs a fail-open watchdog. Second, the bridge's own loading must not depend on Script-versus-eval ordering, and the asset minifier has been reported to strip functions that nothing references (#5183). Third, the transport should sit behind one seam, because the only conflicting upstream item, PR #5194 (wasm-bindgen on desktop, milestone 0.8.0, open but stale and conflicted), would let web-sys work on desktop, and because Blitz makes `eval` a no-op. Fourth, the hooks should carry the names the Rust ecosystem already uses (`use_event_listener`, `use_media_query`, `use_resize_observer`, `use_window_size`) and should reuse `onresize`, `onvisible` and `onscrollend` instead of bridging them again.

Do not depend on dioxus-sdk: its window-size hook is web-sys on web and physical pixels on desktop, it has no media-query, resize or intersection hook, and the repo already dropped `dioxus-sdk-time` for the same reasons. Mirror its names and its SSR-safe initial-value pattern instead. Verdict: adopt, with the four **must** changes in section 7.

---

## 1. Official Dioxus guidance on JS interop

### 1.1 What the docs say

- **`document::eval`** runs JS on web, desktop, mobile and liveview, and returns an `Eval` for two-way messages: JS calls `dioxus.send(v)`, Rust calls `eval.recv::<T>().await`, Rust calls `eval.send(v)`, JS calls `await dioxus.recv()`. Values may be formatted into the script string. Results are awaited. To touch mounted DOM, run it inside `use_effect` or an event handler, never in the component body. The page warns to run only trusted code, with the risk highest on web (XSS) [D2: docs.rs `dioxus-document` 0.7.10 `docs/eval.md`]. The page lists no per-renderer behavioural differences [D2].
- **The "escape hatches" page** names three routes: `eval`, `web-sys` ("currently not portable" to desktop and mobile; for those it suggests Dioxus with Tauri, "can be slower and harder to setup"), and `getElementById` with a generated id. It does not mention wasm-bindgen. It also says elements whose name has a dash render as-is, and recommends wrapping a web component "in a type safe component" [D1].
- **Desktop guide:** `document::eval` is "the" way to run raw JS in the webview, `dioxus.send`/`dioxus.recv` are the two directions, receiving is async, and the app uses the system WebView. It also states the long-term plan to move to a custom renderer ("Blitz") [D3].
- **Assets:** `asset!` takes a path relative to the package root, is `const`, is processed by the `dx` CLI, hashes file names by default, and minifies CSS. The page says nothing about JS or TS processing, and nothing about `document::Script` [D4]. `ScriptProps` documents `src`, `defer`, `type`, `nonce`, `integrity`, and says "Scripts are deduplicated by their src attribute"; it says nothing about load order against `eval` [D5].

### 1.2 Reading against the plan

| Plan element | Official guidance | Verdict |
|---|---|---|
| One generic bridge over `eval`/`dioxus.send`, no web-sys | The only portable route [D1, D3]. | Idiomatic. |
| Calls inside `use_effect` or handlers | Required [D2]. | The plan's hooks must keep it (they do: `use_js_listeners` already does). |
| TS compiled to a committed JS file and loaded as an asset | Same as upstream [S5, S6, S8]. | Idiomatic. Stricter than upstream (manifest hash gate). |
| Bridge loaded by `eval` strings | Docs say nothing; upstream uses `document::Script` for the library and `eval` only to call it [S5]. | Plan is silent on loading: must be specified (section 3.2). |
| Wrap behaviour in typed Rust components | Docs recommend it for web components [D1]. | The typed hooks and the `Rule` enum are this. |

---

## 2. Platform facts that bear on the plan (issues and PRs)

| Item | Finding | Effect on the plan |
|---|---|---|
| #5734 `get_client_rect().await` never completes on Ubuntu/Desktop | Open, filed 2026-08-04 on Dioxus 0.7.1, no maintainer reply; the body also contains unverified AI suggestions [G7]. | A G gate that waits for a Rust read can stay at `opacity: 0` forever on Linux desktop. Needs a fail-open watchdog. |
| #5919 `document::eval` can run before `document::Script` from the same render (desktop) | Open, 2026-10-14, WebKitGTK, 0.7.10. Head insertion is a queued effect; `eval` runs immediately; dynamic scripts load async [G8]. | Loading the bridge as a Script and calling it from the next eval is racy on desktop. Same class applies in principle on web, since web also inserts head elements from `queue_effect` [S9]. [I] |
| #5183 bundled JS "erased" when only called from `eval` | Open bug, 0.7.2: a function never referenced at top level came out with an empty body after bundling; adding a top-level call fixed it [G9]. Not confirmed by a maintainer. | Bridge exports must be assigned to `window` (upstream does: `window.createFocusTrap`) and the bundle checked in the bundled output, not just in `src/js/`. |
| #4590 long-running eval ends unexpectedly on web | Open bug; a `setInterval` keeps it alive [G10]. | Any long-lived per-scope eval loop in the bridge needs a test and a keep-alive. |
| #4064 `document::eval` inside `use_drop` fails an assertion with another eval | Open bug; `spawn_forever` avoids it [G10]. | Teardown must send on the existing channel (as `JsListeners` already does with its `TEARDOWN` message), never start a new eval from `use_drop`. |
| #4655 `Eval` is `Copy` (merged 2025-09-12) | [G10] | Hooks can hold the handle freely. |
| #5313 "Avoid eval on web" (merged 2026-02-16) | Motivated by strict CSP in browser extensions [G11]. Web `eval` still compiles the string with `new Function` and wraps it in an async function [S9]. | An app under a CSP without `unsafe-eval` cannot use any eval-driven library, upstream primitives included. Document it; consider a direct web path later. |
| #2405 window-level event listeners | Open since 2024-05-10, 10 comments, proposes a `dioxus:window` element or `use_window_event`; no maintainer plan seen [G3]. | `listen(window|document, ...)` will not become redundant soon. |
| #1011 capturing event handlers | Open since 2023-05-09 [G4]. | Same: capture and passive options stay a bridge job. |
| #1462 / #1517 event metadata (`onvisible(60)`) | Open and a closed draft [G12]. | `onvisible` keeps default options; IntersectionObserver with `rootMargin`/`threshold` stays a bridge job. |
| `onresize` (#2479, merged 2024-08-15) and `onvisible` (#2911, merged 2024-12-02) | Both shipped [G12]. | Do not rebuild plain element resize/visibility in the bridge. |
| #5258 `ResizeEvent` bubbles on desktop, not on web | Closed; `stop_propagation()` in the child works [G13]. | If `onresize` is used, stop propagation in the child (desktop/web difference). |
| #5194 "Get wasm bindgen working on desktop" | Open, milestone 0.8.0 (description "tbd"), labels enhancement/breaking/desktop, last updated 2026-06-15, `mergeable_state` dirty. Forks wasm-bindgen/web-sys/js-sys as `*-x` crates, moves the vdom to its own thread, uses synchronous cross-thread calls, so `web_sys::Element` mounted data, events and eval could be shared by web and desktop. Maintainer jkelleyrtp asked whether a fork is needed at all [G1, G2]. | The one item that could make the bridge's reason for existing partly obsolete. Not merged, contested, so do not wait for it, but keep the transport swappable. |
| #5117 optional cleanup for `onmounted`, #5113 `onunmounted` | #5117 open, #5113 closed without merging; both web only [G6]. | There is no official per-element unmount hook. Element-bound rules must clean up by dying with the element, not by an unmount callback. |
| #5941 Blitz double borrow on `set_focus` from a spawned task | Closed 2026-10-16 [G6]. | Evidence that MountedData on Blitz is new and fragile; unrelated to the bridge but relevant to the "R" lanes. |
| #4422 "Prevent desktop reloads at the wry level" (merged 2025-07-15) and #6006 (macOS Finder drops, `dragenter` not cancelled in `native.ts`, open 2026-10-25) | The desktop interpreter hard-codes some conditional default-action rules itself [G5]. | Dioxus already ships a small closed rule set in its interpreter (inferred from these two); rules-as-data is the generalisation of what it does. [I] |

**Searches that found nothing:** no open or closed issue asking to change synchronous `prevent_default` on desktop (searches on `prevent_default` + desktop/wry returned #4045, #1477, #157, #4422 only); no planned typed JS bridge or `#[wasm_bindgen]`-like interop other than #5194; no `use_resize`/`use_media_query` proposal. So nothing found contradicts the claim that the 133 existing Rust `prevent_default()` calls keep working. [I, absence of evidence]

**Release state:** 0.7.10 (2026-07-30); 0.8.0-alpha.0 (2026-05-19) to alpha.3 (2026-10-16); 0.8 stable not out. alpha.0 notes mention hotpatch default, desktop non-bubbling event forwarding (#5487), dioxus-native upgrades including custom elements, and nothing on JS interop [G14]. The audit already recorded `onbeforeinput` landing in alpha.1.

---

## 3. Divergences, judged one by one

### 3.1 Eval-driven bridge, no web-sys — justified
Official guidance (section 1) and upstream both do this. The cost is real and the plan already names it: desktop and mobile are an async IPC hop [D3]. No upcoming API removes it before 0.8 stable, and #5194 is stale and disputed.
**Change:** put the transport behind one Rust module (a small trait or a single `bridge::send/recv` pair). The same seam is where `__dxBridgeDelayFrames` hooks in, where a #5194 `web-sys-x` backend could replace eval later, and where a no-op backend answers "unsupported" on Blitz, whose `document::eval` returns `EvalError::Unsupported` (repo `blitz-feasibility-2026-10-25.md`, source D1 there). [I]

### 3.2 How the bridge itself is loaded — unspecified in the plan, must be decided
Three options:
- **(a) Script then eval** (upstream's shape [S5]): idiomatic but racy on desktop (#5919) and exposed to minifier stripping (#5183).
- **(b) Inline bundle in each eval** (what `use_js_listeners` does today with its `PRELUDE`): race-free, no asset pipeline dependency, but re-sends the code on every eval.
- **(c) Recommended [I]:** (a) plus a ready handshake. A root provider renders `document::Script { src: asset!(..) , defer: true }` once; the first eval of every hook awaits `window.__dxBridge?.ready` with a bounded wait (poll or a `load` listener, then fail loudly), so a late script costs a delay, not a missing function. Exports are assigned to `window` and the check-js-bundle gate also asserts the exported names exist in the bundled output.
Multi-window desktop gives each webview its own `window`, so readiness is per webview, not a Rust-side boolean. [I]

### 3.3 Own hooks instead of dioxus-sdk — justified (section 4), but names should match the ecosystem
The plan's `use_window_listener`, `el.show_popover()`, `use_media_query` are mixed. The common vocabulary in Rust frontends (leptos-use, itself a VueUse port; dioxus-sdk) is `use_event_listener`, `use_media_query`, `use_resize_observer`, `use_intersection_observer`, `use_mutation_observer`, `use_window_size`, `use_element_size`, `use_scroll` [X1, S1, S3]. Renaming costs nothing before the first release and saves every user a lookup.

### 3.4 Rules as data — justified, novel to Dioxus, strongly precedented elsewhere (section 6)
Specific idiom points:
- Prefer **rule as an rsx attribute** (`"data-dx-rule": rule.to_attr()`) read by **one delegated capture-phase listener** that finds `closest('[data-dx-rule]')`. That is how Dioxus itself delegates events (one root listener, per the plan's own 0.7.9 source reading of `core.ts`), it is SSR/hydration friendly, needs no install or uninstall, survives re-mounts, and sidesteps the missing unmount hook (#5117/#5113). `rules.install(el, rule)` should be the secondary form. Capture phase also catches non-bubbling events (focus, scroll). [I]
- On `window`, `document` and `body`, `wheel`, `mousewheel`, `touchstart` and `touchmove` listeners are passive by default; a B1 rule that cancels one must set `passive: false` explicitly [X2: MDN `addEventListener`]. The repo's `JsListeners` already enforces this; the bridge's `listen` must keep that default and its `blocking-ok` reasoned marker, not drop them.

### 3.5 G paint-safe gate — justified in concept, unsafe as specified
Floating UI exposes `isPositioned` for the same reason: placement completes asynchronously after open, and the page may need to know when [X3]. (That is a flag's existence; "hide until positioned" is the usual use, my inference.) But Dioxus's own tracker shows the answer can fail to arrive (#5734) or arrive after an unrelated script load (#5919). A permanent `opacity: 0; pointer-events: none` is a worse failure than a one-frame misplacement.
**Change:** every G gate gets a CSS-only fail-open (for example the pending state is a keyframe animation that reaches `opacity: 1` after a bounded delay such as 250-400 ms, cancelled by `done`), and the slow-channel harness gets an `N = never` case that must still end visible.

### 3.6 Reimplementing `observe(resize|intersection)` — partly redundant
`onresize` and `onvisible` exist in 0.7 [G12]; lane 2D is already testing them. Keep `observe` for MutationObserver and for IntersectionObserver with options, and for ResizeObserver only where the target is not a vdom element or the box option differs. Note the desktop bubbling difference (#5258).

### 3.7 `transaction` and `commit` verbs — justified, and the ordering claim has outside support
The repo's own comments say separate evals have no guaranteed order on desktop; #5919 is independent evidence of eval ordering races on WebKitGTK. Make the rule explicit: **all Rust-to-JS bridge traffic for a webview goes over one ordered channel** (one long-lived eval per bridge instance receiving `dioxus.recv()` commands), because ordering inside one channel is the only ordering Dioxus provides. [I] Counterweight: #4590 (long eval loops dying on web), so that pump needs the keep-alive test.

### 3.8 Residue stays TS — no Dioxus-side objection
Upstream has no carousel, scroll lock, message scroller or anchor positioning at all (section 5), so there is no upstream idiom to align with for those blocks.

---

## 4. dioxus-sdk: use it, mirror it, or ignore it

Facts: workspace on Dioxus 0.7.0; crates `storage`, `geolocation`, `notifications`, `window`, `time`, `sync`, `util`; last commit 2026-07-02; `dioxus-sdk-window` has two releases, 0.7.0 (2025-11-06) and 0.8.0-alpha.3 (2026-10-16), and the docs.rs build for the alpha failed [S1, S2, S4].

| Need | What dioxus-sdk has | How |
|---|---|---|
| Window size | `use_window_size() -> ReadSignal<Result<WindowSize, WindowSizeError>>`, `get_window_size()` | web: **web-sys** `innerWidth`/`innerHeight` plus `set_onresize` (replaces any other `onresize` handler on `window`); desktop: **tao** `inner_size()` (physical pixels) and a wry `Resized` handler [S2]. No eval. |
| Root scroll | `use_root_scroll() -> Signal<ScrollMetrics>` | **eval**: a `window.scrollCallback_N` global per instance, listeners on `scroll` and `resize`, `dioxus.send` per event, removal in `use_drop` via a second eval [S3]. Web-oriented. |
| Theme | `theme` module | Web, Windows, Mac per the README [S1]. |
| Media query, ResizeObserver, IntersectionObserver, element size, MutationObserver | **None** [S1] | |

Judgement:
1. **Do not depend on it.** Its window-size hook mixes CSS pixels (web) and physical pixels (desktop), uses web-sys on web (the thing the plan rules out) and `window.onresize =` (a clobbering assignment, not `addEventListener`). Its scroll hook is a strict subset of `listen`, with the leak shape the repo's `JsListeners` was written to prevent. It has no hook for the cases the plan needs. The repo already removed `dioxus-sdk-time` for a local `sleep.rs` (`primitives/Cargo.toml:24`), and each added dependency is also a supply-chain row here.
2. **Mirror the good parts:** the names (section 3.3); the SSR-safe pattern (initial value is a deterministic `Err(Unsupported)`/default, the real value is written in `use_effect`, so hydration never disagrees; the sdk's docs warn that an eager `get_window_size()` outside an effect can break hydration [S2]); and returning `ReadSignal<T>`.
3. **Fix what it gets wrong:** report CSS pixels on every renderer (from `window.innerWidth` through the bridge), use `addEventListener` with derived removal.

---

## 5. Upstream DioxusLabs/components (what this fork started from)

Repository `DioxusLabs/components` (renamed `dioxus-components` in redirects), last commit read 2026-09-08 [S5].

| Area | Upstream today | Source |
|---|---|---|
| JS files | `primitives/src/ts/focus-trap.ts` (2.6 KB) built to `src/js/focus-trap.js` (1.4 KB) plus `hash.txt`; `build.rs` watches `src/ts` and maps the one file. | [S6, S7] |
| Loading | `pub(crate) const FOCUS_TRAP_JS: Asset = asset!("/src/js/focus-trap.js")`; `document::Script { src: FOCUS_TRAP_JS, defer: true }`; the TS puts `window.createFocusTrap` on `window`; eval then calls it. `asset!` is used, `include_str!` and web-sys are not. | [S5, S6, S8] |
| Popover/menu placement | Plain CSS: `position: relative` wrapper plus per-side absolute offsets from `data-side`/`data-align`; no anchor positioning, no flip, no `@position-try`. | [S10] |
| Escape and outside-click | eval strings: `document.addEventListener('keydown', ..)` with `preventDefault()` and `dioxus.send`; `document.addEventListener('pointerdown', f, true)` with `dioxus.send(true)`. | [S5] |
| Exit animations | eval waiting on `element.getAnimations()` `.finished`. | [S5] |
| Pointer tracking | eval with `window` listeners forwarding `[pointerId, clientX, clientY]` via `dioxus.send`; no pointer capture. | [S11] |
| Scroll area | CSS overflow only; no eval, no web-sys. | [S12] |
| Dialog | A plain `div role="dialog"`, not `<dialog>`/`showModal`; no scroll lock; focus trap via eval in `use_effect`. | [S13] |
| Not present | Carousel, drawer, command, scroll lock, anchor positioning, message scroller. | [S14] |

Reading: the plan's chain (TS source, committed JS, hash gate, `asset!`, `Script`, eval to call) is upstream's chain with more gates. Upstream's listeners are the unmanaged `addEventListener`-in-eval shape the repo's `JsListeners` fixed, and its focus-trap `remove()` does not remove its keydown listener (read from the TS, [S6]). There is nothing upstream to adopt for the 22 hard blocks.

---

## 6. Comparable systems for "declarative rules evaluated locally, logic on the other side"

| System | What it does | Bearing on rules-as-data |
|---|---|---|
| **Phoenix LiveView `JS` commands** | The server builds client-side commands (`add_class`, `toggle_attribute`, `show`, `hide`, `dispatch`, `push`) as encodable data; they run without a round trip and, being "DOM-patch aware", "stick to elements across patches from the server" [X4]. | Closest analogue. Validates data-not-code, local execution, and surviving server re-renders. Its lesson: attribute-borne commands survive re-render; imperative changes do not. Supports preferring `data-dx-rule` over `install`. |
| **Elm event decoders** | A handler is built from an event name and a JSON decoder; "Whatever values you successfully decode are then sent on to your `update` function", a failing decoder is ignored, which "keep[s] the conditional logic out of `update`" [X5]. (`preventDefaultOn`'s boolean being decided by the decoder is from my knowledge of the Elm API; the fetch of the Elm docs page failed and the article read does not discuss `preventDefault`.) [I] | The decision runs at event time from event data only, and the program logic never sees non-matching events. Same shape as B1. |
| **React Native Gesture Handler** | "Declarative API exposing platform native touch and gesture system"; gestures "are no longer controlled by the JS responder system, but instead are recognized and tracked in the UI thread" [X6]. | Same split as B2: a declarative config on the thread where the decision must happen, callbacks to the logic thread afterwards. |
| **Floating UI** | `isPositioned` flag after async placement [X3]. | Precedent for G. |
| **Tauri v2 IPC** | "Asynchronous Message Passing", JSON-RPC-like commands, `invoke` compared to `fetch`; no synchronous call offered [X7]. | Confirms the premise that a webview-to-native bridge is async and that synchronous decisions must be taken inside the webview. |
| **Yew `gloo-events`** | Options struct, defaults to `passive: true`, `enable_prevent_default()` is the opt-out [X8]. | Same default the repo's `JsListeners` uses; supports keeping it in `listen`. |
| **Leptos / leptos-use** | web-sys wrappers: `use_event_listener`, `use_media_query`, `use_resize_observer`, `use_intersection_observer`, `use_window_size`, `use_element_size` [X1]. | Naming precedent; also shows a pure web-sys design cannot serve desktop. |

What none of them validates: an open-ended predicate language. LiveView keeps a full-code escape (`phx-hook`), RNGH keeps worklets and JS callbacks, Elm keeps arbitrary decoders. The plan's "residue as named TS modules with exit conditions" is the same escape hatch, and the "no variables, loops or arithmetic" rule is the guard that these systems tend to lose over time. Keep it.

---

## 7. Recommendations

Rating key: **must** = change before lane 3.0 is built, **should** = change in lane 3.0 or the first lanes after it, **could** = optional or later.

| # | Rating | Recommendation | Evidence |
|---|---|---|---|
| 1 | **must** | G gates fail open. Every pending state (`data-dx-place="pending"`, pinned height) has a CSS-only timeout back to visible; add an `N = never` case to the slow-channel harness. | #5734, #5919, async IPC [G7, G8, D3] |
| 2 | **must** | Specify how the bridge loads. Recommended: `document::Script { src: asset!(..), defer: true }` from one root provider, plus a bounded ready handshake before any verb; assign exports to `window`; make `check-js-bundle.sh` (or a sibling) assert the exported names are present in the built output. Per webview, not per process. | #5919, #5183, upstream shape [G8, G9, S5] |
| 3 | **must** | One transport seam: hooks never call `document::eval` directly (the plan's gate already says this); one module owns send/recv, versioned envelope, the `__dxBridgeDelayFrames` hook and the "unsupported" answer for Blitz. | #5194, Blitz no-op eval, #4064 [G1, G2, G10] |
| 4 | **must** | Teardown never starts a new eval. Send the teardown on the existing channel (as `JsListeners` does); window/document listener removal stays derived from the owning scope; element-bound rules die with the element. | #4064, no unmount hook #5117/#5113 [G10, G6] |
| 5 | should | Name hooks for the ecosystem: `use_event_listener`, `use_media_query`, `use_resize_observer`, `use_intersection_observer`, `use_mutation_observer`, `use_window_size`, `use_element_size`; return `ReadSignal<T>`; deterministic initial value, real value in `use_effect` (hydration/SSG safe). | leptos-use, dioxus-sdk [X1, S2] |
| 6 | should | Do not rebuild what Dioxus ships. Use `onresize`, `onvisible` (default options), `onscrollend`, `MountedData`, `onbeforeinput` (0.8) where they suffice; the bridge's `observe` covers MutationObserver and IntersectionObserver with options. Add `stop_propagation()` where `onresize` is used (desktop bubbling). | #2479, #2911, #5258 [G12, G13] |
| 7 | should | Make `data-dx-rule` attributes plus one delegated capture-phase interpreter the primary form of rules; `rules.install` secondary. | LiveView JS, Dioxus root delegation, no unmount hook [X4, G6] |
| 8 | should | Add shared conformance vectors: one JSON file of (rule, event fields, state, expected effect) run by both `cargo test` and `bun test`. The plan's gate compares variant names only, not behaviour. | [I] |
| 9 | should | Keep the passive defaults in `listen` (`wheel`/`touchstart`/`touchmove` on window, document and body are passive by default; B1 needs explicit `passive: false` and a reason marker). | MDN [X2] |
| 10 | should | One ordered Rust-to-JS channel per webview for bridge commands (this is what makes `transaction`/`commit` meaningful on desktop); test that a long-lived eval survives (#4590). | #5919, #4590 [G8, G10] |
| 11 | should | Do not add a dioxus-sdk dependency; mirror names and the SSR-safe pattern; report CSS pixels on every renderer. | section 4 |
| 12 | should | Document the CSP consequence: web `eval` compiles with `new Function`, so strict-CSP apps need `unsafe-eval` (upstream has the same limit). | [S9, G11] |
| 13 | could | Write `Rule::decide(fields, state) -> Effect` in Rust as well (not shipped as a runtime path at first): it is the unit-test oracle for the TS interpreter (item 8) and, because rules are renderer-neutral data, the path a future Blitz or #5194 desktop backend could use synchronously. | [I] |
| 14 | could | Re-check #5194 and the 0.8.0 milestone at each Dioxus release; if it merges, the seam in item 3 is where a `web-sys-x` backend replaces eval for desktop. Do not wait for it. | [G1, G2] |
| 15 | could | Once Dioxus 0.8 stable ships, verify `onbeforeinput` data fields and `prevent_default` on desktop, then move S11's cancel half to a Rust handler. | audit section 3, alpha.1 |
| 16 | could | Consider custom elements (a dashed tag renders as-is [D1]) as the host for B-rule behaviour: `connectedCallback`/`disconnectedCallback` give an element-bound lifecycle without an unmount hook, and dioxus-native alpha.0 lists custom-element support [G14]. Probably unnecessary if item 7 is adopted. | [I] |

### Where the plan should not change
- No web-sys: keep.
- `asset!` plus committed bundle: keep (matches upstream).
- Closed `Rule` enum, backlog row per new variant, no arithmetic: keep.
- Residue modules with exit conditions: keep.
- Upgrading Dioxus for this: not needed; nothing in 0.8 alpha adds a window or element API [audit section 3, G14].

---

## 8. Limits of this review

- Everything about desktop and mobile latency is still inferred, as in the plan; nothing was run.
- GitHub MCP access to `DioxusLabs/*` was refused (repo not granted), so GitHub facts came from `api.github.com` and `raw.githubusercontent.com` pages through WebFetch. Issue bodies and comments were summarised by WebFetch's model, so maintainer replies may be missing: #5919's comments, #5183's comments and #5734's comments were not retrievable. The sdk's `theme` module source and `use_media_query` absence were checked via the module list (`size`, `theme` in window; `scroll` in util) and the README, not by reading every file.
- Fetches that failed or returned nothing usable: the Elm `Html.Events` docs, the RNGH pan-gesture docs page, `docs.rs` `MountedData` page, the Dioxus 0.7 events guide, `leptos-use.rs/elements/use_event_listener`. Claims that rest on those are marked [I] above.
- `dioxus-web` source facts (`new Function`, `PROMISE_WRAPPER`, head elements via `queue_effect`/DOM calls, only `set_title` through eval) came from the `main` branch file, which the fetch summary could not date; the plan's own 0.7.9 citations agree with it on the eval path.
- Upstream `DioxusLabs/components` was read at its default branch on 2026-10-26; its last commit was 2026-09-08.

---

## Sources

Official docs [D]:
- D1 https://dioxuslabs.com/learn/0.7/essentials/ui/escape
- D2 https://docs.rs/crate/dioxus-document/0.7.10/source/docs/eval.md
- D3 https://dioxuslabs.com/learn/0.7/guides/desktop
- D4 https://dioxuslabs.com/learn/0.7/essentials/ui/assets
- D5 https://docs.rs/dioxus-document/latest/dioxus_document/struct.ScriptProps.html
- D6 https://docs.rs/manganis/latest/manganis/struct.JsAssetOptions.html (options list only; says nothing on JS processing)
- D7 https://dioxuslabs.com/blog/release-070 (no JS interop content)

Source and crate pages [S]:
- S1 https://github.com/DioxusLabs/sdk and https://raw.githubusercontent.com/DioxusLabs/sdk/master/Cargo.toml
- S2 https://raw.githubusercontent.com/DioxusLabs/sdk/master/packages/window/src/size.rs
- S3 https://raw.githubusercontent.com/DioxusLabs/sdk/master/packages/util/src/scroll.rs
- S4 https://crates.io/api/v1/crates/dioxus-sdk-window ; https://api.github.com/repos/DioxusLabs/sdk/commits ; https://docs.rs/dioxus-sdk-window/0.8.0-alpha.3/dioxus_sdk_window/
- S5 https://raw.githubusercontent.com/DioxusLabs/components/main/primitives/src/lib.rs ; https://api.github.com/repos/DioxusLabs/components/commits
- S6 https://raw.githubusercontent.com/DioxusLabs/components/main/primitives/src/ts/focus-trap.ts
- S7 https://api.github.com/repos/DioxusLabs/components/contents/primitives/src/ts and .../js ; https://raw.githubusercontent.com/DioxusLabs/components/main/primitives/build.rs
- S8 https://raw.githubusercontent.com/DioxusLabs/components/main/primitives/src/popover.rs
- S9 https://raw.githubusercontent.com/DioxusLabs/dioxus/main/packages/web/src/document.rs ; https://raw.githubusercontent.com/DioxusLabs/dioxus/main/packages/desktop/src/document.rs
- S10 https://raw.githubusercontent.com/DioxusLabs/components/main/preview/src/components/popover/style.css
- S11 https://raw.githubusercontent.com/DioxusLabs/components/main/primitives/src/pointer.rs
- S12 https://raw.githubusercontent.com/DioxusLabs/components/main/primitives/src/scroll_area.rs
- S13 https://raw.githubusercontent.com/DioxusLabs/components/main/primitives/src/dialog.rs
- S14 https://api.github.com/repos/DioxusLabs/components/contents/preview/src/components ; .../primitives/src

GitHub issues and PRs, DioxusLabs/dioxus [G]:
- G1 https://github.com/DioxusLabs/dioxus/pull/5194 (and the api.github.com PR record)
- G2 same, review comments (jkelleyrtp)
- G3 https://github.com/DioxusLabs/dioxus/issues/2405
- G4 https://github.com/DioxusLabs/dioxus/issues/1011
- G5 https://github.com/DioxusLabs/dioxus/pull/4422 ; https://github.com/DioxusLabs/dioxus/issues/6006
- G6 https://github.com/DioxusLabs/dioxus/pull/5117 ; .../pull/5113 ; .../issues/5941 ; .../issues/5574
- G7 https://github.com/DioxusLabs/dioxus/issues/5734
- G8 https://github.com/DioxusLabs/dioxus/issues/5919
- G9 https://github.com/DioxusLabs/dioxus/issues/5183
- G10 https://github.com/DioxusLabs/dioxus/issues/4590 ; .../issues/4064 ; .../pull/4655 ; .../issues/3915 ; .../issues/4454 (from the eval title search)
- G11 https://github.com/DioxusLabs/dioxus/pull/5313
- G12 https://github.com/DioxusLabs/dioxus/pull/2479 ; .../pull/2911 ; .../issues/1462 ; .../pull/1517 ; .../issues/518
- G13 https://github.com/DioxusLabs/dioxus/issues/5258
- G14 https://github.com/DioxusLabs/dioxus/releases/tag/v0.8.0-alpha.0

Other projects [X]:
- X1 https://leptos-use.rs/functions.html
- X2 https://developer.mozilla.org/en-US/docs/Web/API/EventTarget/addEventListener
- X3 https://floating-ui.com/docs/useFloating
- X4 https://phoenix-live-view.hexdocs.pm/Phoenix.LiveView.JS.html
- X5 https://thoughtbot.com/blog/advanced-dom-event-handlers-in-elm
- X6 https://github.com/software-mansion/react-native-gesture-handler
- X7 https://v2.tauri.app/concept/inter-process-communication/
- X8 https://docs.rs/gloo-events/latest/gloo_events/struct.EventListenerOptions.html

Repo files read (read-only): `dev-docs/backlog/434-*.md`, `/mnt/project-files/research/{sync-js-root,js-audit,blitz-feasibility}-2026-10-2*.md`, `primitives/src/{lib.rs,js_listener.rs}`, `primitives/src/js/bundle.manifest`, `primitives/Cargo.toml`.
