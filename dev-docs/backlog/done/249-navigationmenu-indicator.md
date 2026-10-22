---
id: 249
title: "NavigationMenuIndicator (the leftover of closed row 72)"
status: "BACKLOG (2026-10-19)"
size: "S"
area: components
updated: 2026-10-22
---

# Row 249: NavigationMenuIndicator (the leftover of closed row 72)

## Item

NavigationMenuIndicator (the leftover of closed row 72)

## Status / trigger

**BACKLOG (2026-10-19)**

## Size

S

## Detail

Row 72 was closed as done except the NavigationMenu indicator part (Radix NavigationMenu.Indicator, the arrow under the active trigger). Check whether shadcn's current base navigation-menu still ships an Indicator before building it.

## Resolution (lane disclosure, 2026-10-22)
shadcn still ships `NavigationMenuIndicator` (base and radix bases: `top-full z-1 flex h-1.5 items-end justify-center overflow-hidden` with a `bg-border rounded-tl-sm shadow-md` 8px arrow rotated 45deg), so it is built: `NavigationMenuIndicator` primitive (measures the open item against the list, writes `--dx-nav-indicator-x`/`-width`, `data-state` visible/hidden), theme part in the sheet, an `indicator` demo and a docs section. `playwright/navigation-menu-indicator.spec.ts` checks the arrow centred under the open trigger at rest, following a switch, hidden when closed. Known limit: no re-measure on window resize while a panel stays open.
