---
id: 190
title: "oracle/tier2-html/demo-placeholder-links.spec.ts:146 (top_layer) is flaky: link.focus() times out"
status: "FIXED 2026-10-14 (spec bug) -- activating a link closes the NavigationMenu panel, which stays…"
size: "S (diagnosis)"
area: tests
updated: 2026-10-14
---

# Row 190: oracle/tier2-html/demo-placeholder-links.spec.ts:146 (top_layer) is flaky: link.focus() times out

## Item

`oracle/tier2-html/demo-placeholder-links.spec.ts:146` (top_layer) is flaky: `link.focus()` times out

## Status / trigger

**FIXED 2026-10-14 (spec bug)** -- activating a link closes the NavigationMenu panel, which stays mounted and visible (`data-state="closed"`) for ~420 ms of exit animation; the scenario's `isVisible()` said "already open" inside that window and the panel was removed under the next `focus()` (MutationObserver log: click t, closed t+20, removed t+420, focus timeout in between; 11/18 at `--workers=6 --repeat-each=18`). The scenario now returns only for a `[data-state="open"]` panel, waits for a closing one to leave, parks the pointer and hovers the trigger for a fresh `mouseenter`; 18/18 twice, no product bug (`hover_intent.rs` never closed with the pointer inside)

## Size

S (diagnosis)

## Detail

The top_layer page's placeholder links sometimes cannot be focused within 5s (an open overlay or a pending re-render covering them is the first suspect). Not a batch-2 regression. Diagnose before row 152's suite review; do not widen the timeout.
