# JavaScript interop: Rust first, the bridge second, no script strings

**Read this before you write any interactive behaviour.** It is short on purpose. The plan is backlog row 434 step 3
([`research/sync-js-root-2026-10-26.md`](./research/sync-js-root-2026-10-26.md), amended by two adversarial boards); this
page is its agent-facing form.

1. **Rust owns behaviour.** A Dioxus `on*` handler, a signal, a `MountedData` read, `sleep`/`use_interval` and CSS
   (anchor positioning, `scroll-padding`, `overflow-anchor`, `linear()`) come first. If one of them does it, nothing else
   is written.
2. **Browser APIs Dioxus does not wrap go through the bridge** (`primitives/src/bridge/`): closed verbs, and a closed
   registry of **named behaviours** for the few decisions that must be made on the page's thread. Where your code goes is
   decided by the **deadline table** below.
3. **No JS in Rust strings, no new script.** Not `document::eval`, not `document::Script { "..." }`, not `script {}`, not
   `concat!` pieces, not an `include_str!` of a `.js`, not a new `.js`/`.ts` file, not an inline `<script>`, not
   `web_sys`/`js_sys`.
4. **The only new JS is a new verb row or a new named behaviour**, and a behaviour needs a backlog row that exists
   before your lane starts. If what you need is missing, say so in your report; do not route around it.

## I want to ... use this

| I want to | Use this |
|---|---|
| react to a click, key, pointer, focus or input event on an element I render | the Dioxus `on*` prop; `event.prevent_default()` works there on every renderer |
| cancel an event from a window, document or outside target, or in the capture phase (Escape, F6, a page scroll lock, an OTP input filter, drop-anywhere) | a named behaviour via `use_behaviour` (`hotkey`, `scrollLock`, `otpFilter`, `dropAnywhere`); Rust publishes it beforehand and the page decides |
| take pointer capture, or route focus for the next input (drag, swipe, focus on open) | a named behaviour's effect (`pan`, `openFocus`, `settle`); **never** an async verb, **never** `set_focus` from a handler for this |
| open or close a popover or modal, set `indeterminate` or `defaultChecked` | render the state as a `data-dx-*` attribute; the one page reconciler (`reconcile`) applies it and reports back only what the browser did (light dismiss, Escape). Do not call `showPopover`/`showModal` from an effect |
| keep a scroller at its end while content grows | CSS `overflow-anchor` where verified, the Rust invariant behind it, the `stickToEnd` behaviour for engines without it |
| react to a window or document event, or an event type Dioxus has no prop for, when a late answer is harmless | the bridge's `use_event_listener` / `use_window_listener` (always passive; frame-rate events at most once per frame) |
| follow a stream (scroll, resize, pointermove, wheel, touchmove, drag) | `use_sample` / `use_resize_observer` / `use_intersection_observer`: async, coalesced, armed only while a condition holds. **Never a vdom handler** (see the desktop caveats) |
| call `showPopover` once, `scrollIntoView`, `setSelectionRange`, or write a `--dx-*` custom property | `ElementExt` (`el.show_popover()`, `set_var(...)`) over the closed `Method` enum, or `MountedData` where it suffices |
| read size, position, scroll offset or a computed style | `MountedData` where it suffices; `bridge::get(target, [Prop::...])` for the rest; show-and-measure in one JS turn with `show_popover_measured` or a `Transaction` |
| follow a media query or the window size | `use_media_query`, `use_window_size` |
| wait, time out or tick | `sleep`, `use_interval`, `use_timeout_while` (Rust tasks; never a JS timer) |
| place a popup next to its trigger | CSS anchor positioning first, behind the engine floors and the boot probe; Rust placement behind a paint-safe gate on engines that fail |
| run something before wasm loads (the theme and modality writers) | `PREPAINT_JS` only; that class is allowlisted with its reason |

## Where does my code go? The deadline table

Classify each **decision** by what a late answer costs, then by its kind. A block may use several rows for different
decisions. Read top to bottom; the first row whose deadline the decision has is its placement.

