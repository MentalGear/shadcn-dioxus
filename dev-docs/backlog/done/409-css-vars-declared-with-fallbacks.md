---
id: 409
title: "Every CSS custom property read is declared, fallbacks included"
status: "LANDED (merge queue batch 16, lane/css-var-declared). Trigger for reopening: a var(--name, ...)…"
size: "S"
area: build
updated: 2026-10-24
---

# Row 409: Every CSS custom property read is declared, fallbacks included

## Item

`preview/assets/main.css` read the retired token `--primary-color-6` (on a commented-out line), and `check-css-vars-defined.sh` (row 111) let any `var(--x, fallback)` through, so a retired or misspelt token with a fallback was invisible.

## Status / trigger

LANDED (merge queue batch 16, lane/css-var-declared). Trigger for reopening: a `var(--name, ...)` read whose name is declared nowhere and is not a documented caller slot.

## Size

S

## Detail

The dead line is deleted. `scripts/check-css-vars-defined.sh` now requires every name read in `preview` and `primitives` to be declared, fallbacks included, with a reasoned allowlist of 22 entries (19 caller-provided slots, each pointing at its docs, and 3 toast properties written by JS under a computed name); a stale entry (declared, or read nowhere) fails. Negative tests: a misspelt token with a fallback, `--primary-color-6`, and a dropped slot read each turn it red.
