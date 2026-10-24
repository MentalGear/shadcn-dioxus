---
id: 118
title: "shadcn-style /charts/<kind>/ gallery and a pointer-following tooltip (PR #80)"
status: "LANDED 2026-10-03 -- MentalGear/dioxus-components#80 (merged 437e17b). Owner request: model the…"
size: "L"
area: charts
updated: 2026-10-03
---

# Row 118: shadcn-style /charts/<kind>/ gallery and a pointer-following tooltip (PR #80)

## Item

shadcn-style `/charts/<kind>/` gallery and a pointer-following tooltip (PR #80)

## Status / trigger

**LANDED 2026-10-03 -- MentalGear/dioxus-components#80 (merged `437e17b`).** Owner request: model the charts pages on ui.shadcn.com/charts, tooltip included

## Size

L

## Detail

Owner request: model the charts pages on ui.shadcn.com/charts, tooltip included. Landed 2026-10-03 as
MentalGear/dioxus-components#80 (merged `437e17b`). The measured spec was taken first from the live site through curl-routed
requests (Chromium cannot load ui.shadcn.com through the sandbox proxy); it was `dev-docs/archive/research/chart-parity-2026-10-03/`'s
`shadcn-ux-spec.md` (deleted by row 266; `git show 3c584cc2:dev-docs/archive/research/chart-parity-2026-10-03/shadcn-ux-spec.md`),
and the result is carried by this row and row 119.

**Tooltip.** The old one snapped to a fixed anchor per category, pinned polar tooltips at the centre, was not clamped to the card
and stayed closed on a touch tap. Now: area/bar/line x snaps to the nearest category (switching at the midpoint), y follows the
pointer, the box sits +10 px right and below, flips left or above on overflow and clamps inside the chart (pure `place_tooltip`);
horizontal bars swap axes; pie anchors at the slice centroid; radar and radial follow the pointer (polar hover resolves from pointer
coordinates against resting geometry, because arc `pointerenter` blinked when the active arc's scale moved it under the pointer);
200 ms ease-out between positions, no fly-in on (re)open, off under reduced motion; touch taps open and stay, drags scrub, blur or
tap-away closes; keyboard arrows hang the tooltip off the data point and keyboard focus (not pointer focus) opens the first point
like Recharts' `accessibilityLayer`; `default_index` renders it open until the user interacts (used by every Tooltips demo); an
active dot on area and line; a `PointerGate` epoch stops a late pointer result from reopening a closed tooltip (synthetic
move-plus-leave stuck open 30/30 before); correct placement under scaled ancestors. `fit_width` (later superseded by row 119's
`height` prop) gave shadcn's `h-[250px] w-full` hero.

**Gallery.** `/charts/` and `/charts/<kind>/` (area, bar, line, pie, radar, radial, tooltip), all prerendered: a hero, a text tab row
(`aria-current`, scrolls on phones, keeps its offset and focuses the active tab on switch, per-tab titles), a 1/2/3-column grid, and
shadcn's card toolbar (type icon and label, **Copy** the full source, **View Code** in a 700 px `Sheet`, a `Drawer` on phones). One
footer construction for every demo (`CardFooter.dx-chart-footer`; there were five divergent patterns). Also fixed: `area_chart`
links its own stylesheet, line interactive's ticks read "Apr Apr Apr" because its stylesheet was never linked, bar interactive
totals were black on the dark card (bare `<button>` UA colour).

**Three fixes that fell out of it, each by construction.**

1. *A client id collision broke the command palette.* `use_unique_id` allocated `dxc-{N}` from one process-global counter shared
   across concurrently rendered SSG pages while a client-mounted component counted from 0 in the same namespace; the home page
   shipped two `id="dxc-229"` and `getElementById` returned the checkbox (`command.spec.ts:114`, deterministic on that build). Client
   allocations now use a disjoint `dxc-c{N}` namespace, server ids unchanged, hydrated components still take the server's exact id
   through `use_server_cached`.
2. *Scroll-lock horizontal gestures.* The modal scroll lock looked for a scrollable ancestor on the vertical axis only, so a
   horizontal swipe on a wide code block in a bottom drawer was `preventDefault`ed; the lookup is axis-aware (dominant of dx/dy,
   RTL-safe `scrollLeft`).
3. *Copied demo code compiles in a user's app.* Every chart kind imported `crate::components::chart::component::*`, but after
   `dx components add` `component` is a private module, so installed charts failed to compile; `examples!` now declares it
   private like an installed tree and imports use `crate::components::<name>::*`, proven with a throwaway crate in the installed
   layout. Copy, View Code and the CODE tabs show the installed import through one span-remapping transform. New gate
   `scripts/check-installed-paths.sh` blocks the `...::component::*` pattern.

**Evidence.** `charts_gallery.spec.ts` (19 tests including clipboard, sheet and drawer focus return, no overflow 1440-320, three axe
scans), `chart_tooltip_follow.spec.ts` (22 browser tests) plus placement-math unit tests against shadcn's measured offsets; 249
chart and gallery specs on an SSG build, 361/362 targeted specs on the deploy build (the one failure, `navigation_menu.spec.ts:128`,
is row 121); deploy 88/88 routes carried their bootstrap script.

**Deliberate differences from shadcn:** gallery cards are 0.95x shadcn's width at 1440 (the site container is 1280 px against
1336 px); no "Open in v0" action; tooltip motion is 200 ms ease-out (shadcn 400 ms) and never flies in from (0,0); the palette (row
119). **Left open:** rows 124-126 (pre-hydration jumps, small visual nits).

*Trimmed in row 266 (repo cleanup). The full dated narrative of this row is in git: `git show 3c584cc2:dev-docs/backlog/done/118-shadcn-style-charts-kind-gallery-and-a-pointer.md`.*
