# Board round 4, lens `whole-core` (2026-10-26)

**Verdict: HOLDS WITH CHANGES**

**DESIGN findings: 0, DETAIL findings: 17**

Summary:
- I traced the six scenarios end to end on web and desktop against the Dioxus 0.7.9 sources. None of them breaks a core construct. The root rule, the deadline table and its kinds, admission by effect or witness, the closed bridge with one eval sink, A/G/R, and sinks plus ratchet all hold. **The core has converged**: every finding below changes a contract, a mapping, a gate or a parameter.
- The biggest cluster is **order**. Conditions 1 and 3 and the reconciler's generation are three mechanisms for one class, and there are gaps between them. "An absolute set" is called commutative, but it is not commutative when the slot it sets is one the user also writes (findings 1 and 3). Facts from different sessions are processed in **owning-scope height order**, not arrival order, on both renderers (finding 2, V `scheduler.rs`). The fix generalises the reconciler's generation: facts carry the epoch they observed, and writes to slots the page also writes go through that slot's page owner. This is a contract, not a new construct, so it is DETAIL by round 3's own precedent; it is the one place a reader could argue for DESIGN, and I say why not in finding 1.
- Verified desktop breaks that the plan does not name: **`DataTransfer` setters in rsx handlers are silent no-ops on desktop**, with three live sites (finding 4). **`MountedData` is a second Rust-to-page channel that the FIFO pump does not order** (finding 5). **Moving the dialog's exit wait into Rust keeps an invisible modal blocking input** for 1-3 frames (finding 6).
- Structural: a layer that is a DOM descendant of a layer closing in the same batch paints nothing but stays modal (finding 7). G's page watchdog exists only on reconciler-opened surfaces (finding 8). The closure rule and next-frame admission leave `pan`'s output variants with no default before the owner measures (finding 9). A witness spec is written by the same lane as its behaviour (finding 10).

Evidence labels: **V** = verified in the repo or the pinned crate source, **W** = web or spec source, **I** = inference. Dioxus paths are relative to `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`. The plan is `l434d3/dev-docs/research/sync-js-root-2026-10-26.md` ("root") and `sync-js-residue-2026-10-26.md` ("residue"). The repo is `b28`, and the built bridge is `l30` (lane 3.0).

---

## End-to-end traces (the six scenarios)

Shared desktop path (V):
- **Events.** The root listener (`dioxus-interpreter-js-0.7.9/src/ts/core.ts:109-130`) sends a synchronous `POST __events` (`native.ts:574`). Rust handles it in `webview.rs:99-195` and returns `{preventDefault}` (`native.ts:382-383`).
- **Edits.** A render travels as a WebSocket `Blob`, then `arrayBuffer().then(rafEdits)` (`native.ts:482-483`), and is applied in a `requestAnimationFrame` (`:453`). The ack (`markEditsFinished`, `:495`) arrives after that. `poll_vdom` does not run the vdom while a render is unacknowledged (`webview.rs:564-566`).
- **Facts.** Facts (`dioxus.send`) arrive through the ipc handler (`webview.rs:308-317`) and `handle_query_msg` (`app.rs:309-319`), and go onto each eval's own channel (`query.rs:118-137`), whether or not a render is acknowledged. Each wakes a task that `Session::open` spawned in the **calling component's scope** (`l30 bridge/session.rs:424-426`). Tasks are polled by `ScopeOrder` (height, then id) (`dioxus-core-0.7.9/src/scheduler.rs:106-110, 149-163, 233-283`; `virtual_dom.rs:494-536`).
- **Rust-to-page messages.** Each message is one `evaluate_script`. `MountedData` methods call `evaluate_script` at the moment they are called (`dioxus-desktop-0.7.9/src/query.rs:45-60`, `element.rs:26-160`).

Shared web path (V):
- The handler runs inside the DOM event (`dioxus-web-0.7.9/src/dom.rs:88-116`). Renders go through `render_immediate` and `flush_edits` in the run loop with no rAF gate (`lib.rs:245-264`). Mutation observers fire in the same microtask checkpoint, before paint.

### 1. Nested menu in a dialog in a sheet; keyboard and touch; open and close racing

