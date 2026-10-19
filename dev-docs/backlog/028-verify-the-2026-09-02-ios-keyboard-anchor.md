---
id: 28
title: "Verify the 2026-09-02 iOS-keyboard anchor-tracking fix (use_anchor_position_fallback…"
status: "User re-test after deploy — the sandbox has no WebKit, and page.setViewportSize() cannot reproduce…"
size: "S to verify"
area: overlays
updated: 2026-09-03
---

# Row 28: Verify the 2026-09-02 iOS-keyboard anchor-tracking fix (use_anchor_position_fallback…

## Item

Verify the 2026-09-02 iOS-keyboard anchor-tracking fix (`use_anchor_position_fallback`, `primitives/src/top_layer.rs`) on a real iOS/WebKit device

## Status / trigger

User re-test after deploy — the sandbox has no WebKit, and `page.setViewportSize()` cannot reproduce a real iOS keyboard's `window.visualViewport`-only shrink (`window.inner{Width,Height}` stay put on a real device but do change under Playwright's viewport resize, so the oracle's simulation cannot fully exercise the `visualViewport`-vs-`window.inner*` split the fix reads)

## Size

S to verify

## Detail

Same missing calibration as row 4 (first 3-browser CI run): Playwright's `webkit`/`Mobile Safari` project remains unrun here. `playwright/oracle/tier2-html/top-layer.spec.ts` Rule 11 is the closest available proxy — its own header doc records which of the seven `use_anchor_position_fallback` consumers this sandbox's Chromium can be driven into the critical "conforming at open, diverging later" shape for (only `Popover`, via `edge-bottom-popover-*`) and why the rest (Tooltip/HoverCard: real hover-loss on the simulated resize closes them for a legitimate, unrelated reason; DropdownMenu/Select/Combobox/Menubar-at-mobile-width: never land on the CSS-anchor path at open in this sandbox to begin with) could not be driven into it here. **2026-09-03 extension, three more items from the same round of iOS 18 Safari device review, none reproducible in this sandbox for a structural reason (see each's own doc for why), all fixed by construction anyway:** (A) the inline-shift fix (row 10's 2026-09-03 addendum, `top-layer.spec.ts` Rule 12) — needs confirming the home page `ColorPicker` no longer clips on a real narrow iOS Safari viewport; (B) the opening-gesture false positive fix (`use_dialog_backdrop_dismiss`'s deferred listener attachment, `native-dialog.spec.ts` Rule 8) — needs confirming "Show Popover" on the home page stays open through a real touch tap on a real device, since this sandbox's Chromium cannot exhibit the click-retargeting quirk the fix guards against at all; (C) the `Navbar` top-layer migration (`top-layer.spec.ts` Rules 1/5/11) — needs confirming the nav's dropdown actually flips above a bottom-pinned trigger on a real iOS Safari (no CSS Anchor Positioning there), not just under this sandbox's `stripAnchorSupportsBlock` simulation of that condition
