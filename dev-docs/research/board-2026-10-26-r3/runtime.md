# Board round 3, lens `runtime`: the amended plan against each renderer (2026-10-26)

**Verdict: HOLDS WITH CHANGES**

**DESIGN findings: 0, DETAIL findings: 13**

Summary:
- From the runtime lens the main design is solid. I tried hard to break the root rule, the deadline table, the two kinds, admission and the closed bridge on web, desktop, mobile and SSG hydration, and every break repaired inside the existing construct (a protocol field, a mechanism, a condition on one row, a mapping).
- 5 of the 13 are major. They do not change the construct, but they change claims the plan makes: "an overlay open saves a frame" (finding 1), "reconcile converges" (2), the `stickToEnd` mapping (3), the G tier predicate (4) and the first row of the table on Android (5), plus a fail-open that is optional in the built bridge but mandatory in the plan (8).
- Closest to DESIGN, and why they are not: finding 1 (the per-element observer cannot see an insertion; but level-at-attach is correct either way, and the fix is one document-level observer inside `reconcile`) and finding 3 (a page-originated transition that Rust mirrors needs the page to apply it first; that is the root rule applied to an input the W8 mapping missed, and it is one block plus a generation field).
- Sources: dioxus-web, dioxus-desktop, dioxus-interpreter-js, dioxus-document and dioxus-core 0.7.9, read in `~/.cargo/registry/src/*`. Bridge: `/home/user/wt/l30` is branch `lane/js-bridge-3-0`, HEAD `ae2650e8` **plus 14 uncommitted modified files** (the ordered pump and typed `Program` transport are in the uncommitted `transport.rs`, 523 lines against 200 committed). No `origin/lane/js-bridge-3-0` ref exists in that repo; I reviewed the worktree as it stands. Nothing was built or run. Labels: V verified in source, W web spec, I inference.

## What each row of the deadline table does on each renderer

