# Round 2, lens `platform-desktop` (2026-10-26)

**Verdict: HOLDS WITH CHANGES**

DESIGN findings: 1, DETAIL findings: 8

Summary:
- The root rule survives the platform check. Desktop handlers are synchronous (`native.ts:566-593`), `stopPropagation` is never sent by Rust (`webview.rs:630-641`), tasks and effects wait for the render ack (`webview.rs:563-572`), and an eval from a handler runs before its own render's edits (V).
- One design gap. The three-way placement rule sorts by target and event kind, but the root's own question for a stream is "what does a late answer cost?". W8 (follow bottom) is a same-frame layout write. Safari only enabled scroll anchoring in 27 (W), so on the majority Apple webview W8 gets R alone, a frame or more late, and its stated exit forbids page-side JS. Add a fourth row, or a named `stickToEnd` behaviour.
- "Baseline newly available plus a correct fallback" is not an honest support policy as written. Baseline-low dates the first release of each engine, and the first releases carry placement bugs that the vendors' own notes list (finding 2). The T8 outcome check cannot catch state-dependent bugs, and a post-hoc check paints a wrong frame on engines that G leaves ungated.
- G's mechanism contradicts itself: pending "animation: none" kills the delay-based fail-open. The delay change jumps a running animation. The fail-open is defeated by host CSS that the theme itself ships. The synchronous clear does not need `beforetoggle`, which does not exist for dialogs on the fallback engines.
- The desktop contract table is mostly right. It needs five additions: the Blob hop, stalls while the webview is hidden, the app-wide root listener and `onmounted` as blocking XHR inside the rendering step, the Android IO thread, and the Windows drag-drop replacement.
- Stream IPC volume at 120 Hz is acceptable (about 120 messages/s per stream, each a few hundred microseconds on the main thread, I), provided the bridge gets backpressure and latest-wins coalescing. Dioxus vdom handlers at that rate are the worse choice.

Labels: V = read in repo or pinned source, W = web source, I = inference. Repo paths are under `/home/user/wt/b28`; crate paths under `~/.cargo/registry/src/*/`.

---

## 1. DESIGN, major: the placement rule has no "cost of a late answer" axis, so W8 on engines without `overflow-anchor` is R-only by construction

**Claim attacked.** Root doc 2.2 (placement rule: continuous streams go to bridge `subscribe`/`sample`, "never a vdom handler"), 2.3 ("Not for continuous loops: a lagging write during scroll or drag *is* a wrong paint ... Continuous tracking goes to A. B and R ... are never frame-exact"), residue 2.2 ("Engines without `overflow-anchor` ... run on the invariant alone ... a frame late ... which the policy allows") and 2.5 ("The exit ... is that engine's Rust-only path, never a revert to JavaScript").

**Evidence.**
- (W) Safari 27 release notes: "Enabled scroll anchoring, which prevents visible jumps in scroll position ..." (developer.apple.com/documentation/safari-release-notes/safari-27-release-notes.md). WKWebView follows the OS, so macOS/iOS 26 webviews, which are the majority Apple population for months after a September release, have no native anchoring (I for the adoption claim). WebKitGTK is unverified (W: the 2.52.0 notes are silent).
- (V) Follow-bottom is a same-frame write today: `message_scroller.js:840-848` calls `scrollToEnd("auto")` from a ResizeObserver, which is pre-paint. The plan moves it to bridge `observe` plus a Rust `scroll(end)`.
- (V) On desktop that path is IPC out (`native_eval.ts:49-56` to `webview.rs:308` to `app.rs:309`), a Rust task that cannot run while a render is unacked (`webview.rs:563-572`), and an eval back. It costs at least 1 to 2 frames per growth step (I; the plan says so, 1.4 item 2).
- At token rate (50/s, about one line growth per few tokens) the newest line is painted below the fold, then scrolled into view. The acceptance row "follow error at most 1 px over the JS arm" (root doc 5.6, port 1) is unattainable on that path by construction.
- The plan already states the discriminant in G ("a lagging write ... is a wrong paint"). The placement rule does not use it. C5 passes the test (passive, output is a CSS variable, lag is only lag). W8's follow does not.

