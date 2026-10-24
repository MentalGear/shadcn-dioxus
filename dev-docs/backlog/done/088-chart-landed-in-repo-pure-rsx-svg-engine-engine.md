---
id: 88
title: "Chart landed — in-repo pure-RSX SVG engine, engine/layer seam for upstreaming, two integration a11y/visual…"
status: "Landed 2026-09-19 (Chart round; dev-docs/backlog.md row 86's decision made) RESOLVED by the owner…"
size: "L (M engine + S tooltip/legend, MVP…"
area: charts
updated: 2026-10-03
---

# Row 88: Chart landed — in-repo pure-RSX SVG engine, engine/layer seam for upstreaming, two integration a11y/visual…

## Item

Chart landed — in-repo pure-RSX SVG engine, engine/layer seam for upstreaming, two integration a11y/visual fixes

## Status / trigger

Landed 2026-09-19 (Chart round; `dev-docs/backlog.md` row 86's decision made) **RESOLVED by the owner 2026-10-03 (decision closed; history above kept):** the open follow-up this row carried -- upstreaming the engine to `dioxus-community/dioxus-charts` (`plan.md` Open decisions item 6) -- is no longer pending. The owner's call: this repo has its own charts now, built in-repo (rows 90, 117, 118, 119 -- galleries, pointer-following tooltip, Recharts/shadcn projection parity), so nothing is to be offered upstream and no further action is owned by this row

## Size

L (M engine + S tooltip/legend, MVP scope)

## Detail

The chart engine, landed 2026-09-19 (Chart round, after the row 86 decision). Owner decision: build it in-repo, shadcn-shaped
(`ChartConfig` to `--color-<key>` CSS variables, data-driven props, no children-as-configuration, no new Cargo dependency) rather
than adopt `charts-rs`, `dioxus-charts` or a JS-engine wrapper (alternatives table:
`dev-docs/archive/rounds/component-backlog.md`, Chart row). The research is `dev-docs/archive/research/chart-2026-09-19.md`; the
fork and d3 survey is `dev-docs/archive/fork-research/chart-forks-2026-09-19.md`.

- **Engine/layer seam.** `primitives/src/chart/engine/{scale,curve,stack,geometry,table,data}.rs` is pure Rust with no `dioxus`
  types and no `use crate::` of other modules (an upstream-shaped seam); `context.rs` (`ChartContext`/`use_chart`) bridges it to
  reactivity and `components/{container,chart,tooltip,legend}.rs` are the Dioxus-facing pieces.
- **d3-derived math, ported not copied (ISC attribution).** `LinearScale::ticks`/`nice_domain` follow d3-array's `tickSpec`
  arithmetic; the `Monotone` curve follows d3-shape's Steffen tangent formula, cited by file and SHA in the module docs. The
  tangent clamps against both neighbouring slopes (a three-way min); `leptos-chartistry`'s port of the same paper drops one term
  and can overshoot near an extremum, so this repo ports from the canonical JS source, not a sibling library's Rust.
- **A11y contract** (the survey found no accessibility semantics at all in every non-Recharts-lineage chart library): `svg[role="img"]`
  with a required `aria-label` and `<title>`/`<desc>`; a visually hidden `<table>` mirroring series, category and value as the
  keyboard- and screen-reader path; legend swatches `role="graphics-symbol"`; optional arrow/Home/End/Escape stepping of the
  active index (Recharts' `accessibilityLayer` as the tier-3 opinion), never `role="application"`. Oracle:
  `playwright/oracle/tier3-radix/chart.spec.ts` (folder name kept; Chart has no Radix primitive to calibrate against) plus
  `playwright/chart.spec.ts`, 13/13.
- **`<title>` namespace note.** `dioxus-html` 0.7.9 has no SVG-namespaced `<title>`, so a CSR-only page load builds it without the
  namespace; the deployed site (SSR plus hydrate) is unaffected because the HTML parser handles `<title>` in `<svg>` source text
  per the foreign-content rules. Documented in `Chart`'s module doc, not fixable inside a primitive.
- **Three integration fixes found live, not by compilation.** (1) The hidden table's empty corner `<th>` failed axe's
  `empty-table-header`: new `x_label` prop (default "Category"). (2) The 90-datum `main` demo drew 90 overlapping x ticks: new
  `max_x_ticks` prop (default 12) with the pure `x_tick_step` stride (count-based thinning; a width-aware version is the stage-2
  follow-up). (3) The themed wrapper rebuilt its primitive call field by field and silently dropped `x_label`/`max_x_ticks`;
  the forwarding list was fixed.
- **Out of scope, deliberately.** Pie/Radar/Radial (later lanes; the engine split leaves room for a `polar.rs`), tunable margins
  (fixed MVP constants), and memoization: the scale/tick/geometry pipeline is unmemoized and every hover-driven cursor move
  reruns it, a not-yet-measured performance question.
- **Evidence.** All gates plus rustdoc green; 260 primitives tests (75 chart), 31 preview, 142 doctests; SSG release build of 72
  routes with `hydration-parity.spec.ts` 11/11; full suite 910 passed / 7 failed / 12 skipped against the batch-3 baseline: 1
  fixed flake, 5 known reds (rows 21, 45), 2 new and explained load effects.

The follow-up this row carried (upstreaming the engine to `dioxus-community/dioxus-charts`) was resolved by the owner 2026-10-03:
the repo's charts are built in-repo (rows 90, 117, 118, 119), nothing is offered upstream.

*Trimmed in row 266 (repo cleanup). The full dated narrative of this row is in git: `git show 3c584cc2:dev-docs/backlog/done/088-chart-landed-in-repo-pure-rsx-svg-engine-engine.md`.*
