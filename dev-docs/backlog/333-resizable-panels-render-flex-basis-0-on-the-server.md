---
id: 333
title: "Resizable panels render flex-basis 0% on the server and get their real size only in an effect"
status: "BACKLOG (found 2026-10-21 by the row-329 spike)"
size: "S"
area: components
updated: 2026-10-21
---

# Row 333: Resizable panels render flex-basis 0% on the server and get their real size only in an effect

## Item

Resizable panels render flex-basis 0% on the server and get their real size only in an effect.

## Status / trigger

**BACKLOG (found 2026-10-21 by the row-329 spike)**

## Size

S

## Detail

Found 2026-10-21 by the row-329 parity spike (Round 4 thread): the server HTML of the resizable panels has `flex-basis: 0%`, and an effect sets 50%/40% after hydration, so the first paint of every resizable demo collapses the panels and then jumps -- a layout shift and the row 306 class. Fix by computing the default sizes on the server from the panels' `default_size` props.
