---
id: 305
title: "Side drawer: scrolling its content also drags the drawer sideways (iOS)"
status: "WAITING ON OWNER: iPhone re-test of the side drawer (scroll its content, the drawer must not drag…"
size: "S–M"
area: primitives
updated: 2026-10-24
---

# Row 305: Side drawer: scrolling its content also drags the drawer sideways (iOS)

## Item

On iOS, scrolling the content of a left or right drawer also moves the drawer sideways. A vertical scroll and a horizontal dismiss drag aren't told apart.

## Status / trigger

**WAITING ON OWNER: iPhone re-test of the side drawer (scroll its content, the drawer must not drag sideways), together with rows 28 and 30.** Implemented 2026-10-20 (lane b-accdrawer): the drag is pending until the pointer commits to the drag axis (`GESTURE_JS`, `primitives/src/drawer.rs:549`), spec `playwright/drawer-axis-lock.spec.ts`.

## Size

S–M

## Detail

**Cause (inferred from code at main, not reproduced: the sandbox has no WebKit):**
- `GESTURE_JS` in `primitives/src/drawer.rs` (~515-600) starts a drag on `pointerdown` and moves the panel by the raw drag-axis delta on every `pointermove`.
- There is no direction lock. A vertical scroll's small sideways jitter translates a side drawer until the browser claims the pan and sends `pointercancel`.
- The scroll guard (rule 4, ~565-580) only looks for a scrollable ancestor along the drag axis (`overflowX` for a side drawer). So a vertically scrolling list inside a side drawer never blocks the drag.
- `touch-action: pan-y` on side drawers (`drawer/style.css:188,193`) lets the browser scroll, but WebKit still delivers the early pointermoves.

**Target, as in Vaul and Base UI:**
- Make the drag pending on `pointerdown`.
- Commit to dragging only once movement passes a few px *and* the drag-axis delta beats the cross-axis delta.
- If the cross axis wins first, abandon the drag for that pointer so the content just scrolls.
- Don't send `start`/`move` before committing, so the panel never shifts during a scroll.
- Keep rule 4 for the same-axis case.

**Specs:**
- A vertical touch scroll in a side drawer leaves its transform untouched.
- A horizontal swipe still dismisses.
- A diagonal that starts vertical never drags.
- The same in a bottom drawer with a horizontally scrolling row (the mirror case).

Re-check on an iOS device, alongside rows 28/30.

**Files:** `primitives/src/drawer.rs` (`GESTURE_JS`), the drawer specs and `docs.md`. Row 73 (drag hang) touched the same listener.

## Resolution (lane b-accdrawer, 2026-10-20)
Implemented; see the backlog.md row and the new Playwright spec named there.
