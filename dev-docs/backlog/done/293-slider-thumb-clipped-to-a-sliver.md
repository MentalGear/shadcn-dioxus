---
id: 293
title: "Slider thumb clipped to a sliver"
status: "LANDED 2026-10-20 (lane controls, 782baae0, spec fixes 839c07c4). New bug; row 239 only touched the…"
size: "S"
area: primitives
updated: 2026-10-20
---

# Row 293: Slider thumb clipped to a sliver

## Item

The slider knob shows as a thin strip (screenshot 2), and its focus ring is cut off too.

## Status / trigger

LANDED 2026-10-20 (lane controls, 782baae0, spec fixes 839c07c4). New bug; row 239 only touched the thumb's all: unset.

## Size

S

## Detail

- **Cause:** `SliderThumb` renders inside `SliderTrack` (`slider/component.rs:24-26`). The track is 4 px high with `overflow: hidden` (`slider/style.css:30`), copied from Nova, where the thumb sits outside the track. Our thumb is a 12 px absolute circle centred on the track, so the track clips it, along with its `::after` hit area.
- **Fix:** move the clipping to `.dx-slider-range` and give the track `overflow: visible`, for both the horizontal and the vertical orientation.
- **Coarse step**, part of the same row: grow the hit area to at least 44 px (`::after` inset −16px instead of −8px) and optionally the thumb to 20 px under the row-291 media query.
- Add a spec that the thumb's box isn't clipped.

## Landed

- **Root cause:** `.dx-slider-track { overflow: hidden }` (4px) clipped its child `SliderThumb` (12px, absolute, centred), its focus ring and its `::after` hit area. Single, range and vertical sliders share this one rule, so all three are fixed; no other component repeats it (color-picker slider/area thumbs sit in tracks with no `overflow`, the switch thumb has none, resizable's handle is not a track child).
- **Fix:** track `overflow: visible`; the rounded clip moves to `.dx-slider-range` (`border-radius: inherit`), horizontal and vertical. Coarse pointer: thumb 20px and `::after` inset -12px (44px target).
- **Spec:** `playwright/slider-thumb-clip.spec.ts` (no clipping ancestor for every thumb, hit tests at the thumb's four extremes, ring not cut, coarse sizes). Red on the pre-fix build (3 of 3), green after.
