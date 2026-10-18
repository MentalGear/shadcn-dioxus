---
id: 142
title: "The phone theme picker is missing on /demos and /charts (no docs sidebar there)"
status: "LANDED 2026-10-05 (round 3) -- recorded 2026-10-04 with the picker landing (row 111, round-2…"
size: "S"
area: docs
updated: 2026-10-05
---

# Row 142: The phone theme picker is missing on /demos and /charts (no docs sidebar there)

## Item

The phone theme picker is missing on `/demos` and `/charts` (no docs sidebar there)

## Status / trigger

**LANDED 2026-10-05 (round 3)** -- recorded 2026-10-04 with the picker landing (row 111, round-2 addendum)

## Size

S

## Detail

The header's popover trigger is hidden at <= 760px (`main.css`) and the phone entry, `theme::ThemePickerSidebarFooter`, is rendered inside the docs `Sidebar` (`preview/src/main.rs`, under the scrolling nav), so a route that does not render that sidebar has no picker at phone width. Reported for `/demos` and `/charts`; confirm on a 390px viewport and list every route that renders without the docs sidebar before fixing. **Fix by construction:** render the phone trigger from the shared site chrome (the header) instead of a per-layout sidebar, so a route cannot lose it by choosing another layout, while keeping exactly one trigger visible per width (the "exactly one panel mounted" rule in `theme::ThemePicker`'s doc). Spec: a phone-viewport sweep asserting the trigger is visible on every route type. **Landed 2026-10-05 (round 3):** a phone trigger rendered from the shared site chrome, as proposed: the header's own `theme::ThemePicker` renders `ThemePickerPhoneTrigger` when, and only when, the route has no `SidebarCtx` (`/demos`, `/charts` and its tabs, the not-found pages), and it opens the SAME single `ThemePickerDrawer` (one panel mounted, one trigger visible per width). The header overlap at 390 and 320 px is fixed. **Spec:** `playwright/theme-picker-phone.spec.ts`, a sweep over every route type asserting exactly one Theme entry is reachable at phone width, that it opens the same drawer, that the header still fits, and that desktop is unchanged.
