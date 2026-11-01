---
id: 492
title: "Bug: ColorPicker's panel clips to about half its height when it closes"
status: "BACKLOG, bug (owner report 2026-11-01). Reproduce first."
size: "S"
area: overlays
updated: 2026-11-01
---

# Row 492: Bug: ColorPicker's panel clips to about half its height when it closes

## Item

When the ColorPicker closes, its popover panel is cut to roughly half its height for the length of the close
animation. The owner's word for it is "glitch": the panel shrinks and clips its own content, then disappears.

## Status / trigger

BACKLOG, bug (owner report 2026-11-01). Reproduce first.

## Size

S

## Detail

**Not reproduced yet.** This row was written without a build. The first step of the lane is a debug SSG build and a
frame recording (see "Reproduce first" below).

**Where the picker lives.** `primitives/src/color_picker.rs` (the primitive, about 830 lines), its sibling
`primitives/src/color_picker/color_naming.rs`, and the preview at `preview/src/components/color_picker/` (`style.css`,
`variants/main/mod.rs`). The row's premise is that the picker is built on the Popover; the lane confirms which popover
primitive the panel uses and whether the close is the anchored (top-layer) path.

**Reproduce first (the lane's first step, before any fix).**

- Debug SSG build through `scripts/build-ssg.sh` (absolute isolated `CARGO_TARGET_DIR`), served as usual.
- Open the picker, close it (Escape and an outside press, then a trigger press), and record the close frame by frame:
  the panel's `getBoundingClientRect()` height and `getComputedStyle(panel).maxHeight`, plus its `transform`, per
  `requestAnimationFrame` tick until the panel is hidden. Do this at phone width and desktop width, with the picker
  placed above and below its trigger.
- Compare with the previous deploy if the debug build does not show it. A deploy-only difference is still a finding.
- Write the result as measured numbers in the row before choosing a construction.

**First suspects, in the order the row gives them. Each is a hypothesis to test, not a diagnosis.**

1. **Row 477's live `max-height`.** Placement writes `max-height` from the room on the chosen side, on scroll and on
   `visualViewport` resize (`primitives/src/top_layer/anchor_fallback.rs`, the `put('max-height', tall ? hc + 'px' : '')`
   write near line 138). A recompute that runs while the close animation is still going (a scroll, a resize, or the
   panel's own measured size during the scale transform) could cap the panel at about half its height. Check whether
   the value is recomputed during close, and whether the measurement reads the scaled box.
2. **The close animation's transform against `translateY(-100%)`.** For a panel above its trigger, row 477 places the
   top side with `translateY(-100%)`. A close keyframe that sets `transform` replaces the placement transform for as
   long as it runs. The top-layer note in the theme's anchored-overlay section says the drop keyframes therefore animate
   `translate`, not `transform`. A close keyframe that does set `transform` would move or scale the panel off its
   placement, and that can read as a clip. Check the picker's close keyframe against that note.
3. **Row 422's accordion or `data-revealed`.** If the picker contains an accordion (the lane checks), a `data-revealed`
   change during close could change the content's height and the panel's measured height. Check only after 1 and 2 are
   measured.

**Constraint.** The fix goes in the construction that makes the whole overlay class unable to do this, if the cause is
shared (row 477's placement rule or the close keyframes). A picker-only patch is acceptable only if the measurements
show the cause is in the picker.

**Proof (the spec written first, red before the fix).**

- A spec opens the ColorPicker, closes it, and samples the panel's height on every frame of the close (through
  `requestAnimationFrame` in the page). The panel's height stays within 1 px of its open height through the close
  animation, above and below the trigger, at phone and desktop widths, in light and dark.
- The spec is red on the current build before the fix. Its frame recording is kept in the lane report.
- Gate: `scripts/run-gates.sh --fast` in the lane; the full run belongs to the merge batch.

**Files the lane is likely to touch.** `primitives/src/color_picker.rs`, `preview/src/components/color_picker/style.css`
and the placement code `primitives/src/top_layer/anchor_fallback.rs` (only if suspect 1 is confirmed, and then the
construction is row 477's rule, not a per-component patch).