- **Keyboard (ArrowRight opens the submenu).**
  - Desktop: rsx `onkeydown` → synchronous XHR → Rust mints `open:7` → the render lands in rAF N (`native.ts:453`). The reconciler's MutationObserver sees the inserted panel in the checkpoint after that rAF callback and before paint. It calls `showPopover()`; the panel is a DOM descendant of the parent popover, so the auto stack keeps the parent open. On a fallback engine it is pending, reads its rects, and reports `applied{7, rects}` → ipc → processed after the ack → `data-dx-placed=7` in rAF N+1. That is 2 frames, as the plan says.
  - ArrowLeft inside that window mints `close:8`. The stale `placed:7` is dropped by generation. **Holds.**
- **Escape.** The menu's own rsx handler closes the submenu and calls `prevent_default`, which reaches the native event before document-bubble listeners run (web `dom.rs:112-115`; desktop `native.ts:382-383`). So `hotkey{Escape}` must skip `defaultPrevented`, or it closes a second layer. That is the composition rule's reader case, a parameter.
  - Escape followed by a quick tap outside: the tap light-dismisses the menu natively and the reconciler reports it; the `hotkey` fact sits on a different session. Rust may process the report first (scope-height order, finding 2). "Close the top of stack" is then decided from a stack the page already changed, so a **second layer closes**. This is finding 1's class, and the fix is in findings 1 and 2.
- **Touch: tap the dialog's close button while the submenu is open.** Light dismiss happens on the pointer events and is reported. The click is an rsx event handled inline (`webview.rs:191`), so Rust closes the dialog before it processes the report. The volatile rewrite of `open:7` changes nothing. **Converges.**
- **Racing: a button in the sheet opens a dialog and closes the sheet in one render.** The dialog is the sheet's DOM descendant by the popup-boundary rule. `close()` on the sheet gives it `display: none`, and the modal child then generates no box but keeps the page inert. **Broken in either order** (finding 7).

### 2. A Select inside a scrolled virtual list during a fling

- **Open.** The rsx click mints the command. S1 (item-aligned) needs Rust placement **on every engine**, but the gate selector `html:not([data-dx-anchor="a"]) :popover-open:not([data-dx-placed])` (root 2.3 item 1) arms it only on fallback engines (finding 14).
  - Desktop: the open step's rects are read in rAF N. The fling moves the trigger on the compositor, so Rust's answer in N+1 places the list at the old trigger position, until T12's close-on-scroll fact arrives.
  - T12 is mapped "a late close is harmless" (root 3.1). It is not harmless when the user re-opened in between: the reopen click is handled inline, and then the stale scroll fact closes the new open. T12 is a fact-derived write on a page-owned slot (finding 1).
- **Window churn.** Every mounted row carries the `data-dx-indeterminate`/`data-dx-default-checked` commands (S8, S9). Volatile attributes add one record per row per render on top of `childList` (finding 13). The row that holds an open Select unmounts with it. **Converges.**

### 3. A toast swipe while a new toast mounts

- `pan` writes `--toast-swipe-x` with `style.setProperty`. A Rust re-render of a full `style` string keeps properties it does not name (`dioxus-interpreter-js-0.7.9/src/ts/set_attribute.ts:67-83`, V).
- The list is keyed by `toast.id` and rendered newest first (`b28 primitives/src/toast.rs:508-518`). A keyed insert at the front creates a new node and moves no existing one, so pointer capture is not released.
- The new toast is pending (`li:not([data-sized])`, O3). Its fail-open is CSS only, because no reconciler open step arms a watchdog for it (finding 8).
- The fly-out then `send(true)`. Rust marks the toast closing; the other toasts restack.
- **Holds**, except finding 8.

### 4. A chat stream with the user scrolling up

