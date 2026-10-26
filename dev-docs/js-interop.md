# JavaScript interop: Rust first, the bridge second, no script strings

**Read this before you write any interactive behaviour.** It is short on purpose. The plan is backlog row 434 step 3
([`research/sync-js-root-2026-10-26.md`](./research/sync-js-root-2026-10-26.md), amended by seven adversarial boards, the
last being [round 7](./research/board-2026-10-26-r7/SYNTHESIS.md): converged in substance, not at zero; **the paper review
is closed** and the next evidence is built code); this page is its agent-facing form. The plan is backlogged as rows 434
(umbrella) and 444-467, with the live defects the boards found filed as bug rows 468-471 and 473-475, **awaiting the owner's
approval and scope**. **Much of what it
describes is not built.** The next table says what is; the "today" column of the "I want to ..." table says what to do
when the rule you need depends on something that is not.

## Landed and planned

| | on main | on `lane/js-bridge-3-0` (lane 3.0; not on main until its batch merges) | planned (row 434 step 3) |
|---|---|---|---|
| `check-js-volume` | counts per file: eval sinks (`document::eval`, `use ... eval`, `eval(`; **not** a function pointer through a module alias such as `dd::eval`), `document::*`, `Script`/`script {`/`create_script`, `dangerous_inner_html`, `web_sys`/`js_sys`/`wasm_bindgen`/`gloo_utils`/`evaluate_script`/`inline_js`/`srcdoc` (**not** `gloo_events`), JS in Rust strings, every tracked `.js`/`.ts` line (**including** `.d.ts`, `*.gen.ts` and `ts-test`), `JsListeners::install`, inline `<script>` bytes. Universe: `.rs`, `.js`/`.ts` and `.html` files, **not** `.md` or `.svg`. A rise fails against the baseline and against the merge base's; a raise is self-service with a `# raised:` note and prints under `RATCHET RAISES` | the same script and rules; its baseline gains rows for the bridge | the grant rule (a raise needs an open row with `js-grant:` at the merge base), clippy `disallowed_methods` for every sink, coverage of function pointers, `gloo_events`, `.md` and `.svg`, counting what ships, the legacy freeze for scripts and sinks (`js-fix:` grant for a bug fix) |
| older gates | `check-js-bundle`, `check-js-listeners`, `check-uncleared-intervals`, `check-blocking-scroll-listeners`, `check-theme-mode-writer`, `check-open-focus` | the same | their JS halves retire with the last legacy script |
| `check-bridge-behaviours` | none | kind parity with the generated file, behaviour modules pure, budgets (held in a dict inside the script), one place for cancel and frames, closed surface, eval only inside `bridge/`, `Program` closed and sealed, no gesture verbs | `Decision<E>` effect typing and a DOM-less compile for `ts/behaviours/`, a witness test per effect at each `site!` (item 0.3 builds the checks over those artefacts), the lease check on the generated file, single-member-call verb rows, `FORBIDDEN_VERBS` (item 0.1 owns the list; derived from the slot table), `REDUCERS`, budgets in a TSV, `check-hot-events`, the clippy ban on the `DataTransfer` setters, the native-slot sink scan (an rsx and `Attribute` scan with a shrink-only baseline, not clippy: rsx attributes expand to consts) |
| the bridge | `primitives/src/bridge/` does not exist | closed verbs over closed tables; `Program { Serve, Pump, Legacy(&str) }`, sealed to `bridge/`; one FIFO pump for element operations; latest-wins streams; an epoch on every install and a default lease on every non-passive behaviour; one test fixture; the `__dxBridgeDelayFrames` harness with the stuck setting `"never"` | `Program::Frozen`, the rest of the verb rows (each added by the first row that needs it), the closed `REDUCERS` table |
| named behaviours | none | `Behaviour::Fixture`, a test fixture, only | `otpFilter` (proof port A), `scrollLock`, `dropAnywhere`, `pan`, `settle`, `openFocus`, `stickToEnd`, `scrollOwner` (the one-shot kind first), `labelClick`, and `hotkey` (built by the first of 3.OV and 3.GS that opens, only if its witness is red or a UA-reserved shortcut needs it); `roving` only if the phase-0 focus-write witness is red; `scrollStep`, `wheelBand`, `pan`'s page-side output variants and the `release` effect are on the plan's exits list. The executor is a bridge internal, not an entry |
| the executor (the reconciled kinds: popover and modal open, `indeterminate`, `defaultChecked`, then `value`, `checked`, `selected`, `hidden=until-found`) | none | a comment seam (`pageObserver`) | item 0.4 builds the one page executor for every reconciled kind, the native slot kinds included (round 6: no per-kind choice and no Rust `transaction` branch), and owns the hook spreads; item 0.6 builds the witness harness |
| the order runtime (epoch rule, each owner's own rsx handler as the vdom carrier and a fallback port only for `beforematch`, one inbound pump, slot table keyed `(scope, nonce)` and stable across hydration, focus owner, `scrollOwner`) | none (the as-built `Epoch::next` is one process-global Rust counter and cannot lose to the page) | the same | item 0.7 (pump, slot table, per-slot epochs, gesture epoch, focus owner, the one-shot scroll classifier), then the first row that consumes an owner builds it (`scrollOwner` in 3.VS) |
| the paint-safe gate, the boot probe | none | none | lane 3.PL |
| the pre-wasm class | `PREPAINT_JS` and the modality writer, copies kept identical by `check-theme-mode-writer` and `check-open-focus`; their baseline rows carry no reason and `PREPAINT_JS` has inline `listener-ok` markers | the same | one `.ts` generates the copies and the writer becomes a bridge-load internal (lane 3.PW); reasoned allowlist rows, no inline marker |
| hooks that call scripts | `use_popover_sync`, `use_popover_shown_while_mounted`, `use_dialog_open_driver`, `use_global_escape_listener`, `use_global_keydown_listener`, `use_outside_dismiss`, `use_anchor_position_fallback`, `use_scroll_lock`, `use_move_interaction`, `use_js_listeners` | the same | each is ported by its lane; calling one adds nothing to the baseline |

**Scope is the owner's decision** (root doc section 5.0, pending; scope 2 is the default): the construct (the sink, the
ratchet, the deadline rule, admission, a legacy freeze, two proof ports: `otpFilter` and the executor) comes first; every other port is then a backlog
row opened by yield. Nothing here asks you to port a block you are not editing. **The neighbour you copy is a legacy
script. Call its hook; do not copy its string.**

