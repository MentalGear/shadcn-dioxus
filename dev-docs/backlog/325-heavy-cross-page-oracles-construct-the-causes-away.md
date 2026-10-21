---
id: 325
title: "Heavy cross-page oracles: construct the causes away, run what remains per deploy"
status: "PARTLY LANDED 2026-10-21: the popup half (row 259, batch A) and the all-routes half (row 327, batch…"
size: "M"
area: tests
updated: 2026-10-21
---

# Row 325: Heavy cross-page oracles: construct the causes away, run what remains per deploy

## Item

Heavy cross-page oracles: construct the causes away, run what remains per deploy.

## Status / trigger

**PARTLY LANDED 2026-10-21: the popup half (row 259, batch A) and the `all-routes` half (row 327, batch 1). Left: `first-paint-styles`, through row 329's follow-up.**

## Size

M

## Detail

`popup-style-isolation` opens every popup on every page because popups inherit from their DOM host; the popup-root typography reset (row 218's half, lane popup-root-reset, 2026-10-21) removes the inherited channel and `check-popup-boundary.sh` already removes the selector channel, so the spec shrinks to one popup per component. `all-routes` (16% of suite time) and `first-paint-styles` (7.7%) move to the per-deploy run of row 322 once rows 327 and 329 shrink them.

**Update 2026-10-21:** the popup half landed with batch A -- row 259 (lane popup-root-reset 419a2526): one zero-specificity popup-root rule resets 98 inherited properties and `check-popup-root-reset.sh` guards it; `popup-style-isolation` shrank from 85 tests / 592 s to 22 tests / 150 s (one popup per component under a hostile parent) while the old full matrix still passes 89/89 on the new build. Left: `all-routes` and `first-paint-styles` (rows 327 and 329, Round 4 thread).
