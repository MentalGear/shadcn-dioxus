---
id: 222
title: "Specs that measure geometry while an enter animation is still running"
status: "LANDED 2026-10-18 (lane specs 4647fc12), merged on the session branch; deploy pending -- was…"
size: "S"
area: tests
updated: 2026-10-18
---

# Row 222: Specs that measure geometry while an enter animation is still running

## Item

Specs that measure geometry while an enter animation is still running

## Status / trigger

**LANDED 2026-10-18 (lane specs 4647fc12), merged on the session branch; deploy pending** -- was: **Scheduled 2026-10-15 in `dev-docs/research/next-round-plan-2026-10-15.md`** (set up, waiting for the owner's go). **OPEN 2026-10-15 (by-construction fix proposed)**

## Size

S

## Detail

Two instances in one round: `select.spec.ts` :366/:376 read the listbox width mid `dx-pop-in` (142.16 px at scale 0.981 vs 145.89 at rest; fixed in 7c588e7 with `awaitAnimationsSettled`), and `dialog.spec.ts:188` compares the footer band with the dialog width during the 200 ms enter transition (failed 1 in 5 alone, 1.25-1.86 px). `boundingBox()` includes transforms, so any geometry read before the animation settles is a race. **Construction:** one `measureAtRest(locator)` helper in `playwright/animations.ts` (awaits `getAnimations()` settled, then reads the box) used by every geometry assertion on an animated surface, plus a cheap lint in `scripts/check-*.sh` that flags `boundingBox()` in specs of components whose sheet animates on enter without the helper (reasoned allowlist).
