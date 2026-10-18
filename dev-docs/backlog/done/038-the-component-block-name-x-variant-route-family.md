---
id: 38
title: "The /component/block/?name=X&variant=... route family had no page-level <h1>/<main> of its own when visited…"
status: "Landed 2026-09-04"
size: "S–M"
area: tests
updated: 2026-09-04
---

# Row 38: The /component/block/?name=X&variant=... route family had no page-level <h1>/<main> of its own when visited…

## Item

**LANDED 2026-09-04, verified by execution (Chromium).** The `/component/block/?name=X&variant=...` route family had no page-level `<h1>`/`<main>` of its own when visited directly — axe `page-has-heading-one`/`region`, previously RED and documented in-line (`sidebar.spec.ts`'s two "Axe automated scan" tests; the same gap surfaced inside the dashboard email client's own embedded sidebar, `oracle/tier2-html/touch-focus-zoom.spec.ts`'s dashboard test). Turned out to be two defects, not one: `ComponentBlockDemo` now renders a visually-hidden `<h1>` naming the block inside its own `<header>` (banner landmark; the route is `#[end_layout]`-excluded so it is top-level) — deliberately NOT an outer `<main>`, since every block demo already renders one via `SidebarInset` and nesting them is row 34's own already-fixed defect. Separately, the shared `Sidebar` had been rendering its content as an unlandmarked `<div>` all along, which is why the dashboard email client showed the identical `region` finding without ever going through the block route at all — `Sidebar` now renders a `complementary` landmark (desktop and `SidebarCollapsible::None` branches; the mobile `Sheet` branch deliberately left untouched). `sidebar.spec.ts`'s two axe scans **now pass in the Chromium run.** Triage confirmed the dashboard scan's only remaining axe violation is `nested-interactive` (6 nodes) — `region` and `page-has-heading-one` are confirmed ABSENT there — so this row's fix holds and row 37 stays the only blocker on that route

## Status / trigger

Landed 2026-09-04

## Size

S–M

## Detail

Found 2026-09-03 building row 34's axe coverage, visiting `gotoSidebarBlock`'s route directly (as the file's own pre-existing non-axe tests already did) rather than through the iframe. The iframe-embedded case (`sidebar.spec.ts`'s "preview page renders block" test) was and remains a non-issue, unaffected by this change
