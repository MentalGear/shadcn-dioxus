---
id: 327
title: "Audit 1: visit each route once per scheme in all-routes; measure load vs networkidle in gotoHydrated"
status: "PARTLY LANDED 2026-10-21 (queue batch 1, PR #126, c5d02507): all-routes loads each (route, scheme)…"
size: "M"
area: tests
updated: 2026-10-21
---

# Row 327: Audit 1: visit each route once per scheme in all-routes; measure load vs networkidle in gotoHydrated

## Item

Audit 1: visit each route once per scheme in all-routes; measure load vs networkidle in gotoHydrated.

## Status / trigger

**PARTLY LANDED 2026-10-21 (queue batch 1, PR #126, c5d02507): `all-routes` loads each (route, scheme) once. Left: measure `load` plus `html[data-hydrated]` against `networkidle` in `gotoHydrated` on one file.**

## Size

M

## Detail

From `dev-docs/research/test-architecture-audit-2026-10-21.md` section 2 #1: `all-routes` loads every route 6 times (3 probe families x 2 schemes), ~1,000 s saved by one load per (route, scheme) running every probe family; `gotoHydrated` waits for `networkidle` (a 500 ms idle window, `playwright/hydration.ts:47`, 965 call sites) where `load` plus `html[data-hydrated]` may suffice -- measure on one file first. NOT taken: the audit's `reducedMotion` fixture default -- a global context override hides real behaviour (7a066be9 removed the last one, row 228; row 256 decision).
