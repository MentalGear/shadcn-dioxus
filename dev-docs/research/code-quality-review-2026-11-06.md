# Code-quality review of the refactored overlay and shared code (row 436), 2026-11-06

Scope, in priority order: the overlay engine (`dismiss.rs`, `menu_root.rs`, `menu_item.rs`, `menu_sub.rs`,
`menubar.rs`, `dropdown_menu.rs`, `context_menu.rs`, `top_layer/**`, `select/**`, the combobox primitive), the newest
shared code (`form_reset.rs`, `pointer.rs`, `move_interaction.rs`, `bridge/**`, `state_attrs.rs`), and the files the
row 426-435 refactors touched. Criteria: a correctness review (each finding names inputs/state and the wrong output)
and a simplify review (reuse, simplification, efficiency, altitude). The base is main `d040554a`.

**How to read the evidence.** This lane builds and runs nothing. Every finding was verified by reading the code and the
pinned Dioxus 0.7.9 sources along a real caller path; none was reproduced in a browser. "Verified" below means the code
path was traced end to end; "inferred" means a step depends on runtime behaviour that was not observed. Nothing here
is already enforced by a gate, and none is in an open backlog row (checked against `dev-docs/backlog/`).

Counts: 12 findings. By severity: 0 high, 3 medium, 9 low. By size: S 9, M 3, L 0. One class finding (F4) and one
duplicated-construction finding (F7) are fix-by-construction proposals.

## Ranked table

