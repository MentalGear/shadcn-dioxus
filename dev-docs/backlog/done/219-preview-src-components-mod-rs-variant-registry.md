---
id: 219
title: "preview/src/components/mod.rs variant registry is one shared macro list that every demo-adding lane edits"
status: "LANDED 2026-10-18 (lane registry): the registry is derived. preview/build.rs discover_components…"
size: "S-M"
area: process
updated: 2026-10-18
---

# Row 219: preview/src/components/mod.rs variant registry is one shared macro list that every demo-adding lane edits

## Item

`preview/src/components/mod.rs` variant registry is one shared macro list that every demo-adding lane edits

## Status / trigger

**LANDED 2026-10-18 (lane registry): the registry is derived.** `preview/build.rs` `discover_components` walks `src/components/<dir>/` (a `component.json`) and `variants/<name>/` directories and writes the list as the `with_registry!` macro in `OUT_DIR/examples.rs`; `components/mod.rs` just calls `with_registry!(examples)`. Adding a demo is one new `variants/<name>/mod.rs` and touches nothing else; the order after `main` is the component's own `variants/order.txt` (unlisted directories follow, sorted); `variants/demo.css` marks a Block. Editing `mod.rs` (a new component's CATALOG row) re-runs `build.rs`, which closes row 92's manual `touch`. The one order change: `navbar` now precedes `navigation_menu` (sorted by name; it was swapped before). A cfg'd-out block-row mirror stays in `mod.rs` for the three specs that scrape it (build-checked against the directories); delete it when those specs read `variants/`. Earlier status: **Scheduled 2026-10-15 in `dev-docs/research/next-round-plan-2026-10-15.md`**; was OPEN with a by-construction fix proposed

## Size

S-M

## Detail

Adjacent-line merge conflicts recurred on 4 of ~12 merges in batch 2 (chat/sizes, chat/p9parts, sizes/scrollfade twice). **Construction:** each component declares its own variants (a `variants.list` per component, or discovery of `variants/*/` directories in `preview/build.rs`), so no two lanes ever touch the same line; the existing `component-catalog` gate must keep verifying the result. Remember the `preview/build.rs` rewrite rule in CLAUDE.md for new directories. Subsumes: the recurring conflict instances; does not subsume conflicts in `component.json` or the docs pages.
