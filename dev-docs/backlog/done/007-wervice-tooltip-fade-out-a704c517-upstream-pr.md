---
id: 7
title: "Wervice tooltip fade-out (a704c517, upstream PR #293)"
status: "LANDED 2026-09-18. Cherry-picked a704c517 (Wervice, upstream PR #293) via git cherry-pick -x…"
size: "S"
area: overlays
updated: 2026-09-18
---

# Row 7: Wervice tooltip fade-out (a704c517, upstream PR #293)

## Item

Wervice tooltip fade-out (`a704c517`, upstream PR #293)

## Status / trigger

**LANDED 2026-09-18.** Cherry-picked `a704c517` (Wervice, upstream PR #293) via `git cherry-pick -x`, author preserved, no path conflicts — commit `6a5130b` (adds a `dx-tooltip-fade-out` keyframes + `opacity: 0`/`pointer-events: none` in place of `display: none` on Tooltip's closed state, so `animation` can apply). Two adaptations on top of the original diff: the literal `0.2s` duration replaced with the `--dx-motion-duration-slow` token (`scripts/check-css-literals.sh` flags a hard-coded value a token already names; that token is exactly 200ms, matching the same selector's existing fade-in), and a `prefers-reduced-motion` override added, matching the per-component convention `input_otp`/`spinner` already carry (row 31a's shared reduced-motion layer covers interactive roots only, not animated content). Follow-up commit `aad4694` gave HoverCard and Popover the same symmetric close fade: HoverCard had none at all (`display: none` on `[data-state="closed"]`, the same defect the cherry-pick fixed for Tooltip — cannot coexist with `animation`), so it got a `dx-hover-card-fade-out` keyframes built from scratch, using its own existing fade-in's duration/easing tokens for self-symmetry rather than copying Tooltip's token choice; Popover already had a symmetric fade both ways and needed no CSS change at all — only row 19's hook swap to make it observable. Both new stylesheets also gained the `prefers-reduced-motion` override. **Dependency honored:** this CSS alone is invisible on the web arm until row 19 lands (the old `use_popover_sync` called `hidePopover()` before any of these animations got a frame to play) — sequenced 19 → 7 in the same lane per the 2026-09-18 queue evaluation's correction 3. Regression test: `assert-fade-out.ts` + the three component specs (see row 19); demonstrated RED on pre-fix `tooltip.rs`, GREEN restored, integration round 1.

## Size

S

## Detail

Provenance ledger entry: `lifting-from-forks.md` §8, 2026-09-18. `Wervice`'s own fork branch no longer carries `a704c517` — fetched from upstream PR #293's head (`refs/pull/293/head`), per row 66's fork re-check
