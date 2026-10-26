---
id: 434
title: "Cut JavaScript to what Rust cannot do: audit every eval block, move the rest into Rust, TypeScript only for the remainder"
status: "BACKLOG. Approved as P9 of the row 365 sweep; the owner re-scoped it to Rust first on 2026-10-25.…"
size: "L"
area: primitives
updated: 2026-10-26
---

# Row 434: Cut JavaScript to what Rust cannot do

## Item

The owner's principle (2026-10-25): the point of Dioxus is to drive the UI from Rust/wasm, so JavaScript is the exception, not the default. The library has no `web-sys` dependency; every browser API Dioxus does not wrap goes through `document::eval`, the one route that works on web, desktop and mobile. 72 files carry eval. The owner chose **Rust first** over "Rust plus web-sys" (one code path per feature, every renderer) and over the original TypeScript move.

1. **Audit** every eval block and classify it: (a) Dioxus already has it in Rust (`MountedData::get_client_rect`, `scroll_to`, `set_focus`, event data, signals, `onvisible`/`onresize` where they exist), so move it; (b) Dioxus has no Rust API (`showPopover`/`showModal`, `matchMedia`, pointer capture, listener options such as passive/capture, `scrollend`, `getAnimations`), so it stays JS; (c) must run before wasm (`PREPAINT_JS`, the modality writer), so it stays JS; (d) JS on purpose for latency (one-frame measure-and-place in the anchor engine, scroll bridge, virtual find), so it stays only with a measured reason.
2. **Move** every (a) block to Rust, per cluster, with the components' specs on a release build as the proof (timing-sensitive paths compare long tasks before and after).
3. **What stays** (b, c, d) moves into typed `primitives/src/ts/` modules through the existing bundle chain (the original P9 construction), and the copies become one file.

## Audit result (2026-10-25)

[`../research/js-audit-2026-10-25.md`](../research/js-audit-2026-10-25.md): 103 blocks, about 6,205 lines. (a) movable now 21 blocks / 624 lines (10%); (b) no Dioxus API 55 / 1,580 (25%); (c) before wasm 5 / 107 (2%); (d) JS on purpose 22 / 3,894 (63%). Upstream `main`/0.8.0-alpha.3 adds no element or window API, so no upgrade is waited for. Step 2: about 26 agent-hours in 5 lanes (2A styles and toast pause, 2B focus by handle, 2C scroll and measure, 2D virtual bridge experiment with a keep-JS exit, 2E demo sleeps). Step 3: about 66 agent-hours in 8 lanes over two batches (3.0 the typed channel and loading first).

## Step 3 design: one small generic JS bridge (owner 2026-10-26)

The owner asked for a uniform small JS glue layer to Rust instead of many bespoke scripts, which is also how Dioxus frames `document::eval` interop. So step 3 is no longer "every remaining block becomes a TypeScript module". Instead:

- **Lane 3.0 builds the bridge.** One small typed module (`primitives/src/ts/bridge.ts`, through the existing bundle chain) with generic verbs only and no component logic. It covers listen, call, get, observe and match:
  - `listen(target, type, options, fields)`: window, document or element; `capture`, `passive` and `once`; it streams the named event fields to Rust over the eval channel and grows out of `js_listener.rs`'s `listen`, which already owns removal.
  - `call(el, method, args)`: `showPopover`, `hidePopover`, `showModal`, `close`, `setPointerCapture`, `focus({preventScroll})`, `scrollIntoView(opts)`, `setSelectionRange`.
  - `get(el, props)`: computed style values, `offset*`, `activeElement`, `visualViewport`.
  - `observe(el, kind)`: ResizeObserver, IntersectionObserver (with rootMargin and threshold), MutationObserver.
  - `matchMedia(query)`: a subscription.
  On the Rust side these are typed hooks and extension traits (`use_window_listener`, `el.show_popover()`, `use_media_query`), so the logic of every (b) block moves into Rust.
