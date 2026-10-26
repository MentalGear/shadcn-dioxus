# JavaScript interop: Rust first, the bridge second, no script strings

**Read this before you write any interactive behaviour.** It is short on purpose. The plan is backlog row 434 step 3
([`research/sync-js-root-2026-10-26.md`](./research/sync-js-root-2026-10-26.md), amended by four adversarial boards, the
last being [round 4](./research/board-2026-10-26-r4/SYNTHESIS.md)); this page is its agent-facing form. **Much of what it
describes is not built.** The next table says what is; the "today" column of the "I want to ..." table says what to do
when the rule you need depends on something that is not.

## Landed and planned

| | on main | on `lane/js-bridge-3-0` (lane 3.0; not on main until its batch merges) | planned (row 434 step 3) |
|---|---|---|---|
| `check-js-volume` | counts per file: eval sinks (`document::eval`, `use ... eval`, `eval(`; **not** a function pointer through a module alias such as `dd::eval`), `document::*`, `Script`/`script {`/`create_script`, `dangerous_inner_html`, `web_sys`/`js_sys`/`wasm_bindgen`/`gloo_utils`/`evaluate_script`/`inline_js`/`srcdoc` (**not** `gloo_events`), JS in Rust strings, every tracked `.js`/`.ts` line (**including** `.d.ts`, `*.gen.ts` and `ts-test`), `JsListeners::install`, inline `<script>` bytes. Universe: `.rs`, `.js`/`.ts` and `.html` files, **not** `.md` or `.svg`. A rise fails against the baseline and against the merge base's; a raise is self-service with a `# raised:` note and prints under `RATCHET RAISES` | the same script and rules; its baseline gains rows for the bridge | the grant rule (a raise needs an open row with `js-grant:` at the merge base), clippy `disallowed_methods` for every sink, coverage of function pointers, `gloo_events`, `.md` and `.svg`, counting what ships, the legacy freeze for scripts and sinks (`js-fix:` grant for a bug fix) |
| older gates | `check-js-bundle`, `check-js-listeners`, `check-uncleared-intervals`, `check-blocking-scroll-listeners`, `check-theme-mode-writer`, `check-open-focus` | the same | their JS halves retire with the last legacy script |
| `check-bridge-behaviours` | none | kind parity with the generated file, behaviour modules pure, budgets (held in a dict inside the script), one place for cancel and frames, closed surface, eval only inside `bridge/`, `Program` closed and sealed, no gesture verbs | `Decision<E>` effect typing and a DOM-less compile for `ts/behaviours/`, a witness test per effect, the lease check on the generated file, single-member-call verb rows, `FORBIDDEN_VERBS`, `REDUCERS`, budgets in a TSV, `check-hot-events`, the clippy ban on the `DataTransfer` setters |
| the bridge | `primitives/src/bridge/` does not exist | closed verbs over closed tables; `Program { Serve, Pump, Legacy(&str) }`, sealed to `bridge/`; one FIFO pump for element operations; latest-wins streams; an epoch on every install and a default lease on every non-passive behaviour; one test fixture; the `__dxBridgeDelayFrames` harness with the stuck setting `"never"` | `Program::Frozen`, the rest of the verb rows (each added by the first row that needs it), the closed `REDUCERS` table |
| named behaviours | none | `Behaviour::Fixture`, a test fixture, only | `hotkey`, `scrollLock`, `otpFilter`, `dropAnywhere`, `pan`, `settle`, `openFocus`, `stickToEnd`; `reconcile`, `scrollStep`, `roving`, `place` and `wheelBand` only if their witness says so |
| the executor (open and close, `indeterminate`, `defaultChecked`) | none | a comment seam (`pageObserver`) | item 0.4: a page reconciler or a Rust `transaction`, whichever its witness selects |
| the paint-safe gate, the boot probe | none | none | lane 3.PL |
| the pre-wasm class | `PREPAINT_JS` and the modality writer, copies kept identical by `check-theme-mode-writer` and `check-open-focus`; their baseline rows carry no reason and `PREPAINT_JS` has inline `listener-ok` markers | the same | one `.ts` generates the copies and the writer becomes a bridge-load internal (lane 3.PW); reasoned allowlist rows, no inline marker |
| hooks that call scripts | `use_popover_sync`, `use_popover_shown_while_mounted`, `use_dialog_open_driver`, `use_global_escape_listener`, `use_global_keydown_listener`, `use_outside_dismiss`, `use_anchor_position_fallback`, `use_scroll_lock`, `use_move_interaction`, `use_js_listeners` | the same | each is ported by its lane; calling one adds nothing to the baseline |

**Scope is the owner's decision** (root doc section 5.0, pending; scope 2 is the default): the construct (the sink, the
ratchet, the deadline rule, admission, a legacy freeze, two proof ports) comes first; every other port is then a backlog
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

