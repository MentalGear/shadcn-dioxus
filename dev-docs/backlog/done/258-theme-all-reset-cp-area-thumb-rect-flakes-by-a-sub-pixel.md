---
id: 258
title: "theme-all-reset.spec.ts cp-area-thumb 'visual computed style' flakes on a sub-pixel rect (20x19.99999)"
status: "FIXED 2026-10-21 (lane b-sweeps, row 331: rect rounded to 2 decimals in the record and the reader)"
size: "S"
area: playwright
updated: 2026-10-21
---

# Row 258: a sub-pixel `rect` in the colour-picker area thumb snapshot

## Item

`theme-all-reset.spec.ts` "cp-area-thumb > visual computed style equals the pre-change `all: unset` build" compares a computed-style record that includes `rect: "20x20"`; some loads give `"20x19.99999237060547"` (the thumb is `translate(-10px, -10px)` over a fractional position).

## Status / trigger

**FIXED 2026-10-21 (lane b-sweeps, row 331: rect rounded to 2 decimals in the record and the reader)**

## Size

S

## Detail

Fix when it recurs: record `rect` rounded (`Math.round(w * 100) / 100`) in both the snapshot and the reader, or drop `rect` from this record since the width/height computed styles are already in it. Do not loosen anything else. The same file's snapshot for the other thumbs is exact because their layout positions are integral.

**Resolution (2026-10-21):** row 331 asked whether the golden half of this file catches anything the gates and other specs do not; a probe showed it does (a dropped or altered reset line on the four controls goes red only here), so the golden stays and the flake is fixed as proposed above: `rect` is rounded to 2 decimals in both the record (Switch `32x18.39`) and the reader; the width/height computed styles stay exact. Nothing else was loosened.