- **What the bridge cannot carry stays as named JS modules, each with a measured reason.** Bridge messages reach Rust asynchronously, so two kinds of work can't go through it. The first is a decision that must happen inside the listener call, such as a conditional `preventDefault` (wheel band, drag, Escape inside nested layers, OTP filtering). The second is read-then-write in one frame (anchor placement, the virtual scroll bridge unless 2D shows otherwise, carousel drag and wheel). Those become TypeScript modules as originally planned. The audit's (d) rows say which apply.
- **Expected effect:** class (b), about 25% of the JS, becomes Rust plus generic verbs. Class (d) shrinks to the parts that are genuinely synchronous. Class (c), pre-wasm, stays as is. Lane 3.0 is sized again once the bridge exists, and the per-cluster lanes then remove blocks against it.
- **Gate:** besides `check-js-in-rust.sh`, any new `eval` must use a bridge verb, or be a named module listed with its (c) or (d) reason.

## Status / trigger

BACKLOG. Approved as P9 of the row 365 sweep; the owner re-scoped it to Rust first on 2026-10-25. The audit (step 1) is read-only and can run at any time; the moves follow it.

## Size

L

## Detail

Source: [final sweep note, P9](../research/final-sweep-2026-10-25.md). One lane per script cluster (carousel, virtual engine, top layer and overlay, the rest). The bundle chain exists and holds one file.

- **Measured:** `grep -rln "document::eval\|eval("` gives 45 files in `primitives/src`. Blocks include `virtual_grid.rs:1954` (`CELL_CONTROLS_JS`), `:2087` (`FOCUS_CELL_JS`), `virtual/find.rs:135,282`, `virtual/bridge.rs:293` (`SCROLL_BRIDGE_JS`), `virtual/scroll_to.rs:190`, `select/components/list.rs:102` (`ITEM_ALIGNED_JS`), `top_layer/scroll_clamp.rs:58,141,261`, `top_layer/styles.rs:643`, `collection.rs:17`, `modality.rs:70,87` (`:70` is a minified one-liner with `/*listener-ok: ...*/` inside), and `carousel/{autoplay,wheel,drag,scroll}.rs` (7,431 lines under `carousel/`, built with `concat!`).
- **The chain exists:** `primitives/src/ts/` holds one file (`focus-trap.ts`), with `scripts/build-js-bundle.sh`, `primitives/src/js/bundle.manifest` and `check-js-bundle.sh` (committed `.js`, sha256 manifest).
- **Cost shows up as exceptions and copies:** `listener-ok:` 18, `blocking-ok:` 4, `interval-ok:` 3 markers. The modality writer exists in three byte-identical copies (`modality.rs`, `PREPAINT_JS`, `preview/index.html`) that `check-open-focus.sh:34,115` compares; `PREPAINT_JS` exists in two that `check-theme-mode-writer.sh:132` compares.
- **Subsumes:** the 46 blocks over time; the three-copy and two-copy identity checks; the regex-over-Rust complications of four gates; the `listener-ok` markers that sit inside eval strings. **Does not subsume:** one-line `eval` calls that read one value; the carousel needs its own plan because it builds its script with `concat!`/macros.
- **Gate:** `check-js-in-rust.sh`: every eval site carries its audit class; an (a)-class eval, a raw string over N lines, or any `addEventListener` in a `.rs` file fails unless listed with a reason. The JS gates scan `.ts`.
- **Risk:** timing-sensitive drift while moving (scroll bridge, find, wheel band). Each lane's proof is the specs of the components that use its block, on a release build. Typed access to Dioxus's eval channel (`dioxus.send`) needs a declaration file.

## Progress: step 2, lane 2C "scroll and measure" (2026-10-26)