**Concrete change.**
- Add to the placement rule a column or row: "stream whose reply is a same-frame layout write: A where verified, else a named page-side behaviour".
- Registry entry `stickToEnd{endSelector, followAttr}`:
  - A ResizeObserver callback sets `scrollTop = scrollHeight` while Rust-published `data-mode=following-bottom`.
  - It sends the mode transitions to Rust, and Rust keeps the mode machine, intent, opening position and the invariant.
  - It has the standard fail-open and epoch. It is about 40 lines, not 1,065.
- Replace the W8 exit sentence "never a revert to JavaScript" with "the page-side follower on engines without verified anchoring".
- Keep the Rust invariant as the backstop (first overflow at offset 0, DPR, tab return), as already planned.

**Subsumes.** W8's "frame late on desktop, policy allows" caveat. Any later block whose reply is a same-frame layout write (the plan's own O3/V19 are G, which pass).

**Does not subsume.** C5 (passes the discriminant).

---

## 2. DETAIL, major: the support policy and the T8 outcome check

**Claim attacked.** Root doc 2.1 "Web: Baseline newly available ... Older engines must stay correct through a feature-detected fallback", and the T8 row ("A + Rust verification", "tooltip's decoration tier ungated").

**Evidence (W).**
- Baseline-low is the first release of each engine, and the first releases are the buggy ones:
  - Firefox 147 shipped anchor positioning with no `position-try-order`. 148 added it and "`position-area` now correctly keeps the anchored element within the viewport" (developer.mozilla.org Firefox 147 and 148 release notes). T8 uses `position-try-order: most-height`, and the plan's own table says Firefox 148.
  - Safari 26.2 (Dec 2025) fixed:
    - "anchor-positioned elements transitioning from `display: block` to `display: none` can not anchor themselves";
    - "position-area handling to include the in-flow scrollable area of the initial containing block";
    - the order fallbacks are tried;
    - auto margins with `position-area`;
    - and it added `flip-x`/`flip-y`.
  - Safari 27 fixed "`position-area` not being able to anchor to an element positioned using anchor functions" (a submenu anchored to an item in an anchor-positioned parent menu) and added "transform-aware anchor positioning".
- Both Safari bugs bite popovers by construction: reopen is display none to block, and nested menus are the nested case.
- The spec's acceptable-anchor rule itself is satisfied for nested popovers and for anchors inside modal dialogs: top-layer order, then flat-tree order (W, css-anchor-position-1 2.3). The failures are engine bugs, not spec limits.
- (V) `anchor_fallback.rs:21-28` already records that a support check cannot see these failures.
- A one-time check cannot see them either, because they depend on state (reopen, nesting).
- A per-open check is post-hoc. On an A-engine the root doc's tier rule leaves the panel ungated, so every buggy open paints a wrong frame first.
- Not "older": Safari 26 is the current WKWebView until OS 27 adoption. Devices locked to iOS 18 (I: the iPhone XS and XR do not get 26) and macOS 15 are a permanent fallback population. WebView2 109 (Win7/8) and Android WebView on devices without Play updates (I) are the same.

**Concrete change.**
- Replace "Baseline newly available" with a per-feature floor table. T8's set is `position-area`, `position-try-order`, `flip-*`, `anchor-size` and `position-visibility`. Each row names an engine floor and the vendor release note that fixes its known bug.
- Add a boot probe set run once per engine build, before the first popup: viewport-edge shift, reopen x3, nested submenu, anchor under a scaled parent. A failing probe makes that engine count as a fallback engine, so its tier is gated. The per-open measure stays for decoration and verification.
- 3.PL's Firefox and WebKit runs target Firefox 147 and 148 and Safari 26.0, 26.2 and 27, and add a nested-submenu spec and a reopen x3 spec.
- WebKitGTK stays a probe-first unknown.

**Subsumes.** The "Baseline" wording and the open question of which engines are gated. Does not change A/G/R.

