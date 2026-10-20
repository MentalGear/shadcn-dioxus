---
id: 245
title: "Computed-style snapshot harness: tooltip timing nondeterminism and the skip link counted as an open-state item"
status: "LANDED 2026-10-19 (harness timer drain + skip-link exclusion; A/A byte-identical)"
size: "S"
area: tests
updated: 2026-10-19
---

# Row 245: Computed-style snapshot harness: tooltip timing nondeterminism and the skip link counted as an open-state item

## Item

Computed-style snapshot harness: tooltip timing nondeterminism and the skip link counted as an open-state item

## Status / trigger

**LANDED 2026-10-19 (harness timer drain + skip-link exclusion; A/A byte-identical)**

## Size

S

## Detail

From the verify4 validation (session scratch verify4-report.md): two captures of one build differ on dark top_layer hover (196 vs 198 records, a Tooltip trigger chain rendered in 1 of 4 captures) -- tooltip timing the row 166 hover-card fix does not cover; make the harness's opener wait out every popup's configured delay. And harness coverage dropped (item hovers 486 -> 474, skipped 12 -> 123): the "Skip to content" link is now listed as an open-state item (occluded) when an overlay opens and takes hover slots; find why the skip link changes on overlay open (likely row 226's modality/focus work) and fix the harness or the page.

## Update 2026-10-19

Landed in `playwright/computed-style-states.ts` (harness only; `main.css` untouched). **Opener:** no popup delay is guessed any
more. `movePointer` arms a timer-drain (`installStateLib`: the page's `setTimeout` is wrapped, every timeout scheduled
between `arm()` and the end of `drain()` is remembered) and the read waits until all of them fired, so a tooltip / hover
card / submenu opens by hover after whatever delay it is configured with (the old fixed 450/1050 ms was a race). **Skip
link:** `.dx-skip-link` is excluded from the open-state item pool (`ITEM_EXCLUDE`). The page itself was not changed: the
skip link is in every overlay's delta because its style changes with focus, which is harmless once it cannot take hover slots.

Evidence (debug SSG of base 87b5a260, S0 = `/home/user/tgt/snap-S0/`, per scheme because one file for both exceeded Node's 512 MB string limit):
old harness A/A: light 0 differing values, **dark 27** (all `dropdown_menu` `open:4`: the skip-link record present in A only and a
shifted item list), coverage item hovers 499, skipped 131 (the "12 -> 123" erosion). New harness A/A (same build, both schemes):
**byte-identical** (`cmp` of both scheme files and of the coverage files), 0 differing values, item hovers 508, skipped 13,
hover records 8711 (old 8707: the tooltip chain is captured). Gap found: `hover_card`/tooltip chain in 1 of 4 captures did not
recur in the two old light captures, so that half is argued from the construction and from the identical A/A, not reproduced.
