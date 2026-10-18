---
id: 113
title: "calendar.spec.ts:364 \"a range's in-between day keeps its muted look while hovered\" times out (90s) on every…"
status: "LANDED 2026-10-05 (round 3): RESOLVED, and not date-dependent -- Found 2026-09-26 by the audit…"
size: "S (diagnosis)"
area: tests
updated: 2026-10-05
---

# Row 113: calendar.spec.ts:364 "a range's in-between day keeps its muted look while hovered" times out (90s) on every…

## Item

`calendar.spec.ts:364` "a range's in-between day keeps its muted look while hovered" times out (90s) on every run -- likely date-dependent

## Status / trigger

**LANDED 2026-10-05 (round 3): RESOLVED, and not date-dependent** -- Found 2026-09-26 by the audit batch's full-suite run: 3/3 identical timeouts on the branch AND on unmodified main (`78cff69`); the locator `.dx-calendar-grid-cell[...]:not([data-disabled])` `.nth(4)` never resolves

## Size

S (diagnosis)

## Detail

Not root-caused. Suspect the test builds a range relative to today's date, so on some calendar days the in-between cells it expects don't exist or are disabled. Pin the calendar's reference date in the demo or the test, rather than retrying. Full-suite context at the same run: 1307 passed / 5 failed / 12 skipped; the other four failures are the by-design `selftest:unfixed:id` and three CSS Anchor Positioning geometry tests this sandbox's Chromium cannot pass (row 106's class), all identical on unmodified main. **Landed 2026-10-05 (round 3):** the guess above did not hold. With the clock pinned to the day the row was filed and to three other days (2026-05-15, 2026-01-31, 2026-02-28) the locator resolved to all 31 May cells every time, because the `range` demo fixes its own `view_date` to 2026-05-15 and its min/max leave May 2026 available. What reproduces the 90 s timeout deterministically is the URL: the spec opened `/component/block/calendar/range/` (the path form), which only the dev server's SPA fallback serves; the static SSG server has no prerendered file for it and answers 404 (curl: 404 against 200 for the query form), so the calendar never rendered and the first `.click()` waited out the whole test timeout. **Fix, in `playwright/calendar.spec.ts`:** the query form `/component/block/?name=calendar&variant=range&` (both lanes serve it; every other block-page spec uses it), `page.clock.setFixedTime` to a day outside the demo's May 2026 view so `data-today` can never land on one of the cells, and a `toHaveCount(31)` precondition that fails in seconds with a message instead of burning the timeout. Lesson for row 164: a spec URL that 404s on the SSG build looks like a hang, so assert the page's own precondition first, and block demos have no prerendered path form.
