---
id: 174
title: "Home/overview page scroll jank, re-profiled 2026-10-10"
status: "PROFILED (partial, stopped by the owner) -- 2026-10-10; causes 2 and 4 LANDED 2026-10-10 (rows 177…"
size: "S to M per fix (order below)"
area: misc
updated: 2026-10-10
---

# Row 174: Home/overview page scroll jank, re-profiled 2026-10-10

## Item

**Home/overview page scroll jank, re-profiled 2026-10-10**

## Status / trigger

**PROFILED (partial, stopped by the owner)** -- 2026-10-10; **causes 2 and 4 LANDED 2026-10-10 (rows 177, 178, 183), cause 3 PARTLY (entrances wait until seen), causes 1 and 5 still open**; the rest scheduled in **next-session step 1**

## Size

S to M per fix (order below)

## Detail

The owner reported scroll jank on the home/overview page again (rows 116 and 142-143 hold the earlier scroll and main-thread work). Full report: `scroll-profile-2026-10-10.md` (session scratchpad). **Numbers.** At 1x the page is smooth (0 frames >20 ms, rAF p99 16.8 ms) and the main thread is busy 2.3-2.8 s per full scroll, the same as after round 2 (2.1 s); the first scroll is heavier (3.0-3.5 s at 1x). At 4x CPU throttle it is main-thread bound: ~20% of frames >20 ms, input latency p95 90-125 ms. `content-visibility` is a net win (6.7 s busy without it). **Ranked causes.** (1) A per-vsync main frame (~3 ms) remains with all animations off (1.9 s); not isolated. (2) Infinite skeleton/spinner/shimmer animations keep running inside skipped `content-visibility` cards (`skeleton/style.css:3`, `spinner/style.css:11`, `dx-effects.css:430`); disabling them gives -15% busy and 782 -> 429 style recalcs. Fix: pause them in skipped cards via `contentvisibilityautostatechange` -> a data attribute -> `animation-play-state: paused` (S-M). (3) Chart entrance animations replay on first reveal (`chart/style.css:279-321,716-726,826-842`; `chart.rs:616-720`); cold layouts 389 -> 162 without them. Fix: no entrance on the home gallery, or transform/opacity only (S-M). (4) `contain-intrinsic-block-size` ignores the 3rem card padding (`main.css:1616-1625`): the estimate is 476 px against a median of ~256 px, so the page shrinks 38.5k -> 32k px during the first scroll and the scrollbar jumps. Fix: ~284 px content-box estimates (S). (5) Per-card first-reveal work (ResizeObserver + wasm chart re-render, 1 Hz tickers): 480-800 ms at 4x (M). **Suggested order:** 4, then 2, then 3, then 5, then 1. **Not measured:** post-scroll idle, the synthetic baseline, real GPU, 4x cold rAF. Timing numbers count only on a release SSG build (CLAUDE.md). **UPDATE 2026-10-10.** Cause 2 (infinite animations running inside skipped cards) is fixed by construction, not per instance: `primitives/src/activity.rs` answers "is this visible" once per page, one theme rule pauses every loop, timers are `use_interval_while`, and `scripts/check-motion-gating.sh` keeps the ungated shapes out (row 177); the infinite animations are also compositor-only now, enforced by `scripts/check-infinite-animations.sh` (row 178). Cause 3 is half closed: a chart's load animation waits until at least 40% of it is on screen (`use_entered_view_when`, no pre-roll), so a card nobody reached does not animate; it still plays once on reveal. Cause 4 (the card estimate) is landed (row 183). Open: cause 1 (the per-vsync frame with all animations off) and cause 5 (per-card first-reveal work). The motion model is documented in `dev-docs/motion.md` and on `/docs`.
