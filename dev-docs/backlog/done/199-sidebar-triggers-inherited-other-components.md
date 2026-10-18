---
id: 199
title: "Sidebar triggers inherited other components' default trigger recipes"
status: "LANDED 2026-10-14"
size: "S (done)"
area: menus
updated: 2026-10-14
---

# Row 199: Sidebar triggers inherited other components' default trigger recipes

## Item

Sidebar triggers inherited other components' default trigger recipes

## Status / trigger

**LANDED 2026-10-14**

## Size

S (done)

## Detail

`SidebarMenuButton`/`SidebarMenuAction` rendered as `DropdownMenuTrigger`/`CollapsibleTrigger` took every property the sidebar does not declare from those recipes (weight 500, `justify-content`, a 32px height defeating the action's `aspect-ratio`, an inset ring). Construction: the sidebar buttons always render `data-style="sidebar"`, the recipes' existing opt-out (`:where(.dx-...-trigger:not([data-style]))`, Collapsible's now in that shape too). A static gate could catch it by scanning `variants/**` compositions for a composed `as` element carrying `dx-<other>-trigger` without `data-style` -- not built.
