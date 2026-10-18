---
id: 25
title: "ContextMenu's and Menubar's submenu role=\"menu\" popups carry no aria-labelledby/aria-label at all (grepped…"
status: "Landed 2026-09-03 (row 34's own round) — both now carry aria-labelledby from their trigger's id…"
size: "S"
area: menus
updated: 2026-09-03
---

# Row 25: ContextMenu's and Menubar's submenu role="menu" popups carry no aria-labelledby/aria-label at all (grepped…

## Item

`ContextMenu`'s and `Menubar`'s submenu `role="menu"` popups carry no `aria-labelledby`/`aria-label` at all (grepped `context_menu.rs`/`menubar.rs`: zero matches) — an APG menu is required to have an accessible name from one of the two (`menu-and-menubar-pattern.html`, "WAI-ARIA Roles, States, and Properties"); `DropdownMenu`'s popup does carry one

## Status / trigger

**Landed 2026-09-03** (row 34's own round) — both now carry `aria-labelledby` from their trigger's id, mirroring `DropdownMenuContent`'s existing pattern (`context_menu.rs`; `menubar.rs` needed a new `trigger_id` field threaded through `MenubarMenuContext`, since nothing labelled the trigger for reference before). Correction to this row's own original note: verified by execution (temporarily reverting the fix and re-scanning an open menu) that axe-core carries **no rule** that checks a bare `role="menu"` container for an accessible name at all — the earlier expectation that opening the popup would make `preview.spec.ts`'s axe scans catch this was mistaken; the fix landed anyway as a cited APG conformance gap, by construction, not because a scan went red for it

## Size

S

## Detail

Found 2026-09-02 building `oracle/tier1-apg/menu-roles.spec.ts` for row 24. The axe-detectable analogue of this same defect class, found while building row 34: `SelectList`'s `role="listbox"` had the identical gap and *is* caught by axe's `aria-input-field-name` rule — fixed the same way, conditional on the caller not already supplying their own name (`select/components/list.rs`). **Re-verified landed, 2026-09-19 (batch 3, leftovers lane):** confirmed by grepping `aria_labelledby`/`aria-labelledby` usage across all 53 top-level specs (49/53 scanned; the 4 exceptions are this harness's own documented exclusions); no regression
