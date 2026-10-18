---
id: 191
title: "Batch 2 of the demo-parity work surfaced four library classes; all fixed by construction"
status: "LANDED 2026-10-12"
size: "M (done)"
area: overlays
updated: 2026-10-12
---

# Row 191: Batch 2 of the demo-parity work surfaced four library classes; all fixed by construction

## Item

Batch 2 of the demo-parity work surfaced four library classes; all fixed by construction

## Status / trigger

**LANDED 2026-10-12**

## Size

M (done)

## Detail

(1) **`merge_attributes` folds `style`** (primitives/src/lib.rs): a caller's `style` used to REPLACE the primitive's, silently dropping `anchor-name` (an avatar dropdown opened at 0,0; `SelectTrigger` and `ComboboxInput` had the same hole); now joined, primitive declarations first, a later `None`/empty ignored, 9 tests. (2) **Overlay clamp measures the layout viewport minus scrollbars**: one `VIEWPORT_METRICS_JS` helper (`min(visualViewport, documentElement.client*)`) for both clamp sites in `top_layer.rs`, plus a test forbidding raw viewport reads; Chromium here already excluded the scrollbar from `visualViewport`, so the overshoot was reproduced only by simulating an engine that does not (unverified on Safari/Firefox). (3) **`CommandInput` autofocuses only inside `CommandDialog`** (an inline Command stole page focus on load) with an `autofocus: Option<bool>` override; and **Command renders every item in SSR/JS-off** (items registered only in an effect, so the static page showed the empty message): unregistered options count as visible and `CommandEmpty` waits for the first registration. (4) CSS: slider vertical range and thumb sizing, context-menu/menubar item flex+gap (three near-identical menu item blocks: row 145's menu recipe should unify them), toast error text on the destructive fill (3.91 -> 4.83:1), drawer handle hidden for left/right, input-otp separator spacing, a `--dx-combobox-width` slot variable. Open from the same round: `Calendar`'s `today`/`view_date` default to `now()` at render (every demo must pin them to keep SSG and hydration equal; a by-construction default would read the date only after mount); `ProgressLabel`-style gaps listed on row 188.
