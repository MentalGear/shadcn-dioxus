---
id: 54
title: "Nested submenus (DropdownMenu.Sub/ContextMenu.Sub) — LANDED 2026-09-04, verified by execution. Closes the…"
status: "Landed 2026-09-04"
size: "L"
area: tests
updated: 2026-09-04
---

# Row 54: Nested submenus (DropdownMenu.Sub/ContextMenu.Sub) — LANDED 2026-09-04, verified by execution. Closes the…

## Item

**Nested submenus (`DropdownMenu.Sub`/`ContextMenu.Sub`) — LANDED 2026-09-04, verified by execution.** Closes the "Nested submenu" row in [`component-backlog.md`](../../component-backlog.md) (rated **L**, called "the highest-value primitive-needing gap with no clean source") — see that file's own row for the catalog-status update. Built on the landed top-layer/popover engine (Phase 4.4, row 2) and reading its roles from `menu_semantics` (row 24/52), so it inherits both foundations rather than re-deriving them. New shared module `primitives/src/menu_sub.rs` factors the plumbing genuinely identical between `DropdownMenu`'s and `ContextMenu`'s submenus — open/close state, the submenu's own roving-focus item collection, the trigger/content id pair, and the hover-intent open/close timer (`DelayedAction`, generalised from `context_menu.rs`'s pre-existing long-press timer, this crate's only prior art for a timed pointer-driven open/close) — while deliberately leaving each host's own private root context lookup and content-positioning call in `dropdown_menu.rs`/`context_menu.rs` themselves, since Rust's module privacy has no way to let a shared module see either host's private context type without widening its API surface for no reader benefit. Scoped to one level of nesting only, matching the module's own doc and this row's original **L** rating: a `Sub`'s own content may hold plain items but not a further nested `Sub`, since a sub-trigger registers in the *root* menu's collection by reading that root's context type directly, which only resolves correctly at exactly one level of depth. New APG oracle `playwright/oracle/tier1-apg/menu-submenu.spec.ts` (430 lines, cited to the same pinned APG "Menu and Menubar" pattern commit `keyboard-matrix.spec.ts` already uses) — passing. A `use_effect_cleanup` fix was needed in both `dropdown_menu.rs` and `context_menu.rs` to make submenus actually open at all: a submenu's own open-count bookkeeping effect had no cleanup path for the case of the enclosing menu (or an ancestor submenu) unmounting the submenu while it was still open, which runs no further effects, only cleanups — without it the shared open-count state could desync and submenus would fail to open. Keyboard-matrix stayed **64/64** with no regressions

## Status / trigger

Landed 2026-09-04

## Size

L

## Detail

References checked, per `lifting-from-forks.md` hygiene, and NOT taken: `molikto/dioxus-components@ed681439` (stale, shape-only, pre-dates the collection/focus refactors this repo now has)
