# Round 6, lens `order-slots`: the order contract (round-5 form) against Dioxus 0.7.9 and the as-built bridge

Verdict: **HOLDS WITH CHANGES**

DESIGN findings: 0, DETAIL findings: 15

- The two carriers, the one epoch rule and the slot owners survive every trace I ran. No finding replaces any of them. But the round-5 form has five major holes, each with a construction that removes it.
- **Rust-caused transitions** (finding 1, the strongest; borderline DESIGN). The rule counts "a detach" and every other browser-side consequence of a Rust write as a page transition. That makes Rust's own follow-up write lose by construction. Four shipped instances break: collection refocus after a re-mount, the scroll-lock restore after `showModal`'s focus scroll, menubar hover A→B→A (desktop only), and `openFocus` against an explicit focus.
- **The transition port has no home** (finding 2). The bridge renders no element, and section 5.2 rejected "a provider in every app". A port rendered per scope leaks a bubbling synthetic `input` into user handlers. Inside a `<form>` it also serializes the whole form into every transition, which Android drops above the header limit.
- **The vdom-write gate is a spelling gate** (finding 4). Its `initial_*` escape renders `initial_value="…"` literally in SSR, so SSG first paint shows blank fields and open panels. It misses three shipped forms of the write. `data-dx-value` mirrors typed text, OTP codes included, into a DOM attribute that CSS selectors can read.
- **The closed bridge's own verb table bypasses the owners** (finding 5): it still carries `scrollIntoView`/`scrollTo`/`scrollBy`, `setSelectionRange`/`select`, `blur`, and `hidePopover`/`togglePopover`/`close`. **The scrollOwner's classifier is decidable only in a per-intent three-way form** (finding 6): as written it drops keyboard scroll-into-view on desktop.
- Fresh scenarios, each traced on web and desktop: date-segment typing while Rust normalises (breaks on desktop; Backspace and Android IME break on both), OTP paste while the caret advances (breaks on both), a tabs list re-keyed by an async load (breaks under the plan on both), menubar hover-switch while a submenu closes (the renderers diverge), virtual-grid key repeat (breaks on desktop), a virtual list's focused row scrolling out (holds), and a drawer drag end against Escape (the renderers diverge).
- The port itself holds where the transition has a synchronous signal: popover `beforetoggle`, dialog `cancel` and `close` (the latter on the user-interaction task source), `beforematch`, and autofill `input`. Dioxus neither drops nor coalesces it. It also survives an unacknowledged render and a hidden webview, which facts do not.

Labels: V = read in the repo or in the pinned Dioxus 0.7.9 source (`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`), W = web source (URL given), I = inferred. Plan paths are in `/home/user/wt/l434d4/dev-docs/research/sync-js-root-2026-10-26.md` (cited as "root:line"), the bridge is in `/home/user/wt/l30/primitives/src/`, and shipped code is in `/home/user/wt/b28/`.

---

## Findings

### 1. DETAIL (borderline DESIGN), major. Rust-caused transitions fence Rust's own follow-up writes

**Claim attacked.** root:289: "A **page transition** (a light dismiss, the user's wheel, touch or key, a form reset, **a detach**) advances the slot's epoch". The rule (root:290) applies a Rust write iff `based_on >= last_page_transition`.

**Evidence.** Some page-side transitions are consequences of applying a Rust write: Dioxus removing a node, the executor's own `showPopover`/`showModal`, or the focus owner's own `focus()`. Rust decided such a write before the consequence existed, so its next write in the same decision always carries an older `based_on`, and the rule drops it.
- **Refocus after a re-mount (V, I).** `collection.rs:513-523` (`control_mount_focus`) re-applies DOM focus when the focused item mounts again: re-keyed rows, a Suspense swap, async data that replaces the ids.
  - The removal runs the focus fixup, which is "a detach": the epoch moves to e+1.
  - The write from the new element's `onmounted` carries e, so it is dropped on both renderers.
  - Focus stays on `<body>`, and the next arrow key goes nowhere. Today's code recovers.
