---
id: 178
title: "An infinite animation may only animate compositor properties"
status: "LANDED 2026-10-10"
size: "S (done)"
area: theme
updated: 2026-10-10
---

# Row 178: An infinite animation may only animate compositor properties

## Item

An `infinite` animation may only animate compositor properties

## Status / trigger

**LANDED 2026-10-10**

## Size

S (done)

## Detail

Row 174's scroll profile: the skeleton, spinner and text shimmer kept the main thread awake through every scroll (630-730 "Animation" style recalcs per scroll for the skeletons; disabling the infinite animations took 15% off the busy time). `scripts/check-infinite-animations.sh` (in `run-gates.sh`): every `*.css` under `preview/src` and `preview/assets`, an `infinite` rule's `@keyframes` may only touch `transform`, `translate`, `scale`, `rotate` and `opacity`, or the (stylesheet, keyframes) pair is in the script's `ALLOW` with a reason; a stale entry fails. The one exception is `dx-shimmer-sweep` (`background-position`): the highlight is painted inside the glyphs with `background-clip: text`, so only the background can move it; it is paint-only, opt-in, off under reduced motion and paused when unseen (row 177). Documented in `dev-docs/motion.md`.
