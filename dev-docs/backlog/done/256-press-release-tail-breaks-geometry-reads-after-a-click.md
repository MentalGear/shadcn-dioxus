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

LANDED 2026-10-20 (lane/triageA): caused by batch A (row 295 press feedback), reproduced 3 of 3 on the batch and 0 of 3 on main; the specs now read at rest

## Size

S

## Detail

**Cause.**

Row 295 made the press land instantly (`--dx-press-in-duration: 0s`) and release over `--dx-press-duration` (the slow motion duration, 300 ms) on every `dx-` control, `scale: 0.98`. `getBoundingClientRect()` includes `scale`, so a read taken in the 300 ms after a click saw a 15.76 px, 0.1 px shifted checkbox. A debug SSG of the batch fails the three tests 3 of 3 (x/width), a debug SSG of main passes 3 of 3.

The `y 446.80 -> 59.84` read in the suite is the second symptom, and it is a Playwright effect, not a product scroll: the next `click()` on the same checkbox starts while the previous press is still releasing, Playwright logs `element is not stable`, retries, and its scroll-into-view step then moves the page (scrollY 0 -> 65 or 386, so the viewport y of the box jumps). Probe (`DEBUG=pw:api`): scroll appears when the second click follows within ~300 ms, never when 700 ms are waited; no JS `scrollTo`/`scrollIntoView`/`focus` call was made.

**Decision: the specs were wrong, the product is right.**

`scale` never changes layout; the 300 ms release is the intended row 295 behaviour (a visible tail for every tap, however short). These three tests are about LAYOUT, so they read at rest: `readAtRest(checkbox, ...)` (`playwright/animations.ts`) waits for the checkbox's and its ancestors' transitions (a `CSSTransition` is in `document.getAnimations()`) before reading, and so also before the next click starts. 80 of 80 green over 5 repeats at 2 workers on the batch build; 12 of 12 with 3 CPU burners at 4 workers on the release build.

**Class, and what it does not cover.**

Any spec that reads a rect right after clicking a pressable control, or clicks it twice in quick succession, can hit this. Only these three did in a full run. No gate was added: `check-spec-at-rest.sh` covers overlay specs; widening it to every `getBoundingClientRect()` after a `click()` would flag hundreds of reads that measure things a press cannot move. If a fourth case shows up, the construction is a `clickAndSettle` helper in `playwright/animations.ts` (click, then `awaitAnimationsSettled`) for layout-stability specs, enforced the same way as `spec-at-rest`. Not done on two occurrences in two spec files.

**Decision 2026-10-20 (main loop):** a global "press feedback off" default in `playwright/fixtures.ts` was built on lane/triageA (4b043265) and NOT merged. 7a066be9 removed the last global fixture override because forcing a property in every context hid row 228, and a global off would hide press regressions in every spec but three the same way. The three instances keep the at-rest reads above. If a fourth appears, the construction is to widen `check-spec-at-rest.sh` to post-click geometry reads of pressable controls (`readAtRest`/`measureAtRest` or a `rest-ok:` reason), not a fixture override.
