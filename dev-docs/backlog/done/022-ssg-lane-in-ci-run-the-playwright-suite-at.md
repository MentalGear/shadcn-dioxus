---
id: 22
title: "SSG lane in CI — run the Playwright suite (at minimum oracle/hydration-parity.spec.ts) against the…"
status: "CLOSED 2026-10-19 (backlog review 2026-10-18): done in the code -- the SSG lane is in…"
size: "S"
area: ci
updated: 2026-10-19
---

# Row 22: SSG lane in CI — run the Playwright suite (at minimum oracle/hydration-parity.spec.ts) against the…

## Item

SSG lane in CI — run the Playwright suite (at minimum `oracle/hydration-parity.spec.ts`) against the fullstack-SSG-built artifact as its own CI job, not just the `dx serve` dev-server lane the existing `playwright.yml` covers

## Status / trigger

**CLOSED 2026-10-19 (backlog review 2026-10-18): done in the code -- the SSG lane is in playwright.yml and preview/src/main.rs; the commit hashes the row cites do not resolve in this clone**

## Size

S

## Detail

Standing prevention for the 2026-09-01 production incident (`dev-docs/recommended-implementations.md` Caveat 1): the cfg-axis bug that broke the deployed site shipped and passed every existing check, because nothing in CI ever built+served the SSG artifact and hydrated it in a real browser. `scripts/check-cfg-axis.sh` (wired into `main.yml`) catches a *recurrence* of the specific cfg-axis mistake at the source level; this item is the black-box backstop for the *next*, differently-shaped SSG/hydration defect. Recipe: `dev-docs/conformance-harness.md`, "Hydration/deployment parity". **2026-09-04 addendum:** the Chromium calibration run's 5 failures were a dev-server/SSG config mismatch (connection-refused, "not run"), not product defects. **2026-09-19 addendum (batch 3, executed for real):** root cause read from dioxus source, not assumed — `Routable::static_routes()` enumerates only Static/Child route segments (`routable.rs:845-865`) and `FileSystemCache::map_path` strips everything from `?` (`fs_cache.rs:26`), so a query-string route can never become a distinct static file; `dx build --ssg` POSTs `/api/static_routes` to discover what to prerender (`builder.rs:2065-2208`). Construction (`ddcc8b8`): enumerable path-segment routes, `/component/<name>/` (`ComponentDemoPath`) and `/component/block/<name>/<variant>/` (`ComponentBlockDemoPath`), used by every internal link; `server_static_routes()` = `Route::static_routes()` plus 62 demo entries and 3 sidebar variants; the legacy `?name=X&` query routes still work, rendering a loading shell and client-side `navigator().replace`-ing to the canonical path post-mount (new Rule 6 proves this on a static host). A latent defect surfaced and fixed along the way (`83a6a9f`): `GlobalHead` rendered per-route outside `NavigationLayout`, so `document::Link` bookkeeping dropped `main.css`/theme/fonts after a redirect's SPA transition — fixed by hoisting `GlobalHead` into `AppLayout` so it renders exactly once. Three stale hydration-parity rules fixed once real prerendered pages existed to run them against (`c3b4e31`), plus a `navbar.spec.ts`/`sidebar.spec.ts` canonical-URL update. New `SSG_SITE_DIR` env var enumerates every prerendered page for hydration-parity's own extended rules (`02277bd`). New CI job (`4366b43`, `.github/workflows/playwright.yml`) — validated by YAML parse only, not a live Actions run, per the standing CI freeze. `scripts/deploy-preview.sh` now also fails on a shipped not-found shell (`d0afce1`). **Execution result:** release+ssg+fullstack build, 71 routes, 0 not-found shells, 71/71 routes carrying their module bootstrap `<script>`, hydration-parity **11/11** (5 of batch-2's own "not run" results now real and green) — full inventory and the one small defect this execution found cross-referenced at row 46. This docs pass's own production deploy (base-path shape, `--base-path dioxus-components`, this same round) is a second, independent live confirmation of the same route construction: `scripts/deploy-preview.sh`'s own checks reported "OK: all 71 route(s) have their bootstrap script" and "OK: no route shipped the not-found shell" against a real base-path build, 525.64s wall time
