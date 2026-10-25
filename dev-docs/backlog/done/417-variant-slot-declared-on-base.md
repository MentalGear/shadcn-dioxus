---
id: 417
title: "A variant's slot variable is declared on the component's base root too"
status: "LANDED (merge queue batch 19, lane/variant-slot-base). Trigger for reopening: a nested…"
size: "S"
area: build
updated: 2026-10-25
---

# Row 417: A variant's slot variable is declared on the component's base root too

## Item

A horizontal Toolbar nested in a vertical one inherited the outer root's `--dx-toolbar-group-direction: column`, because only the vertical root declared the slot and the group read it with a fallback. `variant-nesting.spec.ts` caught it in the batch-18 release suite; `check-variant-scope.sh` documented the construction (the root re-declares a slot variable for every variant, the default included) but did not enforce it.

## Status / trigger

LANDED (merge queue batch 19, lane/variant-slot-base). Trigger for reopening: a nested other-variant instance that inherits a slot variable from its outer variant.

## Size

S

## Detail

`check-variant-scope.sh` gains an "undeclared-slot" check: for any rule whose selector is one compound of a `dx-` class and a variant attribute (`data-variant`, `data-size`, `data-orientation`, `data-spacing`, `data-align`), looking through `:where()`/`:is()` and inside at-rules, every custom property it sets must also be declared by that class's bare base rule. `SLOT_ALLOW` has one reasoned entry (the sidebar trigger's `--dx-button-radius`, declared by `.dx-button`'s base); stale entries fail; five in-memory mutations of the real sheets must turn red and a control case stays green. The gate flagged Carousel, whose orientation roots declared the Previous/Next placement and fade-timeline slots with none on the base: the horizontal values moved onto the base `.dx-carousel` and `.dx-carousel-fade` rules (computed values unchanged). The Toolbar instance was fixed in batch 18.
