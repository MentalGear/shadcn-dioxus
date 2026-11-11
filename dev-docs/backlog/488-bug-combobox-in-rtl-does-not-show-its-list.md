---
id: 488
title: "Bug: Combobox in RTL does not show its list"
status: "Does not reproduce (2026-11-07, lane/combobox-rtl-488, debug SSG build on main f38662dc). The…"
size: "S"
area: overlays
updated: 2026-10-31
---

# Row 488: Bug: Combobox in RTL does not show its list

## Item

Owner report (2026-10-31): the Combobox's option list does not appear in a right-to-left context. The case is the Combobox RTL demo on its component page, `preview/src/components/combobox/variants/rtl/` (the demo name is `rtl`; its `mod.rs` sets `dir: "rtl"` with a `DirectionProvider` for `Direction::Rtl`), and the page-level direction toggle.

## Status / trigger

Does not reproduce (2026-11-07, `lane/combobox-rtl-488`, debug SSG build on main `f38662dc`). The report is probably fixed by row 477's placement rule (document coordinates, `rtl`-aware alignment in `PLACE_JS`). `playwright/anchored-rtl.spec.ts` stays as the guard.

- **Real RTL demos** (`#component-preview-frame-rtl` on the combobox, select and dropdown_menu pages): the open panel is inside the viewport and overlaps its trigger's columns; the Combobox list's right edge equals the chips field's right edge (inline start), l 562.5 / r 882.5 against the field's 562.5 / 882.5.
- **`top_layer` fixture with the direction flipped** (`html` and every `dir="ltr"` on the way, after hydration): Popover, DropdownMenu and Combobox are inside the viewport and overlap their trigger, on the engine path and on the script path (`stripAnchorSupportsBlock`); the Combobox list's right edge is the input's right edge.
- `combobox-placement.spec.ts`, `anchored-placement.spec.ts` and `anchored-rtl.spec.ts`: 106 passed before the spec was reshaped, 9/9 after.
- **Two traps for the next reproduction.** Setting only `<html dir="rtl">` changes nothing on a component page: the demo frame carries an explicit `dir="ltr"`. A `Select` takes its direction from `DirectionProvider`, not the DOM (its panel carries `dir`/`data-direction` from the context), so a Select on the fixture, which has no provider, stays LTR by design.
- An isolated harness running the real `PLACE_JS`, `PLACEMENT_HELPERS_JS` and `anchor.css` placed a start-aligned list at the input's right edge in all eight combinations (ltr/rtl, engine/script, with and without `data-side`).

## Size

S

## Detail

Size S is provisional: the reproduction confirms it, and a cause in the shared placement code raises it.

**Reproduce first.** On a debug SSG build with `dir="rtl"`, open the RTL demo and the page-level direction toggle, and record which of these happens:

- the list opens off-screen (the placement X is wrong);
- the list is never shown (it is not rendered or it is not opened);
- the list is rendered but hidden behind something (stacking or top-layer order).

**Placement rule.** Row 477 computes the panel's X from the anchor's left edge. In RTL the panel should align to the inline start of the anchor, which is the anchor's right edge. The visible-width clamp may then push the panel out of the viewport when it is computed from the left edge. Compare with Select and DropdownMenu in RTL: they use the same placement, so a Select or DropdownMenu that is correct in RTL points at a Combobox-specific cause (its list host or its open path), and one that fails too points at the shared placement code.

**Likely files (to be confirmed by the reproduction).** `primitives/src/top_layer/anchor_fallback.rs` (placement X for RTL), or the Combobox list component under `primitives/src/combobox/` (its open path and host). Do not edit either before the reproduction names the cause.

**Proof.** An RTL Playwright spec that opens the Combobox and asserts that the list is visible and aligned to the input's inline start (the list's right edge at the input's right edge, within a tolerance). It must be red before the fix and green after. Functional specs may run on a debug SSG build; the fix's gate run is the usual merge-batch run.
