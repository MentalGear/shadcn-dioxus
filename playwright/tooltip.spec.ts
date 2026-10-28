import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { startProbe, readProbe, clearProbe, elapsed } from "./hover-delay-probe";
import { expectNoAxeViolations, EXCLUDE_VENDORED_CODE_HIGHLIGHT } from "./axe";
import { sampleCloseFade, assertFadesOutThenUnmounts } from "./assert-fade-out";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { awaitAnimationsSettled, measureAtRest, readAtRest } from "./animations";

test("test", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/tooltip/`);
  let tooltip = page.getByRole("tooltip");
  // tabbing to the trigger element should show the tooltip
  await page.locator("#component-preview-frame").focus();
  await page.keyboard.press("Tab");
  await expect(tooltip).toBeVisible();
  // tabbing out of the trigger element should hide the tooltip
  await page.keyboard.press("Tab");
  await expect(tooltip).toHaveCount(0);

  // hovering over the trigger element should show the tooltip
  await page.locator("#component-preview-frame").first().getByText("Rich content").hover();
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
  await gotoHydrated(page, `${BASE_URL}/component/tooltip/`);
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
  test("tooltip open has no automatically detectable a11y issues", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/tooltip/`);
    await page.locator("#component-preview-frame").first().getByText("Rich content").hover();
    await expect(page.getByRole("tooltip")).toBeVisible();
    await expectNoAxeViolations(page, "tooltip: open", { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });
});

