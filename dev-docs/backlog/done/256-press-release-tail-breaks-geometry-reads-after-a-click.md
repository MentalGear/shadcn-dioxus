---
id: 256
title: "The press-release `scale` tail broke three layout-stability reads taken right after a click"
status: "LANDED 2026-10-20 (lane/triageA): caused by batch A (row 295 press feedback), reproduced 3 of 3 on…"
size: "S"
area: playwright
updated: 2026-10-20
---

# Row 256: geometry read mid press-release (batch A, row 295)

## Item

Release suite on 3fd7844b: `data_table.spec.ts` "toggling a checkbox moves and resizes nothing" (`checkbox.x 396.5 -> 396.618`), "the selection count gaining a digit ..." (`checkbox.y 446.80 -> 59.84`) and `checkbox.spec.ts` "the box and its indicator keep one constant size ..." (`checkbox width 15.76`, expected 16).

## Status / trigger

LANDED 2026-10-20 (lane/triageA): caused by batch A (row 295 press feedback), reproduced 3 of 3 on the batch and 0 of 3 on main; fixed for the whole class by a `fixtures.ts` default (press feedback off in every spec, opt-in per spec)

## Size

S

## Detail

**Cause.**

Row 295 made the press land instantly (`--dx-press-in-duration: 0s`) and release over `--dx-press-duration` (the slow motion duration, 300 ms) on every `dx-` control, `scale: 0.98`. `getBoundingClientRect()` includes `scale`, so a read taken in the 300 ms after a click saw a 15.76 px, 0.1 px shifted checkbox. A debug SSG of the batch fails the three tests 3 of 3 (x/width), a debug SSG of main passes 3 of 3.

The `y 446.80 -> 59.84` read in the suite is the second symptom, and it is a Playwright effect, not a product scroll: the next `click()` on the same checkbox starts while the previous press is still releasing, Playwright logs `element is not stable`, retries, and its scroll-into-view step then moves the page (scrollY 0 -> 65 or 386, so the viewport y of the box jumps). Probe (`DEBUG=pw:api`): scroll appears when the second click follows within ~300 ms, never when 700 ms are waited; no JS `scrollTo`/`scrollIntoView`/`focus` call was made.

**Decision: the specs were wrong, the product is right.**

`scale` never changes layout; the 300 ms release is the intended row 295 behaviour (a visible tail for every tap, however short). These three tests are about LAYOUT.

**Construction (second commit): press feedback is off by default in every spec.** `playwright/fixtures.ts` overrides the `context` fixture and, unless a spec says `test.use({ pressFeedback: true })`, adds a context init script that stores the visitor preference the product itself uses: the cookie `dx_dx_press-feedback=off`, which `PREPAINT_JS` (the only writer of `data-dx-press-feedback`) reads before first paint and turns into `<html data-dx-press-feedback="off">`. It does not write the attribute: with no cookie PREPAINT_JS removes it again, so an attribute-writing init script would lose to the app, while the cookie goes through the app's own path and survives reloads, SPA navigations and a "Reset all". Specs that test the feedback opt back in with one line: `press-feedback.spec.ts` (file), `theme-all-reset.spec.ts` "Switch gets the theme's press feedback while pressed" and `charts_gallery.spec.ts` "the pill stays while the pointer is held down" (a `describe`). The per-spec `readAtRest` reads of the first commit were reverted: with the feedback off the plain reads in `data_table.spec.ts` and `checkbox.spec.ts` are exact.

**What the construction covers.** Every spec read of a rect, `boundingBox()` or "is the element stable" check taken after a click on a `dx-` control, in any component, including the Playwright actionability retry that scrolled the page (`y 446.8 -> 59.8`): the `:active` rule is simply never applied, so no `scale` tail exists. This subsumes the three observed instances and the unobserved ones in the other pressable controls.

**What it does not cover.**
- A user: a visitor who double-clicks or clicks twice in a row still sees the tail, and nothing is wrong (the tail is the feature; a transform never moves layout or the hit target by more than 1 px). Nothing in the product changed.
- Contexts a spec builds itself with `browser.newContext()` (the JavaScript-off first-paint contexts, touch-emulation specs): they do not pass through the fixture and keep the feedback. They do not read post-click geometry today.
- A regression in the feedback itself is caught only by the opted-in specs (`press-feedback.spec.ts`, the two above), not by the other ~200 files; that is the cost of the default, and the reason the opt-in is one visible line instead of a global mask. The row 228 lesson (a context-wide override of a property the page's bug lived in hid a navigation bug) is why the fixture sets the preference rather than the property, and why `fixtures.ts` documents it.
