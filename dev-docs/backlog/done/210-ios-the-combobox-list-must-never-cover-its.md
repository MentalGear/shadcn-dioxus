---
id: 210
title: "iOS: the Combobox list must never cover its input -- exclusion-zone placement + geometry regression spec"
status: "LANDED 2026-10-14 (overlays lane 51a6aa8, 927fb72; with row 193)"
size: "M"
area: tests
updated: 2026-10-14
---

# Row 210: iOS: the Combobox list must never cover its input -- exclusion-zone placement + geometry regression spec

## Item

iOS: the Combobox list must never cover its input -- exclusion-zone placement + geometry regression spec

## Status / trigger

**LANDED 2026-10-14 (overlays lane 51a6aa8, 927fb72; with row 193)**

## Size

M

## Detail

Owner report, highest priority. Research how shadcn does it first (classic: input inside the popover panel; v4 Base UI: positioner collision avoidance). Placement keeps the anchor box as an exclusion zone inside the visual viewport (shrink max-height, never overlap); geometry spec sweeps viewports incl. keyboard-reduced heights, input positions, scroll offsets, visual-viewport pan/zoom, RTL, typing. No WebKit/real keyboard here: manual device step documented. Item 8. Plan: `dev-docs/research/next-round-plan-2026-10-14.md`. **Landed:** reproduced on main at 390x400 (list 4..210 over input 184..216). Root cause for all ten callers of `use_anchor_position_fallback`: the panel was clamped into the viewport along the anchor's own axis. `reposition()` now picks the side by per-side room in visual-viewport coordinates (the preferred side is kept while it fits), never shifts on the main axis (cross-axis clamp only), caps the panel to the chosen side's room and publishes `--dx-available-height/-width`, trusts a CSS-anchor result only if it is inside the viewport and off the anchor, pins a flipped-up panel by its bottom edge, and a ResizeObserver re-places on size change. `combobox/style.css` `max-height: min(18rem, var(--dx-available-height, 18rem))`; no other sheet had a fixed cap. `combobox-placement.spec` 59/59 (6 plan viewports + 390x320 + 667x375, top/middle/bottom, light/dark, RTL subset, typing shrink/grow, list scroll, keyboard-resize case, CDP pinch-zoom case). Behaviour change: CSS-anchor placement that overflows the cross axis now goes through the JS path. **Limits:** Chromium only, no real iOS keyboard. **Manual iPhone steps:** Combobox page in Safari, field mid/top/bottom + landscape, type "s" and clear; the list must always sit fully above/below the field, shrink and scroll when short of room, and follow the field on drag.
