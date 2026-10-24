---
id: 85
title: "DropdownMenu/ContextMenu — a raw DOM .focus() on a sub-trigger closes the whole menu"
status: "Landed 2026-09-20 (round 4, 3ba793b) — fixed by construction across DropdownMenu, Menubar, and both…"
size: "S to verify"
area: menus
updated: 2026-09-20
---

# Row 85: DropdownMenu/ContextMenu — a raw DOM .focus() on a sub-trigger closes the whole menu

## Item

DropdownMenu/ContextMenu — a raw DOM `.focus()` on a sub-trigger closes the whole menu

## Status / trigger

**Landed 2026-09-20 (round 4, `3ba793b`)** — fixed by construction across DropdownMenu, Menubar, and both Sub levels

## Size

S to verify

## Detail

Found 2026-09-20 by the RTL oracle's submenu coverage (not an RTL defect; it reproduced on the untouched LTR variant): calling
`.focus()` directly on a submenu's own sub-trigger (as a test or an assistive-technology focus-restoration path would) closed the
whole menu and returned focus to the root trigger. Landed 2026-09-20 (round 4, `3ba793b`), root-caused as a class.

**Root cause.** Every affected site shared one shape: a synchronous `onblur` deciding "did focus leave" from a Rust-tracked
roving-focus signal (`ctx.focus.any_focused()`, `menu_ctx.focus.any_focused()`, `sub.focus.any_focused()`) that only this crate's
click and keyboard paths ever write (state first, DOM focus second) and that no item in this menu-button pattern reconciles through
its own `onfocus`. A focus move arriving any other way read stale "nothing focused" state and closed the menu. `DropdownMenuTrigger`'s
root, `MenubarTrigger`/`MenubarItem` and the shared submenu level (`DropdownMenuSubTrigger`/`ContextMenuSubTrigger`,
`menu_sub.rs`'s `SubMenuState`) all carried it (Menubar's was the most directly reproducible). **`ContextMenu`'s own root, the row's
original named instance, could not be reproduced red:** it already dismissed through `use_outside_dismiss` (DOM-containment-based,
immune to the same-tick blur-before-focus hazard), so the original claim did not hold up; stated plainly.

**Fix, by construction.** `DropdownMenu`'s root and `Menubar`'s per-menu wrapper got the identical `use_outside_dismiss` construction
`ContextMenu`'s root already had (a new `root_id` on each), plus a two-id variant `menu_sub::use_sub_outside_dismiss` (a Sub's trigger
and content are two siblings) for both `.Sub` levels; every displaced `onblur` branch was removed rather than left as a second
fragile path. The new ContextMenu tests are documented in-file as construction parity. `navigation_menu.rs` shares the constraint
(the `dioxus-html` 0.7.9 missing-`related_target` gap) but already answers it with a debounced close that every element's `onfocus`
cancels; left untouched. Evidence: RED confirmed for `DropdownMenu` (plain item and sub-trigger) and `Menubar`; `dropdown-menu.spec.ts`,
`context-menu.spec.ts` 13/13, `menubar.spec.ts`, `menu-submenu.spec.ts` 16/16, `keyboard-matrix`, `menu-roles` and
`focus-restore-reference` green (one pre-existing skip); `cargo test --workspace` 433/0.

**The landing itself introduced a regression, found and fixed the same round.** `3ba793b` gave `DropdownMenu`'s root `div` and
`MenubarMenu`'s wrapper a literal `id: root_id.cloned()` (and on Menubar a literal `role:`) beside the unmerged
`..props.attributes` spread, with a comment claiming a caller's `id` "still wins (spread order)", which is false: SSR emits both, a
duplicate-attribute parse error. It reached the prerendered `docs/component/top_layer/index.html` and was caught by
`hydration-parity.spec.ts` "Rule 4 (extended)" on a regenerated site. Fixed by `f2be1d7`: both elements route their literal
attributes through `merge_attributes`, as `context_menu.rs`'s root does (`b35d671`); two SSR tests (one per component) assert a
caller's `id` renders exactly once (RED before with 2 occurrences, GREEN after), and `hydration-parity.spec.ts` is 11/11 on the SSG
build. **Lesson recorded:** the CSR-only suites above could never have caught this class (the browser DOM coalesces duplicate
`setAttribute` calls, so the second attribute is visible only in SSR/SSG's raw text); the hydration-parity SSG lane is the only thing
in the harness that can. This is the instance row 93 generalised into `check-attr-spread-collision.sh`.

*Trimmed in row 266 (repo cleanup). The full dated narrative of this row is in git: `git show 3c584cc2:dev-docs/backlog/done/085-dropdownmenu-contextmenu-a-raw-dom-focus-on-a.md`.*
