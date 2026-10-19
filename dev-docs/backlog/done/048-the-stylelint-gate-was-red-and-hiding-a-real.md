---
id: 48
title: "the stylelint gate was red and hiding a real defect. cd preview && npx stylelint \"/*.css\" --max-warnings=0…"
status: "Landed 2026-09-04"
size: "S"
area: ci
updated: 2026-09-04
---

# Row 48: the stylelint gate was red and hiding a real defect. cd preview && npx stylelint "/*.css" --max-warnings=0…

## Item

**LANDED 2026-09-04: the stylelint gate was red and hiding a real defect.** `cd preview && npx stylelint "**/*.css" --max-warnings=0` (`plan.md`'s definition-of-done item 4) had been failing with 52 errors since the row 42 `touch-action` round, unnoticed because all workflows are `workflow_dispatch`-only under the CI freeze (row 33). Four of those errors were a live bug, not a lint nit: row 42 inserted `touch-action: manipulation` immediately BEFORE `all: unset` in `.dx-color-slider-thumb`, `.dx-color-area-thumb`, `.dx-slider-thumb` and `.dx-switch`, where `all: unset` then reset it straight back to `auto`. Scoped precisely: invisible in the preview app because `main.css` also carries an app-wide `touch-action: manipulation !important`, but real for anyone copying those components out via `dx components add` (they ship without `main.css`) — for them, row 42's iOS double-tap fix was doing nothing on `Switch`, the Slider thumb, and both Color Picker thumbs. Fixed by reordering `all: unset` first (also what the linter wants). The other ~48 violations were swept for the same "shorthand resets a longhand declared above it" hazard — none found — then fixed mechanically (unrelated stylelint conventions, no behavior change). The gate is now green

## Status / trigger

Landed 2026-09-04

## Size

S

## Detail

Found while running `plan.md`'s own definition-of-done item 4, which nothing in CI currently exercises (all workflows `workflow_dispatch`-only, row 33) — recommend `stylelint.yml` goes on a real trigger once the user lifts the CI freeze, so a regression like this is caught immediately rather than sitting unnoticed for a whole round. A different bug mechanism from the `css_module`/`@supports` hashing class (rows 31a/32/47) — a dead-property-order cascade bug, not a selector-scoping one; noted separately so the two classes aren't conflated
