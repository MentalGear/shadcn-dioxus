---
id: 10
title: "Phase 5 — collision detection"
status: "LANDED. Phase 5 is complete: 5.1 FLIP (PR #17) plus the JS-fallback shift and the size clamp…"
size: "S (flip, done) + S (JS-fallback shift…"
area: theme
updated: 2026-10-24
---

# Row 10: Phase 5 — collision detection

## Item

**Phase 5** — collision detection

## Status / trigger

**LANDED.** Phase 5 is complete: 5.1 FLIP (PR #17) plus the JS-fallback shift and the size clamp (`top-layer.spec.ts` rules 12 and 15), 5.2 `ContextMenu`'s point-anchor clamp (`use_point_anchor_clamp`, `context-menu.spec.ts`), 5.3 dropped 2026-09-18 (`dev-docs/plan.md`). Of the two optional ideas in the Detail, bits-ui's deferred nested-lock release landed as row 375 (`primitives/src/scroll_lock.rs`, batch 14); Radix's right-click carve-out for backdrop dismiss was never asked for and is not adopted.

## Size

S (flip, done) + S (JS-fallback shift, done) + S (vertical/size clamp, done) + S (5.2, ContextMenu point-anchor, done 2026-09-14)

## Detail

Keep the CSS clamp (5.3) regardless — still pending, see `dev-docs/reference/platform-rationale.md` §6. Two adoptable ideas from the comparison, not yet taken: bits-ui's debounced nested-lock cleanup (~24ms, survives same-tick unmount/remount races), and Radix's right-click carve-out for backdrop dismiss
