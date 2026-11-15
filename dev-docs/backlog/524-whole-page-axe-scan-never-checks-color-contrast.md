---
id: 524
title: "A whole-page axe scan never checks color-contrast: axe 4.10.2 cannot parse an oklch colour with a `none` channel, so the rule comes back incomplete with no passes and no violations"
status: "BACKLOG (found 2026-11-15). Not started. Small; a test-side row."
size: "S."
area: tests
updated: 2026-11-15
---

# Row 524: the whole-page axe scan's contrast check is silently empty

## Item

On `/component/button/`, `/component/toolbar/` and `/component/table/`, with the docs accordion closed or open, axe
4.10.2 returns `color-contrast` as *incomplete* with no passes and no violations, with the error
`Unable to parse color "oklch(0.995516 0.0000495707 none)"`. A whole-page `expectNoAxeViolations` therefore never
checks contrast in this Chromium, on any component page; only a scan scoped with `include` (as row 521's seeded
self-check is) checks it. The element that carries that colour is not yet identified: it is a computed colour the
engine serialises with a `none` hue (an achromatic `oklch()`), which axe's parser does not accept. Found 2026-11-15 by
the row 521 lane, recorded in row 521.

## Status / trigger

BACKLOG (found 2026-11-15). Not started. Small; a test-side row.

## Size

S.

## Detail

- **The construction.** Two parts. (1) `expectNoAxeViolations` fails when `color-contrast` comes back incomplete for
  a parse error, so the gap cannot reopen silently; axe's `incomplete` list with its `message` is the evidence. (2) Find
  the element whose computed colour serialises with `none` (a theme token that resolves to an achromatic `oklch()` with
  the hue omitted, or an `oklch(... / ...)` mix) and give it a form axe parses, or, if it is axe's parser alone, pin the
  axe version that accepts it and raise. Whichever it is, the fix is by construction: a stylelint or gate rule for
  the token form, not a per-page allowlist.
- **Proof.** The whole-page scan of `/component/button/` reports `color-contrast` with passes, and the seeded
  low-contrast paragraph of row 521's self-check is also found by a whole-page scan.
- **Files.** `playwright/axe.ts`, `playwright/component-catalog.spec.ts`, the theme token (`preview/assets/`), CHANGELOG.
