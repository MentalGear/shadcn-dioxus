---
id: 296
title: "Combobox list drifts from its input on iOS after long scrolling"
status: "OPEN (owner report 2026-10-19), needs device verification. Related rows: 28 (iOS keyboard anchor…"
size: "M + device check"
area: overlays
updated: 2026-10-19
---

# Row 296: Combobox list drifts from its input on iOS after long scrolling

## Item

On iOS, after a lot of scrolling and the keyboard showing and hiding, the open combobox list stops tracking its input.

## Status / trigger

OPEN (owner report 2026-10-19), needs device verification. Related rows: 28 (iOS keyboard anchor fix, still unverified on a device), 10 and done/210.

## Size

M + device check

## Detail

- **Today:** `use_anchor_position_fallback` (`top_layer.rs`, called from `combobox/components/list.rs:~125-150`) works in visual-viewport coordinates. It re-places on window scroll, resize, visualViewport resize/scroll and a ResizeObserver, throttled to one call per animation frame.
- **Likely causes (inferred, not reproduced, since the sandbox has no WebKit):**
  1. `reposition()` returns early when native CSS anchoring already "conforms" (~1480-1500), so if iOS resolves the native anchor wrongly, nothing corrects it.
  2. The one-frame lag during momentum scroll and the keyboard pan, with the collapsing URL bar as a third moving part.
- **Direction:**
  - Skip the native-conformance shortcut on WebKit or touch-primary devices.
  - Re-place synchronously on visualViewport scroll.
  - Add a `scrollend`/`touchend` re-place.
- **Files:** `primitives/src/top_layer.rs`, `combobox/components/list.rs` and the top-layer spec. Close this together with row 28.
