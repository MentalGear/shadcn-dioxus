---
id: 441
title: "The modal scrim fade ignores prefers-reduced-motion: the per-sheet rule never won on specificity"
status: "BACKLOG. Applying reduced motion changes what users with that setting see, so it waits for the…"
size: "S"
area: components
updated: 2026-10-26
---

# Row 441: reduced motion for the modal scrim

## Item

Before row 426, the dialog-family sheets each had a `prefers-reduced-motion` rule for the no-top-layer wrapper scrim. The `[data-state]` animation rule out-ranked it, so the 0.2 s fade played under reduced motion anyway. Row 426 moved the scrim into the theme (`:where([data-dx-scrim])`) and dropped those dead rules without changing behaviour.

## Status / trigger

BACKLOG. Applying reduced motion changes what users with that setting see, so it waits for the owner. shadcn/Radix leave this to the app's animation utilities.

## Size

S

## Detail

- The fix is one theme rule under `@media (prefers-reduced-motion: reduce)` that sets `animation: none` (or zero duration) on `:where([data-dx-scrim])` at a specificity that wins. The fix is in the Outcome section of row 426's done file.
- Check that the top-layer `::backdrop` path behaves the same way, and add a spec with `reducedMotion: 'reduce'` emulation.