- **`stickToEnd` engines (Safari 26 and older).** The wheel listener detaches locally before that frame's ResizeObserver runs. Rust mirrors the detach and re-arms with a newer epoch. But the Rust invariant's own `scroll(end)` backstop write is a bridge verb, not a value published to the behaviour, so the epoch does not filter it. An invariant write in flight lands after the wheel-up.
- **A engines (Chromium, so WebView2 and Android WebView, plus Firefox and Safari 27).** There is no `stickToEnd` (root registry row; residue 2.2), so the release is owned by Rust. The invariant is the follower exactly when A fails: offset 0 on the first overflow, a fractional DPR such as Windows at 125%, and anchoring suppression after a turn starts (residue 2.2 item 1). There it follows 1-2 frames late on desktop, which residue 2.2 calls unacceptable. A wheel-up during those frames is fought, against residue's "never pulled back" and the "no fight frames" pass criterion (findings 1 and 3).

### 5. RTL plus a reduced-motion carousel

- **Reduced motion.** C2's transaction reads `matchMedia` synchronously. `use_media_query` defaults to "reduce", and the G delay is untouched. **Holds.**
- **RTL.** C3's scroll facts and W2's writes take the sign from `use_direction`. That is a Rust context with an LTR default (`b28 primitives/src/direction.rs:190-197`), not the scroller's computed direction. If an app sets `dir` without a `DirectionProvider`, Rust's index and delta are mirrored (finding 16).
- **Next after a fling.** This is the plan's named instance. Remedy (b) needs a page-side program, and admission has no ground for order (finding 11). Remedy (c) works.
- **Rest event.** `pan`'s `dx-carousel-rest` report and C3's facts are two sessions in two scopes, so the shallower scope is processed first (finding 2).

### 6. SSG first paint, then hydration while the user taps

- Before hydration a tap is dropped (row 109). After hydration, a tap before the bridge exists mints `open:1`.
  - Web: the first verb's effect evaluates the bundle synchronously (`l30 bridge/transport.rs:173-180`, eval body synchronous on web). The install-time level read applies the open before paint.
  - Desktop: one frame later.
- The boot probe runs at that first install, so it sits **on the first open's critical path**. If it uses `popover=auto` test elements, it would close an auto popover that is already open (finding 17).
- The generation must match between SSR and the client (hydration-parity). **Converges.**

---

## Findings

### 1. DETAIL, major. Order has three mechanisms and a gap: an absolute set on a page-written slot is not commutative, and fact-derived writes are not fenced

- **Claim attacked.** Root 2.2 condition 1: "commutative ... an absolute set, idempotent". Condition 3: page-owned transitions with epoch acceptance, scoped to "a behaviour". Residue 2.2: "Release is not late ... never pulled back".
- **Evidence.**
  - I: `set(Rust); set(user)` is not `set(user); set(Rust)`. An absolute set commutes only with events that do not write the same slot.
  - The slots the user also writes are scroll position, open state, focus and value.
  - Instances the plan does not list:
    - the W8 invariant's `scroll(end)` (trace 4);
    - T12 close-on-scroll after a reopen (trace 2);
    - Escape's "close the top of stack" after a light dismiss (trace 1);
    - S4 and V6 `scrollTop` setter rows after an asynchronous `get` while the user wheels.
  - The plan has the mechanism for exactly one slot: the reconciler's generation on open state (root 2.2 "Volatile attributes ... gen"). Condition 3's epoch covers only a behaviour's own memory. The bridge `set`/`call`/`transaction` verbs carry no basis (`l30 bridge/transport.rs` `fire`/`ask` send `{id, ops}` only, V).
- **Change** (one contract, three parts):
  - (a) **Facts carry the epoch of the slot they observed**, read on the page at emission. For example, T12's scroll fact names the command generation it saw; `hotkey{Escape}` names the target layer, resolved on the page from the top layer.
  - (b) **A Rust write to a page-written slot goes through that slot's page owner**: the reconciler for open state, `openFocus`/`roving` for focus, the scroller's behaviour or a fenced setter row for scroll position. It carries the epoch it was decided from, and the owner drops a write older than the slot's last page transition.
  - (c) Condition 1's definition becomes "an absolute set on a slot only Rust writes".
