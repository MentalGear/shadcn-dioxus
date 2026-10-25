---
id: 427
title: "Spec page and read kinds become helpers; the exception markers retire"
status: "BACKLOG. Owner approved 2026-10-25 (row 365 sweep, P2)."
size: "S-M"
area: testing
updated: 2026-10-25
---

# Row 427: Spec page and read kinds are helpers, and the exception markers retire

## Item

Playwright specs say what kind of page or box they read in prose: about 124 `goto-ok:` / `rest-ok:` / `viewport-ok:` markers and 37 allowlist entries (20 in `check-demo-text-locators.sh`, 17 in `check-spec-viewport-bound.sh`), one line at a time. The vendored W3C reference-page exception is written in two gates. Construction: one named helper per kind (`gotoPrerendered` and `gotoReference` beside `gotoHydrated`; `staticBox` beside `measureAtRest`; `pageMetrics` beside `fixedViewportBox`). Each helper checks its own claim (origin, running animations, scroll metrics), and each gate accepts only the named helper, so a marker stops being the way out.

## Status / trigger

BACKLOG. Owner approved 2026-10-25 (row 365 sweep, P2).

## Size

S-M

## Detail

Source: [final sweep note, P2](../research/final-sweep-2026-10-25.md). Spec-only; it can run beside Rust work.

- **Measured** (`grep -rn "<marker>:" playwright --include=*.ts`): `goto-ok:` 58 (30 "not the app": 23 vendored W3C APG reference pages in `oracle/tier1-apg/` (menu-roles, disclosure-navigation, sortable-table, select-only-combobox, focus-restore-reference, carousel), four `about:blank`, three static fixtures; 23 "JavaScript off / wasm aborted / first-paint": nothing hydrates by design). `rest-ok:` 47 (about 30 "does not animate"). `viewport-ok:` 19. Allowlists: `check-demo-text-locators.sh:63` (20 entries, 16 "W3C's vendored example page") and `check-spec-viewport-bound.sh:66-84` (17 entries, "scroll extent, not a containment bound").
- **Construction:** `playwright/hydration.ts` gains `gotoPrerendered(page, url)` (no hydration wait; asserts nothing hydrated when JS is off) and `gotoReference(page, url)` (fails if the URL is under the app's origin or base path). `playwright/animations.ts` gains `staticBox(locator)`, which checks `getAnimations()` on the element and its ancestors and throws if one runs, so "does not animate" becomes a checked fact. `playwright/viewport-box.ts` gains `pageMetrics(page)` returning `{ scrollExtent, sidewaysOverflow, gutter }`; `fixedViewportBox` stays the overlay bound.
- **Subsumes:** about 53 of the 58 `goto-ok`, about 30 `rest-ok`, about 15 `viewport-ok`, and the vendored-page locator entries (a locator read through `gotoReference`'s page object needs no exemption). **Does not subsume:** `focus-ok`, `listener-ok` and `key-ok` markers (in Rust and JS, each one-off), and `rest-ok` lines whose reason is that the sampler measures motion itself. The 23 JS-off specs may need two helpers (JS-off vs wasm-aborted); the sweep decides.
- **Gate:** `check-spec-goto-hydrated` accepts `gotoHydrated|gotoPrerendered|gotoReference` and nothing else; `check-spec-at-rest` accepts `measureAtRest|readAtRest|staticBox`; `check-spec-viewport-bound` accepts `fixedViewportBox|pageMetrics`. Each gate's self-test gets a mutation per helper.
- **Inferred, to check:** that `gotoReference` can enforce "not the app" cheaply with an origin check.
