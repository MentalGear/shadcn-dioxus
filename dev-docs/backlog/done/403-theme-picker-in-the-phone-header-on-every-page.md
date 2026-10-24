---
id: 403
title: "The theme picker's palette icon is in the phone header on every page"
status: "LANDED (pending merge) -- owner iOS feedback 2026-10-24: \"The palette icon for customization should…"
size: "S"
area: docs
updated: 2026-10-24
---

# Row 403: the theme picker's palette icon is in the phone header on every page

## Item

On a phone the palette icon that opens the theme customizer is missing from the header on every page with the
docs sidebar (home, docs, component pages, `/demos`, `/charts/`); it is only reachable from a "Theme" entry at
the foot of the navigation sheet.

## Status / trigger

**LANDED (pending merge)** -- owner iOS feedback 2026-10-24: "The palette icon for customization should also be
displayed on mobile".

## Size

S

## Detail

Row 142 put the phone entry in the header only on routes without a `SidebarCtx`; routes with the docs sidebar kept
`theme::ThemePickerSidebarFooter` in the navigation sheet, so their header showed no palette icon at <= 760px.
**Fix:** `theme::ThemePicker` renders `ThemePickerPhoneTrigger` on every route, next to the popover's trigger, and
`ThemePickerSidebarFooter` is gone. The header has one "Theme" button at every width in the same slot (the
popover's above 760px, the drawer's at and below it), so the "one trigger visible, one panel mounted" rule holds
without asking which layout a route uses. The drawer is no longer a modal stacked on the navigation sheet.
**Specs:** `playwright/theme-picker-phone.spec.ts` sweeps every route type (sheet and no-sheet) for the header
entry, asserts the sheet carries none, and the header-fit sweep covers every route at 390, 360 and 320 px;
`playwright/theme-preset.spec.ts` drives the phone drawer from the header entry.
