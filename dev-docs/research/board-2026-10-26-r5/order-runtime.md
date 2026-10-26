# Round 5, lens `order-runtime`: the order contract and the timing contract against the Dioxus 0.7.9 runtimes

Verdict: **HOLDS WITH CHANGES**

DESIGN findings: 1, DETAIL findings: 8

- The order contract fences bridge facts with epochs, but Dioxus events carry no epoch. The interpreter serializes a fixed field set, with no dataset and no timestamp, so a decision an rsx handler makes on a page-written slot cannot know which epoch it decided from. Epochs then turn a reorder into a lost or divergent input instead of one outcome (finding 1, DESIGN). The only path that orders a page transition with a later Dioxus event is the vdom's own event path, which is today's remedy (iii). For this class it has to become a first-class carrier of the contract.
- The contract's minting rule contradicts itself, and the as-built `Epoch` (one global Rust counter, keyed per element object) is a third scheme. As written, Rust's second command of a quick open-then-close is dropped (finding 2).
- The main Rust write channel, the vdom, bypasses every slot owner. `value` is volatile and rewritten on every render of its template. `checked`, `selected`, `open` and `hidden="until-found"` diverge with no record. This loses keystrokes on desktop, resurrects text after a form reset on both renderers, and wipes text typed before hydration (finding 3). `MountedData` `set_focus`/`scroll`/`scroll_to` are a second bypass, and the plan's own ledger routes a focus write through one (finding 4).
- "Exactly one page owner per slot" fails for focus (one document-global slot, four planned writers) and for scroll position. A scroll epoch needs an input classifier, and a "fenced setter row" cannot hold one. If `scrollStep` becomes a transaction, the carousel has no owner (finding 5). The order witness cannot see native transitions (finding 6). Condition (d) is not enforceable as written (finding 7).
- Seven fresh scenarios, each traced on web and desktop: combobox typing under async results (breaks on desktop), slider drag under a clamp (holds), accordion revealed by find-in-page while Rust toggles (breaks under the cross-session reorder and the veto), form reset with a focused controlled input (breaks on both), carousel autoplay against a swipe (breaks on both), a light dismiss and an rsx click in one gesture (the renderers diverge), typing before hydration (breaks on web SSG).
- What held: the root rule, the deadline table, the two kinds, the scheduler row (verified), "two desktop renders never share a frame" (verified), the one eval sink, A/G/R, and enforcement by sinks plus ratchet. Every fix below reuses parts that already exist: the command kind, the slot owners, the transition port, clippy sinks.

Labels: V = read in the repo or in the pinned Dioxus 0.7.9 source (`~/.cargo/registry/src/index.crates.io-*/`), W = web source, I = inferred. Plan paths are relative to `/home/user/wt/l434d4/dev-docs/`, the bridge to `/home/user/wt/l30/primitives/src/`, shipped code to `/home/user/wt/b28/`.

---

## Findings

### 1. DESIGN, major. Dioxus events carry no epoch, so the contract's fence cannot order a decision an rsx handler makes on a page-written slot

**Claim attacked.** Root 2.2, condition 1: "no decision may depend on the arrival order of two inputs". Rust "mints `seen + 1`, where `seen` is the highest epoch it has read from a fact or a report". (b) "the write carries the epoch it was decided from". The instance list includes "`hotkey{Escape}` against an Enter typed right after it" and "Escape's 'close the top of stack' after a light dismiss".