- **Scroll-lock restore (V, I).** `scroll_lock.rs:267-298` captures `window.scrollY` before `showModal()` because the dialog's autofocus scroll "reliably shifts `window.scrollY` by several hundred pixels". It restores the capture on unlock.
  - The jump is a page transition on the document scroller, so the restore (based on the capture's epoch) is dropped.
  - That is exactly the case the mitigation exists for. The document scroller is not in the slot table at all, though Rust writes it here (`:357-360`).
- **Menubar hover A→B→A, desktop (V, W, I).** Menus are `popover="auto"` (`menubar.rs:785-805`). `showPopover(B)` runs "hide all popovers until" and hides A synchronously, which fires `beforetoggle` (W, <https://html.spec.whatwg.org/multipage/popover.html#show-popover>).
  - On desktop Rust decides "open A" from `based_on` eA before the port delivers A's stack-hide (eA+1), so the open is dropped. A shows nothing while Rust holds `open_menu == A`, and `aria-expanded` disagrees with the screen.
  - On web, the port runs synchronously inside the executor's eval, so the later hover event already sees eA+1: the renderers diverge.
- **Executor `openFocus` and dialog focusing steps (I).** The open step moves focus (`openFocus`, autofocus) before the focus owner's flush of Rust's "focus the name field" arrives (a bridge op issued from an effect). If that consequence mints an epoch, the explicit focus is dropped.
- **Anchoring and re-snap after a Rust insert (W, I).** Scroll anchoring (<https://drafts.csswg.org/css-scroll-anchoring-1/>) and re-snapping (<https://drafts.csswg.org/css-scroll-snap-1/#re-snap>) move a scroller as a consequence of Rust's own render. If they count as transitions, they fence S4/V6's "scroll to item" and W8's `scroll(end)` that the same render decided.

**Concrete change.**
- A transition is either **page-originated** or **Rust-caused**.
- A Rust-caused transition is reported, so Rust adopts the resulting level, but it **never advances the fence**. It covers:
  - anything that happens synchronously inside an owner's own apply call (the executor's show and close, the focus owner's `focus()`, the scroll owner's write);
  - a consequence of edit application that the executor's `pageObserver` records name: a removed node that contained `activeElement`, or a removed showing popover, whose hide fires no event.
- "A detach" leaves the list of page transitions.
- In FIFO the causing write always precedes the follow-up, so no sequence number is needed on the page.
- The order witness gains one counterfactual: a Rust follow-up write after its own consequence.

**Subsumes.** All five instances, and the menubar web/desktop divergence. **Why not DESIGN:** the rule, its key and its carriers stand; what counts as a page transition is the rule's parameter. If the owner reads "what advances an epoch" as the rule's ground, this is the one DESIGN item in this lens.

### 2. DETAIL, major. The transition port has no home that does not reverse a section 5.2 decision

**Claim attacked.** root:291 describes "one hidden element that **the bridge's root renders once** and that receives a synthetic `input` event".

**Evidence.**
- V: the bridge renders nothing. It lives in root contexts and root-scope effects (`l30 bridge/transport.rs:163, 177, 267, 275`). An element with a Dioxus listener must come from some component's rsx, and the library cannot insert one into `ScopeId::ROOT`'s output, which is the user's App. `portal.rs` needs a user-placed `PortalOut` for the same reason.
- V: root:667 rejected loading by `asset!` plus `document::Script` because "it needs a provider in every app". A root port is that provider.
- Per-scope ports are not a clean fallback:
  - V: to reach Dioxus, the synthetic event must bubble. `input` is a bubbling name (`dioxus-core-types-0.7.9/src/bubbles.rs`, `"input" => true`), and its only listener is on the app root (`dioxus-interpreter-js-0.7.9/src/ts/core.ts:117-130`).
  - V: Dioxus then calls every vdom ancestor's `oninput` (`dioxus-core-0.7.9/src/runtime.rs:400-470`), for example a user's `form { oninput }` around a Select.
  - V: every DOM ancestor's page listener also sees the event, including the value owner's own document `input` listener (root:294), which recursion must guard against.
- V: inside a `<form>`, desktop serializes the whole form into every port event. `serializeEvent` calls `serializeInputEvent` for type `input` (`serialize.ts:103-111`), which calls `extractSerializedFormValues` and then `new FormData(form)` (`serialize.ts:432-462`).
  - On Android the body travels base64 in one request header (`native.ts:575-587`; root:167 states the limit as about 12 KB), and a failed request loses the event silently.
  - A transition inside a large form is therefore dropped on Android.
- V: before hydration, and inside a streamed suspense subtree that has not hydrated yet, the port has no `data-dioxus-id`. `walk_event_for_id` returns `None`, and the handler returns silently (`dioxus-web-0.7.9/src/dom.rs:93-97, 146-186`).

**Concrete change.** Decide the home explicitly, as an owner call:
- (a) **Recommended:** one root port in a mandatory `BridgeRoot`, placed outside any form. This reverses root:667, so the starter, the registry smoke and the docs need an install step.
- (b) Otherwise, per-scope ports. Their handler calls `stop_propagation()`, which works for the vdom (root:108), and they never sit inside a `<form>`: each one renders as the panel's sibling outside the form when the host is in one, or falls back to (a).
- Either way:
  - add a liveness handshake (transitions queue page-side until the port's first acknowledgement);
  - the value owner ignores the port as a target;
  - a native `stopPropagation` sits on the Dioxus root after Dioxus's own listener, so document listeners and extensions never see the synthetic event.

**Subsumes.** Spurious user `oninput` calls, Android loss inside forms, and silent loss before hydration.

### 3. DETAIL, major. The port's rate rule contradicts the slot table, and its trigger must sometimes predict the level

**Claim attacked.** root:291 ("only **low-rate** transitions qualify (dismiss, toggle, reveal, reset, detach), never a stream") against root:305 ("the transition port for a controlled value that an rsx `oninput` reads") and root:303 (focus: "the port if an rsx handler decides on it").

**Evidence.**
- I: per root:305, every keystroke in a controlled input takes the port. On desktop that is two blocking requests per keystroke: the rsx `oninput`, then the port.
- V: focus is one document-global slot (root:296), and rsx handlers decide on it all over the codebase: `tabs.rs:389` `onfocus`, `date_picker/segment.rs:536-543` `onfocus`, `virtual_list.rs:487-493` `onfocusin`. So "rsx-read" holds globally, and every focus change in the app takes the port.
- Port triggers, checked one by one:
  - popover: `beforetoggle` is synchronous. Holds.
  - dialog: `close` is queued on the user-interaction task source, the same source as the next key, so it runs in FIFO order before that key (W, <https://html.spec.whatwg.org/multipage/interactive-elements.html#close-the-dialog>). Holds. (A popover's `toggle` is on the DOM manipulation source, so it may lag input; use `beforetoggle`.)
  - `beforematch`: synchronous. Holds.
  - `reset` is the exception. It fires **before** the controls reset (W, <https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#concept-form-reset>). For a user-activated reset button, a microtask queued in the listener runs before the reset steps, because the stack is empty after the listener returns (W, <https://html.spec.whatwg.org/multipage/webappapis.html#clean-up-after-running-script>). For a scripted `form.reset()` it runs after them.
- I: a level read at the `reset` event or in its microtask is therefore the pre-reset text, which is root:294's "text that reappears after a form reset" again.

**Concrete change.**
- The port's class is "discrete, human-rate". Keystrokes and focus changes are in; frame streams are out. The extra request per keystroke and per focus change is priced in `bridge-cost`.
- The value owner mints at `reset` with the **predicted** level (`defaultValue`, `defaultChecked`, `defaultSelected`), never the read level.

### 4. DETAIL, major. The vdom-write gate is a spelling gate. Its escape breaks SSR, and its command mirrors secrets into the DOM

**Claim attacked.** root:294: "Rust publishes `data-dx-*` (the desired state and `based_on`)", and "A gate bans rsx `value:`, `checked:`, `selected:`, `open:` and `hidden:` ... outside the `initial_*` forms". `js-interop.md:57` gives the same rule to agents.

**Evidence.**
- **SSR (V).** `dioxus-ssr` writes the attribute name verbatim (`renderer.rs:146-158, 482-492`, with special handling only for `dangerous_inner_html`). So `initial_value: x` reaches the markup as `initial_value="x"` and `initial_checked: true` as `initial_checked=true`: neither is HTML. On the client those names set `defaultValue`/`defaultChecked` (`set_attribute.ts:38-56`), but hydration skips mutations, so they are never set there either.
- **First paint (V).** A `data-dx-*` command is applied only when the bridge installs, after wasm load, hydration, the first effect and the bundle eval (root:667). Under the gate:
  - every controlled field paints blank and every checkbox unchecked on the SSG site and without JS;
  - `collapsible.rs`'s `HIDDEN` becomes a command, so every closed panel paints **open** until install.
- **Shipped writes an rsx scan cannot see (V).**
  - `Attribute::new("HIDDEN", ..)` (`collapsible.rs:41-47`).
  - `attributes!(input { value: text() })`, spread through `InputGroupInput` (`preview/src/components/date_picker/variants/natural_language/mod.rs:50-54`).
  - `Input { value: "{value}", oninput: sanitize }`, a transforming controlled input (`preview/src/components/color_picker/component.rs:370-385`).
  - A quoted `"value"` or `"VALUE"` builds a non-volatile custom attribute (`dioxus-rsx` attribute.rs:191-196). It still goes through `setAttributeInner`'s case-sensitive switch: `"value"` sets the property, while `"VALUE"` sets the content attribute, which is the default value and also the live value until the control is dirty.
- **Secret mirroring (V, W).** Today Dioxus writes only the `value` *property* (`set_attribute.ts:24-35`). A `data-dx-value="<text>|<based_on>"` attribute exposes typed text to CSS attribute selectors, which is the CSS-keylogger class (W, <https://github.com/maxchehab/CSS-Keylogging>; it works where a framework mirrors the value into an attribute). OTP codes are affected (`input_otp.rs:434` is a listed site), and so are a consumer's password `Input`.
- **IME (I).** Setting `value` during composition commits or cancels it. The owner must hold writes until `compositionend` and then re-check `based_on`.

**Concrete change.**
1. `value`, `checked` and `selected` commands travel as owner writes over the bridge (property only, keyed by slot, carrying `based_on`), never as an attribute.
2. One library helper emits the native attribute (`value`, `checked`, `hidden`) at first render from a **mount-frozen** value (`use_hook`), through a non-volatile name. SSR markup and the client mount then agree, and no later diff writes the attribute.
3. The form-control wrappers (Input, Textarea, NativeSelect, Checkbox, Switch, RadioGroup) take `value`/`checked` out of their spread `Vec<Attribute>` at runtime and route them to the owner, so a caller's `Input { value }` is covered.
4. The gate becomes a **sink** gate, as root 5.3 asks everywhere else. It catches any `Attribute` built by rsx, `attributes!`, `Attribute::new` or a quoted name whose name is case-insensitively one of the five, on a page-writable element in library code, except the helper. It is type-aware: a radio's or checkbox's `value` is not page-written.

**Subsumes.** The SSG blank fields and open-panel flash, the three missed sites, CSS exfiltration, and IME cancellation.

### 5. DETAIL, major. The closed bridge's own verbs write page-written slots outside every owner

**Claim attacked.** root:325, "One-shot calls (focus a cell, scroll an item into view, measure now) stay bridge calls ... they have no level to reconcile", against root:290-296 (every Rust write to a page-written slot goes through its owner) and root:665's `FORBIDDEN_VERBS` (capture, `commit`, plain `focus`, and `showPopover`/`showModal` on reconciled kinds).

**Evidence.**
- V: `Method` carries `HidePopover`, `TogglePopover`, `Close`, `Blur`, `ScrollIntoView`, `ScrollTo`, `ScrollBy`, `SetSelectionRange` and `Select` (`l30 bridge/mod.rs:259-301`). The page has matching rows (`ts/bridge.ts:253-260`).
- V: `check-bridge-behaviours.sh:361-374` bans only `SetPointerCapture`, `ReleasePointerCapture`, `Focus` and `Commit`.
- Each remaining row writes the open, focus, scroll or caret slot with no `based_on`.
- **Instance (V, I): OTP click-to-slot.** `snap_caret_to_slot` (`input_otp.rs:603-648`) writes the caret after a click.
  - Desktop: click, then an immediate paste. The caret write arrives after the paste and carries a stale `value_len`, so it clamps to slot 0 and sends 0 back into `selection_start` over the paste's report. The next keystroke replaces character 0.
  - Web: the same race exists across the async `spawn`.

**Concrete change.**
- Partition the verb table by the slot table: a verb that writes a slot in the table exists only as that owner's operation, with `based_on`:
  - `scrollOwner` scroll writes;
  - focus owner `focus`/`blur`;
  - the value owner's selection;
  - the executor for open kinds.
- Derive `FORBIDDEN_VERBS` from the slot table and check it on both `Method` and `METHODS`.
- Delete root:325's sentence. A "focus a cell" call already contradicts the landed gate.

### 6. DETAIL, major. The scrollOwner's classifier: as written it drops keyboard scroll-into-view on desktop, and "no input" is not one class

**Claim attacked.** root:296, "an input classifier (wheel, touchstart, keydown and a scrollbar pointerdown **on the scroller**)".

**Evidence.**
- **(a) Classification by input instead of by realized default action.**
  - V: a listener on the scroller runs before the Dioxus root listener, its ancestor (`core.ts:117-121`).
  - V: grid, listbox and tabs arrow keys are cancelled in rsx (`tabs.rs:391-417` is the pattern).
  - I: so the classifier mints a scroll transition for a key that never scrolls. On web the fact's microtask runs before the root listener, so Rust's `seen` is current. On desktop the fact waits behind the acknowledgement, and the scroll-into-view write that the same keydown decides is dropped.
  - Result: under key repeat in a virtual grid or listbox, the active item leaves the viewport, on desktop only.
- I: with focus on `<body>`, Space and PageDown scroll the last-clicked scroller, so "keydown on the scroller" also misses real user scrolls.
- **(b) Scrolls with no input.** These are anchoring (W8 is built on `overflow-anchor`), re-snap (W, finding 1), find-in-page of visible text (no event at all), screen-reader scrolling (I: gestures consumed by the AT produce no DOM input), and fragment or history restoration.
  - No page signal separates "must not fence" (anchoring, re-snap after a Rust render) from "must fence" (find, AT).
  - Shipped conflict (V): VirtualList's find-in-page reveal (`virtual_list.rs:386-400`) needs find's own scroll to stay neutral, while `stickToEnd` needs an AT or find scroll-up to detach.

**Concrete change.**
- Classify user input by its realized default action, from one page-lifetime passive window bubble listener that reads `defaultPrevented`. Add it to the `listener-inventory` baseline, as `modality.rs`'s listener is.
- Each owner instance declares an **intent kind**, which decides what "unexplained" means:
  - one-shot intents (reveal, ensure-visible): unexplained = neutral;
  - a level mode (`stickToEnd`): detach is decided by the level (distance from the end) at each scroll, not by input;
  - a snap scroller (carousel): realized touch, wheel or key = transition; re-snap = neutral.
- Add the document scroller to the table as a Rust-written scroller (the scroll lock).

### 7. DETAIL, major. Missing slot: a contenteditable's text, written by the user and by the vdom (live defect)

**Evidence.**
- V: date segments are `contenteditable` spans whose text is a vdom child (`date_picker/segment.rs:519-538`, `{display_value}`).
- V: digits cancel the default action (`:271-281`), but Backspace and Delete do not (`:283-299`). Neither does any key outside the match, including Android's `Unidentified` (keyCode 229), which falls to `_ => ()` (`:380`).
- I: so the browser's editing default action edits the same text node that Dioxus rewrites with `SetText`: it deletes characters, inserts IME text, and can replace the node or insert a `<br>`. If the node is replaced, Dioxus keeps writing to a detached node.
- Scope: Backspace on both renderers, every digit on Android (web and the mobile app).
- Neither the slot table nor the gate (finding 4) covers rsx text children.

**Concrete change.**
- Add a slot row, "contenteditable subtree".
- A Rust-rendered contenteditable cancels every editing default action: keys by rsx `prevent_default`; `beforeinput` through an `otpFilter`-like behaviour, since Dioxus 0.7 has no `onbeforeinput`. The alternative is to drop `contenteditable` for a real input with `inputmode`.
- A gate rule: no rsx text child inside a contenteditable without that behaviour.

### 8. DETAIL, major. Auto-advance focus decided by Rust logic is neither roving nor `aria-activedescendant`

**Claim attacked.** root:312, condition 3: "Every roving widget is arrow-key driven", and the "this event's task" row (a focus for the next input is a named behaviour's effect).

**Evidence.**
- V: segments move focus after digits (`FocusNext` from `push_commit_text_effects`, `segment.rs:135-157`) and on an emptying Backspace (`FocusPrevious`). The effect is applied through `ctx.focus` (`:455-466`), which reaches the DOM through `MountedData::set_focus` (`collection.rs:513-523`).
- **Desktop trace (I), typing 1 2 0 5:**
  - after "12", `FocusNext` is an asynchronous write;
  - the "0" keydown targets the month span, which is still DOM-focused;
  - month's handler sees `current_text` "12" at `max_length`, resets, and commits "0", so the month is corrupted;
  - web holds, because `set_focus` is synchronous there.
- I: the plan's placement would put this in a TypeScript behaviour that re-implements the segment parser, which breaks "Rust first".

**Concrete change.**
- Generalise condition 3: when the target of the next keystroke is a Rust decision, the container routes edit keys by Rust's active index. Keys arrive in order on the vdom path on both renderers.
- Every editing default action is cancelled (finding 7).
- DOM focus follows asynchronously through the focus owner.
- `aria-activedescendant` is the special case of this rule where DOM focus never moves.

**Subsumes.** Date segments, and any auto-advance field.

### 9. DETAIL, major. Stream boundaries belong on the port, not on a session

**Claim attacked.** root:293, "A stream that stays on its own session (a drag's start, move and end) is combined with no fact of another source", and root:309, Slider's fix "every phase of the drag through one bridge session, or through the pump".

**Evidence.**
- V: Slider and Resizable start in rsx (`move_interaction.rs:119-146`).
- I: a bridge `up`, whether on a session or on the pump, can never be ordered with the next rsx `pointerdown` on desktop. Round 5's fix works only if the start leaves rsx, which pushes the pointerdown decision into page code.
- I, drawer: the user releases a drag and presses Escape at once.
  - On web, `end` (snap-open) runs, then Escape closes: closed.
  - On desktop, Escape's `cancel` takes the port inline, and then the queued `end` fact's snap decision writes open with a current `based_on`: open.

**Concrete change.**
- Start, end and cancel are discrete transitions on the port. Per-move frames stay latest-wins on the session.
- Reword "one inbound pump is the only page-to-Rust bridge path" as: the port for transitions an rsx decision reads, the pump for facts, and the session for per-frame payload.

**Subsumes.** Round 5's finding 8 without moving Slider's start page-side, and the drawer and carousel drag-end races.

### 10. DETAIL, minor. Slot keys by scope id collide: ScopeIds are reused

**Evidence.**
- V: `self.scopes.vacant_entry()` (`dioxus-core-0.7.9/src/scope_arena.rs:25-26`) reuses freed ids.
- V: root:168 says the desktop vdom routes ahead of the DOM.
- I: a light dismiss of the old panel (scope 42, still painted) reaches the new component that reused 42.

**Change.** Key a slot by `(scope id, mount nonce from use_hook)`.

### 11. DETAIL, minor. A root pump outlives its subscribers, and `Callback::call` panics after the origin scope drops

**Evidence.**
- V: `Callback::call` does `self.callback.write()` (`dioxus-core-0.7.9/src/events.rs:519-529`), and `write()` is `try_write().unwrap()` (`generational-box-0.7.9/src/lib.rs:107-108`).
- V: today a session's receive task lives in the subscriber's scope and dies with it (`l30 bridge/session.rs:424-426`).
- I: a fact still in flight when `Subscription` drops reaches a dead callback.

**Change.** The pump dispatches through a registry entry that `Drop` removes, or through `try_write`.

### 12. DETAIL, minor. Coupled slots need one owner per element, and the focus/scroll coupling rule loses visibility

**Evidence.**
- W: checkbox activation sets checkedness and clears `indeterminate`, and a cancelled activation restores both (<https://html.spec.whatwg.org/multipage/input.html#checkbox-state-(type=checkbox)>). Yet root:303-306 lists `indeterminate` as no page-written slot and splits `checked` (native owner) from `indeterminate` (executor).
- W: `preventScroll` applies to every ancestor at once, and `scrollIntoView` scrolls every ancestor (<https://drafts.csswg.org/cssom-view/#scroll-a-target-into-view>). So root:296's "`preventScroll` wherever the scroll slot has an owner" leaves the focused element off-screen in unowned ancestors.
- V: the `<details open>` row has no consumer (grep: no `details` element in `primitives/src` or `preview/src/components`).

**Change.**
- One owner per element for `checked` and `indeterminate`.
- The focus owner passes `preventScroll` and asks each owned ancestor's `scrollOwner` for an ensure-visible intent, scrolling unowned ancestors itself.
- Drop or mark the `<details>` row.

### 13. DETAIL, minor. Before boot: `data-hydrated` fires before the owners exist, and the install order is unstated

**Evidence.**
- V: `preview/src/main.rs:262` sets `dataset.hydrated` from the App root effect. The bridge bundle installs at first use (root:667), after that.
- I: a spec that uses `gotoHydrated` and presses Escape can run before `hotkey` exists.
- root:294 has owners "report the live level", and the reconciler "reads the level at install" and applies commands from the markup. Unless the report runs first, an SSG command with `based_on` 0 overwrites text typed before hydration.

**Change.**
- Set `html[data-dx-bridge="ready"]` after every owner's install report, and have `gotoHydrated` wait for it.
- Fix the install order: report the level, then apply commands.

### 14. DETAIL, minor. OTP paste: `maxlength` truncates before the filter runs (live defect)

**Evidence.**
- V: `otpFilter`'s `beforeinput` test requires `typeof e.data === 'string'` (`input_otp.rs:376-380`).
- W: `data` is null for `insertFromPaste`; the content is in `dataTransfer` (<https://w3c.github.io/input-events/#overview>).
- I (engine behaviour): the browser trims the paste to `maxlength` first. A pasted "123-456" becomes "123-45", which filters to "12345".

**Change.** `otpFilter` takes `insertFromPaste` and `insertReplacementText` itself: cancel, filter, insert.

### 15. DETAIL, minor. The `auto` popover stack cuts the executor's exit wait

**Evidence.**
- W: `showPopover(B)` hides every non-ancestor `auto` popover synchronously (finding 1's citation).
- W: hiding A cancels its exit animation, because `display: none` terminates animations (<https://drafts.csswg.org/css-animations-1/#animations>), and a cancelled animation rejects `finished` with `AbortError` (<https://drafts.csswg.org/web-animations-1/#cancel-an-animation>).
- I: root:321's "closes before opens" plus the exit wait cannot animate A out while B opens.

**Change.**
- The exit wait treats `AbortError` as done, and the stack hide as Rust-caused (finding 1).
- Menus that must animate out while the next one opens use `popover="manual"`, with Rust's layer-tree invariant as the only closer.

---

## Scenarios traced (fresh; web and desktop)

**A. Date picker: typing segments while Rust normalises.**
- Web: digits hold.
- Desktop: breaks (finding 8).
- Backspace and Delete: the browser and the vdom both write the span on both renderers (finding 7).
- Android IME (web, and the mobile app): every digit lands in the DOM unfiltered (finding 7).

**B. OTP: pasting while the caret advances.**
- Both renderers: a formatted code loses digits (finding 14).
- A click-to-slot just before the paste puts the caret back to slot 0 through a generic `SetSelectionRange` verb (finding 5).
- With the plan's fixes: holds.

**C. Tabs or a collection re-keyed by an async load.**
- Today: refocus holds through `control_mount_focus`.
- Under the plan: the refocus is dropped on both renderers (finding 1).

**D. Menubar: hover-switch while a submenu closes.**
- A→B cuts A's and its submenu's exit (finding 15).
- A→B→A within one acknowledgement window: desktop drops the reopen, web applies it, so the renderers diverge (finding 1).

**E. Virtual grid or listbox under key repeat at the window edge.**
- Desktop: under the plan as written, the active item scrolls out of view (finding 6a).
- Web: holds.

**F. Virtual list: the focused row scrolls out.**
- Holds on both renderers: rsx `onfocusin` pins the row (`virtual_list.rs:378-420`, 487-493), and windowing is latest-wins.
- Removing the pinned item's data drops focus to `<body>`; the plan neither helps nor hurts here, beyond finding 1.

**G. Drawer: drag end against Escape.**
- The renderers diverge (finding 9).

## Attacks that failed

- **Dioxus drops or coalesces the port event.**
  - Web: the handler is `Fn`, so it may be called recursively (`dom.rs:84-87`).
  - Web: a plain `Event` passes `event_type_matches` for form events (`events/mod.rs`, `convert_form_data => web_sys::Event`).
  - Neither renderer checks `isTrusted` (`dom.rs:88-116`, `native.ts` `handleEvent`).
  - Desktop: a synchronous XHR per event, no coalescing. It is handled inline while a render is unacknowledged and while the webview is hidden, so it is more robust than a fact under a stall.
- **Re-entrancy panics when an owner's apply inside a Rust effect fires the port on web.** None in Dioxus: `handle_event` takes only shared borrows (`runtime.rs:369-381`), and a task poll clones its `Rc` first (`tasks.rs:270-288`).
- **Ordering of popover and dialog transitions against the next input.** Holds: `beforetoggle` is synchronous, and `close` shares the user-interaction task source with the next key (finding 3).
- **"A `Callback` runs in its origin scope."** Verified (`events.rs:526`).
- **The `MountedData` write ban is buildable.** Inherent methods (`dioxus-html-0.7.9/src/events/mounted.rs:165-200`), so `disallowed_methods` by path works. There is no web-sys in primitives, so no `downcast` bypass.
- **A Rust write on two channels (a vdom attribute and a bridge op) to one slot.** Ordered by the effect lane rule (root:180) on both renderers.
- **Hydration of the four original reconciled kinds** (round 5): holds. Not repeated here.
