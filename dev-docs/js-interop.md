# JavaScript interop: Rust first, the bridge second, no script strings

**Read this before you write any interactive behaviour.** It is short on purpose; the reasons are in
[`research/sync-js-root-2026-10-26.md`](./research/sync-js-root-2026-10-26.md) and the plan is backlog row 434.

## The rule

1. **Rust owns behaviour.** A Dioxus `on*` handler, a signal, a `MountedData` read (`get_client_rect`, `scroll_to`,
   `set_focus`), `use_interval`/`sleep` and CSS (anchor positioning, `scroll-padding`, `overflow-anchor`, `linear()`)
   come first. If one of them does it, nothing else is written.
2. **Place the rest with the three-way rule** (next section). Browser APIs Dioxus does not wrap go through the bridge
   in `primitives/src/bridge/` ([`bridge` module docs](../primitives/src/bridge/mod.rs)): generic verbs over **closed
   enums** and a registry of **named behaviours**. There is no `document::eval` outside `bridge/transport.rs`.
3. **No JS in Rust strings.** Not `document::eval`, not `document::Script { "..." }`, not `script {}` or
   `dangerous_inner_html` on a script, not `concat!` pieces, not an `include_str!` of a `.js`, not a new `.js`/`.ts`
   file, not an inline `<script>`, not `web_sys`/`js_sys`.
4. **The only new JS is a new bridge verb row or a new named behaviour**, each with a backlog row. If what you need is
   missing, add it to the bridge; do not route around it.

`scripts/check-js-volume.sh` ([`gates.md`](./gates.md)) enforces the last two by counting every place JS can enter and
comparing with the baseline at the merge base with `origin/main`: **the JS volume can only shrink**. A raise is the
owner's decision, never a lane's.

## The placement rule (decide per behaviour, in this order)

| the target and the event | placement | example |
|---|---|---|
| **Inside a Dioxus element, an event Dioxus supports, no listener option needed** | an **rsx handler**: Rust logic, no JavaScript. It decides synchronously on every renderer, so `event.prevent_default()` works. | `onkeydown` on a panel, `oncancel`, `ontoggle` where Rust knows which state it asked for |
| **A window, document or "outside" target; an event Dioxus lacks (`beforeinput` before 0.8, `close`); the capture phase** | a **named [`Behaviour`](../primitives/src/bridge/behaviours.rs)**: a variant with typed parameters and one pure `decide()` in `primitives/src/ts/behaviours/<kind>.ts`, installed by the Rust scope that needs it (`use_behaviour`). The decision must be made inside the event's dispatch, so Rust publishes the behaviour beforehand. | an Escape or F6 hotkey, a page scroll lock, an OTP input filter, drop-anywhere |
| **A continuous stream** (scroll, resize, pointermove, wheel, touchmove, mousemove, drag) | the bridge's **`listen` / `observe` / `sample`** (`use_event_listener`, `use_resize_observer`, `use_intersection_observer`, `use_sample`): async, coalesced per frame or flushed per event task, armed only while a condition holds. **Never a vdom handler**: on desktop each vdom event is a blocking synchronous request. | scroll position, a measurement publisher, a wheel gesture's samples |

**Named behaviours today: none.** The mechanism exists with one test fixture (`fixture`) and no component uses it;
each real behaviour (`hotkey`, `scrollLock`, `otpFilter`, `dropAnywhere`, `pan`, `settle`, ...) is added by the lane that
ports one block, as a variant plus its module, its vectors and its spec. What a behaviour is, concretely:

- a Rust `Behaviour` variant (`bridge/behaviours.rs`). `cargo test` writes `ts/behaviours/behaviours.gen.ts` from the enum
  and fails on drift (`DX_BLESS=1 cargo test -p dioxus-primitives behaviours` rewrites it);
- a module `ts/behaviours/<kind>.ts` that calls `register("<kind>", { fields, reads, decide })` and nothing else: `decide`
  is **pure** over closed event fields and closed attribute reads, and returns `{ prevent?, stop?, attrs?, send? }`;
- Rust **declares** target, event, phase and passivity (`Behaviour::declared()`); `passive` is derived (false only for a
  behaviour that can cancel), never a free option. `prevent` is honoured only when non-passive, `stop` only for a
  non-passive capture behaviour at an element, `defaultPrevented` may be read only in the bubble phase, `attrs` are
  limited to `data-*`/`aria-*`;
- **installation is ref-counted**: the page keeps one DOM listener per (target, type, capture, passive), so a blocking
  listener exists only while a behaviour needs it, and the Rust scope's drop removes it (over the channel it already
  has);
