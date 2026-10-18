---
id: 2
title: "Phase 4.4 — popover/popover=\"auto\" top layer for non-modal overlays"
status: "Landed (Tooltip/HoverCard → popover=\"manual\"; non-modal Popover → <dialog popover=\"auto\">)"
size: "M"
area: overlays
updated: unknown
---

# Row 2: Phase 4.4 — popover/popover="auto" top layer for non-modal overlays

## Item

**Phase 4.4** — `popover`/`popover="auto"` top layer for non-modal overlays

## Status / trigger

**Landed** (Tooltip/HoverCard → `popover="manual"`; non-modal Popover → `<dialog popover="auto">`)

## Size

M

## Detail

Fixes clipping in `overflow:hidden`/transformed ancestors (`playwright/oracle/tier2-html/top-layer.spec.ts` rule 1, red → green); also required a CSS-Anchor-Positioning addition (not originally scoped, but necessary: top-layer promotion changes containing block to the viewport) — Phase 5's flip sub-problem has since landed (row 10: `position-try-fallbacks`, PR #17), shift/size clamping remain the residual scope; CSS Anchor Positioning is Chromium-only as of this writing, but Firefox/WebKit no longer fall back to un-anchored placement — `use_anchor_position_fallback` (row 18, PR #23) self-corrects trigger-relative placement on engines without native support, still pending confirmation on a real non-Chromium engine (item 4 below). Popover-based rendering also cuts the CSS close-fade animation short (shown/hidden is driven by the immediate `open` signal, not the delayed animation-hold signal) — a minor, undone follow-up. Unblocks nested-submenu work in the component queue