- **Subsumes:** the reconciler's "diverged at gen g", `stickToEnd`'s epoch, the W8 fight frame, T12, Escape, S4 and V6. **Does not subsume:** the order inside Rust between a Dioxus event and a fact on a Rust-only slot (the slider's arrow key against queued pointer moves); remedy (c) stays for that.
- **Why DETAIL.** No construct is replaced or added. It states the scope of the existing generation and epoch, and round 3 folded conditions 1 and 3 in as contracts. If the owner counts the conditions' mechanism as core, this is the only candidate for DESIGN in my lens.

### 2. DETAIL, major. Facts from different sessions are processed in owning-scope height order, so "both sides on the bridge" is not ordered

- **Claim attacked.** Root 5.2: "a stream that must order against vdom events carries a sequence number or has both sides on the bridge"; root 1.4 item 8; the harness's "in arrival order per channel".
- **Evidence.**
  - V: each session's receive task is `spawn`ed in the caller's scope (`l30 session.rs:424-426`).
  - V: wakeups are queued as `TaskNotified` and polled by `pop_task` in `ScopeOrder` (height, then id), with FIFO only inside one scope's `VecDeque` (`dioxus-core-0.7.9/src/scheduler.rs:106-110, 149-163, 247-258`; `virtual_dom.rs:445-536`).
  - V (desktop): every fact that arrives while a render is unacknowledged is delivered to its channel (`query.rs:118-137`) and polled together after the ack (`webview.rs:564-566`).
  - I (web): two facts sent in one page task land in one batch the same way.
  - So a report from the root-owned reconciler always runs before a component's `hotkey` or `listen` fact in the same batch, and C3 (carousel root) runs before `pan`'s report (content scope).
- **Change.**
  - Add this row to the timing contract (1.4).
  - Rewrite "both sides on the bridge" as "both sides on one session, or in one scope".
  - Facts that a decision combines share one session. Alternatively, every fact goes through one inbound pump with a page-minted sequence number, dispatched by one root task. That is the plan's own named alternative for Rust-to-page ("carrying session commands as pump messages"), applied to page-to-Rust.
  - The desktop-cadence harness reorders across sessions, not only against events.
- **Subsumes:** cross-session reorders of every R pair.

### 3. DETAIL, major. W8 picks `stickToEnd` by engine, but A's failures depend on state, so Chromium desktop follows late and releases late

- **Claim attacked.** Residue 2.2 and the root registry row: `stickToEnd` runs on "engines without verified `overflow-anchor`".
- **Evidence.**
  - Residue 2.2 item 1 lists A's failures: no anchor at offset 0, a permanent slip at fractional DPR or zoom, and suppression on a computed-size change. They depend on state, not engine.
  - On an engine with A verified, the invariant (a bridge round trip, "1-2 frames per growth step" on desktop, residue 2.2) is the only follower in those states, and the detach is owned by Rust. WebView2 at 125% (a Windows default) is in that class (I).
- **Change.**
  - `stickToEnd` runs on every engine as the one same-frame writer. It is idempotent with A, because both target the bottom.
  - The invariant's corrective writes go through it, so they carry the epoch (finding 1(b)).
  - The page-owned detach then exists everywhere. The probe selects nothing for W8.
- **Subsumes:** trace 4 on A engines.

### 4. DETAIL, major. Commit-time effects other than `prevent_default` in rsx handlers are silent no-ops on desktop, and three ship today

- **Claim attacked.** Root 1.4 item 4: "An rsx handler takes exactly one commit-time effect, `prevent_default`". The rule is stated, but no gate enforces it.
- **Evidence.**
  - V: desktop drag events carry a `SerializedDataTransfer` whose `set_data`, `clear_data`, `set_effect_allowed` and `set_drop_effect` do nothing and return `Ok` (`dioxus-html-0.7.9/src/data_transfer.rs:116-146`; `events/drag.rs:142`; `dioxus-desktop-0.7.9/src/file_upload.rs:348`).
  - Live call sites:
    - `b28 primitives/src/drag_and_drop_list.rs:860-865`: `set_effect_allowed("move")`, `set_drop_effect("move")`, and `set_data("text/html", "")` with the comment "only for Firefox (without it, DnD won't work)";
    - `preview/src/components/data_table/reorder.rs:363-368`;
    - `preview/src/components/virtual_list/variants/reorder/mod.rs:144-149`.
  - The class-(1) inventory (root 1.3) lists only `preventDefault` sites, so it never saw these.
- **Change.**
  - Clippy `disallowed_methods` on the `DataTransfer` setters, by def path (the `check-hot-events` mechanism).
  - The drag source's `dataTransfer` write becomes an effect of a named behaviour, admitted by its detected effect (a `dragSource` half beside `dropAnywhere`).
  - The rule's inventory covers every setter on event data, not only `preventDefault`.
- **Subsumes:** the whole class of setters on event data reached from rsx.

### 5. DETAIL, major. `MountedData` is a second Rust-to-page channel that the FIFO pump does not order

- **Claim attacked.** Root 5.2: "Ordering of element operations is a property of the transport". `js-interop.md:46-47` recommends "`MountedData` where it suffices" next to bridge verbs.
- **Evidence.**
  - V: desktop `MountedData` methods call `evaluate_script` at the moment they are called (`query.rs:45-60` via `element.rs:26-160`). On web they are synchronous at the call.
  - Pump operations are queued and sent from an after-render root effect (`l30 transport.rs` `enqueue`, then `after_render`, then `flush`).
  - So `bridge::set(scrollTop)` followed by `mounted.get_scroll_offset()` reads the old value on **both** renderers.
  - Lanes 2B and 2C moved S3, S19, L17, L19, S5, S6 and V15 onto `MountedData` (row 434 progress notes).
- **Change.** Either the element operations `MountedData` offers (`scroll_to`, `scroll`, `set_focus`, `get_*`) become pump verbs, and clippy bans the `MountedData` methods outside `bridge/`, or a gate forbids mixing the two on one element. The first makes the FIFO claim true by construction.
- **Subsumes:** read-after-write and write-order races between the two channels.

### 6. DETAIL, major. L5 moves the dialog's exit wait into Rust, so an invisible modal blocks input on desktop

- **Claim attacked.** Root 3.1 L5: "bridge `call` (`getAnimations().finished`) with a Rust generation | Async by nature", that is, the "none" deadline.
- **Evidence.**
  - V: today the dialog stays a real modal until its exit animations settle, then calls `close()` in the same script (`b28 lib.rs:621-648, 716-720`).
  - Under L5 the "finished" fact goes over ipc, waits for the ack, Rust issues the close command, and the render applies it in the next rAF. That is 1-3 frames (I) in which the panel and `::backdrop` sit at their final, invisible keyframe while the page is still inert. A click in that window is swallowed.
  - The deadline is the next input (input routing), not "none".
- **Change.** The reconciler's close step owns the exit: Rust's `close:g` starts the closing state, the page waits for the animations and then calls `close()`, and Rust's generation fences a reopen. This is condition 3 applied to the end of the close.
- **Subsumes:** `__dxDialogGen`/`__dxClosing` (L1) and every animated exit of a reconciled kind.

### 7. DETAIL, major. A layer that is a DOM descendant of a layer closing in the same batch paints nothing but stays modal

- **Claim attacked.** Root 2.2: "One reconciler per element, in a fixed order (closes before opens ...)".
- **Evidence.**
  - V: CLAUDE.md's popup-boundary rule says "a popover or dialog stays a DOM descendant of its trigger".
  - I: a closed popover or dialog is `display: none`, and `display: none` suppresses box generation for descendants that are in the top layer. A modal `<dialog>` under it is shown and inert-making but invisible. Not re-fetched: CSS Position 4, top layer.
  - Patterns that hit it: "open a dialog from a sheet button and close the sheet"; "a menu item opens an AlertDialog and the menu closes".
  - The order is per element, and either cross-element order fails.
- **Change.**
  - A Rust layer-tree invariant: closing a layer cascades to its descendant layers, and no layer opens under an ancestor that is closing.
  - The reconciler orders one record batch by tree: closes deepest first, opens shallowest first.
  - A spec for each of the two patterns.
- **Subsumes:** the race of nested opens and closes.

### 8. DETAIL, major. G's page watchdog is armed only by the reconciler, so O3, V19 and W8's opening position fail open in CSS alone

- **Claim attacked.** Root 2.3 item 3: "the reconciler's open step also arms a page-side watchdog".
- **Evidence.**
  - V/plan: O3 (`li:not([data-sized])`), V19 (height pin) and W8's opening position are G surfaces that no reconciler opens.
  - Root 2.3 item 3 itself cites the theme's `animation: none !important` and common user resets.
  - A desktop stall after a rAF exception never acknowledges (root 1.4). A toast's `role="alert"` is then announced but never visible.
- **Change.** The watchdog belongs to the gate. The reconciler's `childList` scan arms it for any node that enters a declared pending state; each G surface in 3.PL's list declares a pending attribute.

### 9. DETAIL, major. The closure rule plus next-frame admission leave `pan`'s output variants with no default before the owner measures

- **Claim attacked.**
  - Root 2.2 condition 2: per-move output is "a next-frame decision placed by that row".
  - Admission ground 2: for next frame, the witness is the owner's `fail` verdict, and "if the spec stays green ... the decision is R".
  - Registry `pan`: effects "`setVar` or `scrollBy` per output".
  - Section 6: "keep their placements".
- **Evidence.** The text contradicts itself. The `css-var`, `scroll-by-with-refusal` and `scroll-absolute` variants are next-frame effects with no verdict, so by admission they cannot be added, and O5, C4 and V17 would become a Rust round trip per frame on desktop. Section 6 says the current placements stay. C5 takes the opposite default (Rust is the attempt, and the legacy arm stays until measured).
- **Change.** One stated rule: absent a verdict, an existing page-side next-frame output keeps its placement under a `pending` witness record bound to row 443. Only a `pass` moves it to R, and only a `fail` admits a new one. This is symmetric with C5.

### 10. DETAIL, major. The witness spec is written by the same lane, and `reconcile` is admitted by lateness for a reason that is convergence

- **Claim attacked.** Admission ground 2: "the behaviour's spec tag must go red in that mode".
- **Evidence.**
  - I: a lane can write a spec that asserts "panel visible in frame N", which goes red under any delay. The witness proves that some spec detects lateness, not that lateness harms the user. The only closure is the human `adds:` row.
  - `reconcile` "passes only by a witness" (`sameFrameWrite`), yet it exists for convergence: lost or early commands, the toggle echo, the two-writer trap (root 2.2). If the owner's budget makes `g-latency` tolerate one more frame, the command kind loses its admission.
- **Change.**
  - The witness spec must exist at the merge base and be green against today's code, or be named in the `adds:` row ("characterisation first", as `scrollLock` already does).
  - `reconcile` becomes part of the closed bridge, as the command plane: a fixed component like the pump, with a closed enum of kinds. It leaves the behaviour registry and admission.

### 11. DETAIL, minor. The condition 1 remedies need two qualifiers

- **(a)** `popovertarget`/`commandfor` on a trigger also activates the native toggle. That makes the browser a second show writer, against G's "every library open is Rust-driven". The trigger therefore also needs rsx `prevent_default` on `click`. A canceled click skips activation behaviour (W, DOM dispatch, "if event's canceled flag is unset, run activation behavior"), and light dismiss still treats the invoker as inside.
- **(b)** "By the owning behaviour" has no admission ground when no behaviour exists, because admission knows effects and lateness, not order. The carousel instance therefore takes (c), or (b) is limited to behaviours that are already admitted.

### 12. DETAIL, minor. L2's bridge `listen(dialog, 'close')` is a second reporter with no generation

- **Claim attacked.** Root 3.1 L2, against 2.2's "Every report carries the generation it answers".
- **Evidence.** The reconciler already reports a browser-initiated dialog close. A `close` fact on a separate session, without a generation, can close a reopen at g+1 and reorders by finding 2.
- **Change.** The dialog's `close` goes only through the reconciler.

### 13. DETAIL, minor. The command attribute value: `volatile` adds nothing to the generation, and the divergence memory is keyed by element

- **Evidence.**
  - V: rsx builds custom attributes with `volatile = false` (`dioxus-rsx-0.7.9/src/attribute.rs:191-196`), so volatile needs a hand-built `Attribute::new(.., true)` spread. Volatile attributes are rewritten on every diff (`dioxus-core-0.7.9/src/diff/node.rs:444-463`).
  - The reconciler "re-applies only for a newer gen", so an equal-value record is a no-op by construction. Volatile costs one record per command per render (virtual-list churn, trace 2) and saves nothing; the veto already works by a generation bump.
  - A panel element that Dioxus re-creates while open (a re-key, a responsive swap) loses the "diverged at g" memory and resurrects a dismissed panel.
- **Change.** Drop volatile. The value carries `(owner, state, gen)`, and the divergence memory is keyed by `owner`.

### 14. DETAIL, minor. The gate predicate is per tier, but some surfaces are always placed by Rust

S1 (item-aligned Select; CSS cannot align by a child's offset) and the iOS keyboard clamp need G on A engines too. The selector `html:not([data-dx-anchor="a"]) ...` misses them. 3.PL's per-surface list declares `tier: always | fallback`, and the selector reads that declaration.

### 15. DETAIL, minor. V14's clipboard copy has no legal home

- **Evidence.** V: the audit row V14 includes `preview/src/components/code_block/component.rs:199`, an installable component (`b28 dev-docs/research/js-audit-2026-10-25.md:202`).
- The plan gives V14 a "preview-only allowlist entry" (root 5.2), and 5.3 item 10 says an installed component contains no sink.
- Clipboard writes are activation-gated (root 1.4 item 4: an effect of a named behaviour only).
- **Change.** A `copyText` behaviour, admitted by its detected activation-gated call; or code_block's copy is declared preview-only and removed from the shipped component.

### 16. DETAIL, minor. The RTL sign for scroll geometry comes from a Rust copy of a page fact

- **Evidence.** V: `use_direction` reads a Rust context with an LTR default (`direction.rs:190-197`), which is Radix parity for keys. C3's index and W2/C2's deltas (root 3.2 W2: "the RTL sign comes from `use_direction`") disagree with the scroller whenever the app sets `dir` itself.
- **Change.** Normalise scroll facts and scroll writes on the page from the scroller's computed direction (W1's reducer already does "RTL normalisation"). Keys stay on `use_direction`.

### 17. DETAIL, minor. The lazy bridge puts the boot probe on the first open's critical path

- The probe runs synchronously at the first install (root 2.1). Its forced layouts (reopen x3, nested, scaled) then land inside the user's first tap.
- If the probe uses `popover=auto`, showing it closes any auto popover that is already open.
- **Change.** Probe with `popover=manual` elements, and warm the bridge from the first idle after hydration.

---

## Attacks that failed

- **Root rule against rsx handlers that read Rust state ahead of the DOM** (desktop routes events to a vdom one render ahead, root 1.4). The handler reads every earlier Dioxus event in order, so its decision matches the user's key sequence. The only same-dispatch signal from a handler to a page behaviour is `defaultPrevented`, and it is applied before document-bubble listeners on both renderers (`dom.rs:112-115`; `native.ts:382-383`). That is enough for Escape and Cmd+B.
- **The reconciler's same-frame claim.** On desktop the `childList` record runs in the checkpoint after the rAF callback that applies the edits (`native.ts:453`), before style and paint. On web it runs in the run loop's microtasks. Both are correct.
- **Open, close, open inside one ack window.** The generation drops the stale `placed`. The page's removal of a vdom-owned attribute is safe because every new value carries a new generation, so Dioxus's equal-value skip (`diff/node.rs:463`) never strands a missing attribute.
- **Toast swipe against a Rust style write.** `set_attribute.ts:67-83` keeps the page's properties. A keyed insert at the front moves no node, so capture holds.
- **Pre-install windows.** `otpFilter` sanitises the current value at install, and the reconciler reads the current state at install. A command that arrives before the bridge converges (trace 6).
- **Sinks.** Clippy's def-path ban catches aliases, function pointers and code `include!`d from `OUT_DIR`. The textual gate is only the backstop. I found no unlisted eval route except `MountedData` (finding 5), which runs JavaScript Dioxus wrote, so it is an ordering concern, not a code-injection one.
- **A/G/R as layers.** No scenario needed a fourth construction. Every break above is solved inside R, G, a behaviour or the reconciler.
