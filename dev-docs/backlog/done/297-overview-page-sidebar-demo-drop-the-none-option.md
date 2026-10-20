---
id: 297
title: "Overview page sidebar demo: drop the \"None\" option"
status: "LANDED 2026-10-20. Overview-page Sidebar card drops the \"None\" button (c49ebdca); the component and…"
size: "S"
area: preview
updated: 2026-10-20
---

# Row 297: Overview page sidebar demo: drop the "None" option

## Item

Remove the "None" collapsible choice from the sidebar card on the docs overview.

## Status / trigger

LANDED 2026-10-20. Overview-page Sidebar card drops the "None" button (c49ebdca); the component and the three variant demos keep it (owner default). Spec: home-overview.spec.ts (94a78461).

Previous status: OPEN (owner review 2026-10-19).

## Size

S

## Detail

- The None button is in `SidebarGalleryCardControls` (`preview/src/main.rs:3738-3744`). The component keeps `SidebarCollapsible::None`.
- The three sidebar variant demos on the component page also have it (`sidebar/variants/{main,inset,floating}/mod.rs:515-518`).
- Owner default (2026-10-19): overview card only.
- Check specs that click "None".
