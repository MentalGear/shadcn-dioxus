---
id: 245
title: "Computed-style snapshot harness: tooltip timing nondeterminism and the skip link counted as an open-state item"
status: "BACKLOG (validation 2026-10-19)"
size: "S"
area: tests
updated: 2026-10-19
---

# Row 245: Computed-style snapshot harness: tooltip timing nondeterminism and the skip link counted as an open-state item

## Item

Computed-style snapshot harness: tooltip timing nondeterminism and the skip link counted as an open-state item

## Status / trigger

**BACKLOG (validation 2026-10-19)**

## Size

S

## Detail

From the verify4 validation (session scratch verify4-report.md): two captures of one build differ on dark top_layer hover (196 vs 198 records, a Tooltip trigger chain rendered in 1 of 4 captures) -- tooltip timing the row 166 hover-card fix does not cover; make the harness's opener wait out every popup's configured delay. And harness coverage dropped (item hovers 486 -> 474, skipped 12 -> 123): the "Skip to content" link is now listed as an open-state item (occluded) when an overlay opens and takes hover slots; find why the skip link changes on overlay open (likely row 226's modality/focus work) and fix the harness or the page.
