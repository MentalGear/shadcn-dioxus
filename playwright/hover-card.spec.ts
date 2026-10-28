import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { startProbe, readProbe, clearProbe, elapsed } from "./hover-delay-probe";
import { expectNoAxeViolations, EXCLUDE_VENDORED_CODE_HIGHLIGHT } from "./axe";
import { sampleCloseFade, assertFadesOutThenUnmounts } from "./assert-fade-out";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { measureAtRest } from "./animations";

test("test", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/hover_card/`);
  let tooltip = page.getByRole("tooltip");
  // tabbing to the trigger element should show the tooltip
  await page.locator("#component-preview-frame").focus();
  await page.keyboard.press("Tab");
  await expect(tooltip).toBeVisible();
  // tabbing out of the trigger element should hide the tooltip
  await page.keyboard.press("Tab");
  await expect(tooltip).toHaveCount(0);

  // hovering over the trigger element should show the tooltip
  await page.getByRole("button", { name: "Dioxus" }).hover();
  await expect(tooltip).toBeVisible();

  // moving the mouse away from the trigger element should hide the tooltip
  await page.mouse.move(0, 0);
  await expect(tooltip).toHaveCount(0);
});

// dev-docs/backlog.md row 115 -- a focus-out must close the content even under
// main-thread load. The popover's `toggle` event is dispatched asynchronously, AFTER `showPopover()`
// returns; `use_popover_shown_while_mounted` must not mirror that event's
// `newState === "open"` back into the `open` signal. When the focus-out (or
// mouse-leave) lands before that queued event -- input events outrank it
// under main-thread load -- the stale "open" echo would re-open the content with
// nothing left to close it (stuck open, forever). Made deterministic here by
// blurring synchronously inside `showPopover()`, i.e. strictly before its
// `toggle` task can run.
test("a focus-out in the same beat as the open stays closed (no stale toggle echo reopening it)", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/hover_card/`);
  await page.evaluate(() => {
    const original = HTMLElement.prototype.showPopover;
    HTMLElement.prototype.showPopover = function (...args) {
      const result = original.apply(this, args);
      (document.activeElement as HTMLElement | null)?.blur();
      return result;
    };
  });
  await page.locator("#component-preview-frame").focus();
  await page.keyboard.press("Tab");
  await expect(page.getByRole("tooltip")).toHaveCount(0);
});

test.describe("Axe automated scan", () => {
  test("card open (hover) has no automatically detectable a11y issues", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/hover_card/`);
    await page.getByRole("button", { name: "Dioxus" }).hover();
    await expect(page.getByRole("tooltip")).toBeVisible();
    await expectNoAxeViolations(page, "hover-card: open", { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });
});

// dev-docs/backlog.md rows 19 and 7: on the web (popover) arm,
// `use_popover_sync` must not call `hidePopover()` the instant `open` goes
// false: the UA's `[popover]:not(:popover-open) { display: none }` rule would
// then hide the content before `dx-hover-card-fade-out`
// (`../preview/src/components/hover_card/style.css`) ever got a chance to
// play (`display: none` from the very first sample, so
// `assertFadesOutThenUnmounts` fails both the "no display:none while
// mounted" invariant and the "a few frames of decreasing opacity" check).
// `primitives/src/top_layer/popover_sync.rs`'s `use_popover_shown_while_mounted`
// (row 19) keeps the popover shown through the animation that CSS defines
// (row 7).
test.describe("Close-fade animation (dev-docs/backlog.md rows 19, 7)", () => {
  test("content fades out (opacity -> 0, still popover-open) before unmounting", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/hover_card/`, { timeout: 20 * 60 * 1000 });
    const trigger = page.getByRole("button", { name: "Dioxus" });
    await trigger.hover();
    const card = page.getByRole("tooltip");
    await expect(card).toBeVisible();
    const contentId = await card.getAttribute("id");
    if (!contentId) throw new Error("hover card content has no id to sample");

    // `sampleCloseFade` settles the OPEN fade-in first (`toBeVisible()` passes
    // at opacity 0), starts sampling before the close trigger so the first
    // frames are never missed, fires the trigger and returns the capture --
    // see assert-fade-out.ts.
    const capture = await sampleCloseFade(page, contentId, () =>
      page.mouse.move(0, 0), // same close trigger as this file's own "test" above
    );

    assertFadesOutThenUnmounts(capture);
  });
});

