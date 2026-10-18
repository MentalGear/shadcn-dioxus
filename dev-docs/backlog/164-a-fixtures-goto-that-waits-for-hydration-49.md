---
id: 164
title: "A fixtures goto that waits for hydration (49 spec files lack the wait), and migrate the 362 legacy ?name=…"
status: "OPEN -- recorded 2026-10-05 (audit); scheduled in the next-session order (step 4, just before row…"
size: "M (the fixture is S; the migration is…"
area: tests
updated: 2026-10-05
---

# Row 164: A fixtures goto that waits for hydration (49 spec files lack the wait), and migrate the 362 legacy ?name=…

## Item

A fixtures `goto` that waits for hydration (49 spec files lack the wait), and migrate the 362 legacy `?name=` spec routes

## Status / trigger

**OPEN** -- recorded 2026-10-05 (audit); scheduled in the next-session order (step 4, just before row 152, which touches the same specs)

## Size

M (the fixture is S; the migration is mechanical but touches ~50 spec files)

## Detail

**The class.** Row 109 fixed the hydration race one spec at a time: `gotoHydrated` (`playwright/hydration.ts`) waits for `html[data-hydrated="true"]`, and the 2026-09-26 sweep moved the 24 interacting SSG specs onto it, but choosing between plain `page.goto` and `gotoHydrated` is still a per-spec decision, so any new or untouched spec can click prerendered markup before wasm has attached its listeners. The audit found **49 spec files that never wait for hydration** (re-count at step 0; some are legitimately render-only or vendored W3C reference pages). **Construction:** make the `page` fixture in `playwright/fixtures.ts` (the file every spec must import from, enforced by `check-playwright-fixture-import.sh`) wrap `page.goto` so a navigation to a preview URL waits for `html[data-hydrated="true"]` by default, with an explicit opt-out (`{ hydrate: false }`) for render-only, vendored and no-wasm pages; `gotoHydrated` then becomes a thin alias, and "a spec forgot the wait" cannot recur. Row 109 confirmed the signal on both `dx serve` and the SSG serve, so the default wait does not hang on either. **Second half:** 362 spec routes still use the legacy form `/component/?name=X&`. Row 46's closure keeps those links resolving client-side, but an SSG build serves them as the not-found shell until hydration corrects the page, so a spec on a legacy route that does not wait reads the wrong page; migrate them to the path form `/component/<name>/` the SSG build prerenders (the form the site and the README link use). Do both in one pass over the same files, in an order that keeps the suite green at each step: the fixture default first, with the legacy routes still working, then the route migration. **Subsumes:** row 109's remaining per-spec follow-ups and row 90's `networkidle` remedy for `pie_chart`/`radial_chart`. **Overlap:** row 152 (the suite slimming review) edits the same files, so 164 goes first; the parity wave adds specs, so take 164 after it or let those lanes write their new specs against the new fixture. **2026-10-05 (round 3) evidence and a trap:** debug builds showed Playwright hydration timeouts under 4-worker load (a spec acting before wasm attaches; release builds did not), which is this row's class appearing as flakes: the fixture default fixes it by construction, and until it lands run functional specs with fewer workers on debug. Round 3's new specs (`all-routes.spec.ts`, `modal-exit.spec.ts`, `first-paint-styles.spec.ts`, `theme-picker-phone.spec.ts`, `listener-inventory.spec.ts`) already use `gotoHydrated`; the 49-file count has not been re-taken. **Trap from row 113:** block demos have no prerendered path form on the SSG build (`/component/block/calendar/range/` answers 404 there), only the query form `/component/block/?name=<c>&variant=<v>&`, so the route migration applies to component pages (`/component/<name>/`), not to block pages. **2026-10-05 (round 3 final verification) sub-item, base-path-naive spec selectors:** `modal-exit.spec.ts` (14 cases) and `listener-inventory.spec.ts` (2 cases) match links with `a[href^="/component/…"]` and `a[href="/?"]`, which hold only when the site is served from the root. They PASS on the root-served verification build and FAIL on the base-path deploy build (`/dioxus-components/…`), the same class as row 100's base-path leak but on the spec side. Fix in the same pass over the same files: a shared helper that builds the link locator from the page's base (or match on a base-path-insensitive suffix such as `a[href$="/component/<name>/"]`), and run the suite once on a `--base-path` build to prove it.
