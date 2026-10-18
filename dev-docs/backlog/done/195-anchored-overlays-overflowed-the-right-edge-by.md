---
id: 195
title: "Anchored overlays overflowed the right edge by the scrollbar gutter; specs measured the same wrong box"
status: "LANDED 2026-10-14"
size: "S (done)"
area: tests
updated: 2026-10-14
---

# Row 195: Anchored overlays overflowed the right edge by the scrollbar gutter; specs measured the same wrong box

## Item

Anchored overlays overflowed the right edge by the scrollbar gutter; specs measured the same wrong box

## Status / trigger

**LANDED 2026-10-14**

## Size

S (done)

## Detail

`html { scrollbar-gutter: stable }` makes `documentElement.clientWidth` ~15px wider than a fixed overlay's containing block; `top_layer.rs` `viewportMetrics()` clamped to it, so the two-month date picker sat 11px past 375/585/885 at 390/600/900px. Fix: one cached `position:fixed; inset:0` probe, width/height = min(visualViewport, clientWidth, probe). The specs that hid it read the same `clientWidth`: one helper `playwright/viewport-box.ts` (`fixedViewportBox`) in 11 spec files and the gate `check-spec-viewport-bound.sh` (a gutter-inclusive read needs `viewport-ok: <reason>` or an allowlist entry). Before/after: date-picker multi-month 4 red -> 54/54.