| I want to | Use this (the design) | Today (what is in the tree) |
|---|---|---|
| react to a click, key, pointer, focus or input event on an element I render | the Dioxus `on*` prop. `prevent_default()` works there on every renderer and is the **only** commit-time effect a handler takes: `stop_propagation()` never reaches the DOM (it stops vdom bubbling only, so a document-level `hotkey` still sees the key) and the `DataTransfer` setters are silent no-ops on desktop | the same |
| a **hotkey on a dialog** (for example Mod+Enter submits its form) | an rsx `onkeydown` on the `<dialog>`. Decide in Rust whether this layer is the top one: with nested layers only the top acts, as Escape's top-of-stack check does. If the key must work with focus outside the dialog it is a `hotkey` behaviour | the rsx handler works now. For a window-level key call `use_global_keydown_listener(key, cb)` (`lib.rs`); if neither fits, stop and report |
| cancel an event from a window, document, outside target or the capture phase (Escape, F6, a page scroll lock, an OTP filter, drop-anywhere) | a named behaviour (`hotkey`, `scrollLock`, `otpFilter`, `dropAnywhere`), each effect admitted by its witness; Rust publishes beforehand and the page decides. On Android an rsx cancel of a payload event (`submit`, `input`, `change`, `paste`, `drop`) can silently fail (the header limit): cancel those declaratively (`method="dialog"`, `inputmode`, CSS) or page-side | none of them is built. Call the existing hook that already does it (`use_global_escape_listener`, `use_outside_dismiss`, `use_scroll_lock`); otherwise stop and report |
| a **drag, swipe or other gesture** | start, move and end all through **one channel**: all rsx for a discrete gesture, or all through one bridge session (the drawer's shape); never an rsx `onpointerup` with bridge moves, because on desktop they are unordered. `touch-action` goes in CSS. If you need pointer capture or a cancel of the committed move, the recogniser is the `pan` behaviour; its per-move output (`set_var`, a `transaction`) is Rust unless a measured `lag` or `order` verdict says otherwise | `use_move_interaction` (`move_interaction.rs`: the window pointer bridge Slider and Resizable use) plus an rsx `onpointerdown`, and `touch-action` in the component's CSS. If you need capture or a cancel, stop and report: `pan` is not built |
| take pointer capture, or route focus for the next input (focus on open) | a named behaviour's effect (`pan`, `openFocus`, `settle`); **never** an async verb, **never** `set_focus` from a handler for this | focus on open is `dxOpenFocus` in `modality.rs`, run by the popover and dialog hooks; leave it alone |
| move focus along a list, menu, tabs, grid or radio group with the arrow keys | a composite whose role supports `aria-activedescendant` keeps DOM focus on the container and renders `aria-activedescendant` plus `data-highlighted`; its `onkeydown` only calls `prevent_default`. Where roving `tabindex` must stay (an owner call), the `roving` behaviour | existing widgets call `set_focus` per arrow key; a new composite does not copy that |
| focus an element once, with no keystroke waiting on it | the bridge's `focus_prevent_scroll` row (lane 3.CA adds it with its grant); a plain `focus` is not a verb | `MountedData::set_focus(true)` through the stored handle (`TriggerHandle`); it has no `preventScroll`, so report if you need it |
| **open or close a popover or modal**, or set `indeterminate` or `defaultChecked` (the **reconciled kinds**) | render the state as a `data-dx-*` attribute carrying `(scope, state, epoch)`; the one executor applies it and reports back what the browser did (light dismiss, Escape) and, on a gated surface, `applied{gen, rects}`. A one-off `show_popover()` is legal only for an element that is none of those kinds | call the existing hook: `use_popover_sync` or `use_popover_shown_while_mounted` for a popover (`top_layer/popover_sync.rs`), `use_dialog_open_driver` for a modal. Do not write your own `show_popover()` |
| **place a popup** next to its trigger | CSS anchor positioning first, behind the engine floors and the boot probe; Rust placement behind a paint-safe gate on engines that fail | `use_anchor_position_fallback` (`top_layer/anchor_fallback.rs`) and `top_layer/anchor.css`; give the popup the existing attributes, not a script |
| keep a scroller at its end while content grows | CSS `overflow-anchor` where verified, the Rust invariant behind it, and `stickToEnd` on every engine | `message_scroller` (a legacy script); for another scroller, stop and report |
| react to a window or document event, or an event type Dioxus has no prop for, when a late answer is harmless | the bridge's `use_event_listener` / `use_window_listener` (always passive; frame-rate events at most once per frame) | `use_js_listeners` is the legacy form; a new need goes in your report |
| follow a stream (scroll, resize, pointermove, wheel, touchmove, drag) | `use_sample` / `use_resize_observer` / `use_intersection_observer`: async, coalesced, armed only while a condition holds. **Never a vdom handler** | existing hooks only; `check-hot-events` is planned, so keep a new `onscroll`, `onresize` or `onpointermove` out by hand |
| call `scrollIntoView`, `setSelectionRange`, or write a `--dx-*` custom property | `ElementExt` over the closed `Method` enum, or `MountedData` where it suffices | `MountedData` where it suffices; otherwise report |
| read size, position, scroll offset or a computed style | `MountedData` where it suffices; `bridge::get` for the rest; several operations and reads in one turn are a `Transaction` | `MountedData` |
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
| **next frame** (drag tracking, momentum) | **bridge stream plus Rust**, under the order contract below and a latency budget the owner sets from one real desktop latency distribution; a named behaviour only where a measured verdict with the reason `lag` or `order` says so |
| **none** | **R**: Rust over the bridge (`listen`, `observe`, `sample`, `get`, `call`) |

Four rules on the table (root doc section 2.2):
1. **The order contract.** On desktop a bridge fact and a Dioxus event are unordered, and so are facts of two bridge sessions
   (the scheduler runs scopes by height, not arrival). So a fact carries the **epoch** of the slot it observed; a Rust write
   to a slot the page also writes (open state, focus, scroll position, an input value) goes through that slot's page-side
   owner with the epoch it was decided from and is dropped if the page has changed the slot since; and "commutative" means
   an absolute set on a slot only Rust writes. Facts that one decision combines share one session. A `MountedData` call is a
   second, unordered channel.
2. **Two deadlines in one gesture are two decisions.** A decision takes the deadline of its earliest effect and includes
   computing its inputs (a recogniser that decides "committed" from the move stream stays on the page); its later outputs
   (`scrollBy`, a spring, a CSS variable) are placed by their own rows, and are Rust by default.
3. **Roving focus** is `aria-activedescendant` where the pattern allows, else one `roving` behaviour.
4. **Android payload events** are not rsx-cancel candidates.

Two kinds of decision, placed by that table:
- A **listener** reacts to events.
- A **command** mirrors Rust state into the page. Rust publishes the desired state as a `data-dx-*` attribute and the one
  executor applies it; it is level-triggered, so a lost, early or reordered command converges, and it never touches the
  native `open` attribute. The **reconciled kinds** are popover open, modal open, `indeterminate` and `defaultChecked`.
  A **one-shot call** (focus this cell, scroll that item into view) is a bridge call, not a command.

## Named behaviours

A behaviour is a closed registry entry, not a script and not a predicate language: a Rust `Behaviour` variant with typed
parameters, one pure `decide(config, memory, snapshot, event) -> Decision<E>` in
`primitives/src/ts/behaviours/<kind>.ts`, and a thin DOM adapter that applies the effects. **Admission is derived per effect
from the deadline table, with one ground: the counterfactual witness** (planned; a declaration admits nothing). For each
effect a behaviour has at a Rust construction site's parameters, a witness test moves only that effect to its next-best
placement (dropped, installed in the bubble phase at the element, delayed by Rust's modelled latency, or reordered after the
next event), and a library probe must go red; the test is an expected failure that passes only with the probe's typed
error. For the next-frame row the witness is the owner's measurement (`research/measure/<port>/verdict.json`,
`{verdict, reason, devices, traces}`): only the reason `lag` or `order` admits. Structural detection is the inventory, not a
ground: behaviour modules compile without the DOM, and the adapter refuses an effect outside the generated set.

