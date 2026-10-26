# Board round 7, lens `whole-core` (2026-10-26)

**Verdict: HOLDS WITH CHANGES**

**DESIGN findings: 0, DETAIL findings: 11**

Summary:
- I traced seven scenarios that no earlier round used, on web and desktop, against the Dioxus 0.7.9 sources and the as-built code (b28, l30). The root rule, the deadline table and its two kinds, admission per effect, the two carriers with one pump, the one executor, the closed bridge with one eval sink, A/G/R, and sinks plus ratchet all held. Every break below can be fixed inside those constructs.
- Every finding sits in the order contract that rounds 4-6 added. All of them are scope or parameter errors, not wrong mechanisms. Five are major:
  - Carrier 1 is chosen by "an rsx decision reads this slot". No gate can check that, and the slot table already gets two shipped rows wrong (carousel Next after a swipe, collapsible trigger after find-in-page), so web and desktop end in different states (finding 1).
  - The named port hooks run after the Dioxus root handler of the same event (finding 2).
  - A write's consequence on another slot is never fenced: a late open steals focus, and a late focus yanks the scroll (finding 3).
  - Per-frame payload carries no stream epoch, so moves from the previous drag land in the next one (finding 4).
  - The focus hook misses focus that moves to no element, such as a click on blank page (finding 5).
- Findings 1 and 3 also simplify. They remove the slot table's "rsx-read" column and its unverifiable gate rule, and they replace two per-instance patches (T12's foreign epoch, Escape's page-side stack resolution) with one rule.
- The main design is solid; what remains is detail. But the order runtime's details are not finished. Item 0.7 must take findings 1-6 before any consumer row builds on it.

Evidence labels: **V** = read in the repo or in the pinned crate source; **W** = spec or web source; **I** = inference. Dioxus paths are relative to `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`. "root" is `l434d4/dev-docs/research/sync-js-root-2026-10-26.md` at 640cad14. "b28" is `/home/user/wt/b28`.

---

## End-to-end traces (fresh; web and desktop)

The shared paths are taken as verified in round 4 and are not repeated here: the root listener (`dioxus-interpreter-js-0.7.9/src/ts/core.ts:117-121`, `this.root.addEventListener`, bubble, no options), the desktop sync XHR, rAF edits with the acknowledgement sent inside the rAF callback (`native.ts:445-457`), and facts queued behind the acknowledgement.

**T1. Combobox inside a Dialog inside a Sheet, with IME composition.**
- Setup: two nested `<dialog>` modals (popup boundary: the Dialog is a DOM descendant of the Sheet), and a `popover="manual"` listbox (`combobox/components/input.rs:108-122`). The input has a controlled `value: display_value()` (`:152`) and an `oninput` that sets the query (`:177-186`).
- Composition, web and desktop: each composing `input` event is a value transition that goes through the port. With the controlled `oninput` that is two sync requests per keystroke on desktop. That is acceptable at human rate. **But** the value owner's document listener runs after Dioxus's root handler, so `oninput` decides from Rust's pre-transition epoch (finding 2).
- Escape while composing: the combobox's rsx handler closes the list and calls `prevent_default`. The close request then does not reach the inner dialog, and L2 `oncancel` is rsx. **Holds.**
- Tapping an option while Android's IME holds a composing region: the label write is held "until `compositionend`". That event may never come while focus stays in the input. When it does come, the epoch has moved on and the write is dropped (finding 8).
- Tapping the Sheet's scrim: the composition commits (a value transition), then the rsx click closes. The cascade closes deepest first (root 2.2, batch by tree). **Holds.**
- Inside a `<form>` in the dialog, the port cannot sit outside the form (finding 7).

**T2. Carousel swiped by touch, then Next tapped within the settle window. Twin case: a panel revealed by find-in-page, then its trigger clicked.**
- V: `selected` is fed by a page fact (`carousel/scroll.rs:381` `dioxus.send(['selected', ...])`, consumed at `:685-687`). The rsx `onclick` computes `step_next((ctx.selected)(), ...)` (`carousel/nav.rs:109-113`; prev at `:62-66`).
- Desktop: the swipe's transitions and settle are facts queued behind the acknowledgement, while the tap is handled inline. Under the plan the target is `old + 1`, sent with the old `based_on`. The scroll owner drops it because the swipe advanced the epoch, so **the tap is lost**.
- Web: the facts run in the swipe's own microtasks, and the tap either goes to `new + 1` or is applied over the swipe. **The renderers diverge.**
- Twin: the collapsible trigger toggles `!open()` (`collapsible.rs:388-391`), and `beforematch` arrives as a page fact (`:64-86`). It is the same race: find reveals the panel, the user clicks to close, and desktop loses the click (finding 1).

