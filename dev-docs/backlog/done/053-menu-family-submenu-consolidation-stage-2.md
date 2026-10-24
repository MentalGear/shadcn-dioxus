---
id: 53
title: "Menu-family submenu consolidation, stage 2 — LANDED 2026-09-14, verified by execution. Following stage 1 (row…"
status: "Landed 2026-09-14"
size: "M (done)"
area: build
updated: 2026-09-14
---

# Row 53: Menu-family submenu consolidation, stage 2 — LANDED 2026-09-14, verified by execution. Following stage 1 (row…

## Item

**Menu-family submenu consolidation, stage 2 -- LANDED 2026-09-14, verified by execution.** After stage 1 (row 52), the question was
whether `DropdownMenu`/`ContextMenu`/`Menubar` should share more plumbing; the answer was to extract three narrow bands, not a
whole shared component, into `primitives/src/menu_root.rs` (the same "identical plumbing shared, host-private everything else"
shape as `menu_sub.rs`, row 54). Details under Detail.

## Status / trigger

Landed 2026-09-14

## Size

M (done)

## Detail

**The three bands in `primitives/src/menu_root.rs`** (registered by one `mod menu_root;` line in `lib.rs`):

1. `use_open_focus_sync`, the effect that resyncs a menu's `open` boolean from whether its roving-focus collection holds DOM
   focus (gated on no submenu being open). It was word-for-word identical in `DropdownMenu` and `ContextMenu`. Deliberately NOT
   used by `Menubar`: its analogous effect resyncs a which-index-is-open signal and has no submenu count, so forcing one generic
   function would invent a translation layer with no duplication to justify it.
2. `use_menu_content_lifecycle`, id resolution (`use_unique_id` plus `use_id_or`) and `use_animated_open`'s open/close-hold gate,
   identical across all three `*Content` components. The positioning half stays host-private: `top_layer.rs` hard-codes the
   `dx-anchor-*` marker classes, so the anchor call still lives in each file's `#[cfg(feature = "web")]` leaf.
3. `content_labelledby_attributes`, the APG accessible-name-from-trigger-id block (row 25), duplicated across six
   `*ContentRendered` arm pairs, now one function.

What stayed host-private: the keyboard state machines and the item components; `Navbar` was not made a consumer, and `menu_sub.rs`
was left untouched.

**The Escape-refocus divergence, root-caused.** `Menubar` refocuses its trigger on Escape and `Navbar` did not; the cause was a
genuine oversight (Navbar supports keyboard-driven opening exactly like Menubar), fixed in `Navbar`'s own `onkeydown`, not gated
to the web arm (read the open index before clearing it, then refocus that trigger through the collection). On this sandbox's
Chromium the new red-first test passed even with the fix reverted, because `NavbarContent` renders via `popover="auto"` and the
Popover API's focus fixup already restores focus to the element focused before opening. The fix is still the only thing providing
the contract on the native (Blitz) arm, which has no Popover API; that arm could not be verified here, and the test is a
regression guard, not a red-to-green proof.

**Verification (Chromium, 2026-09-14).** Existing specs for all three hosts stayed green through the refactor (`dropdown-menu`,
`context-menu`, `menubar`, `keyboard-matrix`); new coverage: `aria-labelledby` assertions for ContextMenu and Menubar in
`menu-roles.spec.ts` and a `NavbarNav -- Escape` case in `keyboard-matrix.spec.ts`. 97 tests: 96 passed, 1 skipped (a pre-existing
Accordion skip), 0 failed; `cargo check` (native and `web`) and `cargo fmt` clean; clippy showed only 5 pre-existing
`doc_lazy_continuation` errors in the untouched `menu_sub.rs`. Firefox and WebKit were not attempted (row 4).

*Trimmed in row 266 (repo cleanup). The full dated narrative of this row is in git: `git show 3c584cc2:dev-docs/backlog/done/053-menu-family-submenu-consolidation-stage-2.md`.*