| deadline (what a late answer costs) | placement |
|---|---|
| this event's **default action** (cancel or not) | an **rsx handler** if Dioxus delivers the event **and every field the decision reads** (`ToggleData`, `FocusData`, `CancelData`, `SelectionData` are empty and `MouseData` has no `detail`: those decisions do not qualify); otherwise a **named behaviour** |
| this event's **task** (pointer capture, focus routing for the next input) | a **named behaviour's effect**; never an async verb, never rsx `set_focus` on desktop |
| **before paint** (a same-frame layout write) | **A** (the platform) where it does it, behind per-feature engine floors and a boot probe; otherwise a **named behaviour** (`stickToEnd`); **G** (a Rust answer behind an invisible pending state) only for one-shot placement where invisible-until-placed is acceptable |
| **next frame** (drag tracking, momentum) | **bridge stream plus Rust**, under the latency budget the owner sets from the desktop measurements; a named behaviour only where the measured lag exceeds it |
| **none** | **R**: Rust over the bridge (`listen`, `observe`, `sample`, `get`, `call`) |

Two kinds of decision, placed by that table:
- A **listener** reacts to events.
- A **command** mirrors Rust state into the page. Rust publishes the desired state as a `data-dx-*` attribute and one page
  reconciler applies it; it is level-triggered, so a lost, early or reordered command converges, and it never touches the
  native `open` attribute. One-shot commands (focus this cell, scroll that item into view) stay bridge calls.

A rsx handler takes **exactly one** commit-time effect, `prevent_default`. On desktop everything else a Rust handler does
reaches the page later: `set_focus` is an async webview query, renders land in the next animation frame, and Dioxus has no
pointer-capture API.

## Named behaviours

A behaviour is a closed registry entry, not a script and not a predicate language: a Rust `Behaviour` variant with typed
parameters, one pure `decide(config, memory, snapshot, event) -> effects` in `primitives/src/ts/behaviours/<kind>.ts`, and
a thin DOM adapter that applies the effects. It exists only because a decision has a **commit-sensitive effect** Rust cannot
take in time: `prevent`, `stop`, `capture`, `focus`, `dataTransfer` or a same-frame write. The gate reads each variant's
declared effect set; a variant with none cannot be added.

Planned registry (each is added by the lane that ports its block; **today none exists but a test fixture**):
`hotkey`, `scrollLock`, `otpFilter`, `dropAnywhere`, `pan`, `settle`, `scrollStep`, `openFocus`, `stickToEnd`, and
`reconcile` for commands. Rust declares target, phase and passivity; `passive` is derived (false only for a behaviour that
cancels). Installation is owned by the Rust scope that needs it and ref-counted, so a blocking listener exists only while
something needs it; every install carries an epoch, a fail-open path (a wedged Rust side must never trap the keyboard,
WCAG 2.1.2) and tears down when its element disconnects. When Dioxus later delivers the missing event or field (0.8's
`onbeforeinput`), the behaviour moves to an rsx handler; a test says so.

A stateful heuristic over a stream (a trackpad classifier, a scroll mode machine) is **not** a behaviour: it is Rust over
`listen`/`sample`, because it may answer late.

## The bridge (what exists)

All of it is in `primitives/src/bridge/` (`dioxus_primitives::bridge`). **Status:** the bridge is lane 3.0's branch
`origin/lane/js-bridge-3-0` and lands in the batch through the main loop; the surface below is that branch, and lane 3.0b
(row 434 phase 0) changes it as marked. Rust sends closed enums; the page maps each through a closed own-property table
with a receiver kind per row (`EVENTS`, `FIELD_GETTERS`, `METHODS`, `PROPS` in `ts/bridge.ts`); a name outside a table is
an error, never a property read. A verb row is **one member call or one member read**; anything bigger is a behaviour.

| verb | Rust |
|---|---|
| `call(el, method, args)` | `ElementExt` (`show_popover`, `hide_popover`, `show_modal`, `close_dialog`, `scroll_into_view`, `set_selection_range`, ...) over `Method`. `set_pointer_capture`, `release_pointer_capture`, `focus_prevent_scroll` and `commit` exist on the branch and are **removed by 3.0b**: they are effects of a behaviour, not verbs (a call after `pointerdown` lands after dispatch) |
| `get(el, props)` | `bridge::get` / `ElementExt::get` with `Prop` (`Rect`, `ActiveElement`, `VisualViewport`, `Style(name)`, `Attr(name)`, ...), answered as `Props` |
| `set(el, --var, value)` | `ElementExt::set_var`; custom properties that start with `--dx-` only |
| `listen(target, event, fields)` | `use_event_listener`, `use_window_listener`, `use_document_listener`, `use_element_listener`, `use_window_size`; always passive; hot events are coalesced per frame |
| `observe(el, kind, opts)` | `use_resize_observer`, `use_intersection_observer`, `use_mutation_observer`, `use_observer` |
| `matchMedia(query)` | `use_media_query` |
| `sample(el, fields, until)` | `use_sample` with `SampleField::read(&Prop)` (coalesced) or `SampleField::batched(..)` (every event with its timestamp, `.flushed_per_task()`); latest-wins, one message in flight, armed only while a gesture or condition holds |
| `behaviour(target, kind, ...)` | `use_behaviour`, `Behaviour` (`bridge/behaviours.rs`) |
| `transaction(ops)` | `Transaction`, `show_popover_measured`, `show_modal_measured`: several operations and their reads in one JS turn |