- every install carries an **epoch** (older than the target's newest is dropped, equal renews) and may carry a **lease**:
  the page removes the listener itself when Rust stops renewing, so a wedged Rust side leaves the page's native
  behaviour (WCAG 2.1.2);
- vectors in `primitives/conformance/behaviours.json` run through the real `decide()` under `bun test`, and `cargo test`
  parses each as a `Behaviour`.

A stateful heuristic over a stream (a trackpad classifier, an anchoring mode machine) is **not** a behaviour: it is
Rust over `listen`/`sample`, because it may answer late.

## The bridge API (what exists)

All of it is in `primitives/src/bridge/` (`dioxus_primitives::bridge`). The surface is closed: Rust sends enums, the page
maps each through a closed own-property table with a receiver kind per row (`EVENTS`, `FIELD_GETTERS`, `METHODS`,
`PROPS` in `ts/bridge.ts`); a name outside a table is an error, never a property read.

| I want to | Use this |
|---|---|
| react to a click, key, pointer, focus or input event on an element | the Dioxus `on*` prop |
| react to a window or document event, or to an event type Dioxus has no prop for, without cancelling | `use_window_listener` / `use_document_listener` / `use_element_listener` / `use_event_listener(target, EventKind, ListenOptions, &[EventField], handler)`; always passive; a frame-rate event arrives at most once per frame |
| cancel or stop an event from the page's side | a named `Behaviour` via `use_behaviour` |
| call `showPopover`, `showModal`, `close`, `focus({ preventScroll })`, `scrollIntoView`, `setSelectionRange`, pointer capture | `ElementExt` (`el.show_popover()` ...) over the closed `Method` enum; **not** at `pointerdown` (the call lands after dispatch): that is a behaviour effect or an rsx handler |
| read size, position, scroll offset or a computed style | `MountedData` where it suffices; `bridge::get(target, [Prop::...])` for the rest (`ActiveElement`, `VisualViewport`, `Style(name)`) |
| show an element and read its rects in one JS turn | `show_popover_measured` / `show_modal_measured`, or a `Transaction` |
| write a theme custom property on an element whose style Rust does not own | `ElementExt::set_var("--dx-name", Some(value))` |
| know when a size, visibility or subtree changes | `use_resize_observer` / `use_intersection_observer` / `use_mutation_observer` (an `onresize`/`onvisible` prop where it suffices) |
| follow a media query, or the window size | `use_media_query`, `use_window_size` |
| sample per frame while a gesture or condition holds | `use_sample` with `SampleField::read(&Prop)` (coalesced) or `SampleField::batched(..)` (every event with its timestamp; `.flushed_per_task()` for the event task) |
| order a write against a later read, or drop a late answer | `Transaction` (one JS turn), `Epoch` on a call, read or install |
| wait, time out or tick | `sleep`, `use_interval`, `use_timeout_while` (Rust tasks; never a JS timer) |
| place a popup next to its trigger | CSS anchor positioning first; the legacy placement script is being ported |
| run something before wasm loads (the theme and modality writers) | `PREPAINT_JS` only; that class is allowlisted with its reason |

Timing per renderer (the table is in `bridge/transport.rs`; web is verified, desktop inferred from Dioxus's sources): an
eval from a handler runs before its own render's edits; on desktop edits apply in a `requestAnimationFrame`, at most one
render per frame, replies arrive an IPC hop later, and `commit()` is only a style flush on web. The
`window.__dxBridgeDelayFrames` harness is a **bridge-latency** harness, not a desktop model: rows say
"web-verified; desktop inferred" until a desktop-cadence mode and a real-webview pass exist.

## Why not a script

Two lines of root cause, from [`research/sync-js-root-2026-10-26.md`](./research/sync-js-root-2026-10-26.md). The browser
commits twice per turn of its main thread: it applies an event's default action when dispatch ends and it paints after
layout, so only code on the page's own thread with data already there can affect either. Dioxus event handlers already
run synchronously, so `prevent_default()` in Rust is fine; what is asynchronous is a message to Rust over an eval
channel, and nothing should be decided in one. The rule is therefore: never ask Rust at commit time; Rust publishes the
behaviour and its state beforehand and a pure `decide()` on the page decides. That is the bridge.

The cost of the alternative showed up in the code. 103 bespoke eval blocks (about 6,200 lines) were written because the
only written convention for "I need the DOM" was `document::eval`, comments next to the code called it the crate's
established pattern, every JS gate checked the shape of the script string (and so blessed the string), and a script
needs no new file, verb or review. The neighbour you copy teaches the way, so this page and `check-js-volume` exist.

## Desktop and mobile caveats

Web is the tested target. Two facts matter for the others (row 434, `research/sync-js-root-2026-10-26.md` section 1.2):

- **Rust event handlers are a blocking synchronous request per event on desktop and mobile.** The webview's JS does a
  synchronous XHR to Rust for every event a vdom handler listens to, including frame-rate ones (`pointermove`, `scroll`,
  `wheel`). A handler that does real work, or a high-frequency event with a handler attached, costs a round trip each
  time. The bridge's coalesced `listen` and batched `sample` exist so per-frame work stays on the page's side.
- **Web `eval` is `new Function`**, so an application with a strict Content-Security-Policy needs `unsafe-eval` for the
  bridge's transport. Pre-wasm scripts and the bridge's bundles (`js/bridge.js` and one `js/behaviours/<kind>.js` each, evaluated through the transport) are the only JavaScript the library ships.

## What is forbidden, and which gate says so

| Forbidden | Gate | Instead |
|---|---|---|
| `document::eval`, `use ... eval as run`, `document().eval`, `document::*` | `check-js-volume` (sink count, aliases included) | the bridge verb |
| `document::Script { "..." }`, `script {}`, `dangerous_inner_html` on a script, `create_script` | `check-js-volume` | `Script { src: <asset> }` for the bridge only, owner-approved |
| JS split into `concat!` pieces, a script held in a `const` | `check-js-volume` (`js_lit`, pieces joined) | a bridge verb row or a named behaviour |
| a new `.js`/`.ts` file, an `include_str!`/`asset!` of one | `check-js-volume` (`js_file`, sink) | a bridge verb row or a named behaviour with a backlog row |
| an inline `<script>` in an `.html` file, an `on...=` attribute, a `javascript:` href | `check-js-volume` (`inline_script`, sink) | `PREPAINT_JS` copies stay as they are |
| `web_sys`, `js_sys`, `wasm_bindgen`, `evaluate_script` | `check-js-volume` (sink) | the bridge's transport is the one place |
| a raw `addEventListener`, an uncleared interval, a blocking wheel listener in a script | `check-js-listeners`, `check-uncleared-intervals`, `check-blocking-scroll-listeners` (until the scripts are gone) | `listen`; Rust timers |
| `preventDefault`, `stopPropagation`, `requestAnimationFrame` or an observer in a `ts/` file outside `bridge.ts` and `ts/behaviours/` | `check-bridge-behaviours` (expiring allowlist, one lane per entry) | a named behaviour or the bridge |
| a behaviour module that reads the DOM, a timer or the clock; `el[name]`, `eval`, `new Function`, `JSON.parse` in `ts/`; `document::eval` outside `bridge/transport.rs`; a Rust variant without its module, bundle, generated kind or vector | `check-bridge-behaviours` | keep `decide` pure; go through the closed tables |
| a hand-edited `js/*.js` or `behaviours.gen.ts`, or a `.ts` edit without `scripts/build-js-bundle.sh` | `check-js-bundle`; `cargo test` (`the_generated_ts_is_current`) | rebuild; `DX_BLESS=1 cargo test -p dioxus-primitives behaviours` |
| a comment that calls `document::eval` the crate's pattern | `check-js-volume` (`comment_eval`) | link this page |

## The baseline and why you do not add to it

`scripts/check-js-volume.baseline.tsv` holds one row per file with a non-zero count. A port lowers its own file's row and
runs `scripts/check-js-volume.sh --update-baseline`; rows belong to files, so lanes that lower different files merge
cleanly. Your report quotes `scripts/check-js-volume.sh --report` before and after. A rise fails, and a baseline edited
to match is compared with the baseline at the merge base: it passes only with a `# raised: row N <reason>` note that the
base does not have (written by `--update-baseline --allow-raise "row N reason"`), and every such raise prints under a
`RATCHET RAISES` heading at each run. That is visibility for the owner, not permission: if you think you must raise, say
so in the report with the row and the measured reason, and wait for the owner. The total `js_lit + js_file` must not
rise either, so moving a script from a Rust string into a new `.ts` lowers one column and raises the other and still
has to come out lower.

## Legacy scripts and the pre-wasm class

Everything counted in the baseline today is legacy, being ported by row 434 in batches; the audit
([`research/js-audit-2026-10-25.md`](./research/js-audit-2026-10-25.md)) classes each block. The one class that stays
JavaScript on purpose is code that must run before wasm loads (`PREPAINT_JS`, the modality writer, and their copies in
`preview/index.html`); `check-theme-mode-writer` and `check-open-focus` keep those copies identical.

## Adding a verb row or a named behaviour

- **A verb row is data.** A variant in the Rust enum (`Method`, `Prop`, `EventKind`, `EventField`), one row in the
  matching TS table (`METHODS`, `PROPS`, `EVENTS`, `FIELD_GETTERS`) with its receiver kind, then
  `scripts/build-js-bundle.sh`. `cargo test -p dioxus-primitives bridge` compares every Rust name with its row (and
  `EVENTS` rows with their receivers and hot flag), `bun test primitives/ts-test` covers the page side.
- **A named behaviour needs a backlog row**, and: the variant in `bridge/behaviours.rs` (`info()`, `declared()`,
  `module()` and `samples()` have no wildcard arm), `ts/behaviours/<kind>.ts` (pure `decide`), vectors in
  `conformance/behaviours.json`, `DX_BLESS=1 cargo test -p dioxus-primitives behaviours` to regenerate the TS types,
  `scripts/build-js-bundle.sh` (one bundle per behaviour, `js/behaviours/<kind>.js`), a Playwright spec tag, and a line
  budget row in the baseline (`check-js-volume`). Behaviour lanes never edit `scripts/check-bridge-behaviours.sh`.
- **The gates, by name:** `check-bridge-behaviours` (this layer), `check-js-bundle` (generated files), `check-js-volume`
  (shrink-only volume), `check-js-listeners`, `check-uncleared-intervals` and `check-blocking-scroll-listeners` (legacy
  scripts, retiring), `check-open-focus` and `check-theme-mode-writer` (the pre-wasm copies).
