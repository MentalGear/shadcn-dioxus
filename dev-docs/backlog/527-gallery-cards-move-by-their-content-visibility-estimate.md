---
id: 527
title: "Gallery cards move by their content-visibility estimate when a script scrolls one into view"
status: "BACKLOG (found 2026-11-15). Not started. The owner's call: tests already settle the cards, so this…"
size: "S: the measured table and its writer…"
area: components
updated: 2026-11-18
---

# Row 527: gallery cards move by their content-visibility estimate

## Item

`.dx-component-card` renders lazily (`content-visibility: auto` with `contain-intrinsic-block-size: auto 376px`, in
`preview/assets/main.css`), so a card that has not rendered yet is 472 px tall with its padding. The first time it
renders it takes its real height, and everything below it moves by the difference: the drawer card is 353 px, 119 px
less (measured 2026-11-15). During an ordinary scroll Chromium's scroll anchoring hides most of this. It shows when a
script scrolls an element into view (a focus move, `scrollIntoView`, Playwright's click) and the cards around it render
one frame later; that is what made batch 7's hydration-parity Rule 3 click land beside the dropdown trigger. Tests settle
the cards first (`settleGalleryCards` in `playwright/hydration.ts`), and a fragment navigation already forces the render
(`html[data-cv-settle]`, `preview/index.html`), so the suite is unaffected.

Proposed construction: each card's estimate is its measured height. A generated table of card heights per width class
(the measurements `playwright/card-fit.spec.ts` already takes) is emitted by the SSG as each card's own
`contain-intrinsic-block-size`, and a check fails when a measured height drifts from the table by more than a set
tolerance. Residue: a width between two measured classes still moves by the difference between them.

## Status / trigger

BACKLOG (found 2026-11-15). Not started. The owner's call: tests already settle the cards, so this is only the visible
move on the site.

## Size

S: the measured table and its writer, the SSG attribute, the drift check.

## Detail

- **Evidence:** the batch 7 reproduction of 2026-11-15 (the trigger moved up 119 px between `pointerdown` and `pointerup`
  with `scrollY` unchanged; the drawer card above it went from its 472 px estimate to 353 px).