// dev-docs/backlog.md rows 19 and 7: on the web (popover) arm,
// `use_popover_sync` must not call `hidePopover()` the instant `open` goes
// false: the UA's `[popover]:not(:popover-open) { display: none }` rule would
// then hide the content before `dx-tooltip-fade-out`
// (`../preview/src/components/tooltip/style.css`, taken from upstream PR
// #293) ever got a chance to play (`display: none` from the very first
// sample, so `assertFadesOutThenUnmounts` fails both the "no display:none
// while mounted" invariant and the "a few frames of decreasing opacity"
// check). `primitives/src/top_layer/popover_sync.rs`'s `use_popover_shown_while_mounted`
// (row 19) keeps the popover shown through the animation that CSS defines
// (row 7).
test.describe("Close-fade animation (dev-docs/backlog.md rows 19, 7)", () => {
  test("content fades out (opacity -> 0, still popover-open) before unmounting", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/tooltip/`, { timeout: 20 * 60 * 1000 });
    const trigger = page.locator("#component-preview-frame").first().getByText("Rich content");
    await trigger.hover();
    const tooltip = page.getByRole("tooltip");
    await expect(tooltip).toBeVisible();
    const contentId = await tooltip.getAttribute("id");
    if (!contentId) throw new Error("tooltip content has no id to sample");

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

// dev-docs/backlog.md row 166 -- `delay` / `close_delay` (Radix `delayDuration`, Base UI
// `Tooltip.Trigger` `delay` / `closeDelay`; the primitive defaults to 600 / 0, the styled tooltip to 0 / 0
// like shadcn's `TooltipProvider`). The `delays` variant renders three tooltips: "Default (0)" (no
// `delay` prop), "Rests (600)" and "Lingers (0 / 600)". Delays are measured in the page
// (hover-delay-probe.ts) and only as a LOWER bound, so a slow debug build cannot turn them red.
test.describe("delay / close_delay (backlog row 166)", () => {
  const URL = `${BASE_URL}/component/tooltip/`;
  const SLOP = 40; // timer clamping / clock granularity between two performance.now() stamps
  const button = (page: Page, name: string) => page.getByRole("button", { name, exact: true });
  const surface = (page: Page) => page.getByRole("tooltip");
  const openSurface = (page: Page) => page.locator('[role="tooltip"][data-state="open"]');

  test("nothing opens before `delay`: a `delay: 600` tooltip waits 600 ms of pointer rest", async ({ page }) => {
    await gotoHydrated(page, URL, { timeout: 20 * 60 * 1000 });
    await startProbe(page);
    await button(page, "Rests (600)").hover();
    await expect(surface(page)).toBeVisible();
    const waited = elapsed(await readProbe(page), "mouseover", "surface:open");
    expect(waited, "the tooltip opened").not.toBeNull();
    expect(waited!).toBeGreaterThanOrEqual(600 - SLOP);
  });

  test("leaving before `delay` cancels the open (a sweep across triggers flashes nothing)", async ({ page }) => {
    await gotoHydrated(page, URL, { timeout: 20 * 60 * 1000 });
    await startProbe(page);
    await button(page, "Rests (600)").hover();
    await page.waitForTimeout(150);
    await page.mouse.move(0, 0);
    // Well past the 600 ms the abandoned hover would have opened at.
    await page.waitForTimeout(1100);
    await expect(surface(page)).toHaveCount(0);
    expect((await readProbe(page)).some((e) => e.what === "surface:open")).toBe(false);
  });

  test("the styled default is delay 0: opens on contact (shadcn's TooltipProvider delay={0})", async ({ page }) => {
    await gotoHydrated(page, URL, { timeout: 20 * 60 * 1000 });
    await startProbe(page);
    await button(page, "Default (0)").hover();
    await expect(surface(page)).toBeVisible();
    // Less than the default's 600 ms; no tighter bound, which would be a debug-build timing claim.
    expect(elapsed(await readProbe(page), "mouseover", "surface:open")!).toBeLessThan(600);
  });

  test("keyboard focus opens at once even with a delay", async ({ page }) => {
    await gotoHydrated(page, URL, { timeout: 20 * 60 * 1000 });
    await startProbe(page);
    await button(page, "Rests (600)").focus();
    await expect(surface(page)).toBeVisible();
    expect(elapsed(await readProbe(page), "focusin", "surface:open")!).toBeLessThan(600);
  });

  test("blur closes at once despite a close_delay", async ({ page }) => {
    await gotoHydrated(page, URL, { timeout: 20 * 60 * 1000 });
    await startProbe(page);
    await button(page, "Lingers (0 / 600)").focus();
    await expect(surface(page)).toBeVisible();
    await clearProbe(page);
    await page.evaluate(() => (document.activeElement as HTMLElement).blur());
    await expect(openSurface(page)).toHaveCount(0);
    expect(elapsed(await readProbe(page), "focusout", "surface:closed")!).toBeLessThan(600);
  });

  test("close_delay holds the tooltip open after the pointer leaves, and entering it cancels the close", async ({
    page,
  }) => {
    await gotoHydrated(page, URL, { timeout: 20 * 60 * 1000 });
    const trigger = button(page, "Lingers (0 / 600)");
    await startProbe(page);
    await trigger.hover();
    await expect(surface(page)).toBeVisible();

    // 1. leave -> closes, but not before close_delay
    await clearProbe(page);
    await page.mouse.move(0, 0);
    await expect(openSurface(page)).toHaveCount(0);
    expect(elapsed(await readProbe(page), "mouseout", "surface:closed")!).toBeGreaterThanOrEqual(600 - SLOP);

    // 2. leave the trigger for the tooltip itself -> the pending close is cancelled
    await expect(surface(page)).toHaveCount(0);
    await trigger.hover();
    await expect(surface(page)).toBeVisible();
    const box = (await measureAtRest(surface(page)));
    await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
    await page.waitForTimeout(1000); // > close_delay
    await expect(openSurface(page)).toHaveCount(1);

    // 3. leaving the tooltip starts the close again
    await page.mouse.move(0, 0);
    await expect(openSurface(page)).toHaveCount(0);
  });

  test("a scroll closes an open tooltip at once, whatever its close_delay (close-on-scroll)", async ({ page }) => {
    await gotoHydrated(page, URL, { timeout: 20 * 60 * 1000 });
    await page.evaluate(() => (document.body.style.paddingBottom = "3000px"));
    const trigger = button(page, "Lingers (0 / 600)");
    await startProbe(page);
    await trigger.hover();
    await expect(surface(page)).toBeVisible();
    await page.waitForTimeout(450); // past the helper's quiet window after an open
    await page.evaluate(() => window.scrollBy(0, 200));
    // Sooner than close_delay (600 ms) could have elapsed even if the scroll had moved the pointer
    // off the trigger at once: the close is the scroll's, not the timer's.
    await expect(openSurface(page)).toHaveCount(0, { timeout: 500 });
  });
});

// dev-docs/backlog.md row 292 -- the "Rich content" demo stacks its title and body. The
// body is a flex row (icon-plus-text and Kbd tooltips rely on it), so a `display: inline-flex`
// keyed on `[data-state="open"]` alone would stack the paragraphs ONLY during the closing fade, when
// the rule stops matching. Two invariants: the demo stacks them at rest, and a tooltip lays out
// the same in the closed (fading) state as in the open one.
test.describe("Rich content layout (row 292)", () => {
  test("title and body are stacked (title above body) at rest, not side by side", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/tooltip/`);
    await page.locator("#component-preview-frame").first().getByText("Rich content").hover();
    const tooltip = page.getByRole("tooltip");
    await expect(tooltip).toBeVisible();
    const rects = await readAtRest(tooltip, (el) => {
      const [title, body] = Array.from(el.querySelectorAll("p")).map((p) => p.getBoundingClientRect());
      return { title: { top: title.top, bottom: title.bottom, left: title.left }, body: { top: body.top, left: body.left } };
    });
    expect(rects.title.bottom).toBeLessThanOrEqual(rects.body.top + 0.5);
    expect(Math.abs(rects.title.left - rects.body.left)).toBeLessThan(1);
  });

  test("the display (and so the layout) is the same while the closing fade plays", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/tooltip/`);
    await page.locator("#component-preview-frame").first().getByText("Rich content").hover();
    const tooltip = page.getByRole("tooltip");
    await expect(tooltip).toBeVisible();
    const contentId = await tooltip.getAttribute("id");
    if (!contentId) throw new Error("tooltip content has no id to sample");
    await awaitAnimationsSettled(tooltip);
    const openDisplay = await tooltip.evaluate((el) => getComputedStyle(el).display);
    // `inline-flex` computes to `flex` on an absolutely positioned box; what matters is that it is a flex row.
    expect(openDisplay).toBe("flex");

    const capture = await sampleCloseFade(page, contentId, () => page.mouse.move(0, 0));
    const mounted = capture.samples.filter((s) => s.exists);
    expect(mounted.length).toBeGreaterThan(0);
    for (const s of mounted) expect(s.display).toBe(openDisplay);
  });
});

// dev-docs/backlog.md row 478 -- Base UI's tooltip defaults: a press on the trigger closes the tooltip
// (`closeOnClick`) and the focus that press moves onto the trigger does not reopen it, a touch pointer
// never opens it, and the panel sits 4px from the trigger (shadcn base-nova `sideOffset={4}`).
test.describe("Base UI defaults (backlog row 478)", () => {
  const URL = `${BASE_URL}/component/tooltip/`;
  const button = (page: Page, name: string) => page.getByRole("button", { name, exact: true });
  const surface = (page: Page) => page.getByRole("tooltip");

  test("a press on the trigger closes the open tooltip and keeps it closed", async ({ page }) => {
    await gotoHydrated(page, URL, { timeout: 20 * 60 * 1000 });
    const trigger = button(page, "Default (0)");
    await trigger.hover();
    await expect(surface(page)).toBeVisible();

    await page.mouse.down();
    await expect(surface(page), "the press closes the tooltip").toHaveCount(0);
    await page.mouse.up();
    await page.waitForTimeout(400);
    await expect(trigger, "the press focused the trigger").toBeFocused();
    await expect(surface(page), "the focus the press moved onto the trigger does not reopen it").toHaveCount(0);

    // Keyboard focus still opens it at once.
    await page.mouse.move(0, 0);
    await page.keyboard.press("Tab");
    await page.keyboard.press("Shift+Tab");
    await expect(trigger).toBeFocused();
    await expect(surface(page), "keyboard focus opens the tooltip").toBeVisible();
  });

  test("a pointerenter of a touch pointer does not start the open; a mouse pointerenter does", async ({ page }) => {
    await gotoHydrated(page, URL, { timeout: 20 * 60 * 1000 });
    const trigger = button(page, "Default (0)");
    await trigger.dispatchEvent("pointerenter", { pointerType: "touch", pointerId: 7, isPrimary: true });
    await page.waitForTimeout(500);
    await expect(surface(page), "a touch pointerenter opens nothing (delay 0 would open at once)").toHaveCount(0);

    await trigger.dispatchEvent("pointerenter", { pointerType: "mouse", pointerId: 1, isPrimary: true });
    await expect(surface(page), "a mouse pointerenter opens it").toBeVisible();
  });

  test.describe("on a touch screen", () => {
    test.use({ hasTouch: true });

    test("a tap does not open the tooltip, by hover or by the focus it moves onto the trigger", async ({ page }) => {
      await gotoHydrated(page, URL, { timeout: 20 * 60 * 1000 });
      await button(page, "Default (0)").tap();
      await page.waitForTimeout(600);
      await expect(surface(page)).toHaveCount(0);
    });
  });

  test("the panel sits 4px from the trigger", async ({ page }) => {
    await gotoHydrated(page, URL, { timeout: 20 * 60 * 1000 });
    const trigger = button(page, "Default (0)");
    await trigger.hover();
    const tooltip = surface(page);
    await expect(tooltip).toBeVisible();
    await awaitAnimationsSettled(tooltip);
    const [t, p] = [await measureAtRest(trigger), await measureAtRest(tooltip)];
    // The default side is `top`; the gap is read along whichever axis separates the two boxes.
    const vertical = Math.max(t.y - (p.y + p.height), p.y - (t.y + t.height));
    const horizontal = Math.max(t.x - (p.x + p.width), p.x - (t.x + t.width));
    const gap = Math.max(vertical, horizontal);
    expect(gap, "side offset 4px").toBeGreaterThan(3.5);
    expect(gap, "side offset 4px").toBeLessThan(4.5);
  });
});
