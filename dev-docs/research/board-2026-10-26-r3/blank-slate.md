# Board round 3, lens `blank-slate` (2026-10-26)

**VERDICT: HOLDS WITH CHANGES**

DESIGN findings: 0, DETAIL findings: 5

- Method: I wrote section 1 (my own construction) from the audit, the code and the Dioxus 0.7.9 sources only, and read the plan after it.
- Convergence: my design and the plan agree on every core element:
  - the root ("publish before, decide locally");
  - placement by deadline, with the strictest target (desktop) deciding;
  - commands as a published desired state with a page reconciler;
  - facts back over one channel with page-side reducers;
  - frame-rate events kept off vdom handlers;
  - one eval sink, sinks plus ratchet, pre-wasm text generated from one source.

  Two independent derivations landing on the same core is evidence that the core holds. I found no DESIGN flaw.
- What breaks are details.
  - **Finding 1:** the rule judges a bridge placement only by latency. On desktop, page facts and Dioxus events are two unordered inputs, so a Rust decision that combines them (carousel paging from a `selected` that bridge facts update, keys after a hotkey fact) resolves differently on desktop than on web. This needs a "commutative or page-side" condition on the R row.
  - **Finding 2:** the command kind's "an overlay open saves a frame" is false for the overlays that mount their content on open (all `use_animated_open` users). There the reconciler is attached over the bridge after the mount render. Element-bound attach and a synchronous `render` memo fix it.
  - **Finding 3:** `stickToEnd` keeps following until Rust republishes the mode, which fights the user's scroll during streaming on desktop. A behaviour must own the transitions its own events cause.
  - **Finding 4:** the roving-focus audit's condition "unless key repeat" is met by every roving widget. Cost it now, and consider `aria-activedescendant` (Rust-only, no JS) for composites.
  - **Finding 5:** the gated open has two show writers (the reconciler and a `show` verb).

## 1. The construction I would choose, designed before reading any plan document

Inputs read for this section, and nothing else: `dev-docs/research/js-audit-2026-10-25.md` (the 103-block inventory), the code it points to (`js_listener.rs`, `top_layer/popover_sync.rs`, `top_layer/anchor_fallback.rs`, `lib.rs` dialog driver, `modality.rs`, `virtual/bridge.rs`, the event handlers in `select/`, `context_menu.rs`, `chart.rs`) and the Dioxus 0.7.9 sources. Labels: V = read in code or crate source, W = web/spec, I = inference.

### 1.1 Facts the design rests on (the ones the audit does not state, or states differently)

F1 (V). **On web, a Rust write does not land "a frame later" when nothing in the chain waits on a timer.** The DOM event runs the Rust handler synchronously (`dioxus-web-0.7.9/src/dom.rs:86-115`). Marking a scope dirty wakes the app future, and `wasm-bindgen-futures` (pinned 0.4.71) re-exports `js_sys::futures`, whose queue schedules with `queueMicrotask` (`js-sys-0.3.98/src/futures/queue.rs:65-72`). The web run loop then does `wait_for_work` → `render_immediate` → `flush_edits` (`dioxus-web-0.7.9/src/lib.rs:169-260`), and effects run on the next turn of the same loop (`dioxus-core-0.7.9/src/virtual_dom.rs:511-535`, `runtime.rs:337-344`). Every step is a microtask. Microtasks drain before the next rendering opportunity (W: HTML, event loop processing model, "perform a microtask checkpoint"). So event → handler → render → patch → effect → eval → a synchronous layout read and style write all complete before the next paint. The audit's sentence "a Rust write goes through a signal, so it lands a render later than a read-then-write JS block" (section 2, last row) is true only of a chain that awaits `sleep` or rAF. On web, latency is not the reason a block has to be JS. The reason is a missing API (no `getAnimations`, `visualViewport`, computed style, `target`, listener options).

