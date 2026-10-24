---
id: 380
title: "Calendars read the visitor's clock after mount, so a prerendered page never keeps the build day"
status: "LANDED in batch 14. primitives::calendar::use_today renders the server's date during hydration…"
size: "M."
area: primitives
updated: 2026-10-24
---

# Row 380: no render-time clock reads

## Item

The SSG prerenders every page on the build day. Calendar, RangeCalendar, DatePickerCalendar and the preview Calendar defaulted `today` and `view_date` to `OffsetDateTime::now_local_date()` while rendering, so the served markup carried the build day, hydration kept it, and a visitor days later saw the build day marked as today (and the week-numbers demo the build's year). Row 377 fixed two date-picker demos one by one; this row fixes the class.

## Status / trigger

LANDED in batch 14. `primitives::calendar::use_today` renders the server's date during hydration (`use_server_cached`) and reads the visitor's clock in an effect after mount. The `today` prop is now `Option<Date>` and `view_date` is `ReadSignal<Option<Date>>`; `None` (the default) means the visitor's clock. That is a public API change. The segment defaults and the week-numbers and presets demos use `use_today`; `LocalDateExt` left `lib.rs`. `scripts/check-render-clock.sh` keeps clock reads out of render paths. `calendar.spec.ts` pins `page.clock` on the prerendered page and is red on a build without the change. Space is not reserved: layout moves only when the visitor's month or year differs from the build's (the week-numbers row count at a year boundary); the today mark is style only.

## Size

M.

## Detail

Verification in the lane: fast gates, the three clippy arms, primitives and preview-server tests, `calendar.spec.ts` 23 passed, `test:changed` 1215 passed with reds that are also red on main.