**Evidence.**
- V: a Dioxus event's payload is a fixed per-type field set. `serializeMouseEvent` and `serializeKeyboardEvent` contain no timestamp, no dataset and no attribute (`dioxus-interpreter-js-0.7.9/src/ts/serialize.ts:279-313`). `serializeEvent` adds only type-specific fields and form `values` (`:15-119`). No hook lets the page append a field. So the only epoch Rust can attach to a decision taken in an rsx handler is its own `seen`, which comes from bridge facts.
- V: on desktop a bridge fact waits for the event loop, and `poll_vdom` returns early while a render is unacknowledged (`dioxus-desktop-0.7.9/src/webview.rs:308-317, 560-563`). A Dioxus event is handled inline, so `seen` lags the page by up to one acknowledgement plus any Rust busy time. On web the fact's microtask runs before any later UA-dispatched event, so `seen` is current there.
- The shipped code already has the same-gesture pattern. `top_layer/popover_sync.rs:200-225`: the trigger's `pointerdown` light-dismisses the `auto` popover natively, "**immediately followed**, same gesture, by the trigger's own `onclick` reading `open`" (V, b28).
- I, trace (desktop, a render in flight). Light dismiss mints 6 and the report is queued. The rsx `onclick` reads Rust's `open == true` and toggles it, so it writes close with "decided from 5". Then one of two things happens:
  - Under (b) the page drops the write (6 > 5). It was a no-op anyway.
  - The report is processed: closed.
  - On web, Rust saw 6 first, so the same click reopens.

  The same input sequence ends in two different states on the two renderers. For "Escape, then Enter on the refocused trigger": desktop drops the reopen (the Enter is lost), web applies it. The epoch fence decides only *which* outcome loses. It does not make the outcome independent of arrival order, which is what the contract states. The window is one acknowledgement plus Rust busy time. Keyboard repeat, touch taps (about 50 ms from pointerdown to click) and heavy renders sit inside it.
- I: the round-4 trace "tap the dialog's close button while the submenu is open: converges" holds only because that rsx decision is absolute and goes in the same direction as the page's transition. A relative decision (toggle) or an opposite one (reopen) does not converge.

