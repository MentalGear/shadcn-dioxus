---
id: 492
title: "Bug: ColorPicker's panel clips to about half its height when it closes"
status: "FIXED on lane/colorpicker-close-492, not merged (2026-11-01). Root cause: a component sheet's…"
size: "S"
area: overlays
updated: 2026-11-01
---

# Row 492: Bug: ColorPicker's panel clips to about half its height when it closes

## Item

When the ColorPicker closes, its popover panel is cut to roughly half its height for the length of the close
animation. The owner's word for it is "glitch": the panel shrinks and clips its own content, then disappears.

## Status / trigger

FIXED on `lane/colorpicker-close-492`, not merged (2026-11-01). Root cause: a component sheet's `display` beat the UA's
`[popover]:not(:popover-open) { display: none }`, so a natively closed popover kept painting outside the top layer for
the whole exit animation. Not suspects 1-3. One shared rule, `primitives/src/top_layer/anchor.css`
(`[data-dx-anchored][popover]:not(:popover-open) { display: none !important }`), closes the class for ColorPicker,
Popover and DatePicker (through Popover). Proof: `playwright/anchored-close-clip.spec.ts`, 16 of its 28 cases red on the
base build (every ColorPicker case, Popover x2, DatePicker x2), all green after the fix.

## Size

S

## Detail

**Measured (debug SSG build of `bc666a1c`, Chromium, per animation frame through the close).** The three suspects are
refuted by numbers: `max-height` stays `none` and the placement `transform` is unchanged on every frame, the panel's
rect height stays 382 px, and the close keyframe animates `translate` only. The cause is the top layer:

- `.dx-color-picker-popover` sets `display: block` (the Popover's `.dx-popover-content` sets `display: flex`), which
  beats the UA's `[popover]:not(:popover-open) { display: none }`. Escape, a light dismiss and a press on the trigger
  (a native light dismiss on `pointerdown`) hide the popover natively, before the exit animation runs, so the panel
  stays painted with `:popover-open` false for the animation's 150 ms: an ordinary positioned box, not in the top layer.
- Desktop, below the trigger (CSS-anchored path): the box does not move (top 125.8 px before and after), but it is
  clipped by `body { overflow: clip }` and no longer above its siblings. Phone, script-placed: it is positioned against
  another containing block and lands at y = 819.8 px of an 844 px viewport (a strip of the panel at the bottom edge;
  with a 3000 px padded page, top 395.9 -> 4000.8 px). That is the owner's "cut off" panel.
- Every case, 4 sides/viewports x 3 closers, shows `display: block` with `:popover-open` false from the first frame after
  the close request until the animation ends.
- Class check (a `[data-dx-anchored][popover]` that is not `:popover-open` but has a `display` other than `none`, sampled
  per frame for 600 ms after Escape and after an outside press): Popover (27 and 26 frames), the date picker through
  Popover (29 and 27), ColorPicker; zero frames for DropdownMenu, Menubar, Combobox, Tooltip, Navbar, NavigationMenu.

**Construction.** The anchored-overlay engine stylesheet restates the UA rule once, with `!important`, so no
component sheet's `display` can keep a panel that left the top layer painted. A panel that closes through its exit
animation stays `:popover-open` until it unmounts, so only an already-gone panel is hidden. Subsumes the ColorPicker,
the Popover and everything composed from it (DatePicker, the theme picker). It does not give a native close an exit
animation (an accepted limit of the plain Popover API, `oracle/tier2-html/top-layer.spec.ts` Rules 2 and 3).

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
