---
id: 470
title: "Bug: the carousel autoplay tick fights a user swipe (the scroller is pulled forward and bounces back)"
status: "BACKLOG. Found by board round 5; reproduce first. Independent of plan approval."
size: "S-M"
area: components
updated: 2026-10-26
---

# Row 470: Bug: the carousel autoplay tick fights a user swipe (the scroller is pulled forward and bounces back)

## Item

Native swipes are deliberately not an "interaction" (`carousel/context.rs:255-270`), and the autoplay tick computes `step_next(selected.peek())` from the last settled index (`carousel/autoplay.rs:166-188`). A swipe from slide 2 back to 1 queues its settle fact while the tick targets 3, so the scroller is pulled forward and then bounces back to 1; on web the tick can fire mid-touch and fight the compositor's scroll.

**Where:** mobile and desktop (`dioxus-desktop`), and web.

**Reproduce first:** a carousel with autoplay on; start a touch swipe backwards just before a tick; watch the position and the selected index. On the desktop-cadence build the settle fact is delayed.

**Source:** [`board-2026-10-26-r5/order-runtime.md`](../research/board-2026-10-26-r5/order-runtime.md) scenario E (finding 5).

Related to [row 434](./434-javascript-in-typescript-modules-not-rust-strings.md) but not part of it: the plan's construct subsumes this defect later, and the fix does not wait for the plan. Source: the last section of [`board-2026-10-26-r5/SYNTHESIS.md`](../research/board-2026-10-26-r5/SYNTHESIS.md) (live defects the board found in today's code).

## Status / trigger

BACKLOG. Found by board round 5; reproduce first. Independent of plan approval.

## Size

S-M

## Detail

- **Files owned:** To be found by the reproduction.
- **Depends on:** None. The plan's `scrollOwner` (row 457: a native swipe is a transition the owner classifies by the realized default action, with the snap scroller's intent kind) subsumes it; a local fix is to pause the tick on `touchstart` and `pointerdown` over the scroller.
- **Proof:** A Playwright spec that fails before the fix and passes after; run on the web build and, where the bug is renderer-dependent, the desktop-cadence build of [row 442](./442-desktop-cadence-harness-and-desktop-pass.md).
- **Hours (inferred):** not estimated (size guess S-M).
