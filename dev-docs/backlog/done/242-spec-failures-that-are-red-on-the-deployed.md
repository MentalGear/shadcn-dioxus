---
id: 242
title: "Spec failures that are red on the deployed baseline too (found by the 2026-10-18 lanes)"
status: "FIXED 2026-10-19 (lane baseline-red a9564042): three stale specs -- date-picker natural-language…"
size: "S each"
area: process
updated: 2026-10-19
---

# Row 242: Spec failures that are red on the deployed baseline too (found by the 2026-10-18 lanes)

## Item

Spec failures that are red on the deployed baseline too (found by the 2026-10-18 lanes)

## Status / trigger

**FIXED 2026-10-19 (lane baseline-red a9564042): three stale specs -- date-picker natural-language now clicks a day that is not the pre-selected one; scroll-main-thread test 2 scrolls the motion-gated player into view; scroll-fade prepend waits for more rows, not scrollHeight -- and one product shift: the data_table page-size Select trigger is static text (CLS 0.0131 gone; the primitive-level class is row 248); all green on a lane build, first-paint-styles green on all routes**

## Size

S each

## Detail

Each fails identically on the 83758d5 baseline build, so none is from this round: `date-picker.spec.ts:2099/2100` natural-language "picking a day writes the formatted date" (field stays empty); `scroll-fade.spec.ts:305` MessageScroller fade after prepending history; `first-paint-styles` for `/component/data_table/`; `scroll-main-thread` test 2 (player readout). Root-cause each (debug-build timing vs real).
