---
id: 90
title: "Chart stage 2 — per-chart-type galleries (shadcn parity), parked mid-flight (LANDED 2026-09-25)"
status: "LANDED 2026-09-25 (round 8) -- all seven parked lanes integrated from the pushed worktree-agent-*…"
size: "L (done 2026-09-25) -- was…"
area: theme
updated: 2026-09-25
---

# Row 90: Chart stage 2 — per-chart-type galleries (shadcn parity), parked mid-flight (LANDED 2026-09-25)

## Item

Chart stage 2 -- per-chart-type galleries (shadcn parity), parked mid-flight (**LANDED 2026-09-25**)

## Status / trigger

**LANDED 2026-09-25 (round 8)** -- all seven parked lanes integrated from the pushed `worktree-agent-*` branches; see this row's own dated addendum below. Original trigger (met): **The user's go-ahead to resume stage 2** (parked 2026-09-20 after a usage-limit interruption and a one-lane-at-a-time process change, `dev-docs/issues/chart-stage2-handoff.md` (deleted by row 266; `git show 3c584cc2:dev-docs/issues/chart-stage2-handoff.md`)); also actionable now, independent of a full resume, for whoever next touches `primitives/src/chart/**`, since that handoff's ownership map and open constructions bind regardless.

## Size

L (done 2026-09-25) -- was: `s2-refactor`/`s2-area` already done; `s2-tooltip`/`s2-radar` S each (mechanical finish, drop-in, verify); `s2-line` S–M; `s2-bar` M; `s2-polar` M–L (`RadialBar` unstarted beyond a stub); the five constructions below and the first-ever live verification of five of the seven lanes add roughly another M on top

## Detail

**What it is.** shadcn ships one gallery page per chart type (70 demos across area, bar, line, pie, radar, radial, tooltip;
`dev-docs/archive/research/chart-2026-09-19.md` section 1.2); stage 1 (row 88) shipped one `/component/chart/` page. The user asked
for one gallery package per type (`area_chart`, `bar_chart`, `line_chart`, `pie_chart` plus `radial_chart`, `radar_chart`,
`chart_tooltip`), dispatched 2026-09-19 as seven parallel lanes (a structural refactor lane plus six family lanes) off stage-1
HEAD `9a6cb0c`. The lanes were parked 2026-09-20 mid-flight after a usage-limit interruption and a one-lane-at-a-time process
change; the handoff note with the ownership map and the preservation commands was
`dev-docs/issues/chart-stage2-handoff.md` (deleted by row 266; `git show 3c584cc2:dev-docs/issues/chart-stage2-handoff.md`).

**Landed 2026-09-25 (round 8, `2d77f2b`..`6a9f936` plus the cross-lane construction commit `b6806f5`).** All seven parked lanes
were integrated onto one tree; the galleries match shadcn's inventory exactly: Area 10/10, Bar 10/10, Line 10/10, Tooltip 9/9,
Radar 14/14, Pie 11/11, Radial 6/6 (70 demos). The five cross-lane constructions that had blocked a clean finish:

1. A struct-update-spread fix so `ChartTooltip`/`ChartLegend`/`ChartContainer` stop silently dropping new props (the class
   stage 1's `Chart` wrapper already had, row 88).
2. `ChartKind::has_full_table()`, decoupling the hidden table's full-vs-reduced choice from `is_cartesian()` so Radar gets its
   categories-by-series table.
3. A `StackMode`-aware `components::layout::build`, so a percent-stacked chart's shared grid, axis and tooltip-anchor y-domain
   agree with its marks.
4. A family-agnostic `ChartLayout::anchor_percent`, so `ChartTooltip` positions correctly for Radar's vertex geometry instead of
   falling back to dead-centre.
5. Both pending CSS blocks (tooltip indicator/icon/total-row from `s2-tooltip`, radar grid rings from `s2-radar`) reconciled into
   `chart/style.css`, with a new polar block, against `03927e3`'s tooltip/legend grid-layout rewrite.

`RadialBar`'s primitive was built out from the empty stub (`RadialOptions{inner_radius, outer_radius, start_angle, end_angle,
corner_radius, grid, labels, stacked, center_text}`); single-series Pie/RadialBar legends key per datum, as shadcn does.

**Evidence.** 207/207 chart-specific tests on the integration tree; all gates green (445 primitives + 38 preview tests, 160
doctests). Browser and SSG validation (round 8b, on `972b32e`): chart-only run 266 tests / 262 passed / 4 failed, full suite 1252
/ 1200 passed / 34 failed / 18 skipped, 30 of the 34 in the settled baseline; the other 4 (`pie_chart.spec.ts:77,116,186`,
`radial_chart.spec.ts:47`) were a test race (a non-retrying `.count()` right after a `load`-event `goto`), fixed in `f468f78` with
`waitUntil: "networkidle"` and verified 102/102 with `--repeat-each=2` (the class of row 109). The release SSG build had 80
prerendered pages (+7 galleries), zero `/dioxus-components/` references, `hydration-parity.spec.ts` 40/41 (only the red-by-design
`selftest:unfixed:id`), and zero duplicated attributes across all 80 pages.

**Follow-on 2026-10-03:** the galleries were re-based on shadcn's real rendered output: display and docs UX (row 117), the
`/charts` gallery and pointer-following tooltip (row 118), Recharts projection parity (row 119).

*Trimmed in row 266 (repo cleanup). The full dated narrative of this row is in git: `git show 3c584cc2:dev-docs/backlog/done/090-chart-stage-2-per-chart-type-galleries-shadcn.md`.*