| # | Sev | Where | Finding | Failure scenario or cost | Proposed fix | Size |
|---|---|---|---|---|---|---|
| F1 | Medium | `menu_sub.rs:614` | A submenu's dismiss layer is registered for as long as its panel is mounted, which includes the close-animation hold, because `active` is `None`. | ArrowLeft (or a hover leave) closes a submenu; Escape within about the animation length plus 250 ms goes to the closed submenu's layer, which is topmost, so the parent menu does not close and the key is swallowed. The user presses Escape twice. | Pass `Some(open.into())` as the host roots already do, so the layer leaves the Escape order when `open` goes false. | S |
| F2 | Medium | `lib.rs:1030-1050` (`use_animated_open`) | Its effect runs once at mount with `open == false` and takes the closing branch: it spawns a `document::eval`, waits a frame, finds no element and answers. | Every closed overlay on the page (each row's DropdownMenu, each Select, Popover, submenu) costs one eval round trip and one rAF at mount. 100 row menus are 100 evals on first render, on desktop 100 IPC hops. Nothing is mounted, so nothing is held. | Skip the closing branch while `show_in_dom` is false (`if !*show_in_dom.peek() { return; }` before the spawn). | S |
| F3 | Medium | `pointer.rs:22,31`, `form_reset.rs:86` | Per-page state lives in `thread_local!`: the pointer bridge guard `BRIDGE`, its `EPOCH`, and the form-reset `QUEUE`. The transport (`bridge/transport.rs`) keeps its state in the root context of the VirtualDom precisely because a second desktop window is a second webview on the same thread. | Desktop, two windows: window A starts a slider drag, so `BRIDGE` is `Some`. Window B's first drag finds `BRIDGE.is_some()`, installs nothing, and never receives `pointermove`; the drag does not move. Likewise a reset in window B queues behind window A's task and the read goes through the wrong runtime's hub. Inferred (multi-window desktop was not run); web is unaffected. | Hold the bridge guard, epoch and queue in a root context (`provide_root_context`), as `Hub` and `Loaded` do. | M |
| F4 | Low | `menu_root.rs:use_menu_panel` (the `Point` arm) with `top_layer/scroll_clamp.rs:179`, `anchor_fallback.rs:175`, `menu_root.rs:use_root_id` | Class: the effect behind `use_js_listeners` re-runs only when a signal it reads changes; the plain values it captured (point `x,y`, side, align, gap, a caller's `id`) are read at that run and not refreshed by a later render. | Context menu open, user right-clicks the trigger area again: `ctx.position` changes, `ContextMenuContentRendered` re-renders `top: {cy}px; left: {cx}px` (`context_menu.rs:721`) over the placement script's `top:0; left:0` (its writer cache does not know), while the script keeps the old point and transform. The panel lands at old transform plus new offset. Same shape: a `dir` flip while a submenu is mounted keeps the old `sub_side`. Inferred: whether the reopen goes through a close first depends on `use_open_focus_sync`. | Make the placement hook key its script on the plain inputs (`use_reactive!` or a memo of `(x, y, side, align, gap)`) so a change reinstalls; or make a re-point of an open context menu an explicit close then open. | M |
| F5 | Low | `top_layer/attributes.rs:38,48`, `anchor_fallback.rs:42`, `scroll_clamp.rs:124`, `popover_sync.rs:148,301`, `modality.rs:211`, `lib.rs:426,658`, `carousel/context.rs:182`, `carousel/root.rs:471` | A caller-supplied `id` is interpolated unescaped into a CSS dashed-ident (`anchor-name: --dxa-{id};`), into a CSS attribute selector (`[style*="anchor-name: --dxa-..."]`) and into single-quoted JS (`getElementById('{id}')`). Generated ids are safe; `DropdownMenuContent { id }`, `SelectList`, dialogs and others accept the caller's. | `id: "file.menu"` or `":r1:"` makes `position-anchor: --dxa-file.menu;` an invalid declaration, so CSS anchoring never resolves and only the JS fallback places the panel. An id with `'` is a syntax error in the popover show script, so `showPopover()` never runs and the panel stays `display:none` with no console cue; an id with `"` throws in `querySelector`. | One `css_ident(id)` for the two style builders and `serde_json`-quoted ids in the scripts (the bridge already sends ids as data, so these sites also move when row 434 ports them); a unit test with `a.b`, `a'b`, `a"b`. | S |
| F6 | Low | `select/components/select.rs:559`, `toolbar.rs:108`, `navigation_menu.rs:304`, `carousel/indicators.rs:88`, `drag_and_drop_list.rs:457`, `slider.rs:247`, `command.rs:256-257` | Class: `ReadSignal::new(Signal::new(<constant>))` written in a component body allocates a new signal in the component's scope on every render; `Signal::new` is owned by the current scope and freed only when the scope drops (dioxus-signals 0.7.9, `Signal::new` doc). | Slider re-renders per drag frame and allocates a signal per render for `min_steps`; the fresh signal also makes the `SliderImpl` props differ on every render, so memoisation never skips it. The others leak one slot per re-render of the root. Small per render; unbounded over a long session. Inferred (no heap measurement). | A `use_constant_signal(value)` helper (`use_hook(|| ReadSignal::new(Signal::new(value)))`) and a grep gate for `ReadSignal::new(Signal::new(` outside `use_hook` and `#[props(default`. Subsumes all seven; the 98 `#[props(default = ReadSignal::new(Signal::new(..)))]` sites are the same allocation in the parent's scope and are not covered. | S |
| F7 | Low | `dismiss.rs:260` (`OpenGap`), `menu_sub.rs:124` (`DelayedAction`), `context_menu.rs:394`, `typeahead.rs:162`, `select/components/select.rs:235`, `date_picker/context.rs:95` | "One cancellable delayed task kept in a `Signal<Option<Task>>`" is written six times. `OpenGap::defer` is `DelayedAction::schedule(Duration::ZERO, ..)` with `pending` on `peek` instead of `read`; the context menu's long press is the same with a position beside it. | Cost is drift, not a crash: the two `pending` flavours already disagree about subscribing, and a fix to one copy (cancel on unmount, clear on run) does not reach the others. | Move `DelayedAction` to `sleep.rs` (public to the crate), build `OpenGap` on it, and port the context menu and typeahead to it. Subsumes `OpenGap`, `DelayedAction`, `long_press_task`; the typeahead and date-picker timers need a look first. | M |
| F8 | Low | `form_reset.rs:39`, `menu_root.rs:85` | Two `caller_id(&[Attribute])` helpers that differ: `menu_root` ignores an empty or namespaced `id`; `form_reset` takes any `Text` named `id`. | `Input { id: "" }` (a conditional id that is empty): `form_reset` writes `""` into the id signal and returns no generated id, so the control carries no id and the reset listener finds nothing; a form reset is never reported to the owner. | One `attribute_text(attributes, "id")` in `lib.rs` with `menu_root`'s rules, used by both. | S |
| F9 | Low | `select/components/list.rs:334,507` | `spawn(async move { _ = listbox_ref.set_focus(true); })` drops the future unawaited; every other `set_focus` site in the crate awaits. The two arms also carry the same effect twice. | Works on web only because dioxus-web's `set_focus` performs the focus before it returns the future; on a renderer whose `set_focus` is lazy the listbox container never takes focus and keys go nowhere. Inferred for desktop (webview query is sent at creation, so probably fine). | `.await` it, and share the effect between the two arms. | S |
| F10 | Low | `move_interaction.rs:119-125` | `start_pointer` calls `prevent_default` and `stop_propagation` before it tests for the primary button. | A pen barrel-button or middle-button press on a slider thumb is consumed (default prevented, ancestors never see the `pointerdown`) and starts nothing. | Test the button first, then consume the event only for a press that starts a drag. | S |
| F11 | Low | `lib.rs:360` (`DISMISS_JS`) | The Escape handler does not check `e.isComposing`. | A Combobox input (Escape policy `close`) in an IME composition: Escape should end the composition; the layer also cancels the key and closes the list, losing the composition's list. Inferred (needs an IME). | `if (e.isComposing) return;` first in the keydown listener; the Rust spec in `dismiss.rs` pins the script text. | S |
| F12 | Low | `ts/bridge.ts:509` (`sweep`) | The element-lifetime observer is a document-wide `MutationObserver` whose callback copies and scans every registration (`[...elementRegs]`, `isConnected`) on every mutation batch. | O(registrations) per batch while any element-tied listener exists; on a streaming chat or a virtual list that mutates every frame this is a steady main-thread cost proportional to open overlays. Inferred (not measured). | Scan only the removed nodes of `records` (`removedNodes`) against a `Set`, or keep the full scan but skip batches with no removals. | S |

## Details

### F1 Submenu layer outlives its open state

`MenuSubContentRendered` calls `use_dismiss_within(sub.content_id, Some(sub.trigger_id), Scope::Element,
Policy::MENU_SUB, None, ..)`. `None` is the `active` argument, so the page-side layer (`layers.push(o)` in
`DISMISS_JS`) is added when the component mounts and removed when it unmounts. The component mounts while
`use_animated_open` says `render()`, and that stays true for the close animation plus a 250 ms hold
(`lib.rs:1043-1075`). In that window the submenu's layer is the highest `seq`, so `layers[layers.length - 1] === o`
holds for it, `e.preventDefault()` runs and `escape-within` goes to `close(Some(cause))`, which closes an already
closed submenu and spawns a focus hand-back to the sub-trigger. The enclosing menu's layer is not the last one, so
it ignores the key. The host layers do not have this problem because they pass `Some(open.into())`. The module
doc in `dismiss.rs` states the rule ("a layer takes its place in the Escape order each time it becomes active"), so
the omission is a slip, not a design choice. Measured: no. Verified: the code path above.

### F2 Closed overlays pay for a close animation that never started

`use_animated_open` runs its effect on mount. With `open == false` it takes the `else` branch and calls
`dioxus::document::eval(..)` with a script that waits `requestAnimationFrame`, looks the element up (absent), and
sends `true`. `show_in_dom` is already `false`, so the result changes nothing. Callers: `DropdownMenuContent`,
`ContextMenuContent`, `MenuSubContent`, `MenubarContent`, `SelectList`, `ComboboxList`, `PopoverContent` and the other
`render()` gates, mounted while closed by design (the gate is inside them). On a table with a menu per row the cost is
one eval per row at first paint; on desktop each is an IPC round trip. The fix keeps the generation counter intact:
bump it, then return when there is nothing to hold.

### F3 Thread-wide state where the rest of the bridge is per-webview

`bridge/transport.rs` documents "Installed is remembered in the root context of the VirtualDom, so it is per webview
... never a process-wide flag". `pointer.rs` and `form_reset.rs` predate or ignore that rule: `BRIDGE`, `EPOCH` and
`QUEUE` are `thread_local!`, which on desktop is shared by every window running on the main thread (and by SSR requests
on a server thread, where they are inert). `ensure_bridge` returns early when `BRIDGE` is `Some`, so a second window
never installs its `pointermove` listeners. `POINTERS` is a `GlobalSignal`, which Dioxus scopes per runtime, so the
two halves of the same feature already disagree about their owner. `dismiss.rs`'s `NEXT_SEQ` is also thread-local but
harmless: it only needs to increase.

### F4 Plain values captured by a signal-driven effect

`use_effect_with_cleanup` wraps the closure in `use_effect`, whose body is a `use_callback` that is replaced every
render, but the effect itself only runs again when a signal read inside it changes. In `use_placement` the closure
reads `open` (a signal) and uses `anchor`, `side`, `align`, `gap_px` (plain values). So a render that changes only
those never reaches the page. The same holds for `menu_root::use_root_id`, which writes a caller's `id` once.
Instances found: the context menu's point (`PanelPlacement::Point { x, y }`), a submenu's `sub_side` after a
direction change, and the caller `id` of a menu root. Not instances: the dismiss layers (they read `root`, `also` and
`active`, all signals).

Fix by construction: let `use_placement` take its inputs through a single `use_memo`/`use_reactive!` tuple, so the
script is reinstalled when any of them changes; this is the same remedy `form_reset.rs` already uses for the caller
id (`use_reactive!(|user_id| ..)`). The reinstall also assigns a new Escape `seq` only for dismiss layers, which this
change does not touch.

### F5 Ids reach CSS and JS unescaped

`anchor_name_style` and `position_anchor_style` build `--dxa-{id}`; a dashed-ident cannot hold `.`, `:`, a space or a
quote without escaping. The placement scripts then look the trigger up with `[style*="anchor-name: --dxa-" + id +
";"]`, which breaks the selector on `"`. Ids generated by `use_unique_id` are safe, which is why no spec catches it.
The seven `getElementById('{id}')` sites are part of row 434's port list; the CSS and selector sites are not, and
moving them to a quoted/escaped form is independent of that port.

### F6 Constant signals built in a render body

`use_collection_provider(ReadSignal::new(Signal::new(true)))` and its siblings create a signal per render. The
collection hook reads the signal reactively, but never needs a fresh one. `Slider` is the visible case: the
`min_steps: ReadSignal::new(Signal::new(0u32))` prop is rebuilt inside `rsx!`, so `SliderImpl`'s props compare
unequal on every `Slider` render. A helper that builds once removes the leak and the defeated memoisation together.

### F7 Delayed single task, six copies

`DelayedAction` in `menu_sub.rs` is already generic (`schedule`, `cancel`, `is_pending`). `OpenGap` in `dismiss.rs`
re-implements `schedule(ZERO, ..)` and a `pending` that peeks. The context menu inlines the same pattern with a
position next to the task and a 700 ms disarm. A shared type removes the divergence in `pending` (read vs peek) and
gives one place to cancel on unmount.

### F8 Two readers of a caller's `id`

`menu_root::caller_id` returns `None` for an empty id and for a namespaced attribute; `form_reset::caller_id` does
not. For `id=""`, `form_reset` sets the id signal to `""`, `use_form_reset_value` returns an empty attribute list
(it believes the caller supplies the id), and the listener registered on `""` finds no element. One reader removes the
disagreement.

### F9 Unawaited focus

`set_focus` returns a boxed future. dioxus-web 0.7.9 does the focus before returning it (`mounted.rs:130`), which
hides the missing `.await`; dioxus-desktop sends its query when the future is created. Awaiting it costs nothing and
matches the other sites, which carry a `focus-ok:` marker for the gate.

### F10 Consume the pointer event after the button check

The early `prevent_default` and `stop_propagation` run for every pointer button. Only a primary press starts a drag.

### F11 Escape during composition

`DISMISS_JS` registers `keydown` on `document`; browsers send Escape with `isComposing === true` while an input method
is composing. The check is one line; the existing script-text test (`the_page_script_speaks_the_messages_rust_reads`)
is the place to pin it.

### F12 Page-wide mutation scan

`watchElement` creates one `MutationObserver` on `documentElement` with `subtree`, shared by all element-tied
registrations; each batch runs `sweep`, which copies `elementRegs` and tests `isConnected` for each. The design
comment (SEAM, row 434 step 3.0b) already expects this to become the single reconciler observer, which is the right
moment to scan `removedNodes` instead.

## Checked and found sound

- `bridge/transport.rs`: the outbox, ask/answer slots, closed-channel settle and `merge` keep the order; a failing
  op in a pump message still answers (`transaction` catches per op), so an `ask` cannot hang on a bad op.
- `bridge/session.rs`: `Subscription` drop cancels the lease task and tears down on the existing channel.
- `form_reset.rs`: one read for the whole form is the right construction; a failed transaction drops the queue and
  the next reset schedules again.
- `menu_item.rs`: one `render_menu_item` for all hosts; the three `SelectOn` models decide per pointer type.
- `menubar.rs`: the always-rendered `role="menu"` wrapper is pinned by `playwright/menubar.spec.ts`, not a finding.
- `dismiss.rs`: `escape_in_gap` and `OpenGap` order (cancel before close) are consistent with the layer's behaviour.

## What happens next

The owner reviews this table. Each accepted finding becomes its own backlog row (S/M as sized); F4 and F6 are the two
by-construction fixes and should be one row each. Nothing here is built.
