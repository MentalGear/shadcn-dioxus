---
id: 421
title: "Toast: a new toast's first measurement never plays as a glide (collapsed pile jumps on insert, Firefox)"
status: "LANDED (lane/toast-insert-jump, for the merge queue). Trigger for reopening: a height or margin-top…"
size: "S"
area: components
updated: 2026-10-25
---

# Row 421: Toast: a new toast's first measurement never plays as a glide

## Item

Owner, 2026-10-25: when new toasts are added, the collapsed stack jumps (Firefox; screenshot and screen recording in the project thread).

## Status / trigger

LANDED (lane/toast-insert-jump, for the merge queue). Trigger for reopening: a height or margin-top transition on a toast's first measurement in any engine.

## Size

S

## Detail

Measured from the owner's recording: on every insert the whole pile jumps up by about one toast height (~78 px) in one frame, then glides back down over 200 ms (`--dx-motion-duration-slow`).

Cause: `TOAST_SIZE_JS` measures a new toast in the mutation callback and wrote `data-sized="pending"`, then `true` in a `requestAnimationFrame`; the stylesheet transitions a toast's height and margin only under `data-sized="true"`. rAF callbacks run before the frame's style recalculation, so unless something else recalculated style in between, the measured `--toast-height` and `data-sized="true"` reached the toast in the same style change and the switch from the unmeasured layout (auto height, margin 0, the toast taking its full height in the pile) to the measured one (margin `-height`) played as a transition. Firefox 136 runs `margin-top: 0px -> -77px` over 200 ms on every new toast of the main demo (debug and release builds); Chromium runs none, which is why the Chromium suite never saw it.

Fix by construction: the first measurement commits the measured height to the toast's style with the transitions still off (a computed-style read), then sets `data-sized="true"`; the two can no longer share a style change in any engine, and no frame timing is involved. After the fix Firefox runs no height or margin-top transition on a new toast. `toast-stack-motion.spec.ts` "collapsed, a newly added toast: its first measurement never plays as a glide" watches `transitionrun` (no layout reads, which would hide the case); it guards the Firefox project.
