---
id: 434
title: "Cut JavaScript to what Rust cannot do: audit every eval block, move the rest into Rust, TypeScript only for the remainder"
status: "BACKLOG. Approved as P9 of the row 365 sweep; the owner re-scoped it to Rust first on 2026-10-25.…"
size: "L"
area: primitives
updated: 2026-10-25
---

# Row 434: Cut JavaScript to what Rust cannot do

## Item

The owner's principle (2026-10-25): the point of Dioxus is to drive the UI from Rust/wasm, so JavaScript is the exception, not the default. The library has no `web-sys` dependency; every browser API Dioxus does not wrap goes through `document::eval`, the one route that works on web, desktop and mobile. 72 files carry eval. The owner chose **Rust first** over "Rust plus web-sys" (one code path per feature, every renderer) and over the original TypeScript move.

1. **Audit** every eval block and classify it: (a) Dioxus already has it in Rust (`MountedData::get_client_rect`, `scroll_to`, `set_focus`, event data, signals, `onvisible`/`onresize` where they exist), so move it; (b) Dioxus has no Rust API (`showPopover`/`showModal`, `matchMedia`, pointer capture, listener options such as passive/capture, `scrollend`, `getAnimations`), so it stays JS; (c) must run before wasm (`PREPAINT_JS`, the modality writer), so it stays JS; (d) JS on purpose for latency (one-frame measure-and-place in the anchor engine, scroll bridge, virtual find), so it stays only with a measured reason.
2. **Move** every (a) block to Rust, per cluster, with the components' specs on a release build as the proof (timing-sensitive paths compare long tasks before and after).
3. **What stays** (b, c, d) moves into typed `primitives/src/ts/` modules through the existing bundle chain (the original P9 construction), and the copies become one file.

## Audit result (2026-10-25)

[`../research/js-audit-2026-10-25.md`](../research/js-audit-2026-10-25.md): 103 blocks, about 6,205 lines. (a) movable now 21 blocks / 624 lines (10%); (b) no Dioxus API 55 / 1,580 (25%); (c) before wasm 5 / 107 (2%); (d) JS on purpose 22 / 3,894 (63%). Upstream `main`/0.8.0-alpha.3 adds no element or window API, so no upgrade is waited for. Step 2: about 26 agent-hours in 5 lanes (2A styles and toast pause, 2B focus by handle, 2C scroll and measure, 2D virtual bridge experiment with a keep-JS exit, 2E demo sleeps). Step 3: about 66 agent-hours in 8 lanes over two batches (3.0 the typed channel and loading first).

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
