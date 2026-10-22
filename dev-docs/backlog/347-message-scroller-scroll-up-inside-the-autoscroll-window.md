---
id: 347
title: "Message scroller: a scroll up inside the 180 ms autoscroll window leaves follow mode armed"
status: "CODE DONE, NOT YET BROWSER-VERIFIED (lane scroller): lastScrollTop only advances downward inside…"
size: "S (a primitive TS change plus the…"
area: primitives
updated: 2026-10-21
---

# Row 347: Message scroller: a scroll up inside the 180 ms autoscroll window leaves follow mode armed

## Item

`primitives/src/ts/message_scroller.ts` (`reconcileFollowMode`) releases `following-bottom` only when `state.end && scrolledUp && !autoscrolling`, and it updates `lastScrollTop` on every commit. A programmatic or scrollbar-drag scroll to the top that lands while `autoscrolling` is true (the 180 ms `AUTOSCROLL_CLEAR_MS` window after every pin, including the opening pin) is therefore consumed: `lastScrollTop` becomes 0, the follow mode stays armed, and the timer's own commit sees no further upward move. The viewport sits at the top, `publish` forces `end: false` while following, so the end button stays `data-active="false"` until the next scroll.

## Status / trigger

CODE DONE, NOT YET BROWSER-VERIFIED (lane scroller): lastScrollTop only advances downward inside the autoscroll window; spec added in message-scroller-api.spec.ts; red/green runs pending (disk)

## Size

S (a primitive TS change plus the bundle rebuild).

## Detail

### Evidence

- `message-scroller-api.spec.ts` "RTL > the jump button stays centred in both directions" failed 5/5 at `--repeat-each=5` on main `16a20ad1` as well as on the batch-2 merge `38ce0b4c` (so not a batch-2 regression): the opening pin removes `data-pending-scroll` and starts the autoscroll window in the same frame, and the spec's `scrollTop = 0` ran 20-100 ms later. Probe: `scrollTop = 0` after 1.5 s gives `data-active="true"`; right after the pin it stays `false` for 3 s.
- The spec now waits for `data-scrollable` and for `data-autoscrolling` to clear. The product edge is unchanged.

### Fix

Candidate fix by construction: do not advance `lastScrollTop` while `autoscrolling`, so the commit that the autoscroll timer runs sees the upward move and releases follow. Needs a check that a smooth streaming pin does not re-arm or release on its own, (correction: `primitives/src/message_scroller.js` is hand-written, not generated, so there is no bundle to rebuild; the fix keeps a high-water baseline instead of freezing it, because freezing would hide an upward move that starts below the old baseline). A real reader cannot scroll within 180 ms of a pin, so the user-visible risk is small.
