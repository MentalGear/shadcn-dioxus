---
id: 207
title: "One edge-panel system for Sheet and Drawer"
status: "LANDED 2026-10-14 (edge lane 0579af5; with rows 208, 197, 155 group 7)"
size: "M"
area: overlays
updated: 2026-10-14
---

# Row 207: One edge-panel system for Sheet and Drawer

## Item

One edge-panel system for Sheet and Drawer

## Status / trigger

**LANDED 2026-10-14 (edge lane 0579af5; with rows 208, 197, 155 group 7)**

## Size

M

## Detail

Both are on the modal dialog primitive but with two side enums, two sheets, two slide animations. One theme recipe (side, slide, inset, radius) + one side type; Drawer = edge panel + drag + handle. Decide Nova's 0 panel padding (row 197) here. Item 4. Plan: `dev-docs/research/next-round-plan-2026-10-14.md`. **Landed:** theme "Edge panel" section keyed on `data-dx-surface="edge"` plus the resolved `data-side` (`data-dx-edge="floating"` floats left/right), slots `--dx-edge-padding/-radius/-inset/-duration-*/-ease-*`, shared `dx-edge-in/out-*` keyframes on `translate` selected via `--dx-edge-enter/-exit`. `primitives/src/drawer.rs`: one `EdgeSide` (Top/Right/Bottom/Left/Auto, default Auto); `DrawerSide` and `SheetSide` are aliases. Sheet: typed `SheetProps { side }`, `SheetPanel` with its own `data-state`; the legacy `data-side` attribute still overrides (Sidebar uses it). Nova's 0 panel padding decided: Sheet 0, Drawer 16px (row 197). Stale allowlist entries removed from `check-duplicate-keyframes.sh` and `check-anchored-keyframes.sh`. `drawer.spec` 46/46, sheet + modal-exit + overlay-switch 52/52. **Open:** `theme.rs` `ThemePickerDrawer` relies on the Drawer default being bottom (phone-only today): pass side Bottom explicitly.
