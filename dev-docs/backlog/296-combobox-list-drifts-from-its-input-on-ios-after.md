---
id: 296
title: "Combobox list drifts from its input on iOS after long scrolling"
status: "Fixed by row 477 (landed in PR #156, deployed); closes with 477 after the owner's iPhone run. Root…"
size: "M + device check"
area: overlays
updated: 2026-10-31
---

# Row 296: Combobox list drifts from its input on iOS after long scrolling

## Item

On iOS, after a lot of scrolling and the keyboard showing and hiding, the open combobox list stops tracking its input.

## Status / trigger

Fixed by [row 477](./477-anchored-overlays-placed-in-document-coordinates.md) (landed in PR #156, deployed); closes with 477 after the owner's iPhone run. Root cause found 2026-10-28 (owner's iPhone runs of the device lab, iOS 18.7 Safari); the inferred causes below are superseded by the Root cause section and the earlier notes stay as the record.

OWNER RE-TEST 2026-10-26 (iPhone): still drifts; the list can also cover its input. Owner's reading: the keyboard limits the space and the list neither shrinks to the height left once the keyboard has fully slid in nor becomes scrollable. Menubar, Navigation Menu and Popover do not drift, so the direction is to place Combobox the way they do. The next session compares the two paths (owner 2026-10-26: fresh session).

Superseded (2026-10-19 note): the owner's repro request and the direction (skip the native-conformance shortcut, re-place on `visualViewport` scroll, add a `scrollend`/`touchend` re-place) are replaced by row 477's rule. Related: rows 28, 10, done/210.

## Size

M + device check

## Detail

**Root cause (measured 2026-10-28).** The panel is a top-layer `popover` with `position: fixed`, placed by JS (`primitives/src/top_layer/anchor_fallback.rs`) that adds `visualViewport` offsets on `visualViewport` events, which iOS fires only about every 260 ms. With the keyboard up the panel drifts by up to `vv.offsetTop` (176 px measured; the maximum `vv.offsetTop` was 354 px). The rect frames also differ: iOS WebKit `getBoundingClientRect` is relative to the visual viewport, Chromium's to the layout viewport, so mixing rect coordinates with scroll or `visualViewport` offsets is wrong by construction. The fix is row 477's placement rule (absolute top-layer panel, document coordinates, transform-only, 0 px drift on the device); Base UI places the same way. Evidence: [`radix-vs-base-ui-2026-10-28.md`](../research/radix-vs-base-ui-2026-10-28.md).

Superseded by row 477 (kept as the record):

- **Then:** `use_anchor_position_fallback` (`primitives/src/top_layer/anchor_fallback.rs`, called from `combobox/components/list.rs`) worked in visual-viewport coordinates, re-placing on window scroll, resize, `visualViewport` events and a ResizeObserver, one call per animation frame. The earlier file `top_layer.rs` no longer exists; the placement code is `top_layer/anchor_fallback.rs` and `top_layer/scroll_clamp.rs`.
- **Inferred causes (superseded):** (1) `reposition()` returned early when native CSS anchoring "conformed"; (2) a one-frame lag during momentum scroll and the keyboard pan. Neither was the cause; see the Root cause.
- **Direction (superseded):** skip the native-conformance shortcut on WebKit or touch-primary devices; re-place synchronously on `visualViewport` scroll; add a `scrollend`/`touchend` re-place. Row 477 removes the `visualViewport` tracking altogether.
- **Files:** the fix is row 477's. Close this together with row 28 after the owner's iPhone run.
