---
id: 521
title: "The accessibility scan skips the closed docs accordion items: Usage notes, Component structure and Props reference are never inspected"
status: "BACKLOG. Small and independent; take it beside any lane."
size: "S."
area: tests
updated: 2026-11-14
---

# Row 521: the accessibility scan opens the docs accordion items

## Item

A closed docs accordion item holds its content in `hidden="until-found"`, which computes to
`content-visibility: hidden`, and axe reports its rules as inapplicable there. `playwright/axe.ts` opens nothing before
it scans, so the content of "Component structure" and "Props reference" is never inspected on any component page, and
"Usage notes" joined them when it stopped being open by default (owner 2026-11-14).

## Status / trigger

BACKLOG. Small and independent; take it beside any lane.

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
