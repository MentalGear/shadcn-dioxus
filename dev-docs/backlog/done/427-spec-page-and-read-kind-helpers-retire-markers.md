---
id: 427
title: "Spec page and read kinds become helpers; the exception markers retire"
status: "DONE 2026-10-25. Owner approved 2026-10-25 (row 365 sweep, P2)."
size: "S-M"
area: testing
updated: 2026-10-25
---

# Row 427: Spec page and read kinds are helpers, and the exception markers retire

## Item

Playwright specs say what kind of page or box they read in prose: about 124 `goto-ok:` / `rest-ok:` / `viewport-ok:` markers and 37 allowlist entries (20 in `check-demo-text-locators.sh`, 17 in `check-spec-viewport-bound.sh`), one line at a time. The vendored W3C reference-page exception is written in two gates. Construction: one named helper per kind (`gotoPrerendered` and `gotoReference` beside `gotoHydrated`; `staticBox` beside `measureAtRest`; `pageMetrics` beside `fixedViewportBox`). Each helper checks its own claim (origin, running animations, scroll metrics), and each gate accepts only the named helper, so a marker stops being the way out.

## Status / trigger

DONE 2026-10-25. Owner approved 2026-10-25 (row 365 sweep, P2).

## Size

S-M

## Detail

Source: [final sweep note, P2](../../research/final-sweep-2026-10-25.md). Spec-only; it can run beside Rust work.

- **Measured** (`grep -rn "<marker>:" playwright --include=*.ts`): `goto-ok:` 58 (30 "not the app": 23 vendored W3C APG reference pages in `oracle/tier1-apg/` (menu-roles, disclosure-navigation, sortable-table, select-only-combobox, focus-restore-reference, carousel), four `about:blank`, three static fixtures; 23 "JavaScript off / wasm aborted / first-paint": nothing hydrates by design). `rest-ok:` 47 (about 30 "does not animate"). `viewport-ok:` 19. Allowlists: `check-demo-text-locators.sh:63` (20 entries, 16 "W3C's vendored example page") and `check-spec-viewport-bound.sh:66-84` (17 entries, "scroll extent, not a containment bound").
- **Construction:** `playwright/hydration.ts` gains `gotoPrerendered(page, url)` (no hydration wait; asserts nothing hydrated when JS is off) and `gotoReference(page, url)` (fails if the URL is under the app's origin or base path). `playwright/animations.ts` gains `staticBox(locator)`, which checks `getAnimations()` on the element and its ancestors and throws if one runs, so "does not animate" becomes a checked fact. `playwright/viewport-box.ts` gains `pageMetrics(page)` returning `{ scrollExtent, sidewaysOverflow, gutter }`; `fixedViewportBox` stays the overlay bound.
- **Subsumes:** about 53 of the 58 `goto-ok`, about 30 `rest-ok`, about 15 `viewport-ok`, and the vendored-page locator entries (a locator read through `gotoReference`'s page object needs no exemption). **Does not subsume:** `focus-ok`, `listener-ok` and `key-ok` markers (in Rust and JS, each one-off), and `rest-ok` lines whose reason is that the sampler measures motion itself. The 23 JS-off specs may need two helpers (JS-off vs wasm-aborted); the sweep decides.
- **Gate:** `check-spec-goto-hydrated` accepts `gotoHydrated|gotoPrerendered|gotoReference` and nothing else; `check-spec-at-rest` accepts `measureAtRest|readAtRest|staticBox`; `check-spec-viewport-bound` accepts `fixedViewportBox|pageMetrics`. Each gate's self-test gets a mutation per helper.
- **Inferred, to check:** that `gotoReference` can enforce "not the app" cheaply with an origin check.

## Outcome

Built as designed, with four differences the sweep left open.

- **Helpers.** `gotoPrerendered` and `gotoReference` are in the new `playwright/page-kinds.ts` (not `hydration.ts`: an edit to
  `hydration.ts` makes `select.mjs` pick every one of the 261 spec files for `test:changed`, and a new file only the specs that import
  it); `gotoReference` throws when the URL is under the app's origin, `gotoPrerendered` throws on `about:`/`file:`. `staticBox` and
  `readStatic` (`animations.ts`) throw when a running keyframe animation (a `CSSAnimation`, not a hover or press transition) sits on
  the element or an ancestor. `pageMetrics(page)` (`viewport-box.ts`) returns the raw page numbers plus the three derived ones
  (`scrollExtent`, `sidewaysOverflow`, `gutter`) and `viewportSize`; code inside `page.evaluate` takes the number it needs as an
  argument.
- **Markers.** `goto-ok` 58 -> 0, `viewport-ok` 19 -> 0, `rest-ok` 47 -> 15 (5 of the 47 sat in `toast-height.spec.ts` on reads that were
  already inside `readAtRest`, so they were never needed). The 15 left are the sampler reads, the drawer drag drivers that place the
  pointer inside their own evaluate, the popup-isolation specs that freeze motion page-wide, and the two selector-driven
  `escapesClip` helpers that settle in the same evaluate.
- **Allowlists.** `check-demo-text-locators` 20 -> 13 (seven W3C entries gone, the eighth retargeted at the indicators variant's own
  `n of 5` panels: a text locator inside a `test(...)` body that calls `gotoReference` is exempt by construction); `check-spec-viewport-bound`
  17 -> 4, all four for code that lives in the page and runs later on its own (the keyboard-inset stub, the chart recorder, two probe
  functions, a rAF sampler). Both now use `scripts/lib/allow.py`, as does `check-spec-at-rest` (0 entries).
- **Gates.** `check-spec-goto-hydrated` (any `.goto(` outside the two helper files, no marker), `check-spec-viewport-bound` (any raw
  source, no marker), `check-spec-at-rest` (accepts `readStatic`/`staticBox`) and `check-demo-text-locators` each got a `--self-test`
  (run by the default run too): the helper is green, the bare call it replaces is red, with and without the retired marker, and each
  gate checks the helper is exported.
- **Not done, as the row said:** `focus-ok`, `listener-ok`, `key-ok`.