**T3. Resizable group: drag, release, re-grab within one acknowledgement window; a layout change mid-drag.**
- V: the drag starts in rsx (`move_interaction.rs:119`). Moves come from a `window` listener keyed by `pointerId` (`pointer.rs:28-38`, upsert `:90`, update `:101`). Each move is converted as `delta_px / group_rect` (`resizable.rs:702-716`), and the rect is refreshed by rsx `onresize` through an async `MountedData` read (`:482-483`, `move_interaction.rs:108`).
- Under the plan the start and end travel on the port and the moves on a session.
- Desktop: frames of drag k still queued when drag k+1 starts update pointer 1, because a mouse's `pointerId` is the same on every press (the plan's own W citation). The first delta of drag k+1 is then measured from drag k's last position, and the handle jumps (finding 4). Web: the frames run before the next `pointerdown`, so no jump.
- A layout change mid-drag mixes a stale rect with fresh moves, so the handle drifts off the pointer. It drifts less on web than on desktop (finding 10).

**T4. Toast swiped while the page goes hidden and comes back (visibility, bfcache; desktop occluded window).**
- V: the countdown runs only while the document is visible, and resets on hide (`toast.rs:1346-1362`). It is a slot only Rust writes. **Holds.**
- The swipe script handles `pointerup` and `pointercancel` only (`toast.rs:1464-1465`).
- Touch app switch: `pointercancel`, which goes through the port. Desktop handles a port event inline even while hidden. **Holds.**
- A mouse drag whose release is never delivered leaves `data-swiping` and the offset stuck until the next pointer event (finding 11).
- bfcache: wasm and page state freeze together. On thaw the lease renewal (due at L/2) runs before the expiry (due at L), so **holds**.
- G in a hidden web tab: the wall-clock fail-open fires while ResizeObserver does not. At the first visible frame the RO callback, and Rust's answer in its microtasks, still land before paint. On desktop no edit lands while hidden. **Holds** (see "Attacks that failed").

**T5. Navigation Menu opened by keyboard, then hovered to another item while the first animates out, then ArrowDown.**
- V: each item's content is `popover="manual"` with `use_popover_shown_while_mounted` (`navigation_menu.rs:1013, 1063`). The hover-intent timers are Rust (`:81-98`).
- Hover-switch A to B is two Rust commands: close A (the exit wait) and open B. No stack hide is involved, because both are manual.
- A hover back to A during A's exit flips A's level to open. The executor re-reads the level when the animation finishes, so the close is skipped.
- Modality stays "keyboard" (hover is not `pointerdown`), so `openFocus` does not steal focus from trigger A.
- ArrowDown on trigger A decides from Rust's open item, which is current because Rust caused it. **Holds on both renderers.**

**T6. Context menu opened by long-press on iOS-like touch, with scroll lock.**
- V: the trigger publishes `touch-action: none` and `-webkit-touch-callout: none` beforehand (`context_menu.rs:501-503`). The long press is a Rust timer (`:445-467`), and the lock is `scrollLock{allowInside: root}` at window capture (L10).
- The gesture-start commit point is met by A: no page scroll for the pressing finger. The lock covers every later gesture.
- The only window is the asynchronous install of the lock after the open, 1-3 frames on mobile, where a second finger could scroll. That is negligible. **Holds.**

**T7. Form submit with Rust validation that focuses the first invalid field inside a virtualised table; the user scrolls or clicks away meanwhile.**
- Native constraint validation sees only mounted rows, so by the root it cannot decide: the data is not present. The form cancels declaratively and Rust validates, so the submit itself holds.
- The chain is: scroll the owner to row N, Rust windows it in, `onmounted`, then a focus-owner write.
- If the user wheels away first, the focus write's `based_on` is still current, because it names the focus slot. `focus()` then yanks the scroller back, since the induced scroll counts as a Rust-caused consequence (finding 3).
- If the user clicks blank page instead, no `focusin` fires, so the late focus write steals focus back (finding 5).

---

## Findings

