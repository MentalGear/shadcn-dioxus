---
id: 483
title: "Playwright waits for a just-opened anchored panel to settle before acting inside it (fixture, not per spec)"
status: "BACKLOG. Scheduled as the first job after the session branch merges into main (owner 2026-10-29).…"
size: "M"
area: tests
updated: 2026-10-29
---

# Row 483: Playwright settle fixture for anchored panels

## Item

Playwright's actionability check waits for a just-opened anchored panel to finish its open animation before a click or hover inside it. The wait belongs in the shared fixture (`playwright/fixtures.ts`), not in each spec.

## Status / trigger

BACKLOG. Scheduled as the first job after the session branch merges into main (owner 2026-10-29). Occurrences so far: 6.

## Size

M

## Detail

- **Cause.** Since row 477, anchored panels are `position: absolute` in document coordinates and animate in. Before that they were `position: fixed`, which could not scroll with the page. When a spec clicks or hovers inside a panel while its open animation runs, Playwright's actionability check sees an unstable element and retries `scrollIntoViewIfNeeded`, which scrolls the window. The panel scrolls with the page, can flip side (row 477, step 5) and leave the pointer outside it. A real user is not affected: nothing scrolls the page on a click.
- **Per-spec fix in use.** `clickSettled` and `hoverSettled` in `playwright/animations.ts` wait for the panel's animations before they act. Occurrences (6): `hover-intent.spec.ts`, `navigation_menu.spec.ts`, `hover-mix-space.spec.ts`, `accent-states.spec.ts`, `chat-demos.spec.ts`, `docs-nav-sort.spec.ts`.
- **Construction.** The shared fixture makes every locator click, hover and tap whose target is inside an open anchored panel wait for that panel's animations (`getAnimations({ subtree: true })` finished) first. No spec does it by hand, and the explicit `clickSettled` and `hoverSettled` calls then become plain clicks and hovers. A spec that measures the animation itself opts out.
- **Schedule (owner 2026-10-29).** First job after the merge into main, not on a count: each occurrence costs its diagnosis (it looks like a product regression), and a count depends on lanes noticing. Until then, a lane that needs the per-spec settle in a new spec file adds it to the occurrence list above.
- **Files owned:** `playwright/fixtures.ts`, `playwright/animations.ts`, and the six specs listed above.
- **Proof:** one full suite run (`playwright/tools/run-suite.sh`) green after the fixture change; the six specs pass without their explicit settle calls.
