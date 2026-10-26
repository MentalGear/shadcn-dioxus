---
id: 423
title: "An open Accordion header still underlines on hover; the header should never underline"
status: "LANDED (lane/accordion-underline-423). Trigger for reopening: a text decoration on an accordion…"
size: "S"
area: components
updated: 2026-10-25
---

# Row 423: An open Accordion header still underlines on hover

## Item

The owner reports (2026-10-25) that an open accordion's header still shows an underline on hover. The header should
show no hover underline in any state.

## Status / trigger

LANDED (lane/accordion-underline-423). Trigger for reopening: a text decoration on an accordion trigger in any state.

## Size

S

## Detail

Find where the underline comes from: the accordion sheet's trigger `:hover` rule, or the docs page's own accordion
styling, which uses the library's Accordion. Fix it where it is declared and keep the rule inside the hover media query
(`check-hover-media.sh`). Add a spec that hovers an open and a closed trigger and reads `text-decoration-line`.

Landed on lane/accordion-underline-423. The underline came from one rule: `preview/src/components/accordion/style.css`,
`.dx-accordion-trigger[aria-expanded="true"]:hover:not(:disabled) { text-decoration-line: underline }`, inside the hover
media query (row 299 kept shadcn's `hover:underline` on open headers only). The rule is removed, so no trigger underlines in
any state; the sheet's comment and the accordion docs sentence now say so. Nothing else in `preview/assets/` paints an underline
on the trigger (the docs accordion heading and the theme only set `text-decoration-skip` and offset). The accordion-chevron
spec's open-header assertion is inverted to `none`, and `playwright/accordion-no-underline.spec.ts` hovers a closed and an
open trigger and reads `text-decoration-line` of the trigger and every element in it.