1. **Rust owns behaviour.** A Dioxus `on*` handler, a signal, a `MountedData` read, `sleep`/`use_interval` and CSS
   (anchor positioning, `scroll-padding`, `overflow-anchor`, `linear()`) come first. If one of them does it, nothing else
   is written.
2. **Browser APIs Dioxus does not wrap go through the bridge** (`primitives/src/bridge/`, not on main yet): closed verbs,
   and a closed registry of **named behaviours** for the few decisions that must be made on the page's thread. Where your
   code goes is decided by the **deadline table** below.
3. **No JS in Rust strings, no new script.** Not `document::eval`, not `document::Script { "..." }`, not `script {}`, not
   `concat!` pieces, not an `include_str!` of a `.js`, not a new `.js`/`.ts` file, not an inline `<script>`, not
   `web_sys`/`js_sys`.
4. **The only new JS is a new verb row or a new named behaviour**, and a behaviour needs a backlog row that exists
   before your lane starts (planned rule). If what you need is missing, say so in your report; do not route around it.

## I want to ... use this

**First step, always: use the existing component.** Dialog, Popover, Tooltip, HoverCard, Select, Combobox, Slider,
Resizable, Drawer, Toast, DragAndDropList, Collapsible, Accordion, Tabs, Carousel and the rest already hold their
behaviour; most requests below are answered by one of them plus its props. The hooks in the "Today" column are
**crate-private** (`fn` or `pub(crate)` in `primitives/src`, callable from library code in that crate); the only `pub` one
is `js_listener::use_js_listeners`, the legacy form. A preview or app lane uses the component, never a hook it cannot name.

