---
id: 18
title: "CSS Anchor Positioning cross-browser fallback for anchored overlays"
status: "Landed 2026-09-01, PR #23 (use_anchor_position_fallback, primitives/src/top_layer.rs), flip-aware…"
size: "S–M"
area: theme
updated: 2026-09-01
---

# Row 18: CSS Anchor Positioning cross-browser fallback for anchored overlays

## Item

CSS Anchor Positioning cross-browser fallback for anchored overlays

## Status / trigger

**Landed 2026-09-01, PR #23 (`use_anchor_position_fallback`, `primitives/src/top_layer.rs`), flip-aware since PR #17's `position-try-fallbacks` addition**

## Size

S–M

## Detail

A JS-measured positioning fallback now self-corrects fully and immediately for any overlay whose engine lacks CSS Anchor Positioning support, byte-identical to native placement — no more viewport-centered UA-default fallback. Item 4 (3-browser CI run, still not executed in this sandbox) remains the way to confirm this concretely on real Firefox/WebKit rather than Chromium's simulated non-conforming path
