---
id: 344
title: "Charts flash the final drawing before their entry animation on first client-side open"
status: "LANDED 2026-10-21 (claude/first-pose): a client-created chart renders data-animate=\"pending\" (the…"
size: "S"
area: charts
updated: 2026-10-21
---

# Row 344: Charts flash the final drawing before their entry animation on first client-side open

## Item

The first time any chart tab is opened (e.g. https://mentalgear.github.io/shadcn-dioxus/charts/area/), the full chart shows for a moment, disappears, and then animates in. Later opens do not flash.

## Status / trigger

LANDED 2026-10-21 (claude/first-pose): a client-created chart renders `data-animate="pending"` (the entrance's start pose) from its first frame; `entrance_pose` in chart.rs, pending rules in chart/style.css; playwright/first-pose.spec.ts red before (first frame `none` = final drawing, 6 kinds), green after.

## Size

S

## Detail

- **Related:** row 211 (done) fixed the same visible symptom for SSG-painted charts on hydration. Above-the-fold charts that are already painted no longer animate, below-the-fold ones animate once on entering view (`use_motion`), and no-JS or reduced motion shows the final drawing. The trigger lives in `primitives/src/chart/components/chart.rs`.
- **Likely cause (inferred, not yet reproduced):** opening a tab is a client-side mount, not a hydration. The chart renders its final geometry in the first frame, and only afterwards does an effect set the animate state, which resets the chart to its start pose and plays the entry. So one frame of the final drawing paints first. "First time only" fits this: once the tab's chart has entered view, it doesn't replay.
- **Fix by construction:** decide the initial pose during render, not in an effect after paint. A chart that mounts on the client after hydration and will animate renders its start pose in the first frame. A server-rendered or already-painted chart renders its final pose as row 211 requires. Add a spec that samples the chart's first painted frames after a client-side tab switch and fails if any frame shows the final pose before the entry starts. It should cover every chart type on the charts page, not only area.
- **Check:** whether other entry-animated components share the effect-after-paint pattern, which would make this a class, not a chart-only bug.
