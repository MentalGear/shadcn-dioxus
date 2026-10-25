/**
 * HoverCard `force_mount` (backlog row 425): Radix's `forceMount` keeps the content in the DOM while the card is
 * closed, hidden (`hidden`, `data-state="closed"`); without it a closed card is not rendered at all. The
 * `force_mount` variant is a quick card (10 ms open delay, 100 ms close delay) whose content is force-mounted.
 * Controls: the main demo (no `force_mount`) has no card in the DOM while closed.
 */
import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { measureAtRest } from "./animations";
import { demoFrame, demoRole } from "./demo-frame";

const URL = `${BASE_URL}/component/hover_card/`;
const NAV_TIMEOUT = 20 * 60 * 1000;
const VARIANT = "force_mount";

// The card is `display: none` while closed, so the role query has to include hidden elements.
const card = (page: Page, variant: string) => demoFrame(page, variant).getByRole("tooltip", { includeHidden: true });
const trigger = (page: Page) => demoRole(page, "button", 0, VARIANT);

test.describe("HoverCard force_mount (backlog row 425)", () => {
  test.beforeEach(async ({ page }) => {
    await gotoHydrated(page, URL, { timeout: NAV_TIMEOUT });
  });

  test("a force-mounted card is in the DOM while closed, hidden", async ({ page }) => {
    const c = card(page, VARIANT);
    await expect(c).toHaveCount(1);
    await expect(c).toBeHidden();
    await expect(c).toHaveAttribute("data-state", "closed");
    await expect(c).toHaveAttribute("hidden", /.*/);
    await expect(c).not.toHaveText("");
  });

  test("it opens on hover and returns to the DOM-but-hidden state after its exit, every time", async ({ page }) => {
    const c = card(page, VARIANT);
    for (let round = 0; round < 2; round++) {
      await trigger(page).hover();
      await expect(c).toBeVisible();
      await expect(c).toHaveAttribute("data-state", "open");
      await expect(c).not.toHaveAttribute("hidden", /.*/);

      await page.mouse.move(0, 0);
      await expect(c).toBeHidden();
      await expect(c).toHaveCount(1);
      await expect(c).toHaveAttribute("data-state", "closed");
      await expect(c).toHaveAttribute("hidden", /.*/);
    }
  });

  test("the open card sits below its trigger, as without force_mount", async ({ page }) => {
    await trigger(page).hover();
    const c = card(page, VARIANT);
    await expect(c).toBeVisible();
    const cardBox = await measureAtRest(c);
    const triggerBox = await measureAtRest(trigger(page));
    expect(cardBox.y).toBeGreaterThanOrEqual(triggerBox.y + triggerBox.height - 1);
  });

  test("Escape on the focused trigger closes it and leaves the hidden card mounted", async ({ page }) => {
    const c = card(page, VARIANT);
    await trigger(page).focus();
    await expect(c).toBeVisible();
    await page.keyboard.press("Escape");
    await expect(c).toBeHidden();
    await expect(c).toHaveCount(1);
  });

  test("control: the default card is not in the DOM at all while closed", async ({ page }) => {
    await expect(card(page, "main")).toHaveCount(0);
    await demoRole(page, "button", 0, "main").hover();
    await expect(demoFrame(page, "main").getByRole("tooltip")).toBeVisible();
    await page.mouse.move(0, 0);
    await expect(card(page, "main")).toHaveCount(0);
  });
});
