/**
 * Responsive Dialog-to-Drawer (backlog row 156; shadcn's "Responsive Dialog"). Demo:
 * `variants/responsive_dialog`: a `Dialog` at and above the mobile breakpoint, a `Drawer` below it
 * (`use_is_mobile`, the Sidebar's hook, `MOBILE_BREAKPOINT` = 768px), the same form in both.
 */
import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { demoRole, demoFrame } from "./demo-frame";
import { awaitAnimationsSettled } from "./animations";

const URL = `${BASE_URL}/component/drawer/`;
const GOTO_OPTS = { timeout: 20 * 60 * 1000 };
const VARIANT = "responsive_dialog";
const DIALOG = '[data-slot="dialog-content"]';
const DRAWER = '[data-slot="drawer-content"]';

async function openAt(page: Page, width: number) {
  await page.setViewportSize({ width, height: 800 });
  await gotoHydrated(page, URL, GOTO_OPTS);
  await demoRole(page, "button", 0, VARIANT).click();
}

test("at 768px and wider the demo opens a Dialog, with the form", async ({ page }) => {
  await openAt(page, 768);
  await expect(page.locator(DIALOG)).toBeVisible();
  await expect(page.locator(DRAWER)).toHaveCount(0);
  await expect(page.locator(DIALOG).getByRole("textbox", { name: "Email" })).toBeVisible();
});

test("below 768px it opens a Drawer, with the same form", async ({ page }) => {
  await openAt(page, 767);
  await expect(page.locator(DRAWER)).toBeVisible();
  await expect(page.locator(DIALOG)).toHaveCount(0);
  await awaitAnimationsSettled(page.locator(DRAWER));
  await expect(page.locator(DRAWER).getByRole("textbox", { name: "Email" })).toBeVisible();
  await expect(page.locator(DRAWER).getByRole("textbox", { name: "Username" })).toBeVisible();
});

test("the switch follows the viewport live: resize across the breakpoint, then open", async ({ page }) => {
  await openAt(page, 1100);
  await expect(page.locator(DIALOG)).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(page.locator(DIALOG)).toHaveCount(0);

  await page.setViewportSize({ width: 600, height: 800 });
  await demoRole(page, "button", 0, VARIANT).click();
  await expect(page.locator(DRAWER)).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(page.locator(DRAWER)).toHaveCount(0);

  await page.setViewportSize({ width: 900, height: 800 });
  await demoRole(page, "button", 0, VARIANT).click();
  await expect(page.locator(DIALOG)).toBeVisible();
});

test("both forms close on Save and the trigger keeps its place", async ({ page }) => {
  for (const [width, panel] of [[1100, DIALOG], [500, DRAWER]] as const) {
    await openAt(page, width);
    await expect(page.locator(panel)).toBeVisible();
    await page.locator(panel).getByRole("button", { name: "Save changes" }).click();
    await expect(page.locator(panel)).toHaveCount(0);
    await expect(demoFrame(page, VARIANT)).toBeVisible();
  }
});
