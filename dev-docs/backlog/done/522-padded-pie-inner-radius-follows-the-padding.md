---
id: 522
title: "A padded pie with no inner radius ends its slices in tips at the centre: the inner circle has to grow with the padding to read as even gaps"
status: "LANDED 2026-11-15 in queue/batch-6 (the batch after 5ebf4803); row 519 without Toast (no placement…"
size: "S."
area: components
updated: 2026-11-15
---

# Row 522: a padded pie's inner radius follows the padding

## Item

The batch of 2026-11-14 made a padded pie's gaps parallel (facing edges offset by half the padding each), so the gap
is one width along its whole length and each slice ends in a tip short of the centre. The tips of a full pie still
converge on the centre, which the owner read as the padding narrowing toward the middle (screenshot of the "Pie Chart -
Padding" demo). The look the owner asks for: the inner circle grows with the padding, so a padded pie is a thin-holed
donut whose hole radius is a function of the padding, not zero.

## Status / trigger

LANDED 2026-11-15 in queue/batch-6 (the batch after 5ebf4803); row 519 without Toast (no placement, caller-wins attributes), row 521 with the whole-page color-contrast gap as row 524 BACKLOG (owner 2026-11-14: "for padding pie the inner circle must increase relative to segment padding to look
correct"). Small; the same chart lane.

## Size

S.

## Detail

- **The construction.** In `primitives/src/chart/engine/polar.rs`, `arc_path` of a full pie with a padding gives the
  slice an inner radius derived from the padding when the chart sets none: the radius at which the slice's two offset
  edges are one gap width apart on the arc, `r_inner = (pad / 2) / sin(theta / 2)` capped at a fraction of the outer
  radius, so a thin slice gets a wider cap and every slice's inner edge sits on one circle. An explicit `inner_radius`
  wins. The too-thin slice that today draws `M0,0Z` then has a real inner arc instead.
- **The formula used (checked against the geometry).** The proposal `(pad / 2) / sin(theta / 2)` is the radius where a
  slice's two inner corners meet: the offset edges are `h = sqrt(r0^2 + r1^2) * sin(pad / 2)` from the radial lines, so
  the corners are `2 * asin(h / r)` apart and the inner arc has positive length only for `r > h / sin(theta / 2)`.
  At the proposed value the thinnest slice is still a tip (and it ignores that `h` grows with `r0`). One circle for all
  slices has to clear the thinnest slice's tip, so `padded_inner_radius` (polar.rs) returns `r0 = 2h / sin(theta_min / 2)`,
  solved in closed form for the `r0` that `h` depends on, capped at half the outer radius; the thinnest slice's inner arc
  then spans about half its angle. `pie.rs` calls it for a single-ring pie whose inner radius resolves to 0.
- **Proof.** `pie_chart.spec.ts` gains a case that reads the inner radius of the padded demo's paths (all equal, greater
  than zero, zero when `padding_angle` is zero) and a screenshot for the owner before the lane hands back; the unit
  tests of `polar.rs` pin the formula at two angles.
- **Files.** `primitives/src/chart/engine/polar.rs`, its tests, `playwright/pie_chart.spec.ts`, CHANGELOG.
