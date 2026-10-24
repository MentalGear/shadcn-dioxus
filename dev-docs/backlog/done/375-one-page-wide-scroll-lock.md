---
id: 375
title: "Scroll lock is one page-wide lock with a deferred final release, so a dialog-to-dialog handoff keeps the page still"
status: "LANDED in batch 14. primitives/src/scroll_lock.rs keeps one count, one saved position and one set…"
size: "M."
area: primitives
updated: 2026-10-24
---

# Row 375: one page-wide scroll lock

## Item

The lock count and saved scroll position were kept per nesting chain, so two sibling surfaces had two counters. When one click closed dialog A and opened dialog B, A's release after its exit animation was the last of its chain and ran `window.scrollTo` to A's saved position while B was open.

## Status / trigger

LANDED in batch 14. `primitives/src/scroll_lock.rs` keeps one count, one saved position and one set of block listeners for the page. The final release waits one frame (rAF, 50 ms fallback) and is cancelled by a lock taken meanwhile, so a surface that releases before the next one locks in the same click keeps the listeners and the saved position. The modal-closed release reads the `active` memo with `try_read`, since `try_peek` can return a dirty memo's stale value. `playwright/oracle/tier3-radix/scroll-lock.spec.ts` carries the handoff case; the top-layer handoff fixture composes the themed Dialog.

## Size

M.

## Detail

See the Status section and the batch 14 commits of the lane.
