---
id: 438
title: "dioxus-native (Blitz) as a supported renderer: parked, research-gated, after row 434"
status: "PARKED. Start when all of these hold: 1. Dioxus 0.8 is stable and the primitives build on it. 2.…"
size: "Estimates from the research: tier 0 M…"
area: primitives
updated: 2026-10-25
---

# Row 438: dioxus-native (Blitz) as a supported renderer

## Item

The owner asked on 2026-10-25 whether the library could target Blitz, the native renderer that has no JavaScript. The research is in [`../research/blitz-feasibility-2026-10-25.md`](../research/blitz-feasibility-2026-10-25.md).

- `document::eval` on dioxus-native returns `Err(EvalError::Unsupported)`, so every eval path needs a Rust arm. Row 434 (Rust first) is the prerequisite.
- Hard blockers in Blitz 0.3.0-beta.4 / dioxus-native 0.8.0-alpha.3 (2026-10-16):
  - no popover or top layer and no `showModal` (Blitz issue #196);
  - `:focus-visible` and `:has()` hard-coded off;
  - no `contenteditable`, so no DatePicker segments;
  - no pointer capture;
  - no resize, toggle or animation events.
- What works: `position: fixed`, transitions and keyframes, `:hover`/`:focus`, the hover and colour-scheme media queries, and `MountedData` rects, scroll and focus, but only on the Dioxus 0.8 line. Inline SVG is static.
- The repo already has a native arm behind `not(feature = "web")` (46 sites in 18 primitives files) that renders overlays inert.

## Status / trigger

PARKED. Start when all of these hold:
1. Dioxus 0.8 is stable and the primitives build on it.
2. Blitz ships `:focus-visible`, `:has()` and either popover or a documented top-layer replacement.
3. A written test strategy exists for a renderer with no DOM oracle, because the Playwright suite and the gates are browser-only.

## Size

Estimates from the research: tier 0 M, static components M-L, interactive non-overlay L, overlays XL, and charts and DatePicker blocked.

## Detail

Per-tier "claim support" gates are in section 4.4 of the research note. Re-check Blitz issues #196 and #887 when the trigger is evaluated; the sandbox could not read them on 2026-10-25.
