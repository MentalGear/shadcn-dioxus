---
id: 41
title: "NavbarContent migrated onto the top-layer engine (2026-09-03, finding C) without also picking up two…"
status: "41a: landed 2026-09-04. 41b: CLOSED 2026-10-03 as a documented non-claim (no device needed; see the…"
size: "S"
area: tests
updated: 2026-10-03
---

# Row 41: NavbarContent migrated onto the top-layer engine (2026-09-03, finding C) without also picking up two…

## Item

`NavbarContent` migrated onto the top-layer engine (2026-09-03, finding C) without also picking up two adjacent, already-landed fixes its closest sibling `MenubarContent` has: an `aria-labelledby` from its own trigger (row 25's fix, APG's "menu must have an accessible name") and an entry in `top-layer.spec.ts` Rule 11(c)'s `CONFORMING_CASES` table (the "conforming at open, diverging later" iOS-keyboard sub-case). **41a — LANDED 2026-09-04, verified by execution (Chromium):** `NavbarContent` (both arms) now gets `aria-labelledby` from its trigger, via a new `NavbarNavContext::trigger_id` — covered by the same new `top-layer.spec.ts` Rule 14 as row 36's fix, which passed in the Chromium run. **41b — landed as a documented non-claim, not independently execution-confirmed:** a Navbar entry was added to Rule 11(c)'s `CONFORMING_CASES`, marked `supported: false`, on structural grounds (it opens by hover + reengage, the shape already execution-confirmed to close via a real `pointerleave` for Tooltip/HoverCard) — that entry is skipped at runtime, so 41b is still NOT independently execution-confirmed for Navbar

## Status / trigger

41a: landed 2026-09-04. **41b: CLOSED 2026-10-03 as a documented non-claim (no device needed; see the 2026-10-03 re-audit in the notes).** Was: the entry exists but is skipped at runtime — still needs either a real device or an engine-simulation change that lets Navbar's hover-driven case reach the critical shape before it can be called execution-confirmed

## Size

S

## Detail

**2026-09-26 (audit correction):** 41b's status made unambiguous — the Navbar `CONFORMING_CASES` entry is `supported: false` (confirmed in `playwright/oracle/tier2-html/top-layer.spec.ts`) and is skipped at runtime, so it is an asserted, documented non-claim, never independently execution-confirmed. Neither was in finding C's own scope (auto-flip when cut off) — noted here so they aren't mistaken for done just because the migration landed. `Navbar`'s own keyboard/role contract (`navbar.spec.ts`, `oracle/tier1-apg/keyboard-matrix.spec.ts`) was deliberately left untouched by the migration and stays green **2026-10-03 re-audit of 41b (main @ 2241da8) -- closed as a non-claim, by construction, not by device.** The thing 41b wanted confirmed is that Navbar's anchor tracking survives a mid-open viewport change. That tracking is not Navbar code: `NavbarContentRendered` calls the same `top_layer::use_anchor_position_fallback` (`navbar.rs:856`) every other anchored overlay shares (11 call sites -- popover, tooltip, hover_card, dropdown_menu x2, context_menu, menubar, navbar, navigation_menu, select, combobox), whose unconditional-tracking construction is execution-verified by `top-layer.spec.ts` Rule 11(c)'s `Popover` case (the sole `supported: true` entry, the mechanism-level `style.top !== ""` assertion). A Navbar-specific copy of that test would re-test the identical hook through a hover-opened surface that Chromium closes via a real `pointerleave` on the simulated resize (the same reason Tooltip/HoverCard are `supported: false`). And the shape cannot arise in practice: it needs the viewport to change *while the overlay stays open*, i.e. an on-screen keyboard from a focused text input, but Navbar has no text input and `NavbarTrigger`/`NavbarItem` close on blur (`navbar.rs:632`, `:1164`), so focusing any input closes it first. The `supported: false` Navbar entry stays as the documented non-claim; no further action owned by this row. 41a unchanged (landed). The one new finding from this pass is row 40's pointerdown-open Navbar exposure, which is a different defect from 41b
