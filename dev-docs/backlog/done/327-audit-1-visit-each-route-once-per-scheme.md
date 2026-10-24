---
id: 327
title: "Audit 1: visit each route once per scheme in all-routes; measure load vs networkidle in gotoHydrated"
status: "LANDED 2026-10-21: all-routes loads once per scheme (PR #126); gotoHydrated load+data-hydrated…"
size: "M"
area: tests
updated: 2026-10-21
---

# Row 327: Audit 1: visit each route once per scheme in all-routes; measure load vs networkidle in gotoHydrated

## Item

Audit 1: visit each route once per scheme in all-routes; measure load vs networkidle in gotoHydrated.

## Status / trigger

**LANDED 2026-10-21: all-routes loads once per scheme (PR #126); gotoHydrated load+data-hydrated measured against networkidle and NOT adopted (see Detail)**

## Size

M

## Detail

From `dev-docs/archive/research/test-architecture-audit-2026-10-21.md` section 2 #1: `all-routes` loads every route 6 times (3 probe families x 2 schemes), ~1,000 s saved by one load per (route, scheme) running every probe family; `gotoHydrated` waits for `networkidle` (a 500 ms idle window, `playwright/hydration.ts:47`, 965 call sites) where `load` plus `html[data-hydrated]` may suffice -- measure on one file first. NOT taken: the audit's `reducedMotion` fixture default -- a global context override hides real behaviour (7a066be9 removed the last one, row 228; row 256 decision).

**Measurement 2026-10-21 (lane test-tools, debug SSG, select.spec + popover.spec, 2 workers, 79 tests, one run each):** networkidle 82 s, load + `html[data-hydrated]` 105 s, both 10 failed / 69 passed (the same ten, on an older template build). `load` was not faster (the 500 ms idle window overlaps the wasm download and hydration the flag waits for anyway) and the extra run was cut short by a container restart, so no repeated-run reliability case could be made. Not adopted; `hydration.ts` unchanged. Item closed.
