---
id: 358
title: "all-routes: one navigation for both schemes (needs a clean forced-colors reset)"
status: "LANDED 2026-10-23 (branch claude/playwright-suite-infra): one load per route for both schemes…"
size: "S"
area: tests
updated: 2026-10-23
---

# Row 358: all-routes: one navigation for both schemes

## Item

`all-routes` navigates every route twice (light, dark): 321 of its ~1,000 worker-seconds. Row 334 showed the DOM is scheme-independent
(the theme is attributes and CSS; `__dxApplyMode` is the one writer), and on 20 routes axe, the 390 px reading and the light forced-colors
sweep read the same on a page switched in place as on a fresh dark load.

## Status / trigger

**LANDED 2026-10-23 (branch `claude/playwright-suite-infra`): one load per route for both schemes, every route checked against fresh loads; all-routes 1,083 -> 859 worker-seconds in one run (-21 %)**

## Size

S

## Detail

See the "Not built" list in row 334 (done/) for the numbers and the other ideas that were measured and rejected.

## Update 2026-10-23: built, without a focus reset

The blocker was not the focus start point alone. The home page's 81-of-83 was the first sweep's `focus()` of a
roving-tabindex container (Menubar, Tag Group) changing it, so no second sweep could be exact. The construction: one
keyboard walk; the skip link and every control are focused once and their outline read in each scheme while focused,
the scheme flipped in the page by `__dxApplyMode`. Overflow reads both schemes in one resize; axe runs per scheme on the
switched page. Compared with fresh loads on all 90 routes x 2 schemes: axe node counts per rule 180/180, forced-colors
verdicts 174/174 (no control gained or lost), overflow verdicts 180/180; `collapsible` and `tabs` joined
`FRESH_PHONE_LOAD` (their closed find-in-page panels lay out unclipped boxes after any second re-layout). Numbers and
method: [`row-358-one-navigation-both-schemes.md`](../../research/row-358-one-navigation-both-schemes.md).
