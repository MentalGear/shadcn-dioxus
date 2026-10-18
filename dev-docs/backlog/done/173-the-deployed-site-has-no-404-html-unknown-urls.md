---
id: 173
title: "The deployed site has no 404.html: unknown URLs (e.g. /dioxus-components/charts/no-such-chart/) get GitHub…"
status: "LANDED 2026-10-11 -- scripts/deploy-preview.sh calls the new scripts/emit-404.sh, which derives…"
size: "S"
area: build
updated: 2026-10-11
---

# Row 173: The deployed site has no 404.html: unknown URLs (e.g. /dioxus-components/charts/no-such-chart/) get GitHub…

## Item

**The deployed site has no `404.html`:** unknown URLs (e.g. `/dioxus-components/charts/no-such-chart/`) get GitHub Pages' own 404 instead of the app's not-found page

## Status / trigger

**LANDED 2026-10-11** -- `scripts/deploy-preview.sh` calls the new `scripts/emit-404.sh`, which derives `404.html` from the prerendered `demos` page (the client always hydrates, so an empty SPA shell throws; `demos` is the header/footer shell without a sidebar, and its asset URLs already carry the base path), hides `main` until `html[data-hydrated]` so the demos list does not flash, and verifies its own output. On a GitHub-Pages-like server the four `theme-picker-phone.spec.ts:138` not-found cases go from red to green (38/38). Plus a catch-all `NotFound` route in `preview/src/main.rs` (not prerendered), so a URL matching no route shows a designed "Page not found" page instead of the router's "Failed to parse route" dump. Was: **OPEN** -- found 2026-10-05 in round 3's final verification; **next-session step 1**

## Size

S

## Detail

`docs/` (the committed deploy output) has no `404.html`, and `scripts/deploy-preview.sh` does not emit one, so every URL the SSG did not prerender is answered by GitHub's generic 404 and never reaches the app's router. The phone-picker spec's 4 not-found cases fail on a plain static server for exactly this reason (a static server answers an unknown path with its own 404, not the app). **Fix:** have `deploy-preview.sh` emit a `404.html` that boots the app (the standard SPA fallback: the app shell, with its asset references correct under the base path) so the router renders its not-found page, then test it on a plain static server with an unknown URL under the base path (the phone-picker spec's 4 not-found cases are the ready-made check). Files: `scripts/deploy-preview.sh`; the new `docs/404.html` is generated output. No overlap with the other step-1 lanes.
