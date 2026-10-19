---
id: 101
title: "oracle/tier1-apg/carousel.spec.ts:224 axe scan fails on the W3C REFERENCE fixture, not on our component"
status: "CLOSED 2026-10-19 (bookkeeping: status already said done). Seen 2026-09-22 (round 7) in a combined…"
size: "Unknown; not yet triaged"
area: tests
updated: 2026-10-19
---

# Row 101: oracle/tier1-apg/carousel.spec.ts:224 axe scan fails on the W3C REFERENCE fixture, not on our component

## Item

`oracle/tier1-apg/carousel.spec.ts:224` axe scan fails on the W3C REFERENCE fixture, not on our component

## Status / trigger

CLOSED 2026-10-19 (bookkeeping: status already said done). Seen 2026-09-22 (round 7) in a combined 68-test run; `carousel.spec.ts` alone is 57/57 green. **FIXED 2026-09-25 (round 8, `1280a7f`)**

## Size

Unknown; not yet triaged

## Detail

A `scrollable-region-focusable` violation on a static `<code>` block inside the imported W3C reference example — the oracle's own baseline, which we do not author. Pre-existing and unrelated to the Carousel component, so it did not block round 7. Decide whether to fix the vendored fixture, scope the axe scan to exclude the reference block, or record it as an accepted upstream defect — the first option edits someone else's reference example, which is the reason it was left alone rather than patched on sight. Also note the filename trap that surfaced it: `npx playwright test carousel.spec.ts` substring-matches BOTH this file and `playwright/carousel.spec.ts`; pass the absolute path to scope a run to one. **FIXED 2026-09-25 (round 8):** took the second option above — scoped the reference axe scan to `#ex1` (the carousel widget itself) via `expectNoAxeViolations`'s own `include` option, rather than editing the vendored reference page at all. Root cause was timing-dependent, not structural: the reference's own script populates `<pre><code id="sc1">` relative to when the scan runs, which is why the violation only showed up in a combined run and not in `carousel.spec.ts` alone — scoping the scan to the widget removes the class regardless of that timing.
