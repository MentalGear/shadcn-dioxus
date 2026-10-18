---
id: 1
title: "Scroll-lock scrollbar-gutter fix (permanent scrollbar-gutter: stable baseline)"
status: "Landed 2026-08-31, PR #10"
size: "S"
area: process
updated: 2026-08-31
---

# Row 1: Scroll-lock scrollbar-gutter fix (permanent scrollbar-gutter: stable baseline)

## Item

Scroll-lock scrollbar-gutter fix (permanent `scrollbar-gutter: stable` baseline)

## Status / trigger

**Landed 2026-08-31, PR #10**

## Size

S

## Detail

Fixed the real 15px shift; construction proven in spike round 2, shipped as `ensure_scrollbar_gutter_baseline` (`primitives/src/scroll_lock.rs`). **Re-verified landed, 2026-09-19 (batch 3, leftovers lane):** confirmed unchanged by source inspection; no regression
