---
id: 69
title: "Axe scan readiness race (playwright/axe.ts), baseline group B"
status: "Found + fixed 2026-09-18 (construction); live verification confirmed round 2"
size: "S"
area: tests
updated: 2026-09-18
---

# Row 69: Axe scan readiness race (playwright/axe.ts), baseline group B

## Item

Axe scan readiness race (`playwright/axe.ts`), baseline group B

## Status / trigger

Found + fixed 2026-09-18 (construction); live verification confirmed round 2

## Size

S

## Detail

`playwright/axe.ts`'s `expectNoAxeViolations()` called `AxeBuilder.analyze()` immediately after `page.goto()` resolved, with no readiness wait. For this CSR wasm app the `load` event can fire before Dioxus paints its first frame into the empty `<div id="main">` shell, so the scan occasionally raced the app and saw the bare `<html lang="en">` — axe then reported `landmark-one-main` + `page-has-heading-one` against `html` itself, a false violation: every failing test's own `error-context.md` snapshot, captured moments later, shows a fully-formed page with exactly one `<main>` and one `<h1>`. Evidence it is a race, not a docs-chrome defect (the task brief's own hypothesis — PRs #52-57's navbar/sidebar rewrite — was checked directly and does not hold): all 8 failures finished in 1.3-2.0s versus 1.8-8.5s for every *passing* axe scan in the same run — fast because there was almost nothing in the DOM yet to scan, not because the page was simpler. Members: `empty`, `alert`, `tabs` (tab 1), `collapsible` (collapsed), `toolbar`, `button_group`, `breadcrumb`, `avatar`. **Construction (Lane I, commit `3cb0b16`):** 72 call sites share `expectNoAxeViolations`, so the fix lives in the helper (`waitForAppReady`), not per spec file — wait for the page's one `<main>` landmark to attach, then for a level-one heading to attach (`state: "attached"` only, no `networkidle`/visibility wait), bounded at 15s so a page that genuinely never renders still fails fast. Two waits, not one, because the race produces two violations and every route this suite scans renders both together as part of the same first paint (verified by reading every route fn in `preview/src/main.rs`). **Not verified against a live browser by the lane** (a mid-task sandbox resource correction ruled out starting a dev server there) — checked only with `playwright test --list` (all 694 tests across 61 files still enumerate, no parse/type error introduced). Live re-run of the 8 Group-B specs plus the axe-adjacent failures this same race caused elsewhere (`native-dialog.spec.ts` Rule 6c/6d/6e, `top-layer.spec.ts` Rule 5 Tooltip/HoverCard — see row 70, same fixture, different mechanism) is confirmed: **round 2's targeted Playwright run against the integrated build found all 8 Group-B specs green**, plus the axe-adjacent failures this same race caused elsewhere. Red/green demonstrated directly, not just inferred: the pre-fix `axe.ts` re-run under 4 workers × 8 (Group B's 8 specs) failed 7/16 with the baseline signature; the current `axe.ts` under the same conditions is 16/16. `pw-baseline/BASELINE-REPORT.md` Group B is the full evidence trail
