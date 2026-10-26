---
id: 434
title: "JavaScript lives in TypeScript modules, not in Rust raw strings"
status: "BACKLOG. Owner approved 2026-10-25 (row 365 sweep, P9)."
size: "L"
area: primitives
updated: 2026-10-25
---

# Row 434: JavaScript lives in TypeScript modules, not in Rust strings

## Item

About 46 blocks (about 2,420 lines) of JavaScript sit in Rust raw strings passed to `document::eval`, so the JS-quality gates regex Rust, and the modality writer and `PREPAINT_JS` exist as identical copies that gates compare. Construction: every non-trivial script is a function in `primitives/src/ts/<module>.ts`, typed, and bundled into the committed `primitives/src/js/<module>.js` by the existing bundle chain, then included with `include_str!`. Rust calls `window.__dx.fn(args)` through one eval wrapper. The copies become one file, and a raw-string JS block longer than N lines fails the gate.

## Status / trigger

BACKLOG. Owner approved 2026-10-25 (row 365 sweep, P9).

## Size

L

## Detail

Source: [final sweep note, P9](../research/final-sweep-2026-10-25.md). One lane per script cluster (carousel, virtual engine, top layer and overlay, the rest). The bundle chain exists and holds one file.

- **Measured:** `grep -rln "document::eval\|eval("` gives 45 files in `primitives/src`. Blocks include `virtual_grid.rs:1954` (`CELL_CONTROLS_JS`), `:2087` (`FOCUS_CELL_JS`), `virtual/find.rs:135,282`, `virtual/bridge.rs:293` (`SCROLL_BRIDGE_JS`), `virtual/scroll_to.rs:190`, `select/components/list.rs:102` (`ITEM_ALIGNED_JS`), `top_layer/scroll_clamp.rs:58,141,261`, `top_layer/styles.rs:643`, `collection.rs:17`, `modality.rs:70,87` (`:70` is a minified one-liner with `/*listener-ok: ...*/` inside), and `carousel/{autoplay,wheel,drag,scroll}.rs` (7,431 lines under `carousel/`, built with `concat!`).
- **The chain exists:** `primitives/src/ts/` holds one file (`focus-trap.ts`), with `scripts/build-js-bundle.sh`, `primitives/src/js/bundle.manifest` and `check-js-bundle.sh` (committed `.js`, sha256 manifest).
- **Cost shows up as exceptions and copies:** `listener-ok:` 18, `blocking-ok:` 4, `interval-ok:` 3 markers. The modality writer exists in three byte-identical copies (`modality.rs`, `PREPAINT_JS`, `preview/index.html`) that `check-open-focus.sh:34,115` compares; `PREPAINT_JS` exists in two that `check-theme-mode-writer.sh:132` compares.
- **Subsumes:** the 46 blocks over time; the three-copy and two-copy identity checks; the regex-over-Rust complications of four gates; the `listener-ok` markers that sit inside eval strings. **Does not subsume:** one-line `eval` calls that read one value; the carousel needs its own plan because it builds its script with `concat!`/macros.
- **Gate:** `check-js-in-rust.sh`: a raw string over N lines, or any `addEventListener` in a `.rs` file, fails unless listed. The JS gates scan `.ts`.
- **Risk:** timing-sensitive drift while moving (scroll bridge, find, wheel band). Each lane's proof is the specs of the components that use its block, on a release build. Typed access to Dioxus's eval channel (`dioxus.send`) needs a declaration file.