---

## 3. DETAIL, major: hardened G's pending style and its fail-open contradict each other, and host CSS can defeat the fail-open

**Claim attacked.** Root doc 2.3 items 2 and 3 ("pending ... `animation: none` semantics" and "fail open through the enter animation's `animation-delay` ... forwards only").

**Evidence.**
- Item 2 says no keyframe may apply. Item 3 needs the enter animation present and delayed. If the pending rule literally says `animation: none` or `animation-name: none`, the delay never runs and a wedged Rust leaves the panel invisible forever.
- The delay alone already satisfies "no keyframe applies" (no backwards fill, W: CSS Animations 1). Item 2's "animation: none" should be dropped.
- A live `animation-delay` change when the gate stops matching does not restart the animation. The effect's local time becomes elapsed minus the new delay, so the panel appears already 20 to 60 percent through its fade (I from the Web Animations timing model). The reduced-motion rule (`dx-components-theme.css:1785-1790`) sets only `animation-duration` (V), so that part holds.
- Host CSS defeats a CSS-only fail-open. The theme ships `[data-dx-motion-key] *, ... { animation: none !important }` under `prefers-reduced-motion: reduce` (`dx-components-theme.css:2421-2432`, V). That matches popups rendered as DOM descendants of a motion-key host (the popup-boundary class). Common user resets do the same (I).
- A Rust-side timer cannot be the alternative, because a render unacked or a wedged runtime stops every task (`webview.rs:563-572`).

**Concrete change.**
- Pending selects its own animation name, `dx-gate-failopen`: delay `--dx-gate-timeout`, forwards fill, keyframe `to { opacity: 1 }`.
- Placed selects the enter animation. The name swap restarts it from 0, so the fade is clean.
- The gate verb (finding 4) arms a page-side watchdog timer of the same length, because the verb lives in the bridge anyway.
- The `g-latency` and stuck-channel specs add a case with `animation: none !important` on an ancestor.

**Subsumes.** The "delay drops to 0" jump. Settles the literal CSS that 3.PL's first-frame spec was to find.

---

## 4. DETAIL, major: the synchronous "placed" clear does not need `beforetoggle`

**Claim attacked.** Root doc 2.3 item 1 ("cleared synchronously in `beforetoggle`", the `dialog[open]` twin).

**Evidence.**
- (W) `beforetoggle` on `<dialog>` is Chrome 132+, Firefox 133+, Safari 26+ only (caniuse mdn-api_htmlelement_beforetoggle_event_dialog_elements). The dialog twin has no clear signal on Safari 18.x and older, which are exactly the fallback engines where G matters.
- (W, HTML) `beforetoggle` does not bubble. A document capture listener is a page-lifetime listener, absent from the closed registry and from `listener-inventory.spec.ts`'s baseline. Dioxus has `onbeforetoggle` (`dioxus-html generated.rs:300`), but an rsx handler's attribute write lands in a later render, not synchronously.
- (V) Every library open is Rust-driven through eval or `transaction` (T5/T7/L16). Native `popovertarget` appears only in preview demos (`preview/src/components/top_layer/component.rs:263,305,582,633`), which carry no placement attribute.
- Rust-driven closes already clear in the hide transaction. Native closes (Esc, light dismiss) report through `toggle` before a human can reopen (I).

**Concrete change.**
- The bridge `show` verb clears `data-dx-placed` in the same JS turn, before `showPopover()`/`showModal()`. It carries the generation token. No `beforetoggle` listener is needed and the dialog twin is covered on every engine.
- Keep a `beforetoggle` capture listener only if the owner wants native `popovertarget` opens gated. It would then be a registry behaviour (`gate`) with a vector, a budget row and a listener-inventory entry.

**Subsumes.** A missing registry member and an engine hole in the dialog twin.

---

## 5. DETAIL, minor: gate scope and the tier predicate

**Claim attacked.** Root doc 2.3 items 1, 5 and 6 ("every open path is gated by default" versus "only placement-critical is gated").