**Concrete change (amends the contract's mechanism).** The contract gets two carriers, one per inbound channel:
- (a') **A page transition on a slot that any rsx-handled decision reads or writes is delivered to Rust on the vdom's ordered path, synchronously at the transition.** It goes as a synthetic `input` event on one hidden **transition port** element that the bridge's root renders once (today's remedy (iii), promoted from last resort). On desktop that is one blocking request per transition. Only low-rate transitions qualify: dismiss, toggle, reveal, reset, detach. Never a stream. Rust's `seen` is then current at every later Dioxus event on both renderers.
- Epochs stay the fence for bridge facts and for Rust-to-page writes.
- A gate rule: a slot is either "rsx-read" (its transitions take the port) or "bridge-only" (facts with epochs). The slot table (finding 5) records which.

The alternative for open/close toggles is the platform: an invoker without `prevent_default`, so the browser toggles from the DOM state the user saw, and the executor reports it as a page transition exactly like a light dismiss. "One show writer" then means one writer of *Rust-originated* shows. The plan's remedy (i) currently forbids this with "with an rsx `prevent_default` on the trigger's `click`".

**Subsumes.** "Escape, then Enter", "Escape after a light dismiss", the same-gesture dismiss-then-click of `popover_sync.rs`, L1/L2-style rsx decisions that read a dismissal, and the round-4 "converges" cases that converge only by luck.

**Why DESIGN.** For this class it replaces the contract's mechanism (an epoch fence) with ordered delivery. The epoch cannot reach the channel that the deadline table's first row uses. If the owner reads this as a scope extension of remedy (iii), it is the one borderline item in this lens.

### 2. DETAIL, major. The epoch rule contradicts itself, and the as-built `Epoch` implements neither half

**Claim attacked.** Root 2.2, "Epoch" bullet: "Rust mints `seen + 1` ... The page applies a Rust write only if its epoch is **strictly newer** than the slot's current epoch". (b): "the owner drops it if a page transition has made the slot's epoch newer" (than the epoch it was decided from).

**Evidence.**
- I, trace under the bullet. The current epoch is 4. Rust opens a tooltip and mints `seen + 1 = 5`, which is applied. Rust's own writes are not "read from a fact or a report", and a non-gated open sends no `applied` report. So the pointerleave close 80 ms later also mints 5, `5 > 5` is false, and the close is dropped. **Every second Rust command between two page reports is lost.**
- I: patching the mint to `max(seen, minted) + 1` still fails with two writes in flight. Rust mints 6 and 7 from a state at 4, the page dismisses (4 → 5... → 6), 6 is dropped and 7 is applied, although 7 never saw the dismissal. A Lamport "strictly newer" test does not capture "decided from".
- V, as built: `Epoch::next` is one process-global `AtomicU64` (`bridge/mod.rs:151-159`). The page's `stale()` compares Rust epochs only with Rust epochs, keyed by element object in a `WeakMap` (`ts/bridge.ts:405-418`). No page transition advances it. A global Rust counter always beats a page epoch, so (b) never drops anything. And the memory is lost when Dioxus re-creates the element. Round 4 asked for the scope key for the reconciler (whole-core 13). The generic verbs still key by element.

**Concrete change.** One rule, the one in (b):
- a Rust write carries `based_on` (the slot's last page epoch Rust has seen) and a Rust sequence;
- the owner applies the write iff `based_on >= last_page_transition`, and otherwise drops it and reports;
- Rust writes are ordered among themselves by the pump's FIFO, or by the command's level;
- epochs are keyed by slot id (the owning scope, or `document` for focus), never by element object.

Replace `Epoch::next` before item 0.4. Delete the "strictly newer" sentence.

**Subsumes.** The reconciler's `gen`, the generic `call`/`set`/`get` epochs, and the stale-op tests of `bridge/mod.rs:980-1006`.

### 3. DETAIL, major. The vdom edit stream writes native page-written attributes outside every slot owner

**Claim attacked.** (b): "a Rust write to a page-written slot goes through that slot's slot owner ... `otpFilter` for the value it filters". The reconciled kinds are a closed set of four (popover open, modal open, `indeterminate`, `defaultChecked`).

**Evidence.**
- V: `value` is `volatile` on `input`, `select` and `textarea`, and `selected` on `option` (`dioxus-html-0.7.9/src/elements.rs:1488, 1536, 1571, 1594`). Volatile attributes are rewritten on every diff (`dioxus-core-0.7.9/src/diff/node.rs:444-463`), and the interpreter assigns `node.value = value` whenever it differs (`dioxus-interpreter-js-0.7.9/src/ts/set_attribute.ts:24-35`), which also moves the caret.
- V: `checked`, `open` (`elements.rs:1428, 1606, 1613`) and this repo's custom `HIDDEN` attribute (`collapsible.rs:41-47`, `volatile: false`) are written only when their value changes. The page writes all three: the user, `form.reset()`, form-state restoration, and `beforematch`, which removes `hidden` itself. So Dioxus's memory diverges with no record. That is the exact problem the command kind solved for popover open.
  - `collapsible.rs:122-124` documents it: "a controlled `open` that ignores `on_open_change(true)` leaves the content revealed while the collapsible reports closed" (V).
- V: hydration rebuilds with `skip_mutations` (`dioxus-web-0.7.9/src/lib.rs:100-148`), so the vdom remembers the server's `value` while the DOM holds whatever the user typed before the wasm arrived. The next render of that template wipes it. The combobox input re-renders on every `open()` or `active_descendant()` change (`combobox/components/input.rs:152-161`, V).
- The command trick of an epoch inside the attribute value cannot apply here. `hidden="until-found"`, `checked` and `value` must hold their literal native values.
- Shipped sites (V, b28): `combobox/components/input.rs:152`, `input_otp.rs:434`, `radio_group.rs:436`, `checkbox.rs:341` (plus a second writer, an eval of `input.checked` at `checkbox.rs:298-316`), and the `HIDDEN` slot in collapsible, accordion and tabs.

**Concrete change.**
- The reconciled kinds become "**every native page-written slot Rust writes**": `value`, `checked`, `selected`, `hidden=until-found` and `<details open>` join the four.
- Rust publishes `data-dx-*` (desired, `based_on`) and the slot owner applies the native property, or drops the write.
- Rust never echoes the user's own input back into the slot: an rsx `oninput` updates Rust, not the DOM.
- At install, each owner reports the live level as a page transition, which covers typing before hydration and form restoration.
- A gate (clippy, or an rsx scan beside `check-hot-events`) bans rsx `value:`, `checked:`, `selected:`, `open:` and `hidden:` on a page-writable element outside the `initial_*` forms.

**Subsumes.** Lost keystrokes on desktop (scenario A), text that reappears after a form reset (D), the wipe of text typed before hydration (G), the `until-found` veto divergence (C), and the two-writer `checked`.

### 4. DETAIL, major. `MountedData` writes are a second owner bypass, and the plan's ledger uses one

**Claim attacked.** (b), and section 5.3 ("clippy `disallowed_methods` for every other route").

**Evidence.**
- V: `MountedData::set_focus`, `scroll_to`, `scroll_to_with_options` and `scroll` write focus and scroll position (`dioxus-html-0.7.9/src/events/mounted.rs:167-200`). On desktop each one is an immediate `evaluate_script`, unordered with the pump (`dioxus-desktop-0.7.9/src/query.rs:45-60`).
- V: b28 has 82 `set_focus(` sites in 36 files and 9 `scroll_to(` sites. `focus()` without `preventScroll` also scrolls ancestor scrollers, so one focus write lands on two slots (W, HTML focusing steps).
- V: the disallowed list names only eval routes and the `DataTransfer` setters (root 5.3, lines 630, 636).
- V: the plan's own ledger places `use_refocus_on_close_unless` as "rsx `set_focus` through `TriggerHandle`" (root, line 465). That is a Rust focus write outside the focus owner. js-interop also tells agents "`MountedData::set_focus(true)`" and to read the scroll offset through `MountedData`, which gives an epoch-less read that then feeds a write.

**Concrete change.** Add `MountedData::{set_focus, scroll, scroll_to, scroll_to_with_options}` to `disallowed_methods` by def path, with per-site `#[expect]` until each site is ported. Focus goes through the focus owner and scroll through the scroll owner (finding 5). Reads of a page-written slot that feed a write go through `bridge::get` with the slot epoch. Fix the ledger row and js-interop.

**Subsumes.** The refocus row, the 78 roving sites' fallback, and the S4/V6 read-then-set.

### 5. DETAIL, major. "One page owner per slot" fails for focus and scroll as mapped. A scroll epoch needs a classifier that no verb row can hold

**Claim attacked.** (b)'s owner list: "`openFocus` and `roving` for focus, the scroller's behaviour (`stickToEnd`, `pan`) or a fenced setter row for scroll position". Also `scrollStep` "is a `transaction` from Rust unless its witness is red", and (c) "a `--dx-*` custom property" as the example of a Rust-only slot.

**Evidence (slot by slot).**

| slot | page writers | planned owner(s) | epoch readable at emission? |
|---|---|---|---|
| open (popover/dialog) | light dismiss, Escape, CloseWatcher (`close` is a post-paint task when `cancel` is skipped) | executor | yes, if the stamp reads the level (`:popover-open`, `dialog.open`), not only the last observed event |
| focus | user, the browser's focus fixup on removal, dialog/popover focusing steps, `autofocus` | `openFocus` **and** `roving`, plus `settle`'s focus, `focus_prevent_scroll` and 82 `set_focus` sites | yes (`activeElement`), but it is one **document-global** slot. The per-element `WeakMap` (finding 2) cannot see a click into another field, so a late roving write steals focus back |
| scroll position | user (compositor-threaded), scroll anchoring, snap re-snap, find-in-page, `#fragment`, `focus()` | `stickToEnd`, `pan`, a "fenced setter row", the carousel's behaviour | **not from `scroll` events**: they do not distinguish user, anchoring, snap and owner writes (W, CSSOM View). It needs the input events (wheel, touchstart, keydown, pointerdown on the scroller) and memory of the owner's own writes |
| value / caret / checked / selected / hidden | user, reset, autofill, restoration, IME, `beforematch` | `otpFilter` only | yes, with `input`/`reset`/`beforematch` listeners (finding 3) |
| `--dx-*` custom property | the page too: `anchor_fallback` publishes `--dx-anchor-width`/`--dx-available-*`, and `pan`/`stickToEnd` write variables | "Rust only" | n/a. (c) must be per property |
| dialog `returnValue`, media state | none in the library (V, grep: no `video`/`audio`, no `returnValue`) | - | n/a |

- A verb row is "one member call or one member read, plus at most one guard `if`" (js-interop, the bridge section). A fenced setter needs listeners plus memory, so it is a behaviour, not a row.
- If `scrollStep` is a transaction, nothing page-side mints the touch-swipe transition. That is scenario E.

**Concrete change.**
- **One focus owner** (document slot, one page-lifetime `focusin` listener in the baseline, as `modality.rs` already has), which `openFocus`, `roving`, `settle`'s focus, refocus and `focus_prevent_scroll` call.
- **One `scrollOwner` behaviour per Rust-written scroller**: it classifies input into transitions, applies `based_on` writes, and reports settle. It subsumes `stickToEnd`'s detach, `pan`'s scroll writes, S4/V6 and C2.
- C2's 114 lines do not leave the registry.
- (c) is decided per property, and `set` refuses a property that a behaviour on the same element declares as an output.

The slot table goes into root 2.2 with one owner per row.

**Subsumes.** S4, V6, W8's `scroll(end)`, carousel autoplay against a swipe, and roving after a click elsewhere.

### 6. DETAIL, major. The order-ground witness reorders facts only against vdom events, but most page transitions are native

**Claim attacked.** Admission table, row "an order-ground decision ... the fact reordered after the next vdom event, by the desktop-cadence harness".

**Evidence.**
- I: the hazards in scenarios C, D and E pit a Rust write against a **native** transition: a touch scroll, find-in-page's reveal, `form.reset()`, a light dismiss on `pointerdown`. None of these is a vdom event or a bridge fact the harness can delay.
- V: `channelDelay` delays bridge messages only (`ts/bridge.ts:331-357`). Row 442's patch delays edits (`442-*.md:28-34`).
- So `scrollStep`'s and `reconcile`'s order witnesses cannot go red on these, and admission could evict the owner the contract needs.

**Concrete change.** Add a counterfactual: "the Rust write delivered after a scripted native transition on its slot (touch scroll, dismiss, reveal, reset, autofill)", with probe `outcomeOrder`. This is the "delayed" counterfactual, applied to order-ground effects. Each slot row in finding 5 names its native transitions.

### 7. DETAIL, major. Condition (d) asks for a per-decision judgment that nothing checks; one inbound pump makes it hold by construction

**Claim attacked.** (d): "facts that one decision combines share one session, or go through one inbound pump".

**Evidence.**
- V: the scheduler orders tasks by height and then scope id (`dioxus-core-0.7.9/src/scheduler.rs:111-115, 147-163`), as row 1.4 says.
- V: today each `Session::open` is its own eval with its own receive task in the caller's scope (`bridge/session.rs:392, 422-426`).
- Which facts "one decision combines" is semantic, so no gate can check it.
- Shipped violation: Accordion in single mode combines the `beforematch` facts of N panel scopes, each with its own `use_js_listeners` session (`collapsible.rs:64-86`, `tabs.rs:499`). Sibling panels are processed in scope-id order, which a re-keyed or inserted panel breaks (scenario C).
- I: a mixed design is worse than either pure one, because the root pump's facts (height 0) always overtake every session's facts in a batch.

**Concrete change.** Every page-to-Rust fact goes through **one** inbound pump: a page-minted global sequence and one root task that dispatches to subscriber callbacks in sequence order. A `Callback` runs in its origin scope (I). Sessions keep their own outbound commands. The scheduler row then names a hazard the construction removes, not a rule lanes must follow.

### 8. DETAIL, minor. Slider and Resizable start a drag in rsx and end it with a bridge fact: a Rust-only slot over two channels

**Evidence.**
- V: `move_interaction.rs:119-146` (`start_pointer` from rsx `onpointerdown`) and `pointer.rs:33-48`, whose window `up` fact calls `remove_pointer(id)`. A mouse's `pointerId` is the same on every press (W, Pointer Events 3).
- I, desktop: click the track twice quickly. Press 1's `up` fact is still queued behind the render that press 1 caused, while press 2 is handled inline. Then that stale `up` removes pointer 1 and `pointer_move` calls `end_pointer`, so drag 2 is dead whenever an effect runs between that `up` and press 2's `down` fact.
- The plan's rule ("start, move and end through one channel") exists, but js-interop marks this module "today", and it is not in the instance list.

**Change.** Add it as an instance. The fix is the plan's own: every phase of the drag through one bridge session, or through the pump of finding 7.

### 9. DETAIL, minor. Legacy fact sources on page-written slots stay outside the contract under scope 2

**Evidence.** V: `use_beforematch` (the hidden slot), `use_form_reset_listener` (`lib.rs:571-588`, the value and checked slots) and `pointer.rs` send epoch-less facts through `Program::Legacy`. The ratchet freezes their volume, not their semantics.

**Change.** Give each one an owner row in the 3.3 ledger and a port lane, ordered by exposure.

---

## Scenarios traced (fresh; web and desktop)

**A. Combobox: typing while async results arrive.**
- Web: holds. `oninput` sets the query synchronously, and the render is a later microtask, so Rust's text is never behind the DOM. Volatile `value` writes are no-ops (`node.value === value`).
- Desktop: **breaks**. The user types 'a' and render R1 (`value "a"`) is in flight. The user types 'b' (DOM "ab", Rust "ab"). R1 lands in the rAF and sets `node.value = "a"`, with the caret moved to the end. 'c' typed before R2 gives "ac", and the 'b' is lost for good. Every results render rewrites `value` again, because the input's template reads `open()` and `active_descendant()`. IME composition is cancelled by any differing write.
- Contract: not covered (finding 3). After the fix, Rust never echoes `value`.

**B. Slider dragged while Rust clamps.**
- Holds on both renderers. The value is a Rust-only slot. The drag accumulates an unclamped "granular" value from absolute pointer positions and clamps once per write (`slider.rs:401-439`), so latest-wins coalescing converges. An arrow key during a drag is overwritten by the pointer, which is acceptable.
- Desktop only: the stale `up` of finding 8.

**C. Accordion: find-in-page `beforematch` while Rust toggles.**
- Web: holds. The fact's microtask runs before the next event.
- Desktop: find-next cycles B, then D (an inserted panel with a lower scope id). Both facts land in one batch and are processed D then B, so B ends open and the find match in D is hidden again: the wrong panel.
- A controlled veto (Rust keeps D closed) leaves `HIDDEN="until-found"` unchanged in the vdom while the browser has removed it, so the panel shows open while `aria-expanded=false`. That happens on both renderers, documented at `collapsible.rs:122-124`.
- Contract: violated by (d) as built (finding 7) and by a missing owner (finding 3).

**D. Form reset while a controlled input is focused.**
- Both renderers. `reset` fires before the controls reset. The checkbox's Rust state is reset through the legacy listener and converges, but on desktop with a frame of skew. The controlled text input has no reset path: the DOM shows "" while Rust keeps "abc", and the next render of its template writes "abc" back at an arbitrary moment. **Breaks.**
- Contract: no owner for value (finding 3). After the fix, the value owner mints on `reset`, which bubbles, through one document listener.

**E. Carousel autoplay tick against a user swipe.**
- V: native swipes are deliberately not an "interaction" (`carousel/context.rs:255-270`). The tick computes `step_next(selected.peek())` from the last settled index (`autoplay.rs:166-188`).
- Mobile/desktop (`mobile = dioxus-desktop`): a swipe from 2 back to 1. The settle fact is queued, the tick targets 3, and the scroller is pulled forward and then bounces back to 1.
- Web: the tick can fire mid-touch and fight the compositor's scroll.
- Contract: needs the scroller's owner to mint a transition on `touchstart`. None exists if `scrollStep` is a transaction (finding 5), and the witness cannot see it (finding 6). **Breaks on both.**

**F. A light dismiss and an rsx click in one gesture.**
- The renderers diverge under the contract (finding 1). The fix is the transition port, or a platform invoker without `prevent_default`.

**G. SSG: typing before hydration.**
- Web SSG: text typed before the wasm arrives is wiped by the first post-hydration render of a template that reads a changing memo (finding 3). `data-hydrated` is set by the root's effect, which runs before descendant effects (effects are popped in `ScopeOrder`, `scheduler.rs:161`), so specs cannot see this window.
- Desktop has no SSG, so it does not apply there.

---

## Attacks that failed
- **The scheduler row** is verified as stated: ordering by height then id, FIFO only inside one scope (`scheduler.rs:111-115, 147-163, 244-251`). The same batching happens on web for facts sent in one page task.
- **"Two desktop renders never share a frame"** is verified: `poll_vdom` waits for `poll_edits_flushed` before `wait_for_work` (`webview.rs:557-563`).
- **The epoch at emission for open state on engines without `beforetoggle` on `<dialog>`, or when CloseWatcher skips `cancel`**: a fact stamped before the post-paint `close` task names the old epoch. The resulting Rust close is redundant, not wrong, provided the owner stamps by reading the level (a parameter, folded into finding 5's table).
- **Eval from handlers as a side channel**: sealed by the one `Program`-typed sink. The lane rule "a bridge op that depends on a render is issued from an effect" holds against 1.4 item 3.
- **Hydration for the four reconciled kinds**: reading the level at install converges, and `skip_mutations` means no attribute record is lost that the install read does not cover.
- **The slider's clamp** (scenario B) and **`--dx-*` variables written only by Rust**: commutative as defined, once (c) is decided per property.
- **Media state and `returnValue`**: no such slots in the library (V).
- **The core constructs** all survived: the root rule, the deadline table, the two kinds, the closed bridge with one eval sink, A/G/R, and enforcement by sinks plus ratchet. No finding needs any of them replaced.
