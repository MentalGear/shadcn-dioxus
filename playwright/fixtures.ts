/**
 * Shared `test`/`expect` for every spec in this suite (dev-docs/backlog.md
 * row 110). Import `test`/`expect`/`devices` from here instead of
 * "@playwright/test" directly; keep importing types (`type Page`,
 * `type Locator`, ...) from "@playwright/test" as usual -- only the
 * runtime `test` object needs to come from this file.
 *
 * WHY THERE IS NO SCROLL OVERRIDE (rows 110, 228). Smooth scrolling is opt-in
 * (`html[data-anchor-scroll] { scroll-behavior: smooth }`, set only for the span of a
 * same-page `a[href^="#"]` jump), so the root scroll box is instant by construction and
 * Playwright's actionability pre-check (`scrollIntoViewIfNeeded`, issued before
 * `.hover()`/`.click()`, an UNQUALIFIED scroll that defers to the scroll box's CSS
 * `scroll-behavior`) cannot animate out from under the cursor. A suite-wide
 * `html { scroll-behavior: auto !important }` would hide the ambient-smooth navigation bug
 * of row 228 (the router's `scrollTo(0, 0)` gliding across the page swap and landing on the
 * short destination's bottom), because it would force the very property that bug lives in
 * to `auto` on every context, so every navigation spec would run in a world users never
 * get. A narrowed override (e.g. only non-root boxes) would still be a second source of
 * truth about scroll-behavior that the page's own CSS can disagree with.
 *
 * What this means for a spec author: the page's own `scroll-behavior` is what
 * runs. A spec that needs the smooth jump (anchor-landing's click cases)
 * asks for it the way the site does -- `data-anchor-scroll` on `<html>`, or an
 * inline `scroll-behavior` it restores itself -- and a spec that must not
 * have it simply never sets it. `scroll-behavior` on an explicit
 * `scrollBy({ behavior })` call (the carousel's) is not affected by CSS.
 * Against a build with an ambient smooth rule, the two row-110 specs
 * (`carousel.spec.ts`'s hover-stops-rotation test and `top-layer.spec.ts`'s `pinNearTop`)
 * are unstable: an ambient smooth rule is a product defect, not a harness problem.
 *
 * The module stays as the one import point so a future
 * suite-wide fixture has a single place to live.
 */
import { test, expect, devices } from "@playwright/test";

export { test, expect, devices };
