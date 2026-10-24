---
id: 405
title: "An iOS rubber-band bounce no longer freezes a windowed renderer: offsets are clamped and a bad message is skipped"
status: "LANDED (pending merge) -- owner iOS feedback 2026-10-24: \"At the boundary on the screenshot other…"
size: "S"
area: primitives
updated: 2026-10-24
---

# Row 405: a scroll bounce cannot freeze a windowed renderer

## Item

On iOS the VirtualGrid "Fade on all edges" demo stopped rendering cells past R15/C8 although it still scrolled. A
rubber-band bounce reports a negative `scrollTop`; the bridge sent it into a `u32` field of `ScrollReport`, the
message failed to decode, and the `while let Ok` loop in `JsListeners::on_message` ended, so the window never moved
again.

## Status / trigger

**LANDED (pending merge)** -- owner iOS feedback 2026-10-24: "At the boundary on the screenshot other items are no
more displayed even though scrollable". Fix: `claude/virtual-grid-ios-bounce` (dbab6ae). `virtual-engine.spec.ts`
replays the bounce: red without the fix, green with it.

## Size

S

## Detail

Two constructions, one per layer: the bridge clamps every offset into the element's scroll range (direction-aware,
so an RTL negative `scrollLeft` maps correctly), and `JsListeners::on_message` skips a message that fails to decode
instead of ending its loop, which covers every listener built on `primitives/src/js_listener.rs`, not only the
virtual engine.
