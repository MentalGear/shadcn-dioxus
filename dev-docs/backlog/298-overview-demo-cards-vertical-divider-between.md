---
id: 298
title: "Overview demo cards: vertical divider between description and example"
status: "OPEN (owner review 2026-10-19)."
size: "S"
area: preview
updated: 2026-10-19
---

# Row 298: Overview demo cards: vertical divider between description and example

## Item

Add a vertical rule between the description column and the example column of the docs overview cards.

## Status / trigger

OPEN (owner review 2026-10-19).

## Size

S

## Detail

- The grid is `main.css:1844-1856`: header | content, then the footer. It stacks below 67rem (container query at about 2020).
- Add `border-inline-start: 1px solid var(--dx-border)` plus padding on the content column, in the two-column layout only. Use logical properties so it flips for RTL.
- Re-run `card-fit.spec.ts`, since the border takes width from the preview column (row 280 just fixed clipping there).
- Owner default (2026-10-19): full height of the header/content row, excluding the footer.
