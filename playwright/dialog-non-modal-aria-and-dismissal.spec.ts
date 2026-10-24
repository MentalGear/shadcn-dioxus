/**
 * Row 264: a non-modal panel reports no aria-modal (modal still does), and
 * `disable_pointer_dismissal` makes an outside press act on the page without closing the drawer.
 * Demo: drawer `non_modal` (button 0 opens the default drawer, 1 is the page button, 2 opens the
 * drawer that ignores outside presses).
 */
import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { demoRole } from "./demo-frame";
import { awaitAnimationsSettled } from "./animations";

const GOTO_OPTS = { timeout: 20 * 60 * 1000 };
test.use({ viewport: { width: 390, height: 844 } });
const CONTENT = '[data-slot="drawer-content"]';
const ROOT = '[data-slot="drawer-root"]';

async function openNonModal(page: Page, index: number) {
  await gotoHydrated(page, `${BASE_URL}/component/drawer/`, GOTO_OPTS);
  await demoRole(page, "button", index, "non_modal").click();
  await expect(page.locator(ROOT)).toHaveAttribute("data-state", "open");
  await awaitAnimationsSettled(page.locator(CONTENT));
}

test("a non-modal panel has no aria-modal and passes axe", async ({ page }) => {
  await openNonModal(page, 0);
  await expect(page.locator(CONTENT)).toHaveAttribute("role", "dialog");
  await expect(page.locator(CONTENT)).not.toHaveAttribute("aria-modal", /.*/);
  const results = await new AxeBuilder({ page }).include(CONTENT).analyze();
  expect(results.violations.filter((v) => v.id.startsWith("aria-"))).toEqual([]);
});

test("a modal dialog has aria-modal=true", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/dialog/`, GOTO_OPTS);
  await demoRole(page, "button", 0).click();
  await expect(page.locator('dialog[open][role="dialog"]')).toHaveAttribute("aria-modal", "true");
});

test("disable_pointer_dismissal: an outside press acts on the page and the drawer stays", async ({ page }) => {
  await openNonModal(page, 2);
  const pageButton = demoRole(page, "button", 1, "non_modal");
  await expect(pageButton).toContainText("Page button: 0");
  await pageButton.click();
  await expect(pageButton).toContainText("Page button: 1");
  await expect(page.locator(ROOT)).toHaveAttribute("data-state", "open");
});

test("disable_pointer_dismissal: Escape and Close close it", async ({ page }) => {
  await openNonModal(page, 2);
  await page.keyboard.press("Escape");
  await expect(page.locator(ROOT)).toHaveCount(0);
  await demoRole(page, "button", 2, "non_modal").click();
  await expect(page.locator(ROOT)).toHaveAttribute("data-state", "open");
  await page.locator(CONTENT).getByRole("button", { name: "Close" }).click();
  await expect(page.locator(ROOT)).toHaveCount(0);
});