Elements are addressed by a `Target` (window, document, an element by `id` or selector, the active element); an element the
bridge acts on carries an `id`. Missing rows (`reset`, `beforematch`, `scrollend`, `dragend`, `invalid`, `reportValidity`,
`getAnimations`, `location.search`, `localStorage`, an `observe` publish option, ...) are added once by 3.0b, so a lane
consumes rows and does not add them; V5, V8, V9, V14 (cookie, `postMessage`, `fetch`, clipboard) take a reasoned preview-only
allowlist entry decided by the owner. Ordering is the transport's job: one FIFO channel, element operations flushed after
the render that issued them. Hooks never call `document::eval`; the transport seam in `bridge/transport.rs` is the one
place (plus three named pre-wasm exceptions). `window.__dxBridgeDelayFrames = N` is a **bridge-latency** harness, not a
desktop model.

## Desktop and mobile caveats

Web is the tested target; every desktop claim in the plan reads "web-verified; desktop inferred" until a
`desktop-pass/<tree>.pass` stamp exists (row 442). What to remember when you write code:

- **A Rust event handler is a blocking synchronous request per event on desktop and mobile**, including frame-rate ones, and
  Dioxus registers one root listener per event name: one `onpointermove` or `onwheel` anywhere makes that event a blocking,
  non-passive request across the whole app. `onmounted` is a blocking request inside the edit application too (one per
  list item in a collection is N round trips in one frame). Use the bridge's `listen`/`sample`/`observe` for frame-rate
  work; `check-hot-events` counts event names app-wide.
- **Rust cannot answer in the same frame.** A render lands in the next animation frame, an eval from a handler runs before
  its own render's edits, a reply costs an IPC hop, and while a render is unacknowledged (or the webview is hidden) no Rust
  task or timer runs. So a fail-open must live on the page, and a same-frame write is a behaviour or the platform.
- **Platform rows Chromium on the web cannot show:** Windows replaces HTML5 drag events under the default file-drop config
  (`dropAnywhere`, the native drag list), Android handles events on an IO thread under a global mutex with a header size
  limit, and release desktop builds install a document `contextmenu` listener that prevents default.
- **Web `eval` is `new Function`**, so an app under a strict Content-Security-Policy needs `unsafe-eval` for the bridge's
  transport.

## What is forbidden, and which gate says so

| Forbidden | Gate | Instead |
|---|---|---|
| `document::eval`, `use ... eval as run`, a bare `eval` function pointer, `document().eval`, `document::*` | `check-js-volume` (sink count, aliases included); clippy `disallowed_methods` once it lands | a bridge verb or behaviour |
| `document::Script { "..." }`, `script {}`, `dangerous_inner_html` on a script, `create_script` | `check-js-volume` | `Script { src: <asset> }` for the bridge only, owner-approved |
| JS split into `concat!` pieces, a script held in a `const` | `check-js-volume` (`js_lit`, pieces joined) | a verb row or a named behaviour |
| a new `.js`/`.ts` file, an `include_str!`/`asset!` of one | `check-js-volume` (`js_file`, sink) | a verb row or a named behaviour with a row |
| an inline `<script>` in an `.html` file, an `on...=` attribute, a `javascript:` href, raw HTML with a handler in a `.md` or `.svg` that reaches `dangerous_inner_html` | `check-js-volume` (`inline_script`, sink) | `PREPAINT_JS` copies stay as they are |
| `web_sys`, `js_sys`, `wasm_bindgen`, `gloo_events`, `evaluate_script` | `check-js-volume` (sink) | the bridge's transport is the one place |
| a vdom `onscroll`, `onresize`, `onpointermove`, `onmousemove`, `onwheel`, `ontouchmove`, `ondrag`, `ondragover` or `onmounted` handler | `check-hot-events` (per event name, app-wide; planned) | `listen`/`observe`/`sample` |
| `preventDefault`, `stopPropagation`, `requestAnimationFrame` or an observer in a `ts/` file outside `bridge.ts` and `ts/behaviours/`; a computed member access or string-form `setTimeout`/`Function` in `ts/` | `check-bridge-behaviours` (compiler-API scan, per-file budget) | a named behaviour or the bridge |
| a behaviour module that reads the DOM, a timer or the clock; a verb row with more than one member call; a behaviour variant with no commit-sensitive effect; a Rust variant without its module, bundle, generated kind or vector | `check-bridge-behaviours` | keep `decide` pure; go through the closed tables |
| a hand-edited `js/*.js` or `behaviours.gen.ts`, or a `.ts` edit without `scripts/build-js-bundle.sh` | `check-js-bundle`; `cargo test` (`the_generated_ts_is_current`) | rebuild; `DX_BLESS=1 cargo test -p dioxus-primitives behaviours` |
| a raw `addEventListener`, an uncleared interval, a blocking wheel listener in a script | `check-js-listeners`, `check-uncleared-intervals`, `check-blocking-scroll-listeners` (retire when the last legacy script goes) | `listen`; Rust timers |
| a comment that calls `document::eval` the crate's pattern | `check-js-volume` (`comment_eval`) | link this page |

