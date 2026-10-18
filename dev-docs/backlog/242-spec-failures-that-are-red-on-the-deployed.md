---
id: 242
title: "Spec failures that are red on the deployed baseline too (found by the 2026-10-18 lanes)"
status: "OPEN"
size: "S each"
area: process
updated: 2026-10-18
---

# Row 242: Spec failures that are red on the deployed baseline too (found by the 2026-10-18 lanes)

## Item

Spec failures that are red on the deployed baseline too (found by the 2026-10-18 lanes)

## Status / trigger

**OPEN**

## Size

S each

## Detail

Each fails identically on the 83758d5 baseline build, so none is from this round: `date-picker.spec.ts:2099/2100` natural-language "picking a day writes the formatted date" (field stays empty); `scroll-fade.spec.ts:305` MessageScroller fade after prepending history; `first-paint-styles` for `/component/data_table/`; `scroll-main-thread` test 2 (player readout). Root-cause each (debug-build timing vs real).
