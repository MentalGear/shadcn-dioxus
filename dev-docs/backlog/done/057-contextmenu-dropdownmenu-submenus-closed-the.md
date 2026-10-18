---
id: 57
title: "ContextMenu/DropdownMenu submenus closed the instant the pointer left the trigger toward the submenu's own…"
status: "Landed 2026-09-15"
size: "M"
area: build
updated: 2026-09-15
---

# Row 57: ContextMenu/DropdownMenu submenus closed the instant the pointer left the trigger toward the submenu's own…

## Item

**`ContextMenu`/`DropdownMenu` submenus closed the instant the pointer left the trigger toward the submenu's own content — LANDED 2026-09-15, user-reported.** The pre-existing hover oracle (`menu-submenu.spec.ts`) stayed green because its one hover test only ever moved the pointer from the trigger to a *sibling root item*, never into the submenu's own content — exactly the path the report described ("on sub-menu mouse hover, the sub-menu disappears / submenus not selectable"). **Root cause 1 (shared plumbing, both hosts):** `SUBMENU_CLOSE_GRACE_DELAY`'s `DelayedAction` timer was private local state inside `*SubTrigger` only (`menu_sub.rs`); the trigger's own `onmouseleave` scheduled a close on it, but **`*SubContentRendered` had no `onmouseenter`/`onmouseleave` handlers at all**, so nothing could ever cancel that timer once the pointer entered the submenu's own content — ~200ms later the stale close fired regardless of where the pointer actually was. Fixed by construction: hoisted `hover_open`/`hover_close: DelayedAction` onto the shared `SubMenuState` (read via context by both trigger and content instead of the trigger's own private locals), then wired `onmouseenter`/`onmouseleave` on all four `*SubContentRendered` functions (`ContextMenuSubContentRendered`/`DropdownMenuSubContentRendered`, web + native arms) to cancel/reschedule the same shared timer the trigger already does — this alone fully fixed `DropdownMenu.Sub`. **Root cause 2 (`ContextMenu`-specific, on top of root cause 1):** `ContextMenu.Sub` still failed after the above — `.dx-context-menu-content` (shared by `ContextMenuContent` and `ContextMenuSubContent`) sets `pointer-events: none` **unconditionally**, unlike `.dx-dropdown-menu-content`'s identical-looking reset, correctly scoped to `[data-state="closed"]` only. `ContextMenuContentRendered` (the root) already works around this with an inline `pointer_events: open().then_some("auto")` override, but `ContextMenuSubContentRendered` never had the equivalent — so an open submenu was non-interactive the entire time, explaining why root cause 1's new hover handlers couldn't even fire. Fixed by mirroring the same inline override onto both of `ContextMenuSubContentRendered`'s arms. `DropdownMenu` never needed this fix (confirmed via grep — no `pointer_events` usage in `dropdown_menu.rs` at all, since its CSS already scopes the reset correctly). **Verified by execution:** two new regression tests in `menu-submenu.spec.ts` (real `page.mouse.move(..., {steps})` sequences, not `.hover()`, from trigger onto the submenu's own item, past the grace delay) — full oracle **14/14 passed** (5 pre-existing unchanged + 2 new, both hosts); `context-menu.spec.ts` + `dropdown-menu.spec.ts` + `keyboard-matrix.spec.ts` combined, independently re-run: **82 passed, 1 pre-existing skip, 0 failed**; `cargo check` clean for both default and `--features web` on `primitives`, `cargo fmt --check` clean, all three composition/class-prefix/css-literal lint scripts green

## Status / trigger

Landed 2026-09-15

## Size

M

## Detail

Found via user report; delegated to a Sonnet subagent, independently re-verified (including a full oracle + broad regression re-run, not just the two new tests) before landing
