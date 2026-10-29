---
id: 296
title: "Combobox list drifts from its input on iOS after long scrolling"
status: "ROOT CAUSE FOUND 2026-10-28 (owner's iPhone runs of the device lab, iOS 18.7 Safari); fixed by row…"
size: "M + device check"
area: overlays
updated: 2026-10-28
---

# Row 296: Combobox list drifts from its input on iOS after long scrolling

## Item

On iOS, after a lot of scrolling and the keyboard showing and hiding, the open combobox list stops tracking its input.

## Status / trigger

ROOT CAUSE FOUND 2026-10-28 (owner's iPhone runs of the device lab, iOS 18.7 Safari); fixed by [row 477](./477-anchored-overlays-placed-in-document-coordinates.md), which closes this row. The inferred causes below are superseded by the Root cause section; the earlier notes stay as the record.

OWNER RE-TEST 2026-10-26 (iPhone): still drifts; the list can also cover its input. Owner's reading: the keyboard limits the space and the list neither shrinks to the height left once the keyboard has fully slid in nor becomes scrollable. Menubar, Navigation Menu and Popover do not drift, so the direction is to place Combobox the way they do. The next session compares the two paths (owner 2026-10-26: fresh session).

**WAITING ON OWNER: reproduce on an iPhone (owner report 2026-10-19); no fix has landed and the cause is inferred, not reproduced.** Direction in the Detail: skip the native-conformance shortcut in `use_anchor_position_fallback` on WebKit or touch-primary devices, re-place on `visualViewport` scroll, add a `scrollend`/`touchend` re-place; close it together with row 28. Related: rows 28, 10, done/210.

## Size

M + device check

## Detail

**Root cause (measured 2026-10-28).** The panel is a top-layer `popover` with `position: fixed`, placed by JS (`primitives/src/top_layer/anchor_fallback.rs`) that adds `visualViewport` offsets on `visualViewport` events, which iOS fires only about every 260 ms. With the keyboard up the panel drifts by up to `vv.offsetTop` (176 px measured; the maximum `vv.offsetTop` was 354 px). The rect frames also differ: iOS WebKit `getBoundingClientRect` is relative to the visual viewport, Chromium's to the layout viewport, so mixing rect coordinates with scroll or `visualViewport` offsets is wrong by construction. The fix is row 477's placement rule (absolute top-layer panel, document coordinates, transform-only, 0 px drift on the device); Base UI places the same way. Evidence: [`radix-vs-base-ui-2026-10-28.md`](../research/radix-vs-base-ui-2026-10-28.md).

- **Today:** `use_anchor_position_fallback` (`top_layer.rs`, called from `combobox/components/list.rs:~125-150`) works in visual-viewport coordinates. It re-places on window scroll, resize, visualViewport resize/scroll and a ResizeObserver, throttled to one call per animation frame.
- **Likely causes (inferred, not reproduced, since the sandbox has no WebKit):**
  1. `reposition()` returns early when native CSS anchoring already "conforms" (~1480-1500), so if iOS resolves the native anchor wrongly, nothing corrects it.
  2. The one-frame lag during momentum scroll and the keyboard pan, with the collapsing URL bar as a third moving part.
- **Direction:**
  - Skip the native-conformance shortcut on WebKit or touch-primary devices.
  - Re-place synchronously on visualViewport scroll.
  - Add a `scrollend`/`touchend` re-place.
- **Files:** `primitives/src/top_layer.rs`, `combobox/components/list.rs` and the top-layer spec. Close this together with row 28.
