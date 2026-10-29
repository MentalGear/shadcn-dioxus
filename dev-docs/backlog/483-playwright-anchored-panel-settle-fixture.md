---
id: 483
title: "Playwright waits for a just-opened anchored panel to settle before acting inside it (fixture, not per spec)"
status: "IMPLEMENTED on lane/settle-fixture-483 (main-loop merge pending). The wait lives in the shared…"
size: "M"
area: tests
updated: 2026-10-29
---

# Row 483: Playwright settle fixture for anchored panels

## Item

Playwright's actionability check waits for a just-opened anchored panel to finish its open animation before a click or hover inside it. The wait belongs in the shared fixture (`playwright/fixtures.ts`), not in each spec.

## Status / trigger

IMPLEMENTED on lane/settle-fixture-483 (main-loop merge pending). The wait lives in the shared fixture; the six specs call `click()`/`hover()` plainly. Proof on a debug lane-ssg build: the six specs pass 3x each (432 tests), and the full suite (`playwright/tools/run-suite.sh`, 8 chunks) ran 3,703 passed, 23 skipped, 1 failed. The one red, `oracle/tier1-apg/keyboard-matrix.spec.ts` "Select listbox — Escape" (a trigger click outside any panel, then Escape), passed 10/10 alone and 60/60 at 4 workers on both the new and the base fixture: a load flake, not the fixture.

## Size

M

## Detail

- **Cause.** Since row 477, anchored panels are `position: absolute` in document coordinates and animate in. Before that they were `position: fixed`, which could not scroll with the page. When a spec clicks or hovers inside a panel while its open animation runs, Playwright's actionability check sees an unstable element and retries `scrollIntoViewIfNeeded`, which scrolls the window. The panel scrolls with the page, can flip side (row 477, step 5) and leave the pointer outside it. A real user is not affected: nothing scrolls the page on a click.
- **Construction (landed).** `playwright/fixtures.ts` wraps `click`, `hover`, `tap` and `dblclick` on the worker's `Locator` class (the class is taken from a throwaway page of the worker's browser, so pages made by `browser.newContext()` are covered too). Before each action, `awaitAnchoredPanelSettled` (`playwright/animations.ts`) looks for `[data-dx-anchored]` ancestors of the target (the attribute row 432 puts on every engine-placed panel) and waits for their finite animations (`getAnimations({ subtree: true })`) to finish, after two rAFs so a just-mounted panel's animations exist. A target outside any panel costs one `evaluateAll` round trip. It never throws: a missing or non-unique target is left to the action's own wait and error. Opt-out: `test.use({ settleAnchoredPanels: false })` for a file, a `describe` or a test (a worker runs one test at a time, so the flag is a module variable set per test).
- **Coverage.** All 264 spec files take `test` from `fixtures.ts`: `scripts/check-playwright-fixture-import.sh` (a gate) fails any spec that imports the runtime `test`/`expect`/`devices` from `@playwright/test`, and no file calls `.extend(`. Not covered: `page.click(selector)`-style page-level actions (no spec uses them) and the other actions (`check`, `fill`, `dragTo`).
- **Removed.** `clickSettled` and `hoverSettled` are gone from `animations.ts`; `awaitAnimationsSettled`, `measureAtRest` and `readAtRest` stay (geometry reads).
- **Files owned:** `playwright/fixtures.ts`, `playwright/animations.ts`, and the six specs listed above.
- **Proof:** one full suite run (`playwright/tools/run-suite.sh`) green after the fixture change; the six specs pass without their explicit settle calls.