## The baseline, and why you do not add to it

`scripts/check-js-volume.baseline.tsv` holds one row per file with a non-zero count. A port lowers its own file's row and
runs `scripts/check-js-volume.sh --update-baseline`; rows belong to files, so lanes that lower different files merge
cleanly. Your report quotes `scripts/check-js-volume.sh --report` before and after. A rise fails, and a baseline edited to
match is compared with the baseline at the merge base. **A raise passes only against a grant row** that exists at the merge
base (a backlog row, open, whose front matter says `js-grant: <column>=<n>`), and every raise prints under `RATCHET RAISES`;
a lane cannot grant itself. A new behaviour likewise needs a row at the merge base that names it (`adds: [<kind>]`); citing
the umbrella row 434 does not count. If you think you must raise, say so in the report with the row and the measured
reason, and wait for the owner. What counts is what ships (bundle bytes and JS held in Rust), not tests, `.d.ts`,
generated files or displayed code samples; moving a script from a Rust string into a new `.ts` still has to come out lower.

## Legacy scripts and the pre-wasm class

Everything counted in the baseline today is legacy, being ported by row 434 in phases
([`research/sync-js-root-2026-10-26.md`](./research/sync-js-root-2026-10-26.md) section 5.4); the audit
([`research/js-audit-2026-10-25.md`](./research/js-audit-2026-10-25.md)) classes each block. The legacy message scroller
and carousel scripts are frozen into `ts/legacy/` behind a non-default `legacy-impl` feature for the owner's A/B
measurement and are removed when the owner's measurement stamp exists, never on a date. The one class that stays JavaScript
on purpose is code that must run before wasm loads (`PREPAINT_JS`, the modality writer, and their copies in
`preview/index.html`, to be generated from one `.ts`); `check-theme-mode-writer` and `check-open-focus` keep those copies
identical until then.

## Adding a verb row or a named behaviour

- **A verb row is data.** A variant in the Rust enum (`Method`, `Prop`, `EventKind`, `EventField`), one row in the matching
  TS table with its receiver kind (one member call or read), then `scripts/build-js-bundle.sh`. `cargo test -p
  dioxus-primitives bridge` compares every Rust name with its row; `bun test primitives/ts-test` covers the page side.
- **A named behaviour needs a backlog row first** (open, at the merge base, `adds:` naming the kind), and: the variant in
  `bridge/behaviours.rs` (its declared effect set must pass admission; `info()`, `declared()`, `module()` and `samples()`
  have no wildcard arm), `ts/behaviours/<kind>.ts` (pure `decide`), vectors in `conformance/behaviours.json`, `DX_BLESS=1
  cargo test -p dioxus-primitives behaviours` to regenerate the TS types, `scripts/build-js-bundle.sh` (one bundle per
  behaviour), a Playwright spec tag, and a budget row. Behaviour lanes never edit a gate script.
- **The gates, by name:** `check-bridge-behaviours` (this layer), `check-js-bundle` (generated files), `check-js-volume`
  (shrink-only volume and the grant rule), `check-hot-events` (frame-rate vdom handlers), the three legacy script gates
  (retiring), `check-open-focus` and `check-theme-mode-writer` (the pre-wasm copies).

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