**Evidence.**
- As written, `:popover-open:not([data-dx-placed])` gates every popover on every engine. The two tiers need a predicate such as `:not([data-dx-anchored])`, which the plan only mentions as a 3.PL attribute (`top_layer/attributes.rs`).
- The enter animation is the theme's `[data-dx-surface="floating"][data-state="open"]` rule (`dx-components-theme.css:1775-1777`). Navbar, NavigationMenu and Toast use their own animations (`navbar/style.css:147,219`, `navigation_menu/style.css:194,218`, `toast/style.css:176,216`), so the swap in finding 3 must be per surface.
- `dialog[open]` twin: the plan never names a Rust-placed dialog. The `::backdrop` is not part of the pending style, so the scrim shows for up to the timeout with no panel.

**Concrete change.**
- Name the predicate.
- List the surfaces the gate covers.
- Say which dialogs are gated, or drop the twin.
- Gate the scrim or declare the flash.

---

## 6. DETAIL, major: the hot-event rule is scoped per call site, but Dioxus pays per event name, app-wide

**Claim attacked.** Root doc 1.4 item 6 and 5.2 ("Hot events stay off the vdom"; `check-hot-events.sh` with a reasoned allowlist).

**Evidence (V).**
- For a bubbling event, Dioxus registers one listener on the app root per event name, with no options (`dioxus-interpreter-js core.ts:109-130`). One `onpointermove` or `onwheel` anywhere makes every such event in the app a blocking XHR on desktop (`native.ts:353-407`, `566-593`) and non-passive.
- `onmounted` on desktop is a synchronous XHR sent from inside the edit application, in the rAF callback (`dioxus-interpreter-js unified_bindings.rs:183-193`). There are 113 `onmounted` sites in `primitives/src`, and `collection.rs:738` gives every list item one. A mounting batch of N rows is N blocking round trips inside one rendering step.

**Concrete change.**
- The gate counts event names app-wide (a per-event-name allowlist, not per file) and adds `onmounted` as a cost class. A per-row handle needs a batched `observe`-style alternative or a single container handle.
- 1.4 item 6 gains the `onmounted` sentence.

---

## 7. DETAIL, minor-major: rows and additions to the section 1.4 table (checked against the 0.7.9 sources)

Rows checked and correct (V): sync XHR event path, the rAF edit application and ack (`native.ts:445-458,495-499`), no new render before the ack (`webview.rs:563-566`), tasks and effects only inside `wait_for_work` (`virtual_dom.rs:441-534`), eval unordered with edits (`element.rs`, `query.rs`), `dioxus.send` through the event loop. Needed corrections and additions:

- **Blob hop.** Edits arrive as a WebSocket binary frame, a `Blob`, and `data.arrayBuffer().then(rafEdits)` (`native.ts:476-484`). There is one extra task hop before the rAF is registered, so a late arrival can miss the next frame. Add it to the "render to DOM" row. The loopback WebSocket (`edits.rs:117-125`) is a platform dependency (local firewalls, iOS sleep kills it).
- **Stall semantics.** While a render is unacked no task or effect runs, including Rust timers. A hidden or occluded webview does not run rAF, so the whole runtime stalls. The fail-open of G and of every behaviour must be page-side, and any "Rust timeout" in the plan (the 1 s ack watchdog is page-side and fine; `crate::sleep` 180 ms in W8) is paused during a stall.
- **Android.** Events are handled on a WebView IO thread under a global mutex (`webview.rs:82,571,583`, `android_sync_lock.rs`). The header limit (`native.ts:586-587`) drops vdom events over about 12 KB raw.
- **Windows drag-drop.** With the default config (`disable_file_drop_handler: false`, `config.rs:127`) WebView2's HTML5 drag events are replaced by synthetic ones: `dragover` only when the element under the pointer changes, no `dataTransfer`, a fake `file.txt` on drop (`webview.rs:329-347`, `launch.rs:61-90`, `native.ts:256-326`). L25 `dropAnywhere` and the native DnD list cannot be proven by Chromium-on-web. Add a platform row and a doc line (`with_disable_file_drop_handler(true)` for apps that need in-page drags).
- **WKWebView sync XHR.** Upgrade from "unconfirmed" to W: WebKit bug 186902 "WKURLSchemeHandler doesn't handle sync XHR" is RESOLVED FIXED (r233113, 2018-06-22). WebKitGTK stays I.
- **Mobile source.** `mobile = ["dep:dioxus-desktop"]` (`dioxus-0.7.9/Cargo.toml:122-125`, V); there is no separate dioxus-mobile crate, so every desktop row applies to iOS and Android, plus the two Android notes above.

