---
id: 499
title: "A safe triangle for the hover menus, so they can take Base UI's short close delays"
status: "BACKLOG. Split out of row 478 item 4, whose open delays now follow Base UI. Land before row 456…"
size: "M"
area: overlays
updated: 2026-11-06
---

# Row 499: A safe triangle for the hover menus

## Item

Base UI closes a hover-opened NavigationMenu panel 50 ms after the pointer leaves and a submenu 0 ms after, because a `safePolygon` keeps it open while the pointer travels from the trigger toward the panel (`@base-ui/react` 1.9.0: `navigation-menu/root/NavigationMenuRoot.mjs:50` `closeDelay = 50`, `NavigationMenuTrigger.mjs:405`; `menu/submenu-trigger/MenuSubmenuTrigger.mjs:41` `closeDelay = 0`, `:154` `safePolygon({ blockPointerEvents: true })`). This crate has no such rule, so its close graces stay at 300 ms (`HOVER_CLOSE_GRACE_DELAY`, `hover_intent.rs`) and 200 ms (`SUBMENU_CLOSE_GRACE_DELAY`, `menu_sub.rs`) as a stand-in: a diagonal move across the gap would otherwise close the panel.

## Status / trigger

BACKLOG. Split out of row [478](./478-overlay-defaults-to-base-ui.md) item 4, whose open delays now follow Base UI. Land before row [456](./456-port-3-ov-overlay-core-escape-dismiss-open-focus.md), which ports the overlay core.

## Size

M

## Detail

Construction: a Rust rule in the hover-intent code. On the pointer leaving a trigger, read the panel's rect (Dioxus `MountedData::get_client_rect`, no new JS), and while the pointer's moves stay inside the triangle between the leave point and the panel's near edge keep the panel open; a move outside it, or a stall, starts the close. With it, `HOVER_CLOSE_GRACE_DELAY` becomes 50 ms and `SUBMENU_CLOSE_GRACE_DELAY` 0 ms, as Base UI's. Applies to NavigationMenu, Navbar (shares `HoverIntent`) and the three submenu hosts.

Proof when done: specs on the page clock (`playwright/app-clock.ts`) for a diagonal move that reaches the panel (stays open) and one that leaves the triangle (closes at the configured delay).
