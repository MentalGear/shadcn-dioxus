---
id: 105
title: "playwright/baseline.local.config.ts (and xvfb.local.config.ts/oracle.local.config.ts/ssg.local.config.ts)…"
status: "CLOSED 2026-10-19 (main-loop review: the fix or lesson is in place, see Size/Detail). Found…"
size: "DONE 2026-09-25 (round 8)"
area: ci
updated: 2026-10-19
---

# Row 105: playwright/baseline.local.config.ts (and xvfb.local.config.ts/oracle.local.config.ts/ssg.local.config.ts)…

## Item

`playwright/baseline.local.config.ts` (and `xvfb.local.config.ts`/`oracle.local.config.ts`/`ssg.local.config.ts`) lack the CI config's `grepInvert: /mobile/` on their chromium project, so every full local run also executes the `... (mobile)` specs and fails them on `.tap()` (needs `hasTouch`, which these launches don't emulate)

## Status / trigger

**CLOSED 2026-10-19 (main-loop review: the fix or lesson is in place, see Size/Detail).** Found 2026-09-25 (round 8) in every full local run this session; reproduced identically on `f103bbd` (pre-existing, not caused by this round's own work)

## Size

**DONE 2026-09-25 (round 8)**

## Detail

`navbar.spec.ts:26`, `select.spec.ts:132` and `sidebar.spec.ts:151` are each named with a `(mobile)` suffix and gated behind `grepInvert: /mobile/` in the CI config (`playwright/playwright.config.ts`), which the four `*.local.config.ts` files never copied — each defines its own single `chromium` project from scratch rather than extending the CI one. Fixed by adding the identical `grepInvert: /mobile/,` line to the `chromium` project in all four: `baseline.local.config.ts`, `xvfb.local.config.ts`, `oracle.local.config.ts`, `ssg.local.config.ts` — none of the four restrict `testDir`/`testMatch` to a subset of specs, so all four run the full suite and all four had the gap. No test file changed