---

## 8. DETAIL, minor-major: stream volume at 120 Hz is acceptable, with conditions

**Claim attacked.** Open question 4 (streams over the bridge versus vdom handlers).

**Evidence.**
- Per JS to Rust message, the desktop path is `JSON.stringify` and `ipc.postMessage` (`native_eval.ts:49-56`), a parse in `ipc_handler` (`webview.rs:308`), a tao user event, `handle_query_msg` with a second parse (`app.rs:309`), an unbounded mpsc send (`query.rs:39`) and a poll event (V). That is 2 JSON parses and at least 3 event-loop turns, a few hundred microseconds of main-thread time per message (I).
- Browsers deliver pointermove, wheel and scroll at most once per frame in practice (I). At 120 Hz that is about 120 messages/s per stream, so a few percent of the main thread per stream. Vdom is worse: a blocking XHR per event on the renderer's main thread, plus app-wide root listening (finding 6).
- Risk: the mpsc is unbounded and nothing polls tasks while a render is unacked (`webview.rs:563-572`), so a stall builds a backlog that replays after.
- Rust to JS is a compiled script per message (`query.rs:139`, `Query::send`).

**Concrete change.**
- `sample` and `subscribe` are latest-wins on the page side with at most one message in flight (send after the previous ack, or after a frame).
- Batched modes (C5) carry a cap and a drop-oldest rule that records the loss.
- High-rate Rust output goes through `set` (CSS custom property batches), not a message per frame.
- `bridge-cost` adds a "Rust busy 30 ms" case at 120 Hz.

---

## 9. DETAIL, minor: release desktop builds install a `contextmenu` preventDefault before any component code

`disable_context_menu` defaults to `!cfg!(debug_assertions)` (`config.rs:117`), and the init script (`webview.rs:441-456`) registers a document `contextmenu` listener that calls `preventDefault()`. A behaviour or test that reads `defaultPrevented` on `contextmenu` (none today, V grep) would see true, and the desktop listener inventory differs from the web baseline. Record it in `desktop-pass.md` and in the `listener-inventory` notes.

---

## Attacks that failed

- **Root on desktop.** Verified: `prevent_default` reaches the native event through the sync XHR response; `stopPropagation` is never sent (`SynchronousEventResponse` has one field); web applies only `prevent_default` (`dioxus-web dom.rs:112-116`).
- **"Two renders never share a frame."** Holds: render k+1 is sent only after the rAF ack, and `wait_for_work` is not polled meanwhile.
- **Eval from an effect runs after its render, from a handler before it.** Holds (effects run inside `poll_tasks`, handlers inline).
- **`toggle` is post-paint, `beforetoggle` is synchronous.** Holds (HTML popover algorithm).
- **Reduced-motion rule leaves `animation-delay` alone.** Holds (`dx-components-theme.css:1785-1790`).
- **Anchors inside modal dialogs and nested popovers are valid anchors by spec.** Holds (top-layer order, then flat-tree order, CSS Anchor Positioning 2.3). The failures are engine bugs (finding 2).
- **WKWebView cannot do sync XHR to a custom scheme.** Fixed in 2018 (finding 7).
- **WebKitGTK ships anchor positioning.** Neither confirmed nor refuted: the 2.52.0 release notes mention none; stays an unknown for the probe.
- **A tooltip's late `data-placed-side` is a design break.** It is a bounded cosmetic jump (a flipped arrow for a few frames); the plan states it, so it is not a design break.