### 1. DETAIL, major: carrier 1 is selected by "an rsx decision reads the slot", which no gate can check; the slot table already misassigns scroll and `hidden`

**Claim attacked.** Root 2.2 condition 1, carrier 1, and the slot table (root :293, :306-313): "The slot table records for each slot whether it is rsx-read ... or bridge-only ... and a gate rule checks the table." The scroll row's carrier is "a fact". The `value`/`hidden` row says "the transition port for a controlled value that an rsx `oninput` reads, else a fact".

**Evidence.**
- V, trace T2: the carousel's rsx `onclick` reads a fact-fed slot (`nav.rs:109-113`, `scroll.rs:381, 685-687`). So does the collapsible trigger (`collapsible.rs:388-391` reading the open state that `beforematch` changes, `:64-86`). Both decisions are relative, so the fence picks a loser on desktop and a different outcome on web. That is exactly the class carrier 1 was added for in round 5.
- V: the documents disagree with themselves. Root :294 and `js-interop.md:94-95` list `beforematch` as a port hook, while the slot table's carrier column sends `hidden` transitions as facts.
- I: whether an rsx handler reads a slot is dataflow through signals and memos (`selected` is a `Memo` fed by a `Callback` that a `js_listener` calls). A grep cannot see it. The plan rejected exactly this kind of per-decision judgement when it built the pump (root :298).

**Change.** Select carrier 1 by the kind of transition, not by its readers:
- **Every discrete page transition of every slot travels on the port**: dismiss, toggle, reveal (`beforematch`), reset, a keystroke on a value Rust writes, a focus change, a stream's start, end and cancel.
- For a scroll slot, the discrete transitions are the first realized user scroll after a quiet period and the settle. The intermediate wheel and touch increments advance the page epoch without a report, so Rust's writes are fenced while the user scrolls and apply after the settle.
- Sessions keep per-frame payload. The pump keeps observations that are not transitions (measurements, mutations, `applied{gen, rects}`), each stamped with the epoch it saw.
- Delete the rsx-read column and its gate rule. The gate checks only "a transition hook fires the port", which can be read from `bridge.ts`.
- Cost: two sync requests per scroll gesture on desktop. Add them to `bridge-cost`.

**Subsumes.** Carousel prev/next after a swipe, a collapsible or accordion trigger after find-in-page, any future rsx reader of a fact-carried slot, and the documents' disagreement about `beforematch`.

### 2. DETAIL, major: the named port hooks run after the Dioxus handler of the same event, so a decision made in that handler is fenced as stale

**Claim attacked.** The port "fires from a synchronous hook (`beforetoggle`, a dialog `open` observer plus `cancel`, `focusin`, `beforematch`)" (`js-interop.md:94-95`). The value owner mints on `input` and `reset` "through one document listener" (root :299, :311).

