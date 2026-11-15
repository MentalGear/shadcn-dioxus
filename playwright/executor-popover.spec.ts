// The page executor's popover kind (row 449) on a menu host, over a slow bridge channel
// (`window.__dxBridgeDelayFrames`, both directions): Rust's open state and the page's `:popover-open` agree once the
// channel settles, and the level the owner decided last is the level the page shows. The executor never echoes its
// own show or hide, and a report the owner has decided over since is dropped (`primitives/src/bridge/executor.rs`).

import type { Page } from "@playwright/test";
import { expect, test } from "./fixtures";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const DELAY_FRAMES = 10;

/** Counts the menu's `:popover-open` flips over the next `frames` frames, then resolves with the count. */
const flipsOver = (page: Page, frames: number) =>
  page.evaluate(
    (n) =>
      new Promise<number>((done) => {
        const menu = document.querySelector('[role="menu"]');
        let last = !!menu?.matches(":popover-open");
        let flips = 0;
        const tick = (left: number) => {
          const now = !!document.querySelector('[role="menu"]')?.matches(":popover-open");
          if (now !== last) flips++;
          last = now;
          if (left <= 0) done(flips);
          else requestAnimationFrame(() => tick(left - 1));
        };
        requestAnimationFrame(() => tick(n));
      }),
    frames,
  );

async function open(page: Page) {
  await gotoHydrated(page, `${BASE_URL}/component/dropdown_menu/`);
  await page.evaluate((n) => ((window as any).__dxBridgeDelayFrames = n), DELAY_FRAMES);
  const trigger = page.getByRole("button", { name: "Open Menu", exact: true });
  const menu = page.locator('[role="menu"]');
  return { trigger, menu };
}

const popoverOpen = (page: Page) => page.evaluate(() => !!document.querySelector('[role="menu"]')?.matches(":popover-open"));

test.describe("the page executor on a menu host, slow channel", () => {
  test("a show followed by the owner's close stays closed: the show is not echoed back", async ({ page }) => {
    const { trigger } = await open(page);
    await trigger.evaluate((el) => (el as HTMLElement).click());
    await expect.poll(() => popoverOpen(page)).toBe(true);
    await trigger.evaluate((el) => (el as HTMLElement).click());
    const flips = await flipsOver(page, DELAY_FRAMES * 4);
    expect(flips, "the menu reopened after the close").toBeLessThanOrEqual(1);
    expect(await popoverOpen(page)).toBe(false);
    await expect(trigger).toHaveAttribute("aria-expanded", "false");
  });

  test("a close then a reopen between two reports ends open: the report of the close is dropped", async ({ page }) => {
    const { trigger } = await open(page);
    await trigger.evaluate((el) => (el as HTMLElement).click());
    await expect.poll(() => popoverOpen(page)).toBe(true);
    // the light dismiss is the page's own close; its report is still on the slow channel while the owner decides twice
    await page.keyboard.press("Escape");
    await trigger.evaluate(
      (el) =>
        new Promise<void>((done) => {
          (el as HTMLElement).click();
          requestAnimationFrame(() => requestAnimationFrame(() => ((el as HTMLElement).click(), done())));
        }),
    );
    await flipsOver(page, DELAY_FRAMES * 4);
    const expanded = await trigger.getAttribute("aria-expanded");
    expect(await popoverOpen(page), "the page and the owner disagree").toBe(expanded === "true");
    expect(await flipsOver(page, DELAY_FRAMES * 2), "the level still flips after the channel settled").toBe(0);
  });

  test("a light dismiss followed by the trigger's own click ends with the owner and the page agreeing", async ({ page }) => {
    const { trigger } = await open(page);
    await trigger.click();
    await expect.poll(() => popoverOpen(page)).toBe(true);
    // a real click: its pointerdown light-dismisses the menu, its click runs the trigger's own toggle
    await trigger.click();
    await flipsOver(page, DELAY_FRAMES * 4);
    const expanded = await trigger.getAttribute("aria-expanded");
    expect(await popoverOpen(page), "the page and the owner disagree").toBe(expanded === "true");
    expect(await flipsOver(page, DELAY_FRAMES * 2), "the level still flips after the channel settled").toBe(0);
  });
});
