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
 * (`carousel-rewind-autoplay.spec.ts`'s hover-stops-rotation test and `top-layer.spec.ts`'s `pinNearTop`)
 * are unstable: an ambient smooth rule is a product defect, not a harness problem.
 *
 * WHAT THE FIXTURE DOES (backlog row 483). Every `locator.click()`, `hover()`, `tap()` and `dblclick()` whose target is
 * inside an anchored panel (`[data-dx-anchored]`) first waits for that panel's animations to finish
 * (`awaitAnchoredPanelSettled`, animations.ts): a retried action on a still-animating absolute panel scrolls the page
 * and can flip the panel away from the pointer. A spec never settles by hand. A spec that measures the open animation
 * itself, or acts on purpose while it runs, opts out with `test.use({ settleAnchoredPanels: false })` (a file, a
 * `describe` or one test). Pages made by `browser.newContext()` are covered too: the wait is installed on the
 * `Locator` class of the worker, once, not on the `page` fixture.
 */
import { test as base, expect, devices } from "@playwright/test";
import type { Locator } from "@playwright/test";
import { awaitAnchoredPanelSettled } from "./animations";

const ACTIONS = ["click", "hover", "tap", "dblclick"] as const;
const INSTALLED = Symbol.for("dx.anchoredPanelSettle");

/** Whether the running test settles anchored panels; set per test by the `_anchoredPanelSettle` fixture (a worker runs one test at a time). */
let settleEnabled = true;

type ActionOptions = { timeout?: number } | undefined;

/** Wraps the four actions on the `Locator` class; idempotent per worker process. */
function installSettle(proto: Record<string | symbol, unknown>): void {
  if (proto[INSTALLED]) return;
  proto[INSTALLED] = true;
  for (const action of ACTIONS) {
    const original = proto[action] as (this: Locator, options?: ActionOptions) => Promise<void>;
    proto[action] = async function (this: Locator, options?: ActionOptions): Promise<void> {
      if (settleEnabled) await awaitAnchoredPanelSettled(this, options?.timeout);
      return original.call(this, options);
    };
  }
}

export const test = base.extend<
  { settleAnchoredPanels: boolean; _anchoredPanelSettle: void },
  { _locatorSettle: void }
>({
  // The `Locator` class is not exported at runtime; one throwaway page in the worker's browser yields it.
  _locatorSettle: [
    async ({ browser }, use) => {
      const context = await browser.newContext();
      const page = await context.newPage();
      installSettle(Object.getPrototypeOf(page.locator("html")));
      await context.close();
      await use();
    },
    { scope: "worker", auto: true },
  ],
  settleAnchoredPanels: [true, { option: true }],
  _anchoredPanelSettle: [
    async ({ settleAnchoredPanels }, use) => {
      settleEnabled = settleAnchoredPanels;
      await use();
      settleEnabled = true;
    },
    { auto: true },
  ],
});

export { expect, devices };
