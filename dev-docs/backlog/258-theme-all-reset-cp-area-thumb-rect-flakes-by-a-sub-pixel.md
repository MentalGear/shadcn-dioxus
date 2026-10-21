---
id: 258
title: "theme-all-reset.spec.ts cp-area-thumb 'visual computed style' flakes on a sub-pixel rect (20x19.99999)"
status: "OPEN. Appeared while running the batch build locally; it was green in the batch-A release suite.…"
size: "S"
area: playwright
updated: 2026-10-20
---

# Row 258: a sub-pixel `rect` in the colour-picker area thumb snapshot

## Item

`theme-all-reset.spec.ts` "cp-area-thumb > visual computed style equals the pre-change `all: unset` build" compares a computed-style record that includes `rect: "20x20"`; some loads give `"20x19.99999237060547"` (the thumb is `translate(-10px, -10px)` over a fractional position).

## Status / trigger

OPEN. Appeared while running the batch build locally; it was green in the batch-A release suite. Not related to press feedback (the thumb is not pressed and the failure reproduces with the fixture change stashed).

## Size

S

## Detail

Fix when it recurs: record `rect` rounded (`Math.round(w * 100) / 100`) in both the snapshot and the reader, or drop `rect` from this record since the width/height computed styles are already in it. Do not loosen anything else. The same file's snapshot for the other thumbs is exact because their layout positions are integral.
