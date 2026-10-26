---
id: 441
title: "The modal scrim fade ignores prefers-reduced-motion: the per-sheet rule never won on specificity"
status: "DONE 2026-10-26. The owner chose \"Stop the fade\". shadcn/Radix leave this to the app's animation…"
size: "S"
area: components
updated: 2026-10-26
---

# Row 441: reduced motion for the modal scrim

## Item

Before row 426, the dialog-family sheets each had a `prefers-reduced-motion` rule for the no-top-layer wrapper scrim. The `[data-state]` animation rule out-ranked it, so the 0.2 s fade played under reduced motion anyway. Row 426 moved the scrim into the theme (`:where([data-dx-scrim])`) and dropped those dead rules without changing behaviour.

## Status / trigger

DONE 2026-10-26. The owner chose "Stop the fade". shadcn/Radix leave this to the app's animation utilities.

## Size

S

## Detail

- The fix is one theme rule under `@media (prefers-reduced-motion: reduce)` that sets `animation: none` (or zero duration) on `:where([data-dx-scrim])` at a specificity that wins. The fix is in the Outcome section of row 426's done file.
- Check that the top-layer `::backdrop` path behaves the same way, and add a spec with `reducedMotion: 'reduce'` emulation.

## Outcome

One zero-specificity theme rule after the two scrim state rules in `preview/assets/dx-components-theme.css`: under `@media (prefers-reduced-motion: reduce)`, `:where([data-dx-scrim][data-state])` takes `animation-duration: var(--dx-motion-duration-reduced)` (0.01 ms, the convention the floating and edge surfaces use). It wins by source order. The duration is shortened rather than the animation removed, because the rest state is `opacity: 0` and `animation: none` would leave the open scrim invisible. The native `<dialog>` `::backdrop` already had its `transition-duration` reduced by the modal-scrim media block. `playwright/scrim-reduced-motion.spec.ts` asserts both paths under `reducedMotion: 'reduce'` and the 200 ms fade without it. Its first run is the batch 28 release suite, because the lane was blocked on disk before its own spec run.
