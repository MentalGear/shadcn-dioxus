---
id: 177
title: "Motion nobody can see must not run: visibility gating for CSS loops, timers and entrances, by construction"
status: "LANDED 2026-10-10"
size: "M (done)"
area: theme
updated: 2026-10-10
---

# Row 177: Motion nobody can see must not run: visibility gating for CSS loops, timers and entrances, by construction

## Item

Motion nobody can see must not run: visibility gating for CSS loops, timers and entrances, by construction

## Status / trigger

**LANDED 2026-10-10**

## Size

M (done)

## Detail

The owner's brief ("animations running in the background are not about cards, but the actual components and their animations"; row 174 cause 2). `primitives/src/activity.rs`: `use_motion` / `use_motion_when` / `use_motion_active_for` (CSS loops and timers, 200px pre-roll), `use_entered_view_when` (one-shot entrances at 40% visible, no pre-roll), `use_document_visible` (tab only); one page-wide `IntersectionObserver` per kind plus `visibilitychange` and `contentvisibilityautostatechange`, reference-counted so the listener inventory returns to baseline. ONE theme rule turns `data-dx-motion="paused"` into `animation-play-state: paused`. `primitives/src/interval.rs`: `use_interval_while` and `use_timeout_while` (no task while inactive, a fresh countdown on return). Wired into spinner, skeleton, progress (indeterminate), avatar (loading), carousel autoplay, chart entrance (waits for view), toast (tab only, a timer that carries meaning), marker and attachment (shimmer hosts), the progress demo and the home page's player ticker. Gate: `scripts/check-motion-gating.sh` (a bare `use_interval` or a hand-rolled `loop`+`sleep` must be the gated form or carry `// motion-ok: <reason>`; a component stylesheet with an `infinite` animation needs `use_motion` or an `ALLOW` reason), in `run-gates.sh`. Spec: `playwright/oracle/tier2-html/motion-gating.spec.ts` (paused state and kept phase, carousel fresh countdown, zero timer callbacks off-screen, toast, chart entrance, listener inventory). Documented in `dev-docs/motion.md` and on `/docs`.