// dev-docs/backlog.md row 166 -- `delay` / `close_delay` (Radix `openDelay` / `closeDelay`, Base UI
// `PreviewCard.Trigger` `delay` / `closeDelay`; defaults 600 / 300). The `delays` variant renders
// "Default (600 / 300)", "Quick (10 / 100)" and "Instant (0 / 0)". Delays are measured in the page
// (hover-delay-probe.ts) and only as a LOWER bound, so a slow debug build cannot turn them red.
test.describe("delay / close_delay (backlog row 166)", () => {
  const URL = `${BASE_URL}/component/hover_card/`;
  const SLOP = 40; // timer clamping / clock granularity between two performance.now() stamps
  const trigger = (page: Page, name: string) => page.getByRole("button", { name, exact: true });
  const surface = (page: Page) => page.getByRole("tooltip");
  const openSurface = (page: Page) => page.locator('[role="tooltip"][data-state="open"]');

  test("nothing opens before `delay`: the default card waits 600 ms of pointer rest", async ({ page }) => {
    await gotoHydrated(page, URL, { timeout: 20 * 60 * 1000 });
    await startProbe(page);
    await trigger(page, "Default (600 / 300)").hover();
    await expect(surface(page)).toBeVisible();
    const waited = elapsed(await readProbe(page), "mouseover", "surface:open");
    expect(waited, "the card opened").not.toBeNull();
    expect(waited!).toBeGreaterThanOrEqual(600 - SLOP);
  });

  test("leaving before `delay` cancels the open (a sweep across triggers flashes nothing)", async ({ page }) => {
    await gotoHydrated(page, URL, { timeout: 20 * 60 * 1000 });
    await startProbe(page);
    await trigger(page, "Default (600 / 300)").hover();
    await page.waitForTimeout(150);
    await page.mouse.move(0, 0);
    await page.waitForTimeout(1100); // past the 600 ms the abandoned hover would have opened at
    await expect(surface(page)).toHaveCount(0);
    expect((await readProbe(page)).some((e) => e.what === "surface:open")).toBe(false);
  });

  test("a short `delay` opens sooner than the default (shadcn's delay={10})", async ({ page }) => {
    await gotoHydrated(page, URL, { timeout: 20 * 60 * 1000 });
    await startProbe(page);
    await trigger(page, "Quick (10 / 100)").hover();
    await expect(surface(page)).toBeVisible();
    expect(elapsed(await readProbe(page), "mouseover", "surface:open")!).toBeLessThan(600);
  });

  test("keyboard focus opens at once even with a delay, and blur closes at once despite a close_delay", async ({
    page,
  }) => {
    await gotoHydrated(page, URL, { timeout: 20 * 60 * 1000 });
    await startProbe(page);
    await trigger(page, "Default (600 / 300)").focus();
    await expect(surface(page)).toBeVisible();
    expect(elapsed(await readProbe(page), "focusin", "surface:open")!).toBeLessThan(600);

    await clearProbe(page);
    await page.evaluate(() => (document.activeElement as HTMLElement).blur()); // off the trigger: blur
    await expect(openSurface(page)).toHaveCount(0);
    expect(elapsed(await readProbe(page), "focusout", "surface:closed")!).toBeLessThan(300);
  });

  test("close waits `close_delay` after the pointer leaves", async ({ page }) => {
    await gotoHydrated(page, URL, { timeout: 20 * 60 * 1000 });
    await startProbe(page);
    await trigger(page, "Default (600 / 300)").hover();
    await expect(surface(page)).toBeVisible();
    await clearProbe(page);
    await page.mouse.move(0, 0);
    await expect(openSurface(page)).toHaveCount(0);
    expect(elapsed(await readProbe(page), "mouseout", "surface:closed")!).toBeGreaterThanOrEqual(300 - SLOP);
  });

  test("re-entering the card cancels the close; leaving it starts the close again", async ({ page }) => {
    await gotoHydrated(page, URL, { timeout: 20 * 60 * 1000 });
    await startProbe(page);
    await trigger(page, "Default (600 / 300)").hover();
    await expect(surface(page)).toBeVisible();
    // Trigger -> card in one move: the trigger's leave schedules the close, the card's enter cancels it.
    const box = (await measureAtRest(surface(page)));
    await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
    await page.waitForTimeout(900); // 3x close_delay
    await expect(openSurface(page)).toHaveCount(1);

    await clearProbe(page);
    await page.mouse.move(0, 0);
    await expect(openSurface(page)).toHaveCount(0);
    expect(elapsed(await readProbe(page), "mouseout", "surface:closed")!).toBeGreaterThanOrEqual(300 - SLOP);
  });

  test("a scroll closes an open card (close-on-scroll)", async ({ page }) => {
    await gotoHydrated(page, URL, { timeout: 20 * 60 * 1000 });
    await page.evaluate(() => (document.body.style.paddingBottom = "3000px"));
    await trigger(page, "Quick (10 / 100)").hover();
    await expect(surface(page)).toBeVisible();
    await page.waitForTimeout(450); // past the helper's quiet window after an open
    await page.evaluate(() => window.scrollBy(0, 200));
    await expect(openSurface(page)).toHaveCount(0, { timeout: 500 });
  });
});

// dev-docs/backlog.md row 478 -- hover is a mouse's (and a pen's): a touch pointer does not start the open
// (Base UI `useHoverReferenceInteraction`).
test("a pointerenter of a touch pointer does not start the open; a mouse pointerenter does", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/hover_card/`, { timeout: 20 * 60 * 1000 });
  const trigger = page.getByRole("button", { name: "Dioxus" });
  await trigger.dispatchEvent("pointerenter", { pointerType: "touch", pointerId: 7, isPrimary: true });
  await page.waitForTimeout(1000); // past the default 600 ms delay
  await expect(page.getByRole("tooltip")).toHaveCount(0);

  await trigger.dispatchEvent("pointerenter", { pointerType: "mouse", pointerId: 1, isPrimary: true });
  await expect(page.getByRole("tooltip")).toBeVisible();
});
