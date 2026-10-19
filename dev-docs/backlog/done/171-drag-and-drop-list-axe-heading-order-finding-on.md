---
id: 171
title: "drag_and_drop_list axe heading-order finding on an h3"
status: "LANDED 2026-10-05 (round 3, final verification)"
size: "S (done)"
area: tests
updated: 2026-10-05
---

# Row 171: drag_and_drop_list axe heading-order finding on an h3

## Item

`drag_and_drop_list` axe `heading-order` finding on an `h3`

## Status / trigger

**LANDED 2026-10-05 (round 3, final verification)**

## Size

S (done)

## Detail

Found by round 3's Sweep B (`playwright/all-routes.spec.ts`: axe, 390 px overflow and forced-colors focus over 80 routes in light and dark, plus 3 block pages), where it was an UNFILED `KNOWN_AXE` entry next to rows 134 and 111 A. **Fixed in round 3:** the main variant's `h3 { class: "dx-tasks-title", "Launch priorities" }` became an `h2` (`preview/src/components/drag_and_drop_list/variants/main/mod.rs`), the `KNOWN_AXE` entry was removed from `all-routes.spec.ts` (the test asserts that a listed finding still reproduces, so a stale entry would fail), and the sweep passes 499/499. Nothing is left to do; the sweep stays entry-free for this route.