Branch `lane/js-measure-2c`. **Moved to Rust (3 of 6):** S2 (Select scroll buttons' can-scroll state, from the list's own `onscroll`/`onresize`), S3 (hover auto-scroll step, `get_scroll_offset` + `scroll(.., Instant)`) and S19 (NavigationMenu indicator, `get_client_rect` of the list and the open item, written through the indicator's own `style`). **Kept in JS, reasons in the audit rows:** S4 (needs list padding/border, layout height for the open animation's scale and `:hover`, none of which Rust can read; `scroll_to` is `scrollIntoView` and scrolls ancestors), L22 (needs per-host wiring in `navbar.rs`/`navigation_menu.rs`, not a hook-only change) and V6 (scroller handle lives in `SidebarContent`, `getComputedStyle`/`offsetParent` reads). Files touched outside the lane list, because only the list element can carry the handlers: `select/components/list.rs` (provides `ListScroll`, adds `onscroll`/`onresize`), and the `onmounted` of `NavigationMenuList`/`NavigationMenuItem`. Release build: `main-thread` select/combobox/navigation_menu open and close 0 long tasks (max interaction 16-40 ms against the 200 ms bound); hover auto-scroll on `/component/select/` 0 long tasks, 60 fps frames (p95 16.7 ms), 346 px in 1.5 s against the nominal 360 px/s. Desktop (two webview round trips per 16 ms tick for S3) was not measured. Pre-existing and unrelated: `native-dialog.spec.ts` 6e reads `viewport.innerHeight` inside a page callback (NaN), red on this branch's base.

## Step 2 progress

