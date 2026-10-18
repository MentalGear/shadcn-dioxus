/**
 * Shared `test`/`expect` for every spec in this suite (dev-docs/backlog.md
 * row 110). Import `test`/`expect`/`devices` from here instead of
 * "@playwright/test" directly; keep importing types (`type Page`,
 * `type Locator`, ...) from "@playwright/test" as before -- only the
 * runtime `test` object needs to come from this file.
 *
 * HISTORY, AND WHY THERE IS NO SCROLL OVERRIDE ANY MORE. Until row 228 this
 * module wrapped `browser.newContext` so every context got an init script
 * forcing `html { scroll-behavior: auto !important }`. The reason was real:
 * Playwright's actionability pre-check (`scrollIntoViewIfNeeded`, issued
 * before `.hover()`/`.click()`) is an UNQUALIFIED scroll, which defers to the
 * scroll box's CSS `scroll-behavior`, and the site then set an ambient
 * `html { scroll-behavior: smooth }` -- so the pre-check animated after the
 * event had been dispatched and dragged the target out from under the cursor
 * (row 110: `carousel.spec.ts`'s hover-stops-rotation test and
 * `top-layer.spec.ts`'s `pinNearTop`).
 *
 * That override was also the reason no spec could see the ambient-smooth
 * navigation bug of row 228 (the router's `scrollTo(0, 0)` glided across the
 * page swap and landed on the short destination's bottom): the suite forced
 * the very property the bug lives in to `auto`, on every context, so every
 * navigation spec ran in a world users never get. Row 228 made smooth
 * scrolling opt-in (`html[data-anchor-scroll] { scroll-behavior: smooth }`,
 * set only for the span of a same-page `a[href^="#"]` jump), so the root
 * scroll box is instant by construction and the Playwright pre-check cannot
 * animate. The override has nothing left to fix and a lot to hide, so it is
 * deleted rather than narrowed: a narrowed one (e.g. only non-root boxes)
 * would still be a second source of truth about scroll-behavior that the
 * page's own CSS can disagree with.
 *
 * What this means for a spec author: the page's own `scroll-behavior` is what
 * runs. A spec that needs the smooth jump (anchor-landing's click cases)
 * asks for it the way the site does -- `data-anchor-scroll` on `<html>`, or an
 * inline `scroll-behavior` it restores itself -- and a spec that must not
 * have it simply never sets it. `scroll-behavior` on an explicit
 * `scrollBy({ behavior })` call (the carousel's) was never affected by CSS.
 * A build older than row 228 (ambient smooth) can flake the two row-110 specs
 * again: that is the old bug showing, not a harness problem.
 *
 * The module stays as the one import point (141 specs import it) so a future
 * suite-wide fixture has a single place to live.
 */
import { test, expect, devices } from "@playwright/test";

export { test, expect, devices };
