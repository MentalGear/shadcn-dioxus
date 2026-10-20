---
id: 255
title: "Two specs drifted from the product and were red on main: the toast countdown shim and the drawer drag viewport"
status: "LANDED 2026-10-20 (lane/triageA): both red on origin/main 4a8847b0 and on the earlier verify6 run…"
size: "S"
area: playwright
updated: 2026-10-20
---

# Row 255: two specs drifted from the product (found by the batch-A release suite, red on main too)

## Item

The release suite on the batch-A merge (3fd7844b) had two reds that batch A did not cause. Both reproduce on a debug SSG of `origin/main` 4a8847b0 and were already in the earlier `verify6` report (`KNOWN`).

## Status / trigger

LANDED 2026-10-20 (lane/triageA): both red on origin/main 4a8847b0 and on the earlier verify6 run of f62953ad, so pre-existing, not batch A; both specs corrected

## Size

S

## Detail

**Detail.**

1. `oracle/tier2-html/main-thread.spec.ts` "drawer (drag): primary interaction is responsive" threw `measureAtRest: the element has no box`. Since `0579af56` (2026-10-14, one edge-panel recipe) the demo drawer takes `EdgeSide::Auto`: the bottom below the `md` breakpoint, a floating right panel at 768 px and up, and `.dx-drawer-handle` is `display: none` on a side panel. The spec ran at the default 1280x720, so there was no handle to grab. `drawer.spec.ts` was moved to a phone viewport in that commit; this subject was missed. Fix: the subject's `goto` sets 390x844 first (the bottom drawer is the gesture under test). Release build, 3 of 3 green.
2. `oracle/tier2-html/motion-gating.spec.ts` "toast auto-dismiss ignores scrolling but not a hidden tab" waited for the toast to vanish. The test sped a 60 s toast up by wrapping `setTimeout(60000)`, but since rows 262/263 the countdown is `ToastTimer`, a 50 ms visibility- and pause-gated tick (`use_interval_while`), so the wrapper touched nothing and the toast lived its full 60 s. Fix: use the `pause` variant (a real 3 s toast) and park the pointer and blur the trigger so the hover/focus pause does not hold it. Debug SSG of main and of the batch, 3 of 3 green each.

**Class.**

A spec that fakes time or viewport for a component has to be re-derived when the component's rule changes; no gate sees it, only a full run does. Not a construction candidate beyond "read the demo's own contract": both specs encoded a fact (the demo's side, the toast's timer) that a one-line probe would have shown.
