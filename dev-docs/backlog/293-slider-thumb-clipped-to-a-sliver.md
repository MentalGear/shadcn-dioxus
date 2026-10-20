---
id: 293
title: "Slider thumb clipped to a sliver"
status: "OPEN (owner review 2026-10-19). New bug; row 239 only touched the thumb's all: unset."
size: "S"
area: primitives
updated: 2026-10-19
---

# Row 293: Slider thumb clipped to a sliver

## Item

The slider knob shows as a thin strip (screenshot 2), and its focus ring is cut off too.

## Status / trigger

OPEN (owner review 2026-10-19). New bug; row 239 only touched the thumb's `all: unset`.

## Size

S

## Detail

- **Cause:** `SliderThumb` renders inside `SliderTrack` (`slider/component.rs:24-26`). The track is 4 px high with `overflow: hidden` (`slider/style.css:30`), copied from Nova, where the thumb sits outside the track. Our thumb is a 12 px absolute circle centred on the track, so the track clips it, along with its `::after` hit area.
- **Fix:** move the clipping to `.dx-slider-range` and give the track `overflow: visible`, for both the horizontal and the vertical orientation.
- **Coarse step**, part of the same row: grow the hit area to at least 44 px (`::after` inset −16px instead of −8px) and optionally the thumb to 20 px under the row-291 media query.
- Add a spec that the thumb's box isn't clipped.