F2 (V). **On desktop and mobile (wry), Rust can meet none of the short deadlines.**
- Every Rust-handled DOM event is a *synchronous* XHR from the webview to the Rust process (`dioxus-interpreter-js-0.7.9/src/ts/native.ts:563-592`, `xhr.open("POST", endpoint, false)`). The webview main thread blocks for the round trip, so a Rust `onscroll`/`onpointermove`/`onwheel` blocks the page once per event.
- Edits travel over a websocket and are applied inside a `requestAnimationFrame` callback (`native.ts:445-457`, `475-485`). Rust then waits for the flush acknowledgement before it polls effects (`dioxus-desktop-0.7.9/src/webview.rs:563-566`).
- An effect's `eval` is a separate `evaluate_script` task (`dioxus-desktop-0.7.9/src/query.rs:60`, `document.rs:27-34`).

So a command issued from an effect lands at least one IPC round trip after the edits it belongs to, and the browser may paint in between.

F3 (V). **Ordering by construction exists in exactly one place: the renderer's own edit batch.** On both platforms the attributes Rust renders are applied in one task: web `flush_edits`, desktop the rAF callback. Synchronous page code that reacts to those attributes runs in that same task: a MutationObserver callback is a microtask of that task, and a custom element's `attributeChangedCallback` runs synchronously. That code is therefore ordered with the DOM it acts on on both platforms. An `eval` is ordered with the edits only on web.

F4 (V). **A Dioxus listener has no options and no target.** It is delegated to the app root (`core.ts:109-130`, `addEventListener(name, handler)` with no options). A Rust `onwheel`/`ontouchmove` therefore installs a non-passive listener on the root div, and Chromium's passive-by-default intervention covers only window, document and body (W: https://developer.chrome.com/blog/passive-event-listeners). Every wheel or touch scroll inside the app then waits for the main thread while such a handler is mounted. `preview/src/components/input/component.rs:50,77` forwards `onwheel` (I: a caller who passes it pays this for the whole page).

F5 (V). `eval` on web is `new Function(...)` per call (`dioxus-web-0.7.9/src/document.rs:209`). Today each open re-compiles and re-sends the whole policy text: the ink sheet, `MODALITY_JS`, the 114-line `OPEN_FOCUS_JS` and `DIALOG_CLOSE_FORWARD_JS` (`lib.rs:663-740`, `popover_sync.rs:291-307`). `globalThis.__channels` grows by one slot per eval and is never compacted (`dioxus-web-0.7.9/src/ts/eval.ts`; V, minor).

F6 (V). Today's split is "Rust state, JS commands, JS expandos": Rust owns `open`, and an effect sends an imperative script with the state `format!`-ed in. The JS keeps a second copy of state on the element (`__dxClosing`, `__dxActivate`, `__dxDialogGen`, `__dxCancelAt`, `lib.rs:690-735`). The echo bug of row 115 (a queued `toggle(open)` arriving after the signal went false, `popover_sync.rs:236-252`) and the "two evals have no ordering" note (`lib.rs:666-676`) are both symptoms of two state holders talking through commands.

F7 (V). The repo already holds the right pattern in two places:
- `modality.rs`: Rust evaluates `initial_focus` for all four modalities at render, *publishes* the table as `data-dx-initial-focus`, and the page decides at open time from the current modality.
- `virtual/bridge.rs`: the page coalesces and deduplicates, then sends a fact (`ScrollReport`). Rust decides and renders only when the window changes.

### 1.2 Where each piece of behaviour runs: placement indexed by deadline × target

The question to ask of a behaviour is not "is there a Rust API?" but "by when must the effect be visible or decided, and on which target is that hardest?". One component source serves web, desktop and mobile, so **the strictest target decides**. That target is desktop/mobile (F2).

