---
id: 183
title: "The home page card estimate: contain-intrinsic-block-size ignored the cards' padding"
status: "LANDED 2026-10-10"
size: "S (done)"
area: misc
updated: 2026-10-10
---

# Row 183: The home page card estimate: contain-intrinsic-block-size ignored the cards' padding

## Item

The home page card estimate: `contain-intrinsic-block-size` ignored the cards' padding

## Status / trigger

**LANDED 2026-10-10**

## Size

S (done)

## Detail

Row 174 cause 4. `content-visibility: auto` lays a never-rendered card out at its `contain-intrinsic-block-size`, which is the CONTENT-box size; `.dx-component-card` estimated `auto 380px` and then added 3rem of padding top and bottom, 476 px against a measured mean of 384 px (median 256 px), so 58 of 75 cards were over 100 px off and the document shrank 38.5k to 32.0k px (-17%) during the first scroll: the scrollbar thumb jumped (64 height steps over 30 px, the largest 1,544 px in one frame). The estimate is now the mean measured border-box height less the padding (`preview/assets/main.css`: cards `auto 290px`, narrow `auto 418px`, widgets `auto 270px`); the mean, not the median, because the sum over the 75 cards keeps the scrollbar steady. Spec: `playwright/oracle/tier2-html/home-scroll-stability.spec.ts` fails when the first scroll changes the document height by 10% or more (re-measure when the gallery changes).
