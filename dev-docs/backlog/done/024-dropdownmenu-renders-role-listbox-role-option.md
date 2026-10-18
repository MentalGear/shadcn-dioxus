---
id: 24
title: "DropdownMenu renders role=\"listbox\" / role=\"option\" (upstream's original markup) where the APG menu-button…"
status: "Landed — dropdown_menu.rs now takes its aria-haspopup/role literals from a shared menu_semantics…"
size: "S"
area: tests
updated: 2026-09-19
---

# Row 24: DropdownMenu renders role="listbox" / role="option" (upstream's original markup) where the APG menu-button…

## Item

DropdownMenu renders `role="listbox"` / `role="option"` (upstream's original markup) where the APG menu-button pattern it is keyboard-tested against (`oracle/tier1-apg/keyboard-matrix.spec.ts`) expects `role="menu"` / `role="menuitem"` — Menubar and ContextMenu already use the menu roles

## Status / trigger

**Landed** — `dropdown_menu.rs` now takes its `aria-haspopup`/`role` literals from a shared `menu_semantics` module also adopted by `context_menu.rs`/`menubar.rs`, so the three implementations of this pattern class cannot drift again; oracle `oracle/tier1-apg/menu-roles.spec.ts` (red on DropdownMenu, green on the vendored reference/ContextMenu/Menubar before the fix; all green after)

## Size

S

## Detail

Found 2026-09-02 verifying the keyboard fixes on the deployed site: focus correctly lands on the first item, but AT announces it as a listbox option. Same class as row 8 (Select trigger role) — role/pattern drift, not behaviour. **Re-verified landed, 2026-09-19 (batch 3, leftovers lane):** confirmed by grepping `menu_semantics` usage across all 53 top-level specs (49/53 scanned; the 4 exceptions are this harness's own documented exclusions in `conformance-harness.md`); no regression
