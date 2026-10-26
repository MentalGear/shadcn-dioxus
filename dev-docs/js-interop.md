# JavaScript interop: Rust first, the bridge second, no script strings

**Read this before you write any interactive behaviour.** It is short on purpose; the reasons are in
[`research/sync-js-root-2026-10-26.md`](./research/sync-js-root-2026-10-26.md) and the plan is backlog row 434.

## The rule

1. **Rust owns behaviour.** A Dioxus `on*` handler, a signal, a `MountedData` read (`get_client_rect`, `scroll_to`,
   `set_focus`), `use_interval`/`sleep` and CSS (anchor positioning, `scroll-padding`, `overflow-anchor`, `linear()`)
   come first. If one of them does it, nothing else is written.
2. **Browser APIs Dioxus does not wrap go through the bridge** in `primitives/src/bridge/`: generic verbs (`listen`,
   `call`, `get`, `set`, `observe`, `matchMedia`, `transaction`, `commit`, `sample`) and rules as data (a closed Rust
   `Rule` enum applied by one interpreter). The bridge lands with row 434 lane 3.0; until it does, **add no JS at all**
   (the legacy scripts below are being ported, not copied).
3. **No JS in Rust strings.** Not `document::eval`, not `document::Script { "..." }`, not `script {}` or
   `dangerous_inner_html` on a script, not `concat!` pieces, not an `include_str!` of a `.js`, not a new `.js`/`.ts`
   file, not an inline `<script>`, not `web_sys`/`js_sys`.
4. **The only new JS is a new bridge verb or `Rule` variant**, each with a backlog row. If the verb or rule you need is
   missing, add it to the bridge; do not route around it.

`scripts/check-js-volume.sh` ([`gates.md`](./gates.md)) enforces the last two by counting every place JS can enter and
comparing with the baseline at the merge base with `origin/main`: **the JS volume can only shrink**. A raise is the
owner's decision, never a lane's.

## I want to ... use this

| I want to | Use this |
|---|---|
| react to a click, key, pointer, focus or input event on an element | the Dioxus `on*` prop; `event.prevent_default()` works there (synchronous on web and desktop) |
| react to a window or document event, or to an event type Dioxus has no prop for | the bridge's `listen` (`use_window_listener` / `use_listener`); passive by default |
| call `showPopover`, `showModal`, `close`, `focus({ preventScroll })`, `scrollIntoView`, `setSelectionRange` | the bridge's `call` (`el.show_popover()` ...); `set_focus` and `scroll_to` from `MountedData` where they suffice |
| read size, position, scroll offset or a computed style | `MountedData::get_client_rect` / `get_scroll_offset`; the bridge's `get` for the rest (`activeElement`, `visualViewport`, computed style) |
| know when a size, visibility or subtree changes | the bridge's `observe` (`use_resize`, `use_intersection`); an `onresize`/`onvisible` prop where Dioxus has one |
| follow a media query | the bridge's `matchMedia` (`use_media_query`) |
| wait, time out or tick | `sleep`, `use_interval`, `use_timeout_while` (Rust tasks; never a JS timer) |
| cancel an event conditionally (a wheel band, Escape in nested layers, an OTP filter) | a `Rule::Gate`: Rust publishes the rule and its state into the DOM beforehand, one interpreter decides at event time |
| drag or swipe | `Rule::Pan` |
| keep a value that must follow layout (a height, an offset) | a CSS rule first; else `Rule::Publish` |
| place a popup next to its trigger | CSS anchor positioning first; the legacy placement script is being ported |
| run something before wasm loads (the theme and modality writers) | `PREPAINT_JS` only; that class is allowlisted with its reason |

## Why not a script

Two lines of root cause, from [`research/sync-js-root-2026-10-26.md`](./research/sync-js-root-2026-10-26.md). The browser
commits twice per turn of its main thread: it applies an event's default action when dispatch ends and it paints after
layout, so only code on the page's own thread with data already there can affect either. Dioxus event handlers already
run synchronously, so `prevent_default()` in Rust is fine; what is asynchronous is a message to Rust over an eval
channel, and nothing should be decided in one. The rule is therefore: never ask Rust at commit time; Rust publishes the
rule and its state beforehand and one local interpreter decides. That is the bridge.

The cost of the alternative showed up in the code. 103 bespoke eval blocks (about 6,200 lines) were written because the
only written convention for "I need the DOM" was `document::eval`, comments next to the code called it the crate's
established pattern, every JS gate checked the shape of the script string (and so blessed the string), and a script
needs no new file, verb or review. The neighbour you copy teaches the way, so this page and `check-js-volume` exist.

## Desktop and mobile caveats

Web is the tested target. Two facts matter for the others (row 434, `research/sync-js-root-2026-10-26.md` section 1.2):

- **Rust event handlers are a blocking synchronous request per event on desktop and mobile.** The webview's JS does a
  synchronous XHR to Rust for every event a vdom handler listens to, including frame-rate ones (`pointermove`, `scroll`,
  `wheel`). A handler that does real work, or a high-frequency event with a handler attached, costs a round trip each
  time. The bridge's batched `sample` and the interpreter exist so per-frame work stays on the page's side.
- **Web `eval` is `new Function`**, so an application with a strict Content-Security-Policy needs `unsafe-eval` for the
  bridge's transport. Pre-wasm scripts and the bridge's asset are the only scripts the library ships.

## What is forbidden, and which gate says so

| Forbidden | Gate | Instead |
|---|---|---|
| `document::eval`, `use ... eval as run`, `document().eval`, `document::*` | `check-js-volume` (sink count, aliases included) | the bridge verb |
| `document::Script { "..." }`, `script {}`, `dangerous_inner_html` on a script, `create_script` | `check-js-volume` | `Script { src: <asset> }` for the bridge only, owner-approved |
| JS split into `concat!` pieces, a script held in a `const` | `check-js-volume` (`js_lit`, pieces joined) | a verb or a rule |
| a new `.js`/`.ts` file, an `include_str!`/`asset!` of one | `check-js-volume` (`js_file`, sink) | a bridge verb or `Rule` variant with a row |
| an inline `<script>` in an `.html` file, an `on...=` attribute, a `javascript:` href | `check-js-volume` (`inline_script`, sink) | `PREPAINT_JS` copies stay as they are |
| `web_sys`, `js_sys`, `wasm_bindgen`, `evaluate_script` | `check-js-volume` (sink) | the bridge's transport is the one place |
| a raw `addEventListener`, an uncleared interval, a blocking wheel listener in a script | `check-js-listeners`, `check-uncleared-intervals`, `check-blocking-scroll-listeners` (until the scripts are gone) | `listen`; Rust timers |
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

## Adding a verb or a rule (once the bridge exists)

A verb is data: one arm in the Rust verb enum, one line in the bridge's receiver table, one conformance vector, then
`scripts/build-js-bundle.sh`. A rule kind is language growth and needs a backlog row, a pure decide function with
vectors, and a budget row in the baseline. This section is filled in by lane 3.0 when `primitives/src/bridge/` lands.
