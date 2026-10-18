---
id: 238
title: "Playwright fixtures force scroll-behavior: auto !important on every context"
status: "LANDED 2026-10-18 (lane specs 7a066be9 (override deleted)), merged on the session branch; deploy…"
size: "S"
area: tests
updated: 2026-10-18
---

# Row 238: Playwright fixtures force scroll-behavior: auto !important on every context

## Item

Playwright fixtures force `scroll-behavior: auto !important` on every context

## Status / trigger

**LANDED 2026-10-18 (lane specs 7a066be9 (override deleted)), merged on the session branch; deploy pending** -- was: **Scheduled 2026-10-18 (lane specs, added to its brief)**

## Size

S

## Detail

Found by lane baseline (row 228): the override in `playwright/fixtures.ts` hid the ambient-smooth navigation bug from every spec, and now masks the anchor opt-in (`html[data-anchor-scroll]`). Narrow or remove it so the suite sees the page's real root `scroll-behavior`; `nav-scroll-instant.spec.ts` keeps its own `unmask()` either way.