- **Lane 2B, focus by handle (branch `lane/js-focus-2b`, 2026-10-26):** L17 (DropdownMenu, ContextMenu; Select keeps the id script until its trigger adopts `TriggerHandle`), L19, S5, S6 and V15 moved to `MountedData::set_focus` (22 script lines removed, no new JS). L18 and L20 stay JS, with the reason in their audit rows: L20 measured (a plain `focus()` moves a half-visible carousel's page by 157 px where `preventScroll` moves it by 0), L18 needs `activeElement` (focus-within) that Rust cannot read. Two files outside the lane's list carry the combobox handle field (`combobox/context.rs`, `combobox/components/combobox.rs`). The row stays open: lanes 2A, 2C, 2D, 2E and step 3.

## Lane 3.0 result: the generic bridge (2026-10-26, branch `lane/js-bridge-3-0`)

What exists now, for the step 3 lanes to build on (none of them migrates a component block yet; `js_listener.rs` is the one consumer):

- **JS half:** `primitives/src/ts/bridge.ts` (bundled to `js/bridge.js`, ~11 KB, a script that installs `window.__dxBridge`) with `scope`, `channel`, `serve`, `pump`, `call` (a closed method list, popover/dialog methods tolerate their state), `get` (computed style, `offset*`/`client*`/`scroll*`, `rect`, `activeElement` with `within`, `visualViewport`, window metrics), `set(el, --var, value)`, `commit`, `transaction`, `sample` (per field **coalesced** or **batched**: every event of the frame with its own `timeStamp`, one message per frame, for the C5 wheel classifier), `observe` (resize, intersection with `rootMargin`/`threshold`, mutation), `matchMedia` and `rules` (`install`/`remove`/`read`/`sync` of `data-dx-rule-<kind>` attributes). `ts/rules/index.ts` is the interpreter: a `switch (rule.kind)` with **no default arm**, empty regions B1, B2, B3, B5 and one test fixture rule. `ts/types/dioxus.d.ts` types the eval channel, `primitives/tsconfig.json` type-checks `ts/` (`tsc`, run by the gate when installed).
- **Rust half:** `primitives/src/bridge/`: `transport.rs` (the ONE module that names `document::eval`), `session.rs` (`Session`/`Subscription`, hooks `use_event_listener`, `use_window_listener`, `use_document_listener`, `use_element_listener`, `use_media_query`, `use_window_size`, `use_resize_observer`, `use_intersection_observer`, `use_mutation_observer`, `use_observer`, `use_sample`, `use_rule`), `mod.rs` (`Target`, `Method`, `Prop`/`Props`, `ElementExt` with `show_popover`, `hide_popover`, `show_modal`, `close_dialog`, `set_pointer_capture`, `focus_prevent_scroll`, `scroll_into_view`, `set_selection_range`, `set_var`, `get`, `Transaction`, `commit`) and `rules.rs` (the closed `Rule` enum with the same regions). `js_listener.rs`'s `listen`/`cleanup` are now the bridge scope's `on`/`cleanup`; its public API is unchanged.
- **Gate:** `scripts/check-bridge-rules.sh` (in `run-gates.sh`): the Rust enum and the interpreter switch match one to one, no default arm, every kind inside a region, a conformance vector (`primitives/conformance/rules.json`, run by `bun test` and `cargo test`) and a `bun test` case per kind; no `preventDefault`/`stopPropagation`/`requestAnimationFrame`/observer in `ts/` outside `bridge.ts`, `ts/rules/` and `scripts/check-bridge-rules.allow.tsv` (seeded with `focus-trap.ts`, reason `3.B1`; every reason must name its step 3 lane, the target is an empty list); `document::eval` only in `transport.rs`; the committed bundle still exports `window.__dxBridge` and every verb (#5183); 27 self-test mutations. `check-js-bundle.sh`/`build-js-bundle.sh` now hash every `.ts` below `ts/` (a module of `ts/rules/` edited without a rebuild is red) and skip `.d.ts`.
- **Tests:** `bun test primitives/ts-test` (47 cases against the shipped bundle and a fake browser, including a thousand commands over one channel), `playwright/bridge-dom.spec.ts` (13 tests on real Chromium: popovers, focus without scroll, a style flush that makes a transition start, observers, `matchMedia`, batched wheel samples, element-bound lifetime, the delay modes), Rust unit tests for every wire shape.

Decisions taken from the best-practice review (`../research/dioxus-best-practice-review-2026-10-26.md`):

1. **One transport seam.** Hooks never call `document::eval`; `transport.rs` owns loading, the ready handshake, the delay mode, the ordered write channel and the `Unsupported` answer (`bridge::is_unsupported()`, `BridgeError::Unsupported`; Blitz subscriptions end quietly). A future web-sys backend replaces the three functions there.
2. **Loading: the bundle stays on the eval route, `asset!` plus `document::Script` was NOT adopted.** Reasons: it needs every consuming app to mount a provider, the script is inserted by a queued head effect while an eval runs immediately (#5919), and a bundler pass may strip a function nothing references at top level (#5183). Not prototyped on desktop (not in CI); reading the Dioxus sources and issues only. Instead: every script starts with a bounded (~2 s) wait for `window.__dxBridge` and fails loudly; "installed" is a root-context marker of the VirtualDom (per webview, not a process flag); the exports are assigned to `window` and the gate checks the built `.js` for them.
3. **Teardown never starts an eval.** Dropping a subscription sends one message on its own channel (#4064). Registrations on an element also end on the page when the element leaves the document (one page-wide `MutationObserver`, created with the first and disconnected with the last), because `onunmounted` is not merged upstream. The first bridge use of a webview creates the pump, so it must not be inside a drop handler.
4. **Fail-open harness.** `window.__dxBridgeDelayFrames = "never"` drops every message to Rust (commands from Rust still arrive); `N` defers both directions by N frames. The CSS-only timeout of a G gate is a later lane's.
5. **One ordered write channel per webview** (`pump`): `call`, `set_var` and `Transaction::fire` run in order; a read is its own request, so an ordered write-then-read is one `Transaction`.
6. **CSP:** web `document::eval` compiles with `new Function`, so a strict-CSP app needs `unsafe-eval` for the bridge, `js_listener` and every eval-driven primitive (upstream has the same limit). Documented in `bridge/mod.rs`.
7. **Use Dioxus's own API first** (`onresize`, `onvisible`, `onscrollend`, `MountedData`, `onbeforeinput` in 0.8); the bridge observers are for `rootMargin`/`threshold`, `MutationObserver`, non-element targets and listener options. On desktop `onresize` bubbles, so stop its propagation in the child.

Not done in this lane, said plainly: the delegated capture-phase interpreter of the review (rules are installed per element, from `rules.install` or synced from `data-dx-rule-<kind>`; the delegation belongs with 3.B1, whose event set decides which events to delegate and whether a blocking `wheel` listener on `document` is acceptable); `Rule::decide` as a Rust oracle (the shared vectors are checked by both sides for shape and by `bun test` for behaviour); a Playwright spec that drives the bridge through Dioxus's real eval channel (needs a page that uses a bridge hook, and `preview/src/**` is lane 2E's; `js_listener`'s existing users are the end-to-end proof); desktop and mobile latency (modelled by the delay mode, not measured); `use_window_size`/`use_media_query` have no SSR-render test beyond the compile. Caveat for the G lanes: `MountedData` reads are synchronous on web, so code that must behave like desktop reads through `get` (or follows its reads with `commit`) for the delay mode to model it.
