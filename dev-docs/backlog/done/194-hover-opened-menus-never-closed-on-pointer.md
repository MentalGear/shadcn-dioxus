---
id: 194
title: "Hover-opened menus never closed on pointer leave (Navbar) -- one shared hover-intent helper"
status: "LANDED 2026-10-14"
size: "S (done)"
area: theme
updated: 2026-10-14
---

# Row 194: Hover-opened menus never closed on pointer leave (Navbar) -- one shared hover-intent helper

## Item

Hover-opened menus never closed on pointer leave (Navbar) -- one shared hover-intent helper

## Status / trigger

**LANDED 2026-10-14**

## Size

S (done)

## Detail

Owner report. `NavbarNav` opened on hover via `set_open_nav` but only closed through `Navbar`'s effect on roving focus, which a hover-open never set, so leaving changed nothing. Construction: `primitives/src/hover_intent.rs` (`HoverIntent`, `use_hover_intent`) owns the open index's single write path, the hover-opened marker, the open-intent (150 ms) and close-grace (300 ms) timers (`menu_sub::DelayedAction`, no raw listeners); Navbar and NavigationMenu both use it. Matches Radix NavigationMenu: leave of trigger or content closes any open (hover, click or keyboard), re-enter cancels, a sibling switch is immediate. `playwright/hover-intent.spec.ts` (4 cases x 2 components) red 3/4 on the old Navbar, 8/8 after. Menubar (switch-only, never closes on leave), HoverCard, Tooltip and submenus already correct. The snapshot harness now records a hover-intent open as such instead of clicking it shut (`computed-style-states.ts`, `HOVER_INTENT_WAIT_MS`).