**Evidence.**
- V: Dioxus delegates bubbling events at the app root (`core.ts:117-121`), which sits below `document`. So a document bubble listener runs after every rsx handler of the same event. `focus` is non-bubbling and is attached on the element itself (`dioxus-core-types-0.7.9/src/bubbles.rs:23`, `core.ts:123-129`). `focusin` fires after `focus` (W, UI Events focus order, <https://w3c.github.io/uievents/#events-focusevent-event-order>).
- I: `based_on` is the epoch "it was decided from" (root :291-292, "decided before the consequence existed"). Consider a Rust write decided inside the rsx handler of the event that is itself the transition:
  - the color picker's range inputs re-render their controlled `value` from `oninput` (V `color_picker.rs:507-514, 543-548, 607-611`);
  - the natural-language date picker's transforming input;
  - a form `onreset` that writes controlled values;
  - `tabs.rs:389` `onfocus` driving the roving index.
- That write carries the pre-port epoch on **both** renderers, and the owner drops it. If `based_on` were instead read at send time, the fence would admit decisions taken before a transition, which is the opposite failure. The witness harness cannot see either: both arms put the port at the same place.

**Change.**
- Fire every port hook from a **window capture** listener, so the port precedes every rsx handler of that event: `input`, `reset`, `focus` with capture (not `focusin`), and `focusout` (finding 5). `beforetoggle`, the dialog observer and `beforematch` already fire before the next input task.
- State in the order contract that `based_on` is captured at decision time.
- Add one spec per hook in which an rsx handler of the same event writes the slot.

**Subsumes.** Transforming controlled inputs, `onreset` writers and `onfocus`-driven roving writes on both renderers.

### 3. DETAIL, major: a write's synchronous consequence on another slot is never fenced, so a late open steals focus and a late focus yanks the scroll

**Claim attacked.** "A Rust-caused transition ... covers anything that happens synchronously inside an owner's own apply call (the executor's show and close, the focus owner's `focus()`, the scroll owner's write)" (root :291). The plan lists "`openFocus` after an explicit focus" among the follow-ups never fenced (:316). The focus owner "asks each owned ancestor's `scrollOwner` for an ensure-visible intent" (:301).

**Evidence (I, traces T7 and T5's mouse variant).**
- An open command is fenced only on the open slot. Rust decides to open a popover; within the 1-3 frames before the command lands on desktop, the user clicks into a text field. That is a focus transition, but on another slot. The open applies and `openFocus`'s mouse policy focuses the panel. The user's next keystrokes go to the panel.
- That is the round-6 red scenario of the focus owner ("a late roving write after a click into another field") entered through another slot's consequence.
- Likewise a focus write fenced on the focus slot carries an ensure-visible scroll that is never checked against the user's wheel (T7).
- The plan already patches two members of this class per instance: T12's scroll fact carries the open-state epoch, and `hotkey{Escape}` names its target layer resolved on the page (:297).

**Change.**
- A command or owner operation carries **`based_on` for every slot it can touch**: open touches focus and the auto stack, focus touches the scroll ancestors. Each owner checks its own entry.
- Per consequence pair, decide the policy once:
  - open then focus: apply the open, skip the focus move and report;
  - focus then ensure-visible: drop both, never focus off-screen;
  - stack hide: always, because it is the platform's.
- Keep "Rust-caused transitions never advance the fence"; this finding is about whether a consequence applies, not whether it mints.

**Subsumes.** Late `openFocus` after a user focus change, the validation focus yank, T12's special stamping and Escape's page-side stack resolution (one rule instead of two patches).

### 4. DETAIL, major: per-frame payload carries no stream epoch, so on desktop the previous drag's frames are applied to the next drag

**Claim attacked.** "Per-frame payload stays on its session, latest-wins, and is combined with no fact of another source; a stream's start, end and cancel are discrete transitions that travel on the port" (root :298; 3.GS :764).

**Evidence.**
- V: `pointer.rs:28-38` sends `[kind, [pointerId, x, y]]`. `update_pointer` writes by id (`:101`), and `pointer_move` takes its delta from the last stored position (`move_interaction.rs:148-160`).
- W: a mouse's `pointerId` is constant across presses (Pointer Events 3, cited by the plan).
- I, desktop: frames of drag k still queued behind an acknowledgement are delivered after drag k+1's rsx `pointerdown`. They overwrite the position of pointer 1, and the first real move of k+1 jumps by the distance between k's last frame and k+1's press. Web delivers them first, so the renderers diverge.
- The port fixes the stale `up`. It does not fix the stale moves.

**Change.**
- The start transition mints a **stream epoch** on the page, and every per-frame payload carries it. Rust drops a frame whose stream epoch is not the current stream's.
- This is carrier 2's rule "facts carry the epoch they observed", extended to per-frame payload.
- Add a re-grab spec at `__dxBridgeDelayFrames = 2`.

**Subsumes.** Slider, Resizable and drawer re-grab jumps, and any `pan` output variant admitted by a `lag` or `order` verdict.

### 5. DETAIL, major: the focus slot's hook misses focus moving to no element, so a late Rust focus write steals focus back after a click on blank page

**Claim attacked.** Slot table, focus row: "epoch source `activeElement`, from `focusin`" (root :309). The focus owner exists because "a late roving write would steal focus back" (:301).

**Evidence.**
- W, I: clicking non-focusable content blurs the active element: `blur` and `focusout` fire with a null `relatedTarget`, focus goes to the viewport, and no `focusin` fires anywhere (UI Events `focusout`, <https://w3c.github.io/uievents/#event-type-focusout>; HTML focusing steps for the viewport). The same holds for a click on a scrollbar or a non-focusable scroller.
- I, desktop: arrow keys in a virtual grid or a roving widget, then a click on blank page. The epoch never advances, so the in-flight focus write lands and focus jumps back into the widget.

**Change.**
- The focus owner also mints on a window-capture `focusout` whose `relatedTarget` is null, after a microtask check that `activeElement` is body or null, so a `focusout` that a `focusin` follows does not mint twice.
- Add a spec: a late roving write after a click on blank page.
- The removal fixup stays Rust-caused, as now.

**Subsumes.** Click-away, scrollbar-click and window-blur-then-click variants of the focus-steal class.

### 6. DETAIL, major: slot ids and the command attribute's `scope` are not SSR-stable, so the pre-install level report and an SSG command use different keys

**Claim attacked.** Slot ids are "the owning scope plus a mount nonce from `use_hook`" (root :291, `js-interop.md:22, 89`), and the attribute carries `(scope, state, based_on)` (:327). Pre-install: "a dismissal before the channel exists is not lost and the level does not reopen it" (:330).

**Evidence.**
- V: hydration rebuilds with `skip_mutations` (`dioxus-web-0.7.9/src/lib.rs:100-148`), so the DOM keeps the server's attribute value until the client's value changes.
- V: the repo already found that an id generated on the server differs on the client unless it goes through `use_server_cached` (`b28 primitives/src/lib.rs:202-218`, `use_unique_id`: "`dxc-c{n}` can never equal any server id"). The SSG server renders every route in one process, so a process-global nonce differs per route.
- I: the executor reads `(scope_s, nonce_s)` at install and reports the level under that key. Rust's slot is `(scope_c, nonce_c)`, so the report is unroutable. Rust's next command under the client key starts a fresh epoch, and a pre-hydration dismissal is lost, which is the divergence the install read exists to prevent.

**Change.**
- Derive the slot id from an SSR-stable source: `use_server_cached`, as `use_unique_id` does, or the component's existing unique id plus a client-only reuse nonce that never appears in markup.
- Add a hydration-parity spec: the server's command attribute equals the client's first value, and a dismissal before install survives.

**Subsumes.** Every reconciled kind on SSG pages, plus the value owner's install-time level.

### 7. DETAIL, minor: "the port is never inside a `<form>`" cannot be built for a form control, and the `input` event type serialises the whole form

**Claim attacked.** "A host inside a form renders the port as the panel's sibling outside it" (root :295; `js-interop.md:96`).

**Evidence.**
- V: Dioxus 0.7.9 has no portal outlet without an app-rendered `PortalOut` (`b28 primitives/src/portal.rs`), and a root provider was rejected (root 5.2).
- V: the popup-boundary rule keeps a panel inside its trigger, so a value owner inside a form renders its port inside the form.
- V: desktop serialises the form for **any** event of type `input`, `change`, `click`, `submit` or `reset` on any `HTMLElement` target, through `target.closest("form")` (`dioxus-interpreter-js-0.7.9/src/ts/serialize.ts:72-81, 432-451`). A `form` attribute does not help.

**Change.** Pick a port event type that the desktop serialiser never extends with form values: a synthetic `AnimationEvent` carrying the payload in `animationName` (`serialize.ts:40-41, 315-321`; Dioxus bubbles it, and the modality writer does not read it). Delete the "outside the form" sentence.

**Subsumes.** The Android header-drop case of the port, and the per-keystroke double form serialisation.

### 8. DETAIL, minor: the value owner's "hold until `compositionend`" can wait indefinitely on composing-region keyboards, then drops the held write

**Claim attacked.** "The owner holds a `value` write during IME composition until `compositionend` and then re-checks `based_on`" (root :299).

**Evidence (I, W).**
- Android keyboards keep a composing region over the current word (W, UI Events composition events, <https://w3c.github.io/uievents/#events-compositionevents>).
- A tap on an option whose `mousedown` keeps focus may leave the composition open. The label write is held, and when `compositionend` finally comes, every composing `input` has advanced the epoch, so the held write is dropped.
- Rust believes "Paris" is selected while the input shows "par".

**Change.**
- A held write is applied when the composition ends **or** when the next discrete non-composition transition arrives on another slot, such as the option's click.
- A user-initiated Rust write ends the composition explicitly (blur and refocus inside the owner's apply call, a Rust-caused transition) instead of waiting.
- Add an Android-emulation spec.

**Subsumes.** Combobox and command-palette selection while composing.

### 9. DETAIL, minor: `value`, `checked` and `selected` have no DOM level, so a re-created element loses the last Rust write

**Claim attacked.** The command kind converges because "the executor reads the level at install ... and at every added node" (root :326). For these three kinds the level is deliberately not in the DOM (:299).

**Evidence (I).**
- An element re-created without its component re-mounting keeps the mount-frozen first value from the helper (an `if` between two `input` branches, a responsive swap).
- The last owner write was addressed to the old element.

**Change.** The value owner keeps the last applied Rust write per slot as the page-side level, and applies it to a newly discovered element of the same slot, unless a page transition is newer.

**Subsumes.** The lost or early convergence guarantee for the three bridge-carried kinds.

### 10. DETAIL, minor: per-frame payload combined with geometry from another source; the rule page contradicts the root on gesture channels

**Claim attacked.** Per-frame payload "is combined with no fact of another source" (root :298). `js-interop.md:51`: "start, move and end all through one channel ... never an rsx `onpointerup` with bridge moves". The root puts start and end on the port and the moves on a session (:298, :764).

**Evidence.**
- V: Resizable divides pointer deltas by a rect refreshed by rsx `onresize` and an async `MountedData` read (`resizable.rs:482-483, 702-716`). Slider uses the same `MoveInteraction`.
- V: the two documents prescribe opposite channel layouts for the same gesture.

**Change.**
- A per-frame sample carries the geometry it is combined with: one `sample` of pointer position plus container rect.
- `js-interop.md:51` states the root's layout: start, end and cancel on the port, frames on one session with the stream epoch of finding 4.

**Subsumes.** Handle drift under a layout change mid-drag.

### 11. DETAIL, minor: stream edges that never arrive, and native controls whose `input` events are frame-rate

**Claim attacked.** The port carries only discrete, human-rate transitions (root :293), and `pan`'s vectors.

**Evidence.**
- V: the toast swipe ends only on `pointerup` or `pointercancel` (`toast.rs:1464-1465`). A release that is never delivered (window blur mid-drag) leaves `data-swiping` and the offset stuck.
- V: the color picker's `type=range` inputs (`color_picker.rs:507, 575`) fire `input` at drag rate under assistive technology and voice control, which would put frame-rate transitions on the port.

**Change.**
- `pan`'s vector set adds `lostpointercapture`, a mouse `pointermove` with `buttons === 0`, and `visibilitychange` to hidden, each as a cancel edge.
- The value owner classifies `range` (and `color`) inputs as streams: edges on the port, values latest-wins.

**Subsumes.** Stuck swipes, and port flooding from native continuous controls.

---

## Attacks that failed

- **The port is the rejected C1 trampoline in disguise.** It is not: it carries discrete transitions and never a cancel, and its `input` event is not read by the modality writer (root 2.4). The remaining cost is reason 1 (abuse of an event type), which finding 7 moves to a less colliding type.
- **Re-entrancy when the executor fires the port from its MutationObserver.** On desktop the acknowledgement is sent inside the rAF callback before the microtask checkpoint (`native.ts:452-455`), so the port's sync request is an ordinary inline event. On web the edit flush has finished before the observer microtask runs. No nesting with a Rust handler.
- **Long-press commit point (T6).** `touch-action: none` is published before the gesture (`context_menu.rs:501-503`), so a lock installed mid-gesture is not needed.
- **Navigation-menu exit wait against hover-back (T5).** The level is re-read when the animation finishes; manual popovers have no stack hide; hover does not change modality.
- **G's fail-open in a hidden tab (T4).** It fires by wall clock while ResizeObserver waits for a rendering opportunity. At the first visible frame the RO callback and Rust's microtask answer still precede paint on web, and desktop applies no edit while hidden. A paint-counted watchdog would be more honest, but no wrong frame was found.
- **Lease under bfcache and intensive timer throttling.** Renewal (L/2) is due before expiry (L), both are throttled together, and renewal reaches the page in its own task's microtasks. The gap after the tab returns is the plan's stated "next renewal reinstalls".
- **Escape during IME in nested modals (T1).** The rsx `prevent_default` stops the close request, and L2 `oncancel` is rsx with Rust's top-of-stack check.
- **The toast countdown against visibility.** A slot only Rust writes, reset on hide (`toast.rs:1349-1362`); commutative as defined.
- **Native validation in a virtualised form (T7).** The root rule already forbids it: the data is not present. A declarative cancel plus Rust validation needs no new construct.
- **A fourth construction beside A/G/R or a second executor.** No trace needed one. Every break above is a parameter of the order contract, a hook, a vector or a stamp.
