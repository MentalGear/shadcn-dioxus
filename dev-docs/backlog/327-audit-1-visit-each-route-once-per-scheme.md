---
id: 327
title: "Audit 1: visit each route once per scheme in all-routes; measure load vs networkidle in gotoHydrated"
status: "BACKLOG (owner 2026-10-21: do it if it holds up)"
size: "M"
area: tests
updated: 2026-10-21
---

# Row 327: Audit 1: visit each route once per scheme in all-routes; measure load vs networkidle in gotoHydrated

## Item

Audit 1: visit each route once per scheme in all-routes; measure load vs networkidle in gotoHydrated.

## Status / trigger

**BACKLOG (owner 2026-10-21: do it if it holds up)**

## Size

M

## Detail

From `dev-docs/research/test-architecture-audit-2026-10-21.md` section 2 #1: `all-routes` loads every route 6 times (3 probe families x 2 schemes), ~1,000 s saved by one load per (route, scheme) running every probe family; `gotoHydrated` waits for `networkidle` (a 500 ms idle window, `playwright/hydration.ts:47`, 965 call sites) where `load` plus `html[data-hydrated]` may suffice -- measure on one file first. NOT taken: the audit's `reducedMotion` fixture default -- a global context override hides real behaviour (7a066be9 removed the last one, row 228; row 256 decision).
