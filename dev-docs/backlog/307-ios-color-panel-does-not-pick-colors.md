---
id: 307
title: "iOS: the Color Panel opens but tapping its color box does not change the color"
status: "WAITING ON OWNER: reproduce on an iPhone (owner report 2026-10-20: the Color Panel opens but…"
size: "M"
area: overlays
updated: 2026-10-24
---

# Row 307: iOS: the Color Panel opens but tapping its color box does not change the color

## Item

On iOS (the preview deployed on the evening of 2026-10-19) the Color Panel opens, but tapping the color box, and probably other controls in the overlay, does not change the color.

## Status / trigger

**WAITING ON OWNER: reproduce on an iPhone (owner report 2026-10-20: the Color Panel opens but tapping its color box does not change the color); no cause is known and no fix has landed** (the sandbox has no WebKit). Start in `primitives/src/color_picker.rs` pointer handling and the `touch-action: none` drag surfaces in `preview/src/components/color_picker/style.css`; related iOS rows 28, 296, 305.

## Size

M

## Detail

Owner report, relayed 2026-10-20: the panel opens; tapping the color box (and, the owner thinks, other controls in the overlay) leaves the color unchanged. Not reproduced here (the sandbox has no WebKit); no cause is claimed. Files involved, named only so the investigation starts in the right place:

- `primitives/src/color_picker.rs` (the picker primitive: saturation/hue/alpha areas and sliders, pointer handling).
- `preview/src/components/color_picker/component.rs` and `style.css` (the styled wrapper; `style.css` sets `touch-action: none` on the drag surfaces).
- `preview/src/components/color_picker/variants/main` (the demo) and `preview/src/components/color_picker/docs.md`.
- The overlay it opens in: `primitives/src/popover.rs` and `primitives/src/top_layer.rs`.

Related iOS rows: 28 (keyboard anchor), 296 (combobox list drift), 305 (side drawer drags while its content scrolls). The home page widget masonry also hosts a `ColorPicker` (see row 10, 2026-09-03 edge-clipping report).
