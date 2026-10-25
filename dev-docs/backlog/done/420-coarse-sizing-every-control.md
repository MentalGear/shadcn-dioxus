---
id: 420
title: "Touch sizing covers every control: a coarse-aware control scale and one hit size"
status: "LANDED (lane/coarse-controls-420, for merge queue batch 24). Trigger for reopening: a control with…"
size: "M"
area: components
updated: 2026-10-25
---

# Row 420: Touch sizing covers every control: a coarse-aware control scale and one hit size

## Item

Owner, 2026-10-25: the `data-dx-coarse` switch (Auto/On/Off in the palette) resized only menus, the text-entry font, the slider and Resizable; buttons, toggles, tabs, pagination, inputs, selects, checkbox, radio, switch and TagGroup's remove button kept their desktop size on touch. Every control follows the one switch; touch targets aim at 44px; text entry is 17px on touch; heights stay tokens (fixed heights keep mixed controls in a row aligned) and the tokens follow the switch. TagGroup's remove button was too small on desktop too. Upstream shadcn has no touch sizing at all.

## Status / trigger

LANDED (lane/coarse-controls-420, for merge queue batch 24). Trigger for reopening: a control with a fixed size that ignores the switch, or a row of mixed controls that misaligns in either mode.

## Size

M

## Detail

- **Control scale** (theme, `:root, [data-dx-coarse]` block, so a forced container re-evaluates): `--dx-control-height-xs/sm/default/lg` are 24/28/32/36px fine (unchanged) and 36/40/44/48px coarse. About 22 components read the scale and follow with no change of their own; adjusted: Toolbar button `min-width` (stays square), Input OTP slot font, Calendar caption row (`max(cell, xs)`, nav buttons centred), and the docs header at 320px keeps its sidebar trigger at 1.75rem. Calendar day cells use `--dx-calendar-cell-size` and do not change.
- **Hit size**: `--dx-control-hit-size`, 24px fine (WCAG 2.5.8) and 44px coarse, for glyph-sized controls whose visible box stays its own size: Checkbox and Switch are a transparent frame around the glyph (the switch track is the button's `::before`), Radio items take it as `min-height`, TagGroup's remove button is a hit-size square (24px on desktop, inside the tag's padding; the tag grows to hold 44px on touch), the slider thumb's `::after` hit area, the accordion trigger's `min-height` (default control height), and the Toast close (the larger of sm and the hit size). On fine pointers negative margins and `translate` keep today's layout. On coarse the block axis is real layout, so stacked targets cannot overlap.
- **Text entry**: `--dx-coarse-text-entry-font-size` is 17px coarse (14px fine), inside the 44px coarse fields.
- **Evidence**: `coarse-alignment.spec.ts` (fine heights equal the pre-change CSS, coarse heights, mixed rows equal height and edges within 0.5px in both modes, no clipping, shipped demos) and `coarse-glyph-controls.spec.ts` (fine boxes equal the baseline, 24px/44px targets, glyph size kept, no overlapping neighbour targets, slider hit test). A box dump of 11,188 controls on every component page from the base build (`c8fe20ce`, project file `coarse-420/baseline-c8fe20ce.json`) is the baseline for the fine-mode comparison.
