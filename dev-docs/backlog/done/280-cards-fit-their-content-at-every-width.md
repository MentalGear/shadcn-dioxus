---
id: 280
title: "Home-page cards fit their content at every width"
status: "LANDED 2026-10-19 (owner bug report: the composer card clipped its Post button; card-fit spec)"
size: "S"
area: preview
updated: 2026-10-19
---

# Row 280: Home-page cards fit their content at every width

## Item

Owner preview review 2026-10-19: "cards can be cut off in a weird way". The "Reply to roadmap thread" showcase card cut
its footer row (B, I, U, Save draft, Post) at the right edge, Post clipped in half by `.dx-widget-card`'s
`overflow: hidden`.

## Status / trigger

**LANDED 2026-10-19 (owner bug report: the composer card clipped its Post button; card-fit spec)**

## Size

S

## Detail

A width sweep of the deployed site (320-1920 px, step 20) found a class of nine cards, not one: content wider than its
card, clipped (showcase cards) or spilling past the preview frame (component cards).

- Composer: the footer was a non-wrapping flex row (320, 340, 660, 780 px). Now `flex-wrap: wrap`.
- Schedule block and the Calendar card: every calendar demo carried its own `padding: 20px` inside a frame that already
  pads (320-360, 660-820, 1120-1160 px). Removed from all eleven calendar variants.
- Field, Input Group, Pagination, Skeleton, Chart Tooltip, Resizable (780-1160 px): the gallery went to two columns on a
  viewport breakpoint, so with the docs sidebar open a preview column was 286 px wide at 780 px. Now the gallery is a
  size container and a card stacks below a 67rem gallery (Resizable's 516 px preview, twice, plus the gap).
- Input Group at 320-360 px: `.dx-input` lacked shadcn's `min-w-0`, so the input kept its UA intrinsic width in a flex
  row. Added to the Input sheet, which every flex-placed Input inherits.
- Skeleton at 320-360 px and Resizable below 520 px: fixed widths capped by the viewport (`min(250px, 100vw - 6rem)`,
  `min(450px, 100vw - 6rem)`), allowing for the 74 px of phone-card chrome and a desktop browser's 15 px scrollbar
  gutter that `100vw` includes.

Construction: `playwright/card-fit.spec.ts` loads `/` and, at every width from 320 to 1920 (step 16, plus each home-page
breakpoint and its neighbours), fails if any `.dx-widget-card` or `.dx-component-card-preview` has
`scrollWidth > clientWidth`. A new block, demo or breakpoint that overflows fails there at the width where it shows.
Not covered: the component pages' variant frames (the same demos, but `/component/?name=...` pages are wider and were
not swept).
