---
id: 423
title: "An open Accordion header still underlines on hover; the header should never underline"
status: "BACKLOG, bug. Stage 1 (bugs) of the adopted order."
size: "S"
area: components
updated: 2026-10-25
---

# Row 423: An open Accordion header still underlines on hover

## Item

The owner reports (2026-10-25) that an open accordion's header still shows an underline on hover. The header should
show no hover underline in any state.

## Status / trigger

BACKLOG, bug. Stage 1 (bugs) of the adopted order.

## Size

S

## Detail

Find where the underline comes from: the accordion sheet's trigger `:hover` rule, or the docs page's own accordion
styling, which uses the library's Accordion. Fix it where it is declared and keep the rule inside the hover media query
(`check-hover-media.sh`). Add a spec that hovers an open and a closed trigger and reads `text-decoration-line`.