| row | web (wasm) | desktop (WebView2, WKWebView, WebKitGTK) | mobile | SSG hydration |
|---|---|---|---|---|
| default action, rsx `prevent_default` | holds (V `dioxus-web/src/dom.rs:88-116`, a handler that awaits before `prevent_default` loses it on both: `dioxus-core/src/events.rs:339-347`) | holds (V sync XHR `native.ts handleVirtualdomEventSync`, answer `SynchronousEventResponse` `webview.rs:630-641`); runs against a vdom that can be ahead of the DOM (finding 6) | **Android: fails silently for payload events** (finding 5); iOS I | no handler exists before hydration; native defaults run (submit, links) |
| default action, behaviour | holds (page-local) | holds, but installs arrive after render + ack + eval hop, at least one frame after the Rust state that needs it | same; iOS cannot raise the keyboard from non-gesture focus, which the `openFocus` touch policy already avoids | none until hydration plus the first effect (finding 11) |
| this event's task (capture, focus) | holds | holds (page-local; the only correct place, `set_focus` is a webview query, `element.rs:137-160`) | same | same |
| before paint, A | holds where the engine passes | WebView2 and Android WebView hold; **WKWebView and WebKitGTK cannot be classified by floor** (finding 4) | iOS 26.2 floor unobservable on macOS WKWebView | n/a (popups are not SSR'd) |
| before paint, G | zero extra frames (microtasks, V `lib.rs:250-264`) | at least 2 frames plus IPC per open on a fallback engine (finding 4); fail-open watchdog on the page is the right place (V `webview.rs:563-572`) | same | n/a |
| next frame, bridge stream | per-frame, latest-wins (V `bridge.ts` `listen`, hot rows) | per-frame messages, no in-flight limit built; can reorder against vdom events (finding 9) | same | n/a |
| none, R | holds | holds | holds | n/a |

## Findings

### 1. DETAIL, major. The per-element observer of `reconcile` cannot discover an insertion; "an open saves a frame" holds only if discovery is page-side

- **Claim attacked:** 2.2 "Kinds": "one attribute-filtered MutationObserver per element ... reconciles at attach ... on desktop it runs before paint in the frame the edit lands, so an overlay open saves a frame".
- **Evidence:**
  - V `primitives/src/top_layer/popover_sync.rs:222-232`: the library mounts popover content only while open ("none of which keep their content mounted"). So for the common case the element appears already carrying `data-dx-open`, and the attribute never changes afterwards.
  - I: Dioxus sets dynamic attributes on the cloned, still detached template before appending it, so an observer on the element, or a subtree observer on an ancestor, never gets an attribute record for it. Only a `childList` record of the insertion exists.
  - V `dioxus-interpreter-js/src/ts/native.ts rafEdits`: all edits of a render land in one rAF callback, and the microtask checkpoint after the callback is where a MutationObserver fires (W HTML "update the rendering"; the ack `markEditsFinished()` was already sent synchronously in the same callback, so the callback's microtasks run before any later Rust message, which is a later task). So the frame claim is true when something page-side sees the insertion in that checkpoint.
  - If attach is instead issued by Rust (`onmounted` is a blocking request inside the edit application on desktop, `unified_bindings.rs:183-193`, then an effect after the ack, then a pump message), the first open of conditionally mounted content costs what today's effect path costs. Nothing is saved.
- **Change:** specify discovery. The reconcile module starts one document-level `{childList, subtree}` observer that scans added subtrees for `[data-dx-cmd]` (cost proportional to added nodes, reported in `bridge-cost`) and does the level read at attach and once at install for markup already in the page. Keep a per-element attribute observer only for always-mounted or animated-exit content (`use_popover_shown_while_mounted`). Count observers in an inventory next to `listener-inventory.spec.ts`. Otherwise drop the frame claim from the plan and keep Rust attach.
- **Subsumes:** the SSG attach question (the install-time scan handles server markup), "when the runtime loads late (#5919)", finding 12's stale-answer rejection.
- **Why not DESIGN:** level-triggered attach is correct with either mechanism, and the kinds, admission and layering do not move.

### 2. DETAIL, major. A browser-initiated change leaves the attribute unchanged, so "level-triggered, converges" does not cover a vetoed close

- **Claim attacked:** 2.2 "reconciles ... on every change, so a command that is lost, early or reordered converges".
- **Evidence:** a light dismiss or Escape changes the DOM state, not the attribute. V `dioxus-core/src/diff/node.rs:416-470`: the vdom writes an attribute only when its value changed (or the attribute is `volatile`). For a controlled `open` whose `on_open_change(false)` is ignored the signal stays true, the attribute stays `true`, no edit and no record exist, and the DOM stays closed while Rust says open (the failure the plan says it removes). Separately, the report is a `dioxus.send` and a following click is a synchronous XHR handled inline, so the click can run first (`protocol.rs:62-63`; plan 1.4 table row `dioxus.send`): Rust toggles from a stale value. The window is about one Rust round trip, so minor in practice.
- **Change:** publish command attributes as `volatile` (`Attribute::new(name, value, None, true)`; the vdom then rewrites them on every render of that scope and the observer fires even for an equal value, W DOM "change an attribute" queues a record for any set), and have a vetoing handler bump a signal that the scope reads so it re-renders. State that the page's report carries the attribute generation it answers, and Rust drops a report older than the last publish. Add the veto case to the reconcile spec.
- **Subsumes:** the stale-toggle residue left after the row 115 echo is removed, and the generation blind spot in finding 12 (a same-value publish is invisible to the vdom).

### 3. DETAIL, major. `stickToEnd` reads a Rust-published mode that is stale after a page-originated detach, so it fights the user on desktop

- **Claim attacked:** registry row `stickToEnd` ("while the Rust-published `data-mode` is `following-bottom`"), residue 2.5 ("a late arm or release delays the start or stop of following by a frame").
- **Evidence:** the user's wheel-up is a page event. The detach reaches Rust over the bridge (IPC), Rust re-renders, and the attribute lands in the next rAF after the ack (V `webview.rs:537-590`, `native.ts rafEdits`): at least two to three frames on desktop. At token rate (about one growth per frame) every ResizeObserver callback in that window still sees `following-bottom` and writes `scrollTop = scrollHeight`, cancelling the user's scroll. "Late release" is not a late arm: it is a write against the user's input. The legacy controller avoids it by deciding locally. The plan's own pass column lists "no fight frames".
- **Change:** `stickToEnd` owns detach locally: follow only if the viewport was at the end before this growth (computed from `scrollTop`, `clientHeight`, `scrollHeight` the callback already has), cleared by the page's own passive wheel, touch, key and scroll-up observation; the Rust mode only gates (not following in `anchored-to-message`) and re-arms. General rule to state beside the root rule: **a transition caused by a page event is applied on the page first; Rust mirrors it and re-publishes only for Rust-originated changes, with a generation.** Apply it to `reconcile` (finding 2), `pan` and `settle`'s takeover.
- **Subsumes:** the W8 desktop fight, the ownership question for any bidirectional state.
- **Why not DESIGN:** it is the root rule applied to an input one mapping missed; placement, kinds and admission are unchanged.

### 4. DETAIL, major. The tier predicate, the floors and the cost of G on Apple desktop

- **Claim attacked:** 2.1 floors and probe, 2.3 item 1 and 5 ("`data-dx-anchored` is the tier predicate ... a boot-probe failure leaves it off").
- **Evidence:**
  - V `primitives/src/top_layer/attributes.rs:142-159`: `data-dx-anchored` is written unconditionally by Rust for every anchored popup (its value is the kind), and the engine sheet scopes on it. It is not an engine tier today. A vdom-written attribute cannot carry a client-only probe result without a signal round trip, and the first popup can precede the probe.
  - I: the macOS WKWebView user agent is frozen (`Macintosh; Intel Mac OS X 10_15_7`, `AppleWebKit/605.1.15`, no `Version/`), so a "Safari 26.2 floor" cannot be read there; iOS exposes `OS 26_2`; WebKitGTK is unverified. On those the probe is the only classifier, and the probe has never run on a webview.
  - I: a gated open on desktop is event, render, rAF edit, ack, effect, pump, webview task, reply over IPC, render, rAF: two to three frames plus IPC (plan 1.4 item 2). Today's T8 fallback measures and places in one JS turn (V `anchor_fallback.rs` doc). Safari 26 WKWebView is the plan's own "majority for months", so G on the fallback tier is the default experience on Apple desktop, a strict slowdown there.
- **Change:** put the probe result in a page-owned root attribute (`html[data-dx-anchor="a|fallback"]`) written by a synchronous probe at bridge load, and key the CSS gate on that, not on a vdom attribute; define the pre-probe default as fallback (gated, fail-safe); do not cache the result across loads (the engine build is unobservable). Before 3.PL starts, decide whether the fallback tier's default is G or the reserved `place` (in-turn re-place); measure `g-latency` on a WebKit desktop build first. Both are mappings of T8.
- **Subsumes:** "once per engine build" (unobservable on WKWebView), the pre-probe window, the question of what `place` is reserved for.

### 5. DETAIL, major. Row 1 of the deadline table fails silently on Android for payload events

- **Claim attacked:** row 1 "an rsx handler if Dioxus delivers the event and every field the decision reads"; 1.2 "Works" for desktop and mobile.
- **Evidence:** V `native.ts handleVirtualdomEventSync`: the serialized event travels in a base64 request header (the code's own comment: 16 KB per header, "we lose the event due to header max size"), and `serializeEvent` includes every form value on `submit` and the value on `input`, `change` and `paste`. When the request fails, `JSON.parse(xhr.responseText)` throws inside the listener and the default action proceeds; on the Rust side a parse error answers `SynchronousEventResponse::new(false)` (V `webview.rs:60-95`). So a `prevent_default` on a large `submit` loses the cancel and the page navigates. (The plan lists the 12 KB drop in the Android row of 1.4 but not what it does to row 1.) `DemoForm` is immune by construction (`method="dialog"`, V `preview/src/components/form/component.rs:118`), which is the better pattern.
- **Change:** add a condition to row 1: the serialized event stays small. Events that carry user text or form values (`submit`, `input`, `change`, `paste`, `drop`) are not rsx-cancel candidates on mobile; cancel declaratively (`method="dialog"`, `inputmode`, CSS) or page-side. List the event names in the gate that counts rsx cancels.
- **Subsumes:** the 133 `prevent_default` sites get one extra audit column.

### 6. DETAIL, minor. Corrections and omissions in the desktop timing contract (1.4)

- **Citation:** the ack gate is in `poll_vdom` (V `webview.rs:537-590`). `dioxus-core virtual_dom.rs:445-536` (`wait_for_work`, `poll_tasks`) has no gate: row "tasks and effects" should cite the first.
- **Events run against a vdom ahead of the DOM:** a render in flight is not yet applied, yet a synchronous XHR event is routed by `data-dioxus-id` to the current vdom. If the in-flight diff freed and reused an id, a click on the old, still visible element reaches the new element's handler (I; ids come from a reclaiming slab, `dioxus-core diff/node.rs` `dom.reclaim`). Add a row; the rsx `prevent_default` rows read Rust state that can be one render ahead.
- **Rust to page is one script evaluation per message** with the JSON inside the script text (V `query.rs:104-106`, `rustSend({data})`), and the pump's FIFO is the webview's order of `evaluate_script` calls, which no Dioxus code guarantees (I; each platform backend orders them, the plan relies on it).
- **Bridge writes are ack-gated like tasks:** a verb flush is an effect, and effects run only after the ack (`edits.rs` comment "We don't run the virtual dom while this is true to avoid running effects before the dom has been updated", V). Not a flaw, but it is where a handler's bridge write waits.
- **A lost ack stalls the runtime for good:** if an edit application throws inside the rAF callback, `markEditsFinished()` never runs (V `native.ts rafEdits`, no try/catch). The page-side fail-opens are then the only recovery, which supports putting them there; add a diagnostic.
- **Change:** edit the table and the rows above; no construct moves.

### 7. DETAIL, minor. The built bridge has one FIFO channel for element operations only

- **Claim attacked:** 5.2 "All Rust-to-JS traffic goes over one long-lived channel, so messages are FIFO".
- **Evidence:** V `l30/primitives/src/bridge/transport.rs` (working tree): `fire`/`ask` use the single `Pump` eval, but every hook's `Session::open` opens its own `Program::Serve` eval and sends its commands immediately from an effect (V `session.rs:382-420`), `JsListeners` opens one eval per script, and `ensure_module` and `ensure_installed` are separate evals. Order across them is the order of the `evaluate_script` calls. Effect order also differs from program order: `pending_effects` pops the lowest scope height first (V `dioxus-core scheduler.rs:167-169`), so the root-owned flush effect runs ahead of descendants' effects. Observable consequence is small (installs and writes are issued from effects after the ack, and webview evals are ordered), but the sentence is false as written.
- **Change:** say "element operations" in the plan, `js-interop.md:103` and the transport doc, or carry session commands as pump messages. Keep the transport doc's honest table.

### 8. DETAIL, major. Fail-open in the built bridge is an optional lease, not the plan's mandatory acknowledgement

- **Claim attacked:** 2.2 "Every behaviour has a fail-open path and an epoch ... an ack outstanding for about one second makes the behaviour stop preventing".
- **Evidence:** V `bridge/behaviours.rs:94-97` and `session.rs:392-427`: `lease_ms` is `Option` per variant, renewed by a Rust `sleep` loop at half the lease; `check-bridge-behaviours.sh` has no check that a `prevent` variant declares a lease (V: the script's only mention of lease is a comment, line 16), and `Info` carries no effect set (the plan's admission source). A `prevent` variant with no lease passes every gate. A lease is a heartbeat, which is better than an ack (it also covers a stalled render, a hidden webview and a dead channel with no per-event traffic), but it covers "Rust stopped", not "Rust is alive and wrong". A hidden webview expires the lease by design and the next renewal reinstalls it (V `bridge.ts installBehaviour`: after `off` the slot is empty, an equal epoch passes `stale()`).
- **Change:** make the lease mandatory for any variant whose generated effect set contains `prevent` or `stop` (a gate on the generated file), size it above one render plus ack worst case, and rewrite the plan's sentence to the heartbeat. Keep a stuck-channel spec (`__dxBridgeDelayFrames = "never"`) per `prevent` behaviour.

### 9. DETAIL, minor. Streams are latest-wins per frame, not "one message in flight"

- **Claim attacked:** 5.2 and `js-interop.md:95` ("latest-wins, one message in flight").
- **Evidence:** V `bridge.ts` `listen`: for a `hot` event `pending = message; if (!frame) frame = requestAnimationFrame(flushPending)`, so at most one message per frame, with no acknowledgement. `observe` has no coalescing, `sample` sends once per frame when changed. When Rust is busy for 30 ms on desktop the page keeps posting one message per frame, and `dioxus.send` is unbounded and handled on the event loop (plan 1.4 row `dioxus.send`). V `webview.rs`/`protocol.rs`: the synchronous XHR bypasses that queue, so a stream message can be handled after the vdom event that followed it.
- **Change:** either implement an in-flight window (Rust acks a stream sequence number, the page sends the next only after an ack or a frame budget) or reword both documents to "at most one message per frame per stream". Streams that must order against vdom events carry a sequence number or both use the bridge (L30's start/move/end all go through the bridge, so it is ordered; mixing a bridge move with a vdom `onpointerup` is not).

### 10. DETAIL, minor. The built bridge loads by eval, against the plan's `asset!` plus `document::Script`

- **Claim attacked:** 434 "Dioxus best-practice review ... Folded into lane 3.0: ... loading via `asset!` + `document::Script` with a bounded ready handshake (#5919)"; 5.2 "One transport seam".
- **Evidence:** V `l30 transport.rs` doc, "Loading": `asset!` plus `Script` "was not adopted" (needs a provider in every app, #5919, #5183); `ensure_installed` evals the bundle on first use. Consequences: the bridge exists only after wasm boot and the first effect, so no page-side code (probe, reconcile discovery, pre-hydration queueing) can start earlier; every consumer needs `new Function`/`unsafe-eval` on web (plan already notes the CSP); the bundle is parsed on first use per webview.
- **Change:** decide and write one: either amend the plan to "evaluated through the transport, post-boot" (then finding 4's probe and finding 1's discovery start at install, and SSG pre-hydration never sees them), or have 3.0b add the `Script` route. The decision interacts with findings 1, 4 and 11, so it is not a free choice.

### 11. DETAIL, minor. SSG hydration: what the plan can rely on

- **Evidence:** V `dioxus-web/src/lib.rs:100,148` and `mutations.rs`: the hydration rebuild runs with `skip_mutations`, so the client writes no attribute and no node. V `dioxus-interpreter-js/src/ts/core.ts:204-225` `hydrate_node`: it only reads `data-node-hydration`, sets `data-dioxus-id` and creates listeners. V `lib.rs:153-156`: mounted events flush after hydration, and effects run at the first `wait_for_work` after `rehydrate`. So a hydrated page produces no attribute record for `data-dx-*`, and convergence rests only on the level read at attach (the plan says so; correct).
- **Gaps:**
  - If the bridge is loaded pre-hydration (finding 10's alternative), an install-time scan runs `showModal()`/`openFocus` on server-rendered default-open content before wasm, and a pre-hydration light dismiss is reported to no listener; at attach the level (`true`) reopens what the user dismissed. Queue reports page-side until the channel opens, or load post-hydration.
  - A client first render that differs from the server's is never written to the DOM (the vdom compares against its own first value); `hydration-parity` covers it, and client-only values must arrive through an effect.
  - Behaviours have a pre-install window on SSG pages (`otpFilter` cannot reject characters typed before install; `indeterminate` shows only after install). Install must sanitise the current value and state.
- **Change:** one paragraph in 2.2 "Kinds" and a line each in `otpFilter` and `reconcile` rows.

### 12. DETAIL, minor. G: the stale-answer rule needs an observer, and "one writer per attribute" is not what is described

- **Claim attacked:** 2.3 items 1, 4: "the page owns the generation and the clear, Rust owns the answer. Exactly one writer per attribute".
- **Evidence:** the page clears `data-dx-placed` and Rust writes it through the vdom: two writers. The CSS shows the panel the moment the attribute exists, so a stale answer (older generation) is visible unless the page removes it before paint. Dropping it requires a page-side observer on that attribute in the same microtask checkpoint (the finding 1 mechanism); nothing in the plan names one. V `diff/node.rs:416-470`: an equal-value write is not emitted, so the generation must differ every open (it does) and the attribute should be `volatile`.
- **Change:** two attributes (`data-dx-gen`, page-owned and minted in the `show` verb's turn; `data-dx-placed`, Rust's answer carrying that generation), a rule that the reconciler removes a `placed` older than `gen` before paint, and the same on the fail-open path. Lane 3.PL's first-frame spec asserts it. Deferring the literal wiring is fine; the invariant should be written now.

### 13. DETAIL, minor. The lane's docs still teach the dropped rule

- **Evidence:** V `l30 transport.rs` "Consequences" ("Pointer capture and `focus` are not verbs ... They are rsx handlers") and `bridge/mod.rs` ("What the bridge can and cannot carry"). Under the amended rule rsx cannot take pointer capture (no API in `dioxus-html`) or reliable focus on desktop (`set_focus` is a webview query that runs after the handler, `element.rs:137-160`); both are effects of a named behaviour. The gate `check-bridge-behaviours.sh` already forbids the verbs, so only the prose is wrong.
- **Change:** fix both docs before an agent reads them (`js-interop.md` is the first thing CLAUDE.md sends them to).

## Attacks that failed

- **rsx `prevent_default` is synchronous on web and desktop (V).** `dom.rs:112-116` and `native.ts:380-384` with `webview.rs:191-195`. `stop_propagation` reaches the DOM on neither (V: web applies only `prevent_default`; the desktop struct has no such field). Holds.
- **`oncancel` works on desktop.** `cancel` is in `EventData` and deserialises from `{}` (`dioxus-html/src/transit.rs`, `events/cancel.rs`), so L2's veto is reachable on desktop.
- **Effects run only after the DOM is updated on desktop (V).** `poll_vdom` stops calling `wait_for_work` while a render is unacked (`webview.rs:537-590`, `edits.rs` comment). The plan's "queue_effect flush after the render's edits" holds on desktop and on web (`dioxus-web lib.rs` renders and flushes before the next `wait_for_work`). It costs a frame on desktop, as 1.4 says.
- **An eval from a handler runs before that render's edits (V).** `Query::new_query` calls `evaluate_script` at call time (`query.rs:60-105`) while edits wait for socket, Blob and rAF. Holds on both renderers.
- **A MutationObserver callback runs before paint in the rAF's edit frame (W).** The checkpoint follows the rAF callback; IntersectionObserver and `toggle` are queued tasks, so post-paint. The plan is right.
- **The edit socket cannot lose edits** (V `edits.rs`): unacked edits are re-queued when the socket closes. A hidden webview stalling every Rust timer is real and the plan already puts every fail-open on the page.
- **The closed `Program` transport.** `Serve`, `Pump`, `Legacy`; `pub(in crate::bridge)`; `document::eval` named only in `transport.rs`. The one caller-composed body is `Program::Legacy`, constructed only in `legacy.rs` and used only by `js_listener.rs`: it matches the plan's reasoned callers row. Holds, as a transitional hole.
- **Epochs.** `stale()` per element and key, an equal epoch renews, a lease expiry then reinstalls on the next renewal, replies echo the epoch: implemented as the plan describes.
- **SSG server side.** Effects never run on the server, so no `Session::open` or `ensure_installed` eval happens during prerender; `fire`/`ask` only queue. I found no caller that awaits `ask` from a render-time task.
- **iOS keyboard from non-gesture focus.** Consistent with the `openFocus` rule that touch never focuses an editable element.
