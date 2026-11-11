---
id: 499
title: "A safe triangle for the hover menus, so they can take Base UI's short close delays"
status: "BACKLOG. Split out of row 478 item 4, whose open delays now follow Base UI. Land before row 456…"
size: "M"
area: overlays
updated: 2026-11-07
---

# Row 499: A safe triangle for the hover menus

## Item

Base UI closes a hover-opened NavigationMenu panel 50 ms after the pointer leaves and a submenu 0 ms after, because a `safePolygon` keeps it open while the pointer travels from the trigger toward the panel (`@base-ui/react` 1.9.0: `navigation-menu/root/NavigationMenuRoot.mjs:50` `closeDelay = 50`, `NavigationMenuTrigger.mjs:405`; `menu/submenu-trigger/MenuSubmenuTrigger.mjs:41` `closeDelay = 0`, `:154` `safePolygon({ blockPointerEvents: true })`). This crate has no such rule, so its close graces stay at 300 ms (`HOVER_CLOSE_GRACE_DELAY`, `hover_intent.rs`) and 200 ms (`SUBMENU_CLOSE_GRACE_DELAY`, `menu_sub.rs`) as a stand-in: a diagonal move across the gap would otherwise close the panel.

## Status / trigger

BACKLOG. Split out of row [478](./478-overlay-defaults-to-base-ui.md) item 4, whose open delays now follow Base UI. Land before row [456](./456-port-3-ov-overlay-core-escape-dismiss-open-focus.md), which ports the overlay core.

**What exists (2026-11-07, `lane/safe-triangle-499`, not yet merged).** `primitives/src/safe_triangle.rs` is the one rule: pure geometry (`Point`, `Rect`, `Triangle::toward(origin, panel)`, `contains`, unit-tested, 2 px slack) and `SafeClose`, the thin wiring. On a pointer leaving a trigger, `SafeClose::leave` reads the panel's rect (`MountedData::get_client_rect`, a `PanelHandle` the host puts on its panel's `onmounted`) and builds the triangle from the leave point to the panel's near edge. While a close is pending it listens to the document's `mousemove` through the bridge (`use_event_listener` with `EventKind::Mousemove`, at most one message per frame, installed only while a close is pending; no new JS). A move inside the triangle keeps the panel open and restarts a 100 ms stall timer (`SAFE_TRIANGLE_STALL`); a move outside it, or no move for the stall, starts the close after the host's delay; the host's existing enter/hold/cancel calls drop it. Without a mounted panel, an unreadable rect or a leave from inside the panel, it is the plain delay.
- **Users, one rule:** `HoverIntent` (`hover_intent.rs`: NavigationMenu and Navbar, trigger/nav leave passes the panel; leaving the panel itself is the plain delay) and `SubMenuState::hover_leave_trigger` (`menu_sub.rs`: the submenus of DropdownMenu and ContextMenu; Menubar has no submenus). `HOVER_CLOSE_GRACE_DELAY` is 50 ms and `SUBMENU_CLOSE_GRACE_DELAY` 0 ms.
- **Not covered:** leaving a *panel* back toward its trigger crosses the gap on the plain delay (Base UI's `safePolygon` is also only on the reference). Base UI's `blockPointerEvents` (other triggers ignore the pointer inside the triangle) is not done: a diagonal over a neighbouring NavigationMenu trigger still switches panels, as before.
- **Proof:** `playwright/safe-triangle.spec.ts` on the page clock: NavigationMenu and DropdownMenu submenu, (a) a slow crossing that outlasts the old graces stays open, (b) a move out of the triangle closes at the configured delay, (c) a stall in the gap closes it. On the base build 4 of 5 fail (the diagonal passes there: the old 300 ms grace covers it); on the change all 5 pass, 6 of 6 repeated runs.

## Size

M

## Detail

Construction: a Rust rule in the hover-intent code. On the pointer leaving a trigger, read the panel's rect (Dioxus `MountedData::get_client_rect`, no new JS), and while the pointer's moves stay inside the triangle between the leave point and the panel's near edge keep the panel open; a move outside it, or a stall, starts the close. With it, `HOVER_CLOSE_GRACE_DELAY` becomes 50 ms and `SUBMENU_CLOSE_GRACE_DELAY` 0 ms, as Base UI's. Applies to NavigationMenu, Navbar (shares `HoverIntent`) and the three submenu hosts.

Proof when done: specs on the page clock (`playwright/app-clock.ts`) for a diagonal move that reaches the panel (stays open) and one that leaves the triangle (closes at the configured delay).
