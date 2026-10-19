---
id: 202
title: "Extra components explain why they are Extra"
status: "LANDED 2026-10-14"
size: "S (done)"
area: tests
updated: 2026-10-14
---

# Row 202: Extra components explain why they are Extra

## Item

Extra components explain why they are Extra

## Status / trigger

**LANDED 2026-10-14**

## Size

S (done)

## Detail

Owner request. `Origin::Extra(&'static str)` makes the reason part of the type (an Extra row without one does not compile; unit test: non-empty, < 120 chars); component page shows a visible line under the title and a Tooltip on the badge; sidebar/card badges carry it as `title`. `check-component-catalog.sh` requires the payload; the specs read reasons via one shared parser `playwright/catalog.ts` that throws on a row-count mismatch (the old regex had silently dropped the 8 Extra pages from `all-routes`: +48 tests).
