---
id: 345
title: "carousel-nav-opacity first-slide spec reads Next mid-fade (red on main)"
status: "Done 2026-10-21 (branch claude/carousel-nav-fix): the spec waits for Next to be enabled and for…"
size: "S"
area: tests
updated: 2026-10-21
---

# Row 345: carousel-nav-opacity first-slide spec reads Next mid-fade (red on main)

## Item

`playwright/carousel-nav-opacity.spec.ts` "at the first slide Previous is disabled, transparent and hidden; Next is untouched" failed deterministically on main `fd083a2d` (debug SSG).

## Status / trigger

**Done 2026-10-21 (branch `claude/carousel-nav-fix`): the spec waits for Next to be enabled and for both arrows' animations to settle before reading.**

## Size

S

## Detail

Symptom: `look(next)` returned `{ opacity: 0.843628, visibility: "visible", disabled: false }`, expected `opacity: 1`.

Mechanism: the carousel counts its slides in an effect after mount, so the server HTML (and the first client frame) renders Next `disabled` (`can_scroll_next` is false for `count == 0`, 0.5 opacity). Once the items register the arrow enables and `.dx-carousel-next` fades 0.5 -> 1 over `--dx-motion-duration-slow` (200 ms). The spec read it right after `Previous` became disabled, i.e. inside that fade. No carousel file changed since the spec last passed; the page got a later/different hydration timing (large theme, select, virtual-grid and questionnaire changes in the same window), so a pre-existing race became deterministic. A read of an element that plays a transition belongs at rest (`awaitAnimationsSettled`, the same rule as `check-spec-at-rest.sh` for geometry).

Fix: spec only, `await expect(next).toBeEnabled()` then `awaitAnimationsSettled` on both arrows.

Open remainder: the 0.5 -> 1 fade of Next after hydration is itself a first-paint flicker (same class as the chart and vgrid first-paint rows). By construction it would need the server to know the slide count, or the fade suppressed for the first registration; not done here.
