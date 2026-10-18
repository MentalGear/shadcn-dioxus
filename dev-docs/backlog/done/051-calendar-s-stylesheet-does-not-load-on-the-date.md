---
id: 51
title: "Calendar's stylesheet does not load on the date_picker route — LANDED 2026-09-04. Found incidentally during…"
status: "Landed 2026-09-04"
size: "S"
area: tests
updated: 2026-09-04
---

# Row 51: Calendar's stylesheet does not load on the date_picker route — LANDED 2026-09-04. Found incidentally during…

## Item

**`Calendar`'s stylesheet does not load on the `date_picker` route — LANDED 2026-09-04.** Found incidentally during row 50's ink investigation: `document.styleSheets` on `/component/?name=date_picker&` contained no `calendar`-named entry at all, so `DatePicker`'s calendar day-grid cells (`.dx-calendar-grid-cell`, real `<button>`s) had no author `color` — or any other `Calendar` styling — to fall back to on that route, only their own UA default. Pre-existing and unrelated to row 50's fix (which no longer causes black text there regardless, since the extended rule reaches top-layer-descendant form controls independent of whether their own stylesheet loaded). **Root cause:** `date_picker` composes `CalendarRoot`/`CalendarMonthView` — the calendar primitive — rather than the themed `Calendar` wrapper, so after row 32 moved CSS delivery into each themed wrapper's own `document::Link`, `date_picker`'s composition path never rendered `Calendar`'s wrapper and never rendered its link either. **The audit matters more than the fix.** Checked as a class, not patched as a one-off bug: two shapes were considered — a raw `dioxus_primitives::` component composed outside its owning wrapper (already guarded by `check-preview-composition.sh`, zero violations across the codebase) and a themed component composing another's `pub(crate)` internals directly, which that script exempts wholesale rather than checking. Six components do the latter; of them, `date_picker`/`calendar` was the **only** one missing CSS. **Fixed** by moving the `document::Link` onto `CalendarRoot`/`RangeCalendarRoot` themselves — the `pub(crate)` roots every consumer (including `Calendar`/`RangeCalendar` themselves) funnels through — mirroring `popover`'s existing pattern of a `Link` on every one of its own composable entry points, not just its "main" one; `document::Link`'s `(href, rel)` dedup makes the now-redundant `Link` that used to sit only on `Calendar`/`RangeCalendar` harmless to leave, so it was simply removed. **The guard is generic, not a calendar special case:** `playwright/dx-class-migration.spec.ts` now asserts, per route, that every `dx-<component>` prefix actually rendered in that route's own demo content has its owning component's stylesheet reachable from the page — no per-component "what composes what" list needed, and it would have caught this pair red before the fix

## Status / trigger

Landed 2026-09-04

## Size

S

## Detail

Found 2026-09-04 building row 50's oracle; landed the same date rather than folded into row 50, since it is a separate, unrelated defect