| I want to | Use this (the design) | Today (what is in the tree) |
|---|---|---|
| react to a click, key, pointer, focus or input event on an element I render | the Dioxus `on*` prop. `prevent_default()` works there on every renderer and is the **only** commit-time effect a handler takes: `stop_propagation()` never reaches the DOM (it stops vdom bubbling only, so a document-level `hotkey` still sees the key) and the `DataTransfer` setters are silent no-ops on desktop. **Frame-rate events (`onscroll`, `onresize`, `onpointermove`, `onwheel`, `ontouchmove`, `ondrag`, `ondragover`) never take a vdom handler, whatever their deadline** | the same |
| a **hotkey on a dialog** (for example Mod+Enter submits its form) | an rsx `onkeydown` on the `<dialog>`. Decide in Rust whether this layer is the top one: with nested layers only the top acts, as Escape's top-of-stack check does. If the key must work with focus outside the dialog, it is a bridge `listen` (document-wide, passive) plus an rsx `prevent_default` where a cancel is needed (L6 most likely decomposes so); `hotkey` exists only if that decomposition's witness is red, or for a UA-reserved shortcut (a platform rule row) | the rsx handler works now. For a window-level key `use_global_keydown_listener(key, cb)` (`lib.rs`, crate-private); if neither fits, stop and report |
| cancel an event from a window, document, outside target or the capture phase (Escape, F6, a page scroll lock, an OTP filter, drop-anywhere) | a named behaviour (`scrollLock`, `otpFilter`, `dropAnywhere`, and `hotkey` where admitted), each effect admitted by a counterfactual witness or a platform rule row, exactly one of the two per effect; Rust publishes beforehand and the page decides. On Android an rsx cancel of a payload event (`submit`, `input`, `change`, `paste`, `drop`) can silently fail (the header limit): cancel those declaratively (`method="dialog"`, `inputmode`, CSS) or page-side | none of them is built. Call the existing crate-private hook that already does it (`use_global_escape_listener`, `use_outside_dismiss`, `use_scroll_lock`); otherwise stop and report |
| a **drag, swipe or other gesture** | start, move and end all through **one channel**: all rsx for a discrete gesture, or one bridge session whose per-move frames carry a **gesture epoch**: the start is the rsx `pointerdown` that mints it, the end and cancel settle by dropping the per-gesture subscription, and anything the gesture reads once (Resizable's container rect) travels on the same channel as its deltas. Never an rsx `onpointerup` with bridge moves, because on desktop they are unordered. `touch-action` goes in CSS. A swipe whose release never arrives ends by timeout or `visibilitychange`. If you need pointer capture or a cancel of the committed move, the recogniser is the `pan` behaviour; its per-move output (`set_var`, a `transaction`) is Rust unless a measured `lag` or `order` verdict says otherwise | `use_move_interaction` (`move_interaction.rs`: the window pointer bridge Slider and Resizable use) plus an rsx `onpointerdown`, and `touch-action` in the component's CSS. **Known live defect: on desktop a quick re-grab can receive the previous drag's queued frames (bug row 475; reproduce first).** If you need capture or a cancel, stop and report: `pan` is not built |
| take pointer capture, or route focus for the next input (focus on open) | a named behaviour's effect (`pan`, `openFocus`, `settle`); **never** an async verb, **never** `set_focus` from a handler for this | focus on open is `dxOpenFocus` in `modality.rs`, run by the popover and dialog hooks; leave it alone |
| move focus along a list, menu, tabs, grid or radio group with the arrow keys | a composite whose role supports `aria-activedescendant` keeps DOM focus on the container and renders `aria-activedescendant` plus `data-highlighted`; its `onkeydown` only calls `prevent_default`. Where roving `tabindex` must stay (an owner call), the `roving` behaviour, which exists only if the phase-0 focus-write witness is red. A focus write goes through the **focus owner** once it exists | existing widgets call `set_focus` per arrow key (about 28 real DOM focus writes in 18 files); a new composite does not copy that |
| a **type-ahead** in a Listbox or menu (typing a letter highlights the next matching item) | Rust: an rsx `onkeydown` collects the characters and matches in Rust (`typeahead.rs`'s model), a `sleep` clears the buffer, and the highlight is `aria-activedescendant` plus `data-highlighted`, scrolled into view by the container's `scrollOwner` (one-shot kind). A printable key has no default action to cancel, so nothing here is commit-time | `typeahead.rs` and `listbox.rs` move focus with `set_focus` per match: use the existing component, do not copy the focus call, and do not write a script |
| a **copy-to-clipboard button** | the library has no clipboard verb and no library code uses the clipboard, so the platform category for it (an activation-gated API effect) is not in the registry today. A clipboard write must be issued inside the user gesture (WebKit enforces it; a call from Rust arrives late, and on WKWebView through `evaluate_script`, outside the gesture), so a library copy button would be a new `copyText` behaviour on a platform rule row, **an owner decision, not a lane's**. Until then the component takes a callback (`code_block` is the model: it carries no sink) and the app supplies the write; the preview's own writer is one reasoned preview-only allowlist row (V14, lane 3.PV) | `CodeBlockCopyButton` still holds an inline script with `navigator.clipboard` plus an `execCommand` fallback (a legacy block); call the component, do not copy its string |
| an **auto-growing textarea** | CSS first: `field-sizing: content` (Chrome 123, Firefox 152, Safari 26.2) behind `@supports`; on an older engine the textarea keeps its fixed height and `resize: vertical` handle, which is a fine fallback. A Rust fallback (`oninput` plus a `transaction` of `get scrollHeight` and `set_var`) grows a frame late, so a one-frame scrollbar can flash: it needs an owner decision, not a lane's | `Textarea` does not adopt `field-sizing` (its sheet says so); do not write a script, report if you need it |
| a **sortable list with keyboard reorder** | use `DragAndDropList` first. Keyboard reorder is Rust: an rsx `onkeydown` on the item (Space lifts, arrows move, Escape cancels; `prevent_default` on the arrow keys, a small event), the order is a signal, the move is announced in an `aria-live` region, and focus follows the moved item after its re-mount as a Rust-caused follow-up (never fenced). The pointer drag's `dragover` is frame-rate: the drop target's cancel is the `dropAnywhere` behaviour's, and the `DataTransfer` setters are no-ops on desktop, so the source half is `dropAnywhere`'s `dragSource` (planned) | `drag_and_drop_list.rs` holds its own `dragover` and `set_data` code (a legacy block; the three `DataTransfer` setter sites are known desktop no-ops); do not copy it |
| a **hover-content tooltip** (content the pointer can enter) | `Tooltip` or `HoverCard`. Hover intent is rsx `onpointerenter`/`onpointerleave` plus a `sleep` (`hover_intent.rs`); these are discrete, human-rate events, not frame-rate. The panel is a popover that stays a DOM descendant of its trigger, so moving the pointer into it keeps the hover, and placement is CSS anchor positioning behind the engine floors. Opening it is the popover command below. Touch has no hover: read the component's docs for what it does there, and never add a script for it | the component |
| **wheel-to-increment** (a number field that changes with the mouse wheel) | **Do not build it.** An rsx `onwheel` is a frame-rate event and never takes a vdom handler; a bridge `listen` is always passive, so it cannot cancel the page scroll that the wheel would also cause; so the only form is a named behaviour installed while the field is focused, with its `prevent` admitted by a witness, and it is a scroll trap for everyone else. Use the `+`/`-` buttons and arrow keys (rsx `onkeydown`, `prevent_default`). If the owner wants it, that is a new behaviour with a backlog row, not a lane's choice | nothing in the tree does this; report if you need it |
| a **sticky header that measures its columns** | CSS first: one table with `position: sticky` on the header cells, or `table-layout: fixed` with widths from the header, needs no measuring. Where body columns must drive the header, measure in Rust: `MountedData` reads for the first paint, `use_resize_observer` (a bridge `observe`, coalesced per frame) for changes, and publish the widths as `--dx-col-*` through `set_var`: R, next-frame or none, because a header that lags one frame is a small wrong paint, not a lost default action. **Never a per-row `onresize` in rsx** (a frame-rate event) | `table_rows.rs` (the virtual table) measures rows with a Rust `onresize` that the planned `check-hot-events` baseline will hold; use that component, do not copy its handler |
| a **print expand-all** (every collapsed panel open when printing) | CSS first: an `@media print` rule that shows `[hidden]` and `until-found` panels (the theme owns it, not a component sheet). `beforeprint` is not a Dioxus event and a Rust answer over the bridge arrives after the print layout, so **the deadline is the print layout, which Rust cannot meet**; the component must keep the content mounted (Rust decides that at render) so that CSS has something to show. A page-side `beforeprint` behaviour only if CSS cannot do it, as an owner decision | no `@media print` rule exists in the primitives, the preview sources or the preview assets today; report if you need it |
| focus an element once, with no keystroke waiting on it | an owner call to the **focus owner**, which carries `preventScroll` (planned, item 0.7; there is no `focus_prevent_scroll` verb). A plain `focus` is not a verb | `MountedData::set_focus(true)` through the stored handle (`TriggerHandle`); it has no `preventScroll`, so report if you need it |
| **open or close a popover or modal**, or set `indeterminate` or `defaultChecked` or write a native page-written attribute (the **reconciled kinds**) | render the state as a `data-dx-*` attribute carrying `(scope, state, based_on)`; the one page executor applies it (one executor per element, native slot kinds included), and a transition the page made (light dismiss, Escape, a focus change, an input) reaches Rust **on the vdom's ordered path as a Dioxus event on the element you already render**: `onbeforetoggle` for a popover, `oncancel` for a dialog, `onfocusin`/`onfocusout`, `oninput`, `onscrollend`; never the queued `toggle` or `close`. Only an event Dioxus lacks (`beforematch`) takes a hidden fallback port that the owner renders. On a gated surface the executor also reports `applied{gen, rects}`. A controlled `value`, `checked` or `selected` is an owner write over the bridge, never an attribute; a mount-frozen helper emits the native attribute once, at first render. `hidden=until-found` is a reconciled kind too. A one-off `show_popover()` is legal only for an element that is none of those kinds | call the existing hook: `use_popover_sync` or `use_popover_shown_while_mounted` for a popover (`top_layer/popover_sync.rs`, crate-private), `use_dialog_open_driver` for a modal. Do not write your own `show_popover()` |
| **place a popup** next to its trigger | CSS anchor positioning first, behind the engine floors and the boot probe; Rust placement behind a paint-safe gate on engines that fail | `use_anchor_position_fallback` (`top_layer/anchor_fallback.rs`) and `top_layer/anchor.css`; give the popup the existing attributes, not a script |
| keep a scroller at its end while content grows | CSS `overflow-anchor` where verified, the Rust invariant behind it, and `stickToEnd` on every engine | `message_scroller` (a legacy script); for another scroller, stop and report |
| react to a window or document event, or an event type Dioxus has no prop for, when a late answer is harmless | the bridge's `use_event_listener` / `use_window_listener` (always passive; frame-rate events at most once per frame) | `use_js_listeners` (`pub`, the legacy form); a new need goes in your report |
| follow a stream (scroll, resize, pointermove, wheel, touchmove, drag) | `use_sample` / `use_resize_observer` / `use_intersection_observer`: async, coalesced, armed only while a condition holds. **Never a vdom handler** | existing hooks only; `check-hot-events` is planned, so keep a new `onscroll`, `onresize` or `onpointermove` out by hand |
| call `scrollIntoView`, `setSelectionRange`, or write a `--dx-*` custom property | `ElementExt` over the closed `Method` enum; a scroll, selection, blur or popover call on a slot goes through the slot's owner. `MountedData` is for **reads**; its writes go through the owners once the phase-0 focus-write witness says the race is real | `MountedData` where it suffices (existing code); otherwise report |
| read size, position, scroll offset or a computed style | `MountedData` reads where they suffice; `bridge::get` for the rest; several operations and reads in one turn are a `Transaction` | `MountedData` |
| follow a media query or the window size | `use_media_query`, `use_window_size` | none on main; report |
| wait, time out or tick | `sleep`, `use_interval`, `use_timeout_while` (Rust tasks; never a JS timer) | the same |
| run something before wasm loads (the theme and modality writers) | `PREPAINT_JS` only | the same; edit it only with its copies, as `check-theme-mode-writer` and `check-open-focus` require |

## Where does my code go? The deadline table

Classify each **decision** by what a late answer costs, then by its kind. A block may use several rows for different
decisions. Read top to bottom; the first row whose deadline the decision has is its placement.

| deadline (what a late answer costs) | placement |
|---|---|
| this event's **default action** (cancel or not) | an **rsx handler** if Dioxus delivers the event, **every field the decision reads** (`ToggleData`, `FocusData`, `CancelData`, `SelectionData` are empty and `MouseData` has no `detail`: those decisions do not qualify) **and the serialized event stays small** (a payload event on Android can lose the cancel); otherwise a **named behaviour** |
| this event's **task** (pointer capture, focus routing for the next input) | a **named behaviour's effect**; never an async verb, never rsx `set_focus` on desktop; for arrow-key focus, `aria-activedescendant` first |
| **before paint** (a same-frame layout write) | **A** (the platform) where it does it, behind per-feature engine floors and a boot probe; a **named behaviour** where A does not cover every state (`stickToEnd`); **G** (a Rust answer behind an invisible pending state) only for one-shot placement where invisible-until-placed is acceptable |
| **next frame** (drag tracking, momentum) | **bridge stream plus Rust**, under the order contract below and a latency budget the owner sets from one real desktop latency distribution; a named behaviour only where a measured verdict with the reason `lag` or `order` says so. **A frame-rate event never takes a vdom handler, whatever its deadline** |
| **none** | **rsx first** where Dioxus delivers every field the decision reads and its state is Rust's; otherwise **R**: Rust over the bridge (`listen`, `observe`, `sample`, `get`, `call`) |

Four rules on the table (root doc section 2.2):
1. **The order contract: one epoch rule, two carriers, one inbound pump.** On desktop a bridge fact and a Dioxus event are
   unordered, and so are facts of two bridge sessions (the scheduler runs scopes by height, not arrival). So no decision may
   depend on the arrival order of two inputs. A **slot** is page state that Rust writes and the page can also write (open
   state, focus, scroll position, an input value, a behaviour's mode), with one epoch per slot id kept on the page. A page
   transition (light dismiss, the user's wheel, a form reset, focus moving to no element) advances it; a transition **Rust
   caused** (a refocus after a re-mount, a detach that leaves a list) is reported but never advances it. Slot keys are
   `(scope, nonce)`, because Dioxus reuses scope ids, and are stable across server render and hydration. A Rust write
   carries `based_on` for **every slot it can touch** (a late popover open steals focus, a late focus write scrolls a list
   back), captured when the decision is made, and the slot's owner applies it only if `based_on >= last_page_transition`;
   Rust writes do not mint epochs, they are ordered by the pump's FIFO. **Carrier 1: every discrete page transition reaches
   Rust on the vdom's own ordered path, as a Dioxus event on the element its owner already renders**: `onbeforetoggle` for
   a popover, `oncancel` for a dialog, `onfocusin`/`onfocusout` for focus, `oninput` for a value (the value owner
   re-dispatches a real `input` for a reset, a restoration and the install-time level), `onscrollend`. Dioxus delivers all
   of these synchronously and in order, so an epoch on this carrier needs no payload (each side counts the same events),
   and a port exists only for an event Dioxus lacks (today `beforematch`: a hidden element the owner renders, fired from a
   window-capture listener, not a form control). Never the queued `toggle` or `close`. **Carrier 2:** bridge facts carry
   the epoch of the slot they observed, and every page-to-Rust fact goes through **one inbound pump** (a page-minted
   sequence number), so facts that one decision combines are ordered by construction. A stream carries a **gesture epoch**
   minted at its start on every per-move frame (a mouse reuses its `pointerId`, so a previous drag's queued frames would
   otherwise land in the next drag). The vdom is also a writer of native slots (`value`, `checked`, `selected`, ...): those
   are owner writes over the bridge that keep a level, and a sink scan keeps the vdom from writing them (and from copying
   typed values into a CSS-readable attribute). A `MountedData` write (`set_focus`, `scroll_to`) is an owner bypass; it is
   banned **if** the phase-0 focus-write witness is red, and until then `MountedData` is for reads and the race is
   unreproduced. Scroll writes go through the slot's `scrollOwner`, which classifies the user's input by the realized
   default action; the document scroller is not a slot (the executor saves and restores scroll around `showModal()`), and
   the contenteditable date segments are. "Commutative" means an absolute set on a slot only Rust writes.
2. **Two deadlines in one gesture are two decisions.** A decision takes the deadline of its earliest effect and includes
   computing its inputs (a recogniser that decides "committed" from the move stream stays on the page); its later outputs
   (`scrollBy`, a spring, a CSS variable) are placed by their own rows, and are Rust by default.
3. **Roving focus** is `aria-activedescendant` where the pattern allows, else one `roving` behaviour (built only if the focus-write witness is red), and a focus write goes through the focus owner.
4. **Android payload events** are not rsx-cancel candidates.

Two kinds of decision, placed by that table:
- A **listener** reacts to events.
- A **command** mirrors Rust state into the page. Rust publishes the desired state as a `data-dx-*` attribute and the one
  page executor applies it (one executor for every kind, decided by no witness); it is level-triggered, so a lost, early or
  reordered command converges, and it never touches the native `open` attribute. The **reconciled kinds** are popover open,
  modal open, `indeterminate` and `defaultChecked`, and the native page-written state Rust writes (`value`, `checked`,
  `selected`, which travel as owner writes rather than an attribute, and `hidden=until-found`), because the vdom is a writer
  of them too.
  A **one-shot call** (focus this cell, scroll that item into view) is a bridge call, not a command.

## Named behaviours

A behaviour is a closed registry entry, not a script and not a predicate language: a Rust `Behaviour` variant with typed
parameters, one pure `decide(config, memory, snapshot, event) -> Decision<E>` in
`primitives/src/ts/behaviours/<kind>.ts`, and a thin DOM adapter that applies the effects. **Admission is derived per effect from the deadline table, with two grounds, never a declaration** (planned): the
**counterfactual witness**, or a **platform rule row** from a closed, cited category table: (a) `prevent` of a UA-reserved
shortcut (F6, Ctrl/Cmd+B), (b) an activation-gated API effect (`clipboardWrite`: on WKWebView the call arrives by
`evaluate_script`, outside the gesture, so no delay counterfactual can witness it; **not in use** while no library code uses
the clipboard, so it is on the plan's exits list), (c) an engine-specific default action (`dataTransfer`). **Every effect has exactly one ground**; the owner's `verdict.json` (`research/measure/<port>/`,
only the reason `lag` or `order` admits) is the next-frame row's form of the witness, not a third ground. A platform row sits
only on a `site!` of a registry behaviour, names the engine that could observe it (`observable_on`) and goes stale when that
engine joins a gate config; its check line lives in `desktop-pass.md`, with an owner-approved grant row at the merge base.
A construction site is named, not inferred: `site!("name")` with one scenario file per site. For each effect a witness test
moves only that effect to its next-best placement (dropped, installed in the bubble phase **at the site's own host element**,
delayed by Rust's modelled latency, or reordered after the next event), and one criterion reads the result, **arm
divergence** over library-defined observables (a per-element trajectory over a wide projection that sees `checked`,
`indeterminate` and `value`, unrelated animation paused through `[data-dx-motion="paused"]`, a closed `defaultAction`
observable table per event type, `outcomeOrder` with harness sentinels), with a real-against-real control arm; the test is an
expected failure that passes only with the probe's typed error, and a `test.fixme` is keyed to the site list. Witness mode is
a Playwright `addInitScript` plus a `{site, effect}` marker the adapter sets while applying an effect: no second build, no
cargo feature. Witnesses run on Chromium (the only engine in the lane container) and under fallback emulation where the effect serves
another engine; Firefox and WebKit runs are wired at deploy and cannot execute in a lane. On a debug build a witness pauses
the host subtree's own animations, and `ts/witness.ts` never ships as a bundle entry. Structural detection is the inventory, not a ground: `site!` ids, behaviour modules that compile without the
DOM (purity is a tsc scope analysis: no `any`, no `constructor`, `__proto__` or `prototype`), and an adapter that refuses an
effect a site did not declare (no per-site typestate and no coverage hook: round 7). `window.__dxBridge` is closed after boot (no `register`, no live table, no verbs). The bridge's own
internals (the modality writer, the one page executor, its exit wait, G's watchdog) are admitted as wholes by the reds they
already have, not per kind.

Planned registry (each is added by the row that ports its block; **today none exists but a test fixture on lane 3.0**):
`otpFilter` (the proof port), `scrollLock`, `dropAnywhere`, `pan`, `settle`, `openFocus`, `stickToEnd`, `scrollOwner` (the
page-side half of every Rust-written scroller, the one-shot kind first: it classifies the user's input by the realized
default action into transitions and applies `based_on` writes) and `labelClick` (the label's double-click default action; its user-select half
stays CSS), and `hotkey` only if its witness is red or a UA-reserved shortcut needs it (the first of 3.OV and 3.GS to open
builds it). `scrollStep` (C2, a `transaction` from Rust), `wheelBand`, the page-side output variants of `pan` and the
`release` effect are on the plan's exits list; `roving` exists only if the focus-write witness is red; the executor, `place`
and `dialogKeyForward` are not entries (a row nothing uses is dead surface).
Rust declares target, phase and passivity; `passive` is derived (false only for a behaviour that cancels). Installation is
owned by the Rust scope that needs it and ref-counted, so a blocking listener exists only while something needs it; every
install carries an epoch, a fail-open lease Rust renews (a wedged Rust side must never trap the keyboard, WCAG 2.1.2) and
tears down when its host element disconnects. When Dioxus later delivers the missing event or field (0.8's `onbeforeinput`),
the behaviour moves to an rsx handler; a test says so.

A stateful heuristic over a stream (a trackpad classifier, a scroll mode machine) is **not** a behaviour: it is Rust over
`listen`/`sample`, because it may answer late.

## The bridge (lane 3.0's branch; not on main)

It lives in `primitives/src/bridge/` (`dioxus_primitives::bridge`) on `origin/lane/js-bridge-3-0`. Rust sends closed enums;
the page maps each through a closed own-property table with a receiver kind per row (`EVENTS`, `FIELD_GETTERS`, `METHODS`,
`PROPS` in `ts/bridge.ts`); a name outside a table is an error, never a property read. A verb row is **one member call or one
member read, plus at most one guard `if`**; anything bigger is a behaviour. There are no gesture verbs and no `commit`:
capture and a focus for the next keystroke are effects of a behaviour.

| verb | Rust |
|---|---|
| `call(el, method, args)` | `ElementExt` (`show_popover`, `hide_popover`, `show_modal`, `close_dialog`, `scroll_into_view`, `set_selection_range`, ...) over `Method`; for a reconciled element the executor is the only show writer, and a verb that writes a slot (scroll, selection, blur, popover hide or toggle, close) is an owner operation |
| `get(el, props)` | `bridge::get` / `ElementExt::get` with `Prop` (`Rect`, `ActiveElement`, `VisualViewport`, `Style(name)`, `Attr(name)`, ...) |
| `set(el, --var, value)` | `ElementExt::set_var`; custom properties that start with `--dx-` only |
| `listen(target, event, fields)` | `use_event_listener`, `use_window_listener`, `use_document_listener`, `use_element_listener`, `use_window_size`; always passive; hot events are coalesced per frame |
| `observe(el, kind, opts)` | `use_resize_observer`, `use_intersection_observer`, `use_mutation_observer`, `use_observer` |
| `matchMedia(query)` | `use_media_query` |
| `sample(el, fields, until)` | `use_sample` with `SampleField::read(&Prop)` or `SampleField::batched(..)`; latest-wins per frame, armed only while a gesture or condition holds |
| `behaviour(target, kind, ...)` | `use_behaviour`, `Behaviour` (`bridge/behaviours.rs`) |
| `transaction(ops)` | `Transaction`: several operations and their reads in one JS turn |

Elements are addressed by a `Target` (window, document, an element by `id` or selector, the active element); an element the
bridge acts on carries an `id`. **The row that first needs a missing verb adds it, with its own grant** (`reset`,
`beforematch`, `scrollend`, `dragend`, `invalid`, `reportValidity`, `getAnimations`, `scrollTop`
setters, `location.search`, `localStorage`, an `observe` publish option, ...); V5, V8, V9 and V14 (cookie, `postMessage`,
`fetch`, clipboard) take a reasoned preview-only allowlist entry decided by the owner. Ordering is the transport's job for
**element operations only**: they share one pump and flush after the render that issued them, each `Session` has its own
eval, and streams are latest-wins per frame; the order contract above covers everything else (a stream's start, end and
cancel are the owner's own rsx events or a dropped subscription, the pump dispatches through a registry entry that `Drop`
removes, so it never calls a dropped scope's `Callback`). The bridge loads by eval after
boot, so reports queue on the page until the channel opens. Hooks never call `document::eval`; the seam in
`bridge/transport.rs` (a `Program`-typed sink) is the one place, plus two named exceptions (`eager_head.rs` and the
preview-only `theme_seed`). `window.__dxBridgeDelayFrames = N` (or `"never"`) is a **bridge-latency** harness, not a
desktop model.

## Desktop and mobile caveats

Web is the tested target; every desktop claim in the plan reads "web-verified; desktop inferred" until a
`desktop-pass/<tree>.pass` stamp exists (row 442). What to remember when you write code:

- **A Rust event handler is a blocking synchronous request per event on desktop and mobile**, including frame-rate ones, and
  Dioxus registers one root listener per event name: one `onpointermove` or `onwheel` anywhere makes that event a blocking,
  non-passive request across the whole app. `onmounted` is a blocking request inside the edit application too (one per
  list item in a collection is N round trips in one frame). Use the bridge's `listen`/`sample`/`observe` for frame-rate
  work.
- **Rust cannot answer in the same frame.** A render lands in the next animation frame, an eval from a handler runs before
  its own render's edits, a reply costs an IPC hop, and while a render is unacknowledged (or the webview is hidden) no Rust
  task or timer runs. So a fail-open must live on the page, and a same-frame write is a behaviour or the platform.
- **Platform rows Chromium on the web cannot show:** Windows replaces HTML5 drag events under the default file-drop config
  (`dropAnywhere`, the native drag list), Android handles events on an IO thread under a global mutex with a header size
  limit, and release desktop builds install a document `contextmenu` listener that prevents default.
- **Web `eval` is `new Function`**, so an app under a strict Content-Security-Policy needs `unsafe-eval` for the bridge's
  transport.

## What is forbidden, and which gate says so

"Landed" means the gate exists on main; the first table says what lane 3.0's branch adds.

| Forbidden | Gate | Instead |
|---|---|---|
| `document::eval`, `use ... eval as run`, `eval(`, `document::*` | `check-js-volume` (landed). A function pointer through a module alias (`dd::eval`) is **not** counted; clippy `disallowed_methods` and the `Program` argument check are planned | a bridge verb or behaviour |
| `document::Script { "..." }`, `script {}`, `dangerous_inner_html` on a script, `create_script` | `check-js-volume` (landed) | `Script { src: <asset> }` for the bridge only, owner-approved |
| JS split into `concat!` pieces, a script held in a `const` | `check-js-volume` (`js_lit`, pieces joined; landed) | a verb row or a named behaviour |
| a new `.js`/`.ts` file, an `include_str!`/`asset!` of one | `check-js-volume` (`js_file`, sink; landed) | a verb row or a named behaviour with a row |
| an inline `<script>` in an `.html` file, an `on...=` attribute, a `javascript:` href | `check-js-volume` (`inline_script`, sink; landed). Raw HTML with a handler in a `.md` or `.svg` is **not** scanned yet (planned) | `PREPAINT_JS` copies stay as they are |
| `web_sys`, `js_sys`, `wasm_bindgen`, `gloo_utils`, `evaluate_script` | `check-js-volume` (sink; landed). `gloo_events` and `gloo_file` are planned | the bridge's transport is the one place |
| a vdom `onscroll`, `onresize`, `onpointermove`, `onmousemove`, `onwheel`, `ontouchmove`, `ondrag`, `ondragover` or `onmounted` handler | `check-hot-events` (**planned**; per event name, app-wide, shrink-only baseline) | `listen`/`observe`/`sample` |
| an rsx `prevent_default` on a payload event (`submit`, `input`, `change`, `paste`, `drop`) | the `check-hot-events` family (**planned**) | a declarative cancel or a page-side behaviour |
| a `DataTransfer` setter in an rsx handler | clippy `disallowed_methods` (**planned**; three sites ship today) | `dropAnywhere`'s `dragSource` half |
| a plain `focus`, `setPointerCapture` or `commit` verb | `check-bridge-behaviours` on lane 3.0's branch (landed there) | a behaviour's effect |
| a verb that writes a slot (`showPopover`, `showModal`, popover hide or toggle, close, scroll, selection, blur) outside its owner or the executor | `FORBIDDEN_VERBS` in `check-bridge-behaviours` (**planned**; derived from the slot table) | the owner operation or the executor |
| a vdom write of a native slot (`value`, `checked`, `selected`, `indeterminate`) through an rsx attribute, `Attribute::new`, `attributes!` or a caller spread; more than one owner for `checked` or `indeterminate` on an element | the native-slot sink scan (**planned**; shrink-only baseline) | the owner write over the bridge, with the mount-frozen helper for the first render |
| a `MountedData` write (`set_focus`, `scroll_to`) | **only if** item 0.6's focus-write witness is red: then clippy `disallowed_methods` with per-site `#[expect]`; otherwise not banned (**planned**) | the focus owner and the slot's `scrollOwner`; `MountedData` for reads |
| `preventDefault`, `stopPropagation`, `requestAnimationFrame` or an observer in a `ts/` file outside `bridge.ts` and `ts/behaviours/`; a computed member access or string-form `setTimeout`/`Function` in `ts/` | `check-bridge-behaviours` (identifier part on lane 3.0's branch; the compiler-API scan is **planned**) | a named behaviour or the bridge |
| a behaviour module that reads the DOM, a timer or the clock; a verb row with more than one member call and guard; an effect with no witness; a `prevent`/`stop` variant with no lease; a Rust variant without its module, bundle, generated kind or vector | `check-bridge-behaviours` (purity, parity and vectors on lane 3.0's branch; the rest **planned**) | keep `decide` pure; go through the closed tables |
| a hand-edited `js/*.js` or `behaviours.gen.ts`, or a `.ts` edit without `scripts/build-js-bundle.sh` | `check-js-bundle`; `cargo test` (`the_generated_ts_is_current`, on lane 3.0's branch) | rebuild; `DX_BLESS=1 cargo test -p dioxus-primitives behaviours` |
| a raw `addEventListener`, an uncleared interval, a blocking wheel listener in a script | `check-js-listeners`, `check-uncleared-intervals`, `check-blocking-scroll-listeners` (landed; retire when the last legacy script goes) | `listen`; Rust timers |
| a comment that calls `document::eval` the crate's pattern | `check-js-volume` (`comment_eval`; landed) | link this page |

## The baseline, and why you do not add to it

`scripts/check-js-volume.baseline.tsv` holds one row per file with a non-zero count. A port lowers its own file's row and
runs `scripts/check-js-volume.sh --update-baseline`; rows belong to files, so lanes that lower different files merge
cleanly. Your report quotes `scripts/check-js-volume.sh --report` before and after. A rise fails, and a baseline edited to
match is compared with the baseline at the merge base. **Today** a raise passes with a note on the baseline row and prints
under `RATCHET RAISES`; **planned:** a raise passes only against a grant row that exists at the merge base (a backlog row,
open, whose front matter says `js-grant: <column>=<n>`), a lane cannot grant itself, and a new behaviour likewise needs a row
at the merge base with `adds: [<kind>]`; citing the umbrella row 434 does not count. If you think you must raise, say so in
the report with the row and the measured reason, and wait for the owner. **Today the volume counts every tracked `.ts` line,
so `.d.ts`, generated files and tests are in it;** the plan counts what ships (bundle bytes and JS held in Rust). Moving a
script from a Rust string into a new `.ts` still has to come out lower.

## Legacy scripts and the pre-wasm class

Everything counted in the baseline today is legacy, being ported by row 434
([`research/sync-js-root-2026-10-26.md`](./research/sync-js-root-2026-10-26.md) section 5.4); the audit
([`research/js-audit-2026-10-25.md`](./research/js-audit-2026-10-25.md)) classes each block, and root section 3.3 gives every
id its placement and lane. **Do not edit a legacy script except to port it:** the legacy freeze (planned, item 0.2) will make
an edit a red gate, but today an edit that keeps the line count passes. The legacy message scroller and carousel scripts are
(planned, with the owner-measured ports) frozen into `ts/legacy/` behind a non-default `legacy-impl` feature for the owner's
A/B measurement, and removed when the owner's `verdict.json` says `pass`, never on a date and never on a `fail`. The one
class that stays JavaScript on purpose is code that must run before wasm loads (`PREPAINT_JS`, the modality writer's
pre-wasm copy and the `preview/index.html` scripts); `check-theme-mode-writer` and `check-open-focus` keep the copies
identical until they are generated from one `.ts`.

## Adding a verb row or a named behaviour

- **A verb row is data.** (This describes lane 3.0's branch.) A variant in the Rust enum (`Method`, `Prop`, `EventKind`,
  `EventField`), one row in the matching TS table with its receiver kind, then `scripts/build-js-bundle.sh`. **The row that
  first needs the verb adds it, with its grant row**: a new row raises `bridge.ts`'s count. `cargo test -p dioxus-primitives
  bridge` compares every Rust name with its row; `bun test primitives/ts-test` covers the page side.
- **A named behaviour needs a backlog row first** (open, at the merge base, `adds:` naming the kind; planned rule), and: the
  variant in `bridge/behaviours.rs` (`info()`, `declared()`, `module()` and `samples()` have no wildcard arm),
  `ts/behaviours/<kind>.ts` (pure `decide`), vectors in `primitives/conformance/behaviours/<kind>.json` (one Rust file and one vectors file per behaviour), `DX_BLESS=1 cargo test -p
  dioxus-primitives behaviours` to regenerate the TS types, `scripts/build-js-bundle.sh` (one bundle per behaviour), one
  witness per effect at its `site!` (authored by item 0.6 or a lane other than the behaviour's own; an unbuilt variant's
  witness is a `test.fixme` keyed to the site list), a Playwright spec tag, and a budget row. Behaviour lanes never edit a gate script.
- **The gates, by name.** Landed on main: `check-js-bundle`, `check-js-volume`, the three legacy script gates,
  `check-open-focus` and `check-theme-mode-writer`. On lane 3.0's branch: `check-bridge-behaviours`. Planned: the grant rule
  in `check-js-volume`, `check-hot-events`, the legacy freeze, the witness check, the native-slot sink scan.

## Why not a script

The browser takes its irreversible decisions at fixed points: when an event's dispatch ends (the default action), at
gesture start, at input routing, at paint, at the compositor frame. Only code on the page's own thread, with data already
there, can affect them. Dioxus event handlers are already synchronous, so `prevent_default()` in Rust is fine; what is
asynchronous is a message to Rust, and nothing should be decided in one. The rule is: **never ask Rust at commit time;
Rust publishes beforehand and the page decides locally.**

The cost of the alternative showed up in the code: 103 bespoke eval blocks (about 6,200 lines) were written because the only
written convention for "I need the DOM" was `document::eval`, every JS gate checked the shape of the script string (and so
blessed the string), and a script needs no new file, verb or review. The neighbour you copy teaches the way, so this page
and the gates exist.
