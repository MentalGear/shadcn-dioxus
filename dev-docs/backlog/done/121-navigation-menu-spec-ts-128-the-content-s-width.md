---
id: 121
title: "navigation_menu.spec.ts:128 \"the content's width never changes while its open animation plays\" fails on every…"
status: "LANDED 2026-10-11: a test-methodology defect, not a component bug -- the spec sampled…"
size: "S (diagnosis)"
area: build
updated: 2026-10-11
---

# Row 121: navigation_menu.spec.ts:128 "the content's width never changes while its open animation plays" fails on every…

## Item

`navigation_menu.spec.ts:128` "the content's width never changes while its open animation plays" fails on every SSG run

## Status / trigger

**LANDED 2026-10-11: a test-methodology defect, not a component bug** -- the spec sampled `getBoundingClientRect().width`, which follows the open animation's `scale(.98 -> 1)`, so the old 825px panel's rect swung ~2% while its layout width never changed (reproduced 5/5 on the pre-compact-panel deploy; current code passed because the 508px panel's rect happens not to follow the scale, i.e. one panel-size change from failing again). The sampler now reads the computed layout width (still red on an injected width keyframe, 3/3). Same file: `openPanel`'s `click()` raced the 150ms hover-intent open under load (the click then closed the just-opened panel); it now hovers, waits for `aria-expanded`, and focuses. Final: navigation_menu + popover + select specs, `--repeat-each=10 --workers=4` on a release SSG: 680/680. The product behaviour it exposed is row 189. Was: **OPEN** -- pre-existing: failed on the deploy before PR #80 (361/362) and in PR #81 (373/374); not caused by the chart work

## Size

S (diagnosis)

## Detail

Seen 2026-10-03 as the single failure of both targeted runs on the SSG build. Not root-caused here; first check whether it is a sampling race of the same family as row 112 (a test that samples a live animation across an async gap -- the helper at the top of the spec resolves samples on `requestAnimationFrame`) or a real width change during the open transition under the SSG build. Needs a fresh read of the failing sample values before any fix; do not widen the threshold to make it pass.
