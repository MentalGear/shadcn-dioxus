---
id: 52
title: "Menu-family role consolidation, stage 1 — LANDED 2026-09-04. menu_semantics exists, in its own module doc's…"
status: "Landed 2026-09-04"
size: "S"
area: tests
updated: 2026-09-04
---

# Row 52: Menu-family role consolidation, stage 1 — LANDED 2026-09-04. menu_semantics exists, in its own module doc's…

## Item

**Menu-family role consolidation, stage 1 — LANDED 2026-09-04.** `menu_semantics` exists, in its own module doc's words, so that fixing `dropdown_menu`'s roles in place (row 24) would not "leave the *next* menu-pattern component free to hand-write a fourth, possibly-different set" — and Navbar was exactly that fourth set: `primitives/src/navbar.rs` hand-wrote `role="menu"` (×3) and `role="menuitem"` (×2) at five sites while `dropdown_menu.rs`, `context_menu.rs`, and `menubar.rs` all already read theirs from the shared module. All five now read from `menu_semantics::MENU_ROLE`/`MENU_ITEM_ROLE`, each with a comment naming the `menubar.rs` counterpart it mirrors; no new constant was needed, and Navbar never used `aria-haspopup` at all. Two literals stay inline on purpose, not by oversight: `role="navigation"` is a landmark, not part of this pattern class, and `role="menubar"` is the Menu-and-Menubar pattern's own container role, which the module's own Scope section explicitly disclaims — `menubar.rs` keeps its own inline `role="menubar"` for the identical reason. The module doc now lists Navbar among its consumers and names it in that carve-out, since listing only three consumers is how this gap went unnoticed in the first place. `playwright/oracle/tier1-apg/menu-roles.spec.ts` did **not** cover Navbar at all — the omission that let the drift happen unnoticed — and now does, opening by hover (Navbar's real gesture, not Menubar's click), with the same two-elements-both-`role="menu"` disambiguation the existing Menubar block already needed. Emitted markup is unchanged; only its source moved — menu-roles, navbar, and keyboard-matrix all stay green

## Status / trigger

Landed 2026-09-04

## Size

S

## Detail

Found while assessing whether Navbar and Menubar should share a submenu primitive (row 53) — the drift check that assessment required surfaced this gap directly. See row 53 for the larger stage-2 question this stage was scoped out of
