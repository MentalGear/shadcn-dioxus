---
id: 521
title: "The accessibility scan skips the closed docs accordion items: Usage notes, Component structure and Props reference are never inspected"
status: "DONE on lane/axe-docs-items-521 (unmerged). Open residue: the whole-page color-contrast run is…"
size: "S."
area: tests
updated: 2026-11-15
---

# Row 521: the accessibility scan opens the docs accordion items

## Item

A closed docs accordion item holds its content in `hidden="until-found"`, which computes to
`content-visibility: hidden`, and axe reports its rules as inapplicable there. `playwright/axe.ts` opens nothing before
it scans, so the content of "Component structure" and "Props reference" is never inspected on any component page, and
"Usage notes" joined them when it stopped being open by default (owner 2026-11-14).

## Status / trigger

DONE on `lane/axe-docs-items-521` (unmerged). Open residue: the whole-page `color-contrast` run is incomplete (below).

## Size

S.

## Detail

- **Measured 2026-11-14** on `/component/button/`: `color-contrast` on the Usage notes content passes on one node with
  the item open and is inapplicable with it closed.
- **The construction.** The scan helper opens every `.dx-docs-accordion` item through its trigger and waits for the
  open state before it scans, so the rule holds for every caller (`all-routes.spec.ts`, `component-catalog.spec.ts`)
  and for an item added later. If the full sweep grows too slow, one route per component is the fallback, measured
  first.
- **Proof.** A case that fails on the tree before the change: a seeded contrast violation inside a closed item is
  reported.
- **Files.** `playwright/axe.ts`, the two specs that call it.

## Result (2026-11-15)

- **Built.** `openDocsAccordionItems` in `playwright/axe.ts`, called by `expectNoAxeViolations` before every scan: clicks
  each closed item's trigger, waits for `data-open` and for the items' own finite animations (5 s bound). It does nothing
  under an open modal (`:modal`: the page behind it is inert; the alert-dialog scan otherwise waited out its 90 s timeout).
- **Proof.** `component-catalog.spec.ts` "the scan inspects the content of a closed docs accordion item" seeds a `#bbb` on
  `#fff` paragraph into a closed item of `/component/button/` and scans `.dx-docs-accordion`. With the opening call
  commented out the scan reports nothing (red); with it, `[color-contrast]` on the seeded node (green).
- **Found by the opening.** Five docs tables had empty header cells (`empty-table-header`: questionnaire, table, markdown,
  virtual_grid, virtual_list); their headers are named now.
- **Cost.** `all-routes.spec.ts -g axe`, 91 tests, 2 workers, debug build, beside other lanes: before 7.6 min wall
  (user+sys 3 m 09 s), after 9.1 min (4 m 00 s) with the five failures, 14.3 min (3 m 53 s) on a later run when the machine
  was busier. CPU time is about +23 %; wall time is dominated by the load of the other lanes. A per-component fallback would
  change nothing: the sweep has one docs page per component already, and the other routes have no accordion.
- **Residue: whole-page `color-contrast` is incomplete.** On `/component/button/`, `toolbar` and `table`, closed or open,
  axe 4.10.2 reports `color-contrast` as incomplete with the error `Unable to parse color "oklch(0.995516 0.0000495707
  none)"` and no pass or violation at all. The rule is a no-op on a whole-page scan in this Chromium (the colour is a
  computed value of some element; its source is not found yet). A scan scoped with `include` is unaffected, which is why the
  seeded self-check scopes to the accordion. Separate row to open: find the element, give it a parseable colour or
  pre-process it, and make the helper fail on an incomplete `color-contrast`.