| tier | deadline | runs in | why (evidence) |
|---|---|---|---|
| L0 | none: the platform does it declaratively | **markup + CSS, rendered by Rust**: `popover`, `<dialog>`, `popovertarget`, invoker `command`/`commandfor`, `closedby`, anchor positioning + `position-try`, `@starting-style` + `transition-behavior: allow-discrete`, `overscroll-behavior`, scroll-snap, scroll-driven animations, `hidden=until-found` | zero script on capable engines. Where an engine lacks it (iOS Safari: no anchor positioning, the reason T8 exists), a page fallback ships and runs **only when the measured outcome is wrong**, as T8 already does (`anchor_fallback.rs:20-36`). |
| L1 | before wasm / first paint | inline pre-paint script, generated from the same TS source as the runtime | nothing else exists before hydration (class c). |
| L2 | inside the event (cancel, stop, focus before the default), or same frame (read layout → write style before paint), or per frame (gesture, rAF loop), or a high-frequency stream (scroll, pointermove, wheel) | **page behaviour (TS)** that decides from state Rust has already published as attributes | desktop: sync XHR per event, commands one IPC late (F2); web: the API is missing (F4, audit section 2). |
| L2' | inside the event, but low-frequency, on an element Dioxus delegates, with the data it needs (click, keydown, pointerdown on the component's own element) | **Rust handler** | `prevent_default` works on both targets (web `dom.rs:113-115`; desktop's sync XHR returns `preventDefault`, `native.ts:380-383`); one sync round trip per keypress is acceptable. |
| L3 | eventual (the next frame or later is fine): state sync, form reset, visibility gating, windowing input | **Rust**, fed facts by page events | the only tier where Rust's latency on desktop is harmless. |

Two consequences:
- A behaviour moves to Rust only if it is L2' or L3 *on desktop*. Being expressible in Rust on web is not enough.
- A per-event stream (S2's `onscroll` on the Select list, `context_menu.rs:531` and `chart.rs:701` `onpointermove`) is the wrong kind of Rust handler on desktop, even though it is fine on web.

### 1.3 How Rust state reaches the page: render, reconcile, and command only for queries

Four forms, in order of preference. The construction forbids the later ones where an earlier one fits.

1. **Render it.** Anything with an attribute or property spelling (`data-state`, `aria-*`, `style`, `checked`) goes out in the renderer's edit stream. That stream is atomic and ordered with the DOM (F3).
2. **Desired-state attribute + page reconciler** for platform state that has no attribute (open as popover or modal, `indeterminate`, `defaultChecked`, scroll position, focus target):
   - Rust renders the *desired* state (`data-dx-want-open="true"`). One page runtime reacts in the same task as the edit, compares desired with actual (`:popover-open`, `dialog.open`) and acts idempotently.
   - It knows which changes it made itself, synchronously, so it reports to Rust only the divergences the browser caused (light dismiss, Escape). The row-115 echo cannot exist.
   - "Reopen during the exit animation" is a desired-state flip seen by the same controller, so no generation counter is needed.
   - A one-shot intent (scroll item 7 into view, focus the trigger) becomes a sequence-numbered request attribute (`data-dx-reveal="12:7:center"`). It is processed once and ordered with the render that mounted its target. That retires the "selector, 30 tries, up to 20 frames" retry loops of W3/W5/W7 for the mounted case.
3. **Facts from the page, over one channel.** The page coalesces and deduplicates (the `SCROLL_BRIDGE_JS` pattern) and sends typed facts keyed by a component key. Rust routes each fact to the hook that registered that key. These are inputs to Rust state, never instructions. This one channel replaces N per-hook evals (46 `JsListeners::install` sites today). On desktop it is also cheaper than Dioxus's own events: an eval channel message is asynchronous, a Dioxus event is a sync XHR (F2).
4. **Command over the channel** only for a genuine request/response query whose answer Rust needs (measure now, read the clipboard), or a one-shot payload too large for an attribute. Never for a state.

**Events versus commands.** Rust → page is *state* (forms 1-2). Page → Rust is *facts* (form 3). Imperative commands are the exception (form 4). The reason, by construction: a command duplicates the state, and its ordering against the DOM holds only on web (F2, F3). A published desired state has a single owner and is ordered on every target.

### 1.4 The page side, its lifetime and its enforcement

- **Behaviours bind to elements, not to scopes.** An element that needs one carries `data-dx-b="dialog popover-anchor ..."`. The runtime attaches a controller when the element connects and detaches it when the element disconnects. Two candidate mechanisms:
  - (I) one page-lifetime MutationObserver on the app root (childList + an `attributeFilter` of the `data-dx-*` behaviour attributes);
  - (I) an autonomous custom-element marker child, whose callbacks are synchronous CEReactions.

  Either way the lifetime follows the element, and nothing is targeted by id.
- The runtime is loaded once, in the head, from one generated `.js`, so behaviour keyed off the SSG markup also works **before hydration**: an L1 bonus that reduces the `gotoHydrated` "click dropped before hydration" class.
- The **in-event state problem on desktop** stays even with publication. The attributes of the state an event just changed arrive at the next rAF. A second event in the same burst (key repeat, a pointerup right after its pointerdown) sees the old state. So a transition that a following event of the same interaction depends on must be **owned locally**: the page behaviour flips it optimistically and Rust is reconciled after. "Decide locally" therefore sometimes means "transition locally". (I, from F2.)
- **Enforcement by sinks:**
  - `document::eval` appears only in the bridge module;
  - `addEventListener` appears only in the runtime;
  - JS exists only in `.ts` files;
  - a gate checks that every `data-dx-b` name rendered by Rust has a registered controller and vice versa;
  - a volume ratchet records the JS line count.
- **Blitz** has no eval, no popover and no dialog. The L0 markup is rendered only under the `web` feature (already the case, `popover_sync.rs:69-96`), and L2 behaviours are absent there.

---

## 2. Comparison with the amended plan

Read after section 1 was written:
- the root doc (sections 1-5);
- the residue doc;
- the row 434 "Step 3 adopted" section;
- both syntheses and the round-2 `blank-slate`, `core-construct` and `platform-desktop` reports, so as not to repeat them.

### 2.1 Where the two designs converge

| element | my design (section 1) | plan | verdict |
|---|---|---|---|
| root | decide where both inputs are present; strictest target decides (F2) | "never ask Rust at the moment of commit; publish before, decide locally" (root 1.1) | converges |
| web microtask fact (F1) | Rust→DOM is pre-paint on web, but irrelevant because desktop decides | same, labelled "not a contract" (root 1.2, C4) | converges |
| placement | L0 declarative → L1 pre-wasm → L2 page / L2' rsx (prevent only, human rate) → L3 Rust | deadline table (root 2.2:242-248) + delete/rsx/A/R/G/B order (5.1) | converges. The plan is more Rust-first: "next frame" goes to R under a measured budget where I had the page. The owner's bar favours the plan, and the budget plus a `place`/behaviour exit makes it falsifiable |
| Rust→page | render, then a desired-state attribute plus a reconciler, then commands only for queries | rsx/attributes; `reconcile{kind}` for state mirrors; one-shot ops as bridge calls (2.2:252-258) | converges for state. Diverges for timing-critical one-shots and for attach (finding 2) |
| page→Rust | typed facts, coalesced page-side, one channel | bridge `listen`/`observe`/`sample` with reducers, latest-wins, one in flight (5.2) | converges. The plan is better specified (back-pressure). **Neither design orders facts against Dioxus events** (finding 1) |
| hot events | F2/F4: never `onscroll`/`onpointermove`/`onwheel`/per-row `onresize` in rsx | 1.4 item 6, 5.2, `check-hot-events`, lane 3.HV | converges. The plan is stronger (counts per event name, `onmounted`) |
| A first, measured fallback | L0 with the outcome measured (`anchor_fallback.rs:20-36`) | A behind floors plus a boot probe, a Rust verification step (2.1) | converges. The plan is more careful (floors) |
| enforcement | eval and `addEventListener` sinks, `.ts` only, name parity, ratchet | 5.3 items 1-11 | converges. The plan is far stronger |
| pre-wasm | generated from the runtime's TS | generated from one `.ts`, identity gates become generated-file checks (section 4) | converges |
| G (paint-safe gate) | not in my design: fallback placement page-side (L2) | Rust behind an invisible pending state, `place` reserved | acceptable (attacks that failed) |
| lifetime | bound to the element (attach on connect) | "the owner's" Rust scope, with an `isConnected` backstop (2.2:291) | diverges, DETAIL (finding 2) |
| published state staleness | "a transition the next same-burst event depends on is owned locally" (1.4) | not stated; `stickToEnd` follows the Rust-published mode | diverges, DETAIL (finding 3) |

### 2.2 Findings

#### Finding 1. DETAIL, major: the placement rule judges a bridge placement only by deadline; on desktop page facts and Dioxus events are unordered, so a Rust decision combining them is order-dependent

**Claim attacked.** Root 2.2:248 (row "none" → R: "Rust over the bridge"), 3.1:374 (L3/L7/L8/L9 "No cancel, so async is fine"), C3 → R (3:360). The root treats lateness, not order. The reorder is named in the timing table (1.4:136: "a streamed `pointermove` can be processed after the `pointerup` that followed it") and in 5.6:561 only as something the desktop-cadence harness must model. No construction prevents it.

**Evidence.**
- (V) The two inbound paths on desktop:
  - A Dioxus event is a synchronous request handled inline: `handle_html_event` calls `runtime.handle_event` at once (`dioxus-desktop-0.7.9/src/webview.rs:191`, routed at `protocol.rs:62-63`).
  - A bridge fact (`dioxus.send`) is posted to the event loop (`webview.rs:315`, `proxy.send_event(UserWindowEvent::Ipc ..)`) and handed to the channel by `handle_query_msg` (`app.rs:318`). The task that consumes it runs only inside `poll_vdom`, which returns early while a render is unacknowledged (`webview.rs:564`).

  So a fact posted in an earlier page task can be applied after a Dioxus event from a later task: up to one render acknowledgement (about a frame) later, plus any Rust busy time.
- (I, from js-sys `queue.rs:65` and the HTML event loop) On web the fact's handler runs in the microtasks of its own task, before any later event. The two renderers therefore resolve the same input sequence differently, and a Chromium-only CI cannot see it (the plan's own 5.6 point).
- (V) Instances in the plan's mapping, each a Rust decision taken relative to state that bridge facts update:
  - The carousel's next and previous buttons step from Rust's `selected`: `nav.rs:111` `set_selected(step_next((ctx.selected)(), ..))`. That `selected` is written by the scroll-tracking facts (`content.rs:469-481` → `use_carousel_scroll_tracking`, C3 → R). A tap on "next" right after a fling settles steps from the pre-settle index on desktop when the settle fact is still queued.
  - The slider's window pointer bridge (L21 → R) streams the drag's last moves. An arrow key pressed within the frame is handled inline from the pre-move value, and the queued moves then overwrite the step.
  - `hotkey{Escape}` sends a fact. An Enter typed immediately after it is an inline rsx keydown that still sees the layer open.
- (I) The window is short, about a frame or the length of a busy Rust render, but it is a correctness class rather than a lag. It grows exactly when Rust is busiest (token streaming re-renders every frame).

**Concrete change.** Add an ordering condition to the R row, in the same form round 2 added "every field the decision reads" to the rsx row:

> R holds only if every Rust decision that reads fact-fed state is commutative with the Dioxus events around it: an absolute set, idempotent, never "toggle" or "step from current".

A non-commutative combination is decided where both inputs are ordered:
- (a) A: for open/close toggles, the invoker relationship (`commandfor`/`popovertarget`). HTML's light dismiss treats the invoker as inside its popover (W: https://html.spec.whatwg.org/multipage/popover.html#nearest-inclusive-target-popover).
- (b) The owning behaviour, which sees both events in DOM order. The carousel step becomes "step from the settled index the page knows", sent as an absolute target.
- (c) Only if neither fits: deliver the low-rate fact on the vdom's own ordered path. That is an `input` event on a hidden port element whose value carries the typed fact: `FormData.value` is serialised on both renderers (`dioxus-interpreter-js-0.7.9/src/ts/serialize.ts:183-191`; `webview.rs:125-128` `DesktopFormData { value }`). It costs one blocking request per fact on desktop, so it is never for streams.

The desktop-cadence harness (5.6 step 1 already reorders "not across channels") gets one spec per instance above.

**What it subsumes.** The facts-versus-events reorder class on every R mapping (L3, L7-L9, L21, L30, C3, W1, O1's F6 cycle), and the web/desktop divergence on fast succession.

**Not subsumed.** Pure streams whose consumers are latest-wins (W1's windowing is absolute).

**Why DETAIL and not DESIGN.** It is a condition on one row of the existing rule and a mapping change for a handful of decisions. Option (c) would add an inbound transport, but (a) and (b) suffice and stay inside the construct (round 2 classified the analogous rsx-row condition as DETAIL).

**Attack that failed inside this finding.** The re-click of an open `popover=auto`'s trigger (`popover.rs:995` toggles from Rust state). Light dismiss acts at pointerup, and the click is dispatched in the same task. Both renderers see the click before the reconciler's report, so the outcome is the same.

#### Finding 2. DETAIL, major: "an overlay open saves a frame" is false for the overlays that mount on open; attach element-bound behaviours on connect, and derive `render` synchronously

**Claim attacked.** Root 2.2:257 ("On desktop a MutationObserver callback ... runs before paint in the frame the edit lands, so an overlay open saves a frame"), repeated in row 434 and the round-2 synthesis. The reconciler "reconciles at attach" (2.2:255), and attach belongs to the owning Rust scope ("Lifetime is the owner's", 2.2:291; round-2 blank-slate: "attached by the owning scope like every behaviour").

**Evidence.**
- (V) Overlay content is rendered only `if render()`. `use_animated_open` sets `show_in_dom` inside an effect (`lib.rs:1074-1080`), so the content mounts one render after `open` flips. The sync hooks live in the mounted leaf: `popover.rs:388,402-411` (`PopoverContentRendered`), and `use_animated_open` is likewise used by `dialog.rs:258`, `tooltip.rs:371` and `dropdown_menu.rs:1872`. The audit lists the same hook for Select, Combobox, Menubar, Navbar, NavigationMenu, ContextMenu, AlertDialog, Accordion, Drawer and Listbox.
- (I, from the plan's own 1.4 V chain) The desktop open is:
  1. click (sync) → render 1 (`open`) in rAF N → acknowledgement;
  2. effect sets `show_in_dom` → render 2 (mount) in rAF N+1 → acknowledgement;
  3. effect → bridge attach → `evaluate_script` → the reconciler attaches → `showPopover` in a later task: frame N+2 at best.

  The MutationObserver path gains nothing here, because the element did not exist when the attribute changed. Its same-frame property serves only elements kept mounted while closed, which no default open path does.
- (V) Attach by the scope also means:
  - one bridge install per instance, at mount, for every S8/S9 checkbox and switch that takes `reconcile{indeterminate|default-checked}`;
  - the `isConnected` backstop and the 200-mount leak spec (2.2:291), which exist because the lifetime is not the element's.

**Concrete change.**
- (i) Rust only: `render` = `open || exiting`, as a memo. The effect handles only the exit wait. This saves one desktop frame per open with no JS.
- (ii) Element-bound behaviours (`reconcile`, the `pan` and `settle` targets) attach **on connect**: the element carries `data-dx-b="<kind>"` and its config attributes. One page-lifetime childList observer on the app root, restricted to added nodes that match `[data-dx-b]`, attaches inside the task that applies the edits. On desktop that is the rAF callback (`native.ts:445-457`), so the open lands in frame N, or N+1 without (i).
  - Window or document behaviours (`hotkey`, `scrollLock`, `dropAnywhere`) stay scope-owned and ref-counted as planned.
  - Round 2 rejected document-wide discovery for its scan cost without measuring it. `bridge-cost` measures the observer with a virtual list scrolling, and the scope-attach path is the fallback if it costs more than the budget.
- (iii) A timing-critical one-shot intent rides the edit stream as a sequence-numbered attribute (`data-dx-reveal="12:7:center"`) instead of a bridge call. It then has a level (the last sequence number applied), so "one-shot commands have no level to reconcile" (2.2:258) stops being true. It also saves the acknowledgement plus IPC hop for keyboard grid focus and reveal (W7) on desktop.

**What it subsumes.**
- The frame claim becomes true for every overlay.
- The per-instance install traffic.
- The leak class for element-bound behaviours: lifetime equals the element, by construction.
- Round 1's #5919 "runtime loads late" case, which still converges because attach is level-triggered.

**Why DETAIL.** The kinds, the reconciler semantics and the bridge are unchanged. This changes the attach mechanism, a hook's render derivation and a timing claim.

#### Finding 3. DETAIL, major: published state is stale inside an interaction burst on desktop; a behaviour must own the transitions its own events cause (`stickToEnd` fights the user's scroll)

**Claim attacked.** The registry row `stickToEnd` (2.2:280): "While the Rust-published `data-mode` is `following-bottom`, the callback sets `scrollTop = scrollHeight` and sends the mode transitions to Rust". The residue doc (:15, :62) says the same, and at :113 it accepts that "a late ... release delays the ... stop of following by a frame".

**Evidence.**
- (V) Desktop delivers the user's intent to Rust only through a bridge fact. That fact waits while a render is unacknowledged (`webview.rs:564`), and the republished `data-mode` comes back in a later rAF (`native.ts:445-457`).
- (I) During token streaming Rust re-renders on every growth, so the release waits at least 1-2 frames plus render time. Meanwhile every ResizeObserver growth runs `scrollTop = scrollHeight`, undoing the user's upward wheel or touch scroll at token rate. That is the familiar "cannot scroll up while it streams" defect, and it is worst exactly on the engines that take this path (Safari 26 and older WKWebView, residue :33).
- (I) The same class elsewhere:
  - `reconcile` keeps desired = open after a browser-initiated close until Rust republishes. Any reconcile pass in that window (a re-attach after a keyed remount, a change of another filtered attribute) re-opens the dismissed panel.
  - Roving focus (finding 4).

**Concrete change.** A rule under "How a behaviour is built" (2.2): **published state is a floor, not the truth.**
- A behaviour applies at once, in its own per-install memory, every transition its own events cause, and reports it.
- It accepts Rust's published value only when that value's epoch (the existing install epoch, 2.2:290) is newer than the local transition.

Applied:
- `stickToEnd` flips to free locally on user intent: wheel or touch toward the start, scroll keys, or a `scrollTop` decrease it did not write itself. Rust re-arms it through the attribute.
- `reconcile` records the epoch it saw dismissed and never re-applies that epoch.

Vectors: "growth after wheel-up with Rust stalled" (at `?dx-bridge-delay=never`, the stuck setting of 5.6) must not move `scrollTop`.

**What it subsumes.** The scroll-jail risk of W8's highest-risk port, the reconciler's zombie reopen, and the general case of a decision reading its own stale echo.

**Why DETAIL.** The root ("decide locally") already permits it: per-install memory and epochs exist. This changes one behaviour's specification and adds a construction rule plus vectors.

#### Finding 4. DETAIL, major: the roving-focus audit will find key repeat everywhere; cost it now, and the Rust-only construction for composites is `aria-activedescendant`

**Claim attacked.** 3.1:390: "a focus after human-rate input stays `set_focus`. Lane 3.OV audits the sites ... no new behaviour unless the audit finds a key-repeat case". The deadline row "this event's task ... never rsx `set_focus` on desktop" (2.2:245).

**Evidence.**
- (V) Every roving widget is arrow-key driven, and arrow keys auto-repeat, so the audit's condition holds for all of them. 78 `set_focus` call sites sit in 32 primitives files (grep).
- (V) `tabs.rs:389` `onfocus: ctx.focus.set_focus(Some(index()))` writes DOM focus back into Rust.
- (I) With desktop's async `set_focus` (plan V, `element.rs:137-160`), a late focus event for item k+1 that arrives after Rust moved to k+2 resets the index to k+1, so a key-repeat step is lost.
- (V, I) A native Enter or Space then activates the DOM-focused (stale) button, whose own `onclick` selects its own value (`tabs.rs:381-386`).

**Concrete change.**
- For composites whose ARIA role supports it (listbox, menu, menubar, grid, tree, radiogroup, tablist, combobox popup; W: https://www.w3.org/TR/wai-aria-1.2/#aria-activedescendant): keep DOM focus on the container and render the active item as `aria-activedescendant` plus `data-highlighted`, through the edit stream.
  - Key handling stays rsx on the container (`prevent_default` only). Scrolling the active item into view is next-frame R.
  - Zero JS, Rust-only, and the focus-routing deadline disappears because focus never moves.
- Where roving tabindex must stay (Radix parity for tabs or radio, an owner call, since screen-reader support for `aria-activedescendant` differs), add one `roving` behaviour that moves focus in the keydown's task.
- Either way, put the 32 files into the lane plan and the estimate now. 3.OV's 23-29 h does not contain them.

**What it subsumes.** The whole `set_focus`-on-key class: the stale activation and the write-back regression.

**Why DETAIL.** It is a mapping and an estimate. The rule row is right; its audit condition is vacuous.

#### Finding 5. DETAIL, minor: the gated open has two show writers

**Claim attacked.**
- 2.2:256: the reconciler "is the **only** caller of show/hide for its element".
- 3.1:386 (T5/T7): "a Rust-initiated measured open is the `show` verb plus the rects in one transaction (G)".
- 2.3 item 1: "the reconciler applies opens through the same function".

**Evidence (V, plan text).** A gated element has both a published desired state (the reconciler) and a `show` verb issued in a transaction. Whichever runs second finds the panel already open. The reconciler's echo bookkeeping ("suppresses the toggle events it caused") does not cover the verb's show, which is the two-writer class the command kind was introduced to remove.

**Concrete change.**
- Remove the `show` verb for elements a reconciler owns.
- The reconciler's open step mints the generation, clears pending, shows, reads the rects in the same turn and reports them with the open. That is one transaction by construction, and one writer.
- Add a `FORBIDDEN_VERBS` entry for `showPopover`/`showModal` on reconciled kinds.

**What it subsumes.** The separate G transaction and its ordering against the attribute.

### 2.3 Attacks that failed

- **"Rust is a frame late on web."** False on web (F1). The plan does not rely on it either way, and designs for desktop. No finding.
- **Custom elements as behaviour hosts.** Synchronous callbacks, but they need wrapper or marker elements (no customised built-ins in Safari), cannot sit in void elements, and break `>` selectors and `:empty`. The plan's rejection holds; finding 2 takes the attach-on-connect win without them.
- **Direct `wasm-bindgen` calls on web** (synchronous, no `new Function` per eval, no CSP `unsafe-eval`). This gives two transports and does not exist on desktop. The plan's transport seam already reserves a web-sys backend (5.2, PR #5194). Holds.
- **G versus a page `place` behaviour on fallback engines.** G costs 1-3 hidden frames on desktop and mobile (iOS below 26.2 is a real population). It is bounded, specified (`g-latency`), fails open twice, and `place` is reserved as the exit. Acceptable under the owner's Rust-first bar.
- **"Next frame → R" lag for drags.** It is budgeted by the owner's measurement, with a behaviour exit. Acceptable.
- **Trigger re-click of an open auto popover as a reorder instance.** Fails (finding 1, last paragraph).
- **Right-click elsewhere while a context menu is open.** The new open itself awaits a bridge query (`context_menu.rs:432-438`, `visual_viewport_offset().await`), so the dismiss fact, posted first, wins on both renderers. Not an instance.
- **One eval sink and the ratchet.** I found no route around the sinks of 5.3 that the plan does not already list. Not my lens; no finding.
- **Eval channel slot growth** (`globalThis.__channels` never compacted, `dioxus-web-0.7.9/src/ts/eval.ts`). Moot once one long-lived channel replaces per-call evals (5.2).
