---
id: 358
title: "all-routes: one navigation for both schemes (needs a clean forced-colors reset)"
status: "BACKLOG. Blocked on one thing: the forced-colors sweep presses Tab from the focus start point…"
size: "S"
area: tests
updated: 2026-10-22
---

# Row 358: all-routes: one navigation for both schemes

## Item

`all-routes` navigates every route twice (light, dark): 321 of its ~1,000 worker-seconds. Row 334 showed the DOM is scheme-independent
(the theme is attributes and CSS; `__dxApplyMode` is the one writer), and on 20 routes axe, the 390 px reading and the light forced-colors
sweep read the same on a page switched in place as on a fresh dark load.

## Status / trigger

BACKLOG. Blocked on one thing: the forced-colors sweep presses Tab from the focus start point, which the first sweep moves into `<main>`; a
reset (blur, strip the hash, scroll to 0, prepend and remove a button) restores it on 3 of 4 routes tried but the home page then judges 81
controls instead of 83. Build it only with a reset that is exact on every route, or an explicit fresh-load set like `FRESH_PHONE_LOAD`, each
route verified against a fresh load as in `dev-docs/research/row-327-all-routes-and-goto-hydrated.md`.

## Size

S

## Detail

See the "Not built" list in row 334 (done/) for the numbers and the other ideas that were measured and rejected.