Planned registry (each is added by the row that ports its block; **today none exists but a test fixture on lane 3.0**):
`hotkey`, `scrollLock`, `otpFilter`, `dropAnywhere`, `pan`, `settle`, `openFocus`, `stickToEnd`. `reconcile` is a registry
behaviour only if its witness keeps the executor out of the bridge, `scrollStep` (C2) is a `transaction` from Rust unless its
witness is red, `roving`, `place` and `wheelBand` exist only on a red witness or verdict, and `dialogKeyForward` is reserved.
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
| `call(el, method, args)` | `ElementExt` (`show_popover`, `hide_popover`, `show_modal`, `close_dialog`, `scroll_into_view`, `set_selection_range`, ...) over `Method`; for a reconciled element the executor is the only show writer |
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
`beforematch`, `scrollend`, `dragend`, `invalid`, `reportValidity`, `getAnimations`, `focus_prevent_scroll`, `scrollTop`
setters, `location.search`, `localStorage`, an `observe` publish option, ...); V5, V8, V9 and V14 (cookie, `postMessage`,
`fetch`, clipboard) take a reasoned preview-only allowlist entry decided by the owner. Ordering is the transport's job for
**element operations only**: they share one pump and flush after the render that issued them, each `Session` has its own
eval, and streams are latest-wins per frame; the order contract above covers everything else. The bridge loads by eval after
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
| a `showPopover` or `showModal` verb on a reconciled kind | `FORBIDDEN_VERBS` in `check-bridge-behaviours` (**planned**) | the executor |
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
  `ts/behaviours/<kind>.ts` (pure `decide`), vectors in `primitives/conformance/behaviours.json`, `DX_BLESS=1 cargo test -p
  dioxus-primitives behaviours` to regenerate the TS types, `scripts/build-js-bundle.sh` (one bundle per behaviour), one
  witness test per effect, a Playwright spec tag, and a budget row. Behaviour lanes never edit a gate script.
- **The gates, by name.** Landed on main: `check-js-bundle`, `check-js-volume`, the three legacy script gates,
  `check-open-focus` and `check-theme-mode-writer`. On lane 3.0's branch: `check-bridge-behaviours`. Planned: the grant rule
  in `check-js-volume`, `check-hot-events`, the legacy freeze, the witness check.

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
