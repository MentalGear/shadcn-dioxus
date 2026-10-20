---
id: 126
title: "Link's aria_current can emit a duplicated attribute under a base path; the charts gallery tab row uses plain…"
status: "LANDED 2026-10-11 (root-caused and fixed by construction; one library residue below) -- the…"
size: "S (diagnosis)"
area: charts
updated: 2026-10-20
---

# Row 126: Link's aria_current can emit a duplicated attribute under a base path; the charts gallery tab row uses plain…

## Item

`Link`'s `aria_current` can emit a duplicated attribute under a base path; the charts gallery tab row uses plain anchors to avoid it

## Status / trigger

**LANDED 2026-10-11 (root-caused and fixed by construction; one library residue below)** -- the construction: `preview/src/nav_link.rs` `NavLink` renders its own anchor with `Link`'s click semantics (only an unmodified primary click is client-routed; modifier and middle clicks fall to the browser) and exactly one `aria-current` from a route comparison with the query stripped, identical on server and client (an `active` override covers the Area tab on `/charts/`; a caller's `aria-current` is dropped from the spread). Navbar, footer, docs sidebar and `ChartsTab` use it (the hand-rolled anchor is gone). Guard: `scripts/check-internal-hrefs.sh` rule 3, every raw router `Link {` in `preview/src` needs a `// link-ok: <reason>` marker and none may carry `aria_current` (11 external/content sites marked). Evidence on a release base-path build: 0 duplicate `aria-current` over 7517 anchors; new `playwright/nav_link.spec.ts` (prerendered with JS off and hydrated agree on `/`, `/docs`, `/demos`, `/charts/`, `/charts/bar/`, `/component/button/`; the mark follows client navigation; ctrl/middle click open a tab). **Residue:** `primitives/src/navbar.rs` `NavbarItem` (a shipped component) wraps raw `Link`, so its current-page mark has the same hydration loss (the home-page navbar demo item); needs its own fix in the library. Not matched by the guard: the `Link(props)` call form (one dead `GotoIcon` in `main.rs`). Original finding: not the base path: dioxus-router 0.7.9's `Link` writes its own `aria_current` literal (`href == router.full_route_string()`) beside `..attributes`, so a caller's `aria_current` duplicates it (7 active chart tabs had `aria-current` twice in the SSG HTML), and after hydration the computed value is lost site-wide: routes carry `?:dark_mode`, the server compares `/charts/?` with `/charts/?` but the client's route string has no `?`, so every navbar/footer `Link` that is `aria-current="page"` in the prerender has none once hydrated. `ChartsTab`'s doc comment now states this; the construction is one `NavLink` with a route-level comparison. Was: **OPEN** -- hazard noted 2026-10-03 while building the gallery tab row; not reproduced or root-caused here **2026-10-20 sync: checked, `primitives/src/navbar.rs` `NavbarItem` still wraps raw `Link` (the residue is unfixed).**

## Size

S (diagnosis)

## Detail

`preview/src/charts_gallery.rs` (`ChartsTab`, ~line 431) renders `a { href: prefix + route, aria_current: ..., onclick -> navigator().push }` instead of the router `Link`, after the `Link` + `aria_current` + base-path combination was found to risk a duplicated `aria-current` attribute. The plain anchor re-implements `Link`'s modifier/non-primary-click behavior by hand (a drift risk) and the prefix by hand (`router().prefix()`, the construction row 117's link fix standardised). **Next step:** reproduce on an SSG build with `--base-path`, count `aria-current` attributes per `<a>` (the `check-attr-spread-collision.sh` family, row 93; and `merge_attributes`, row 108), and either fix `Link` or document why the gallery is the exception. Do not copy the plain-anchor pattern elsewhere until then.
