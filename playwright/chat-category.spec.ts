/**
 * Row 148: the docs sidebar has a "Chat" category holding the chat kit, instead of burying five chat parts in
 * "Data display" beside Table, Avatar and Badge. `category_of` (preview/src/components/mod.rs) is the one
 * mapping; the sidebar loops `ComponentCategory::ALL`, so these specs read the rendered result.
 */
import { test, expect } from "./fixtures";
import type { Page } from "@playwright/test";
import { gotoHydrated } from "./hydration";
import { BASE_URL } from "./base-url";

const PAGE = `${BASE_URL}/component/button/`;
const CHAT = ["attachment", "bubble", "markdown", "marker", "message", "message scroller"];

const group = (page: Page, label: string) =>
  page.locator(".dx-nav-category").filter({
    has: page.locator(".dx-sidebar-group-label", { hasText: new RegExp(`^\\s*${label}\\s*$`) }),
  });

async function names(page: Page, label: string): Promise<string[]> {
  const items = await group(page, label).locator(".dx-sidebar-menu-item a").evaluateAll((els) =>
    els.map((el) => (el.firstChild?.textContent ?? "").trim()),
  );
  return items.sort();
}

test("the category view has a Chat group between Data display and Charts, holding the chat kit", async ({ page }) => {
  await gotoHydrated(page, PAGE);
  await page.locator('.dx-nav-sort-option[data-nav-sort-option="category"]').click();
  const labels = (await page.locator(".dx-nav-category .dx-sidebar-group-label").allTextContents()).map((l) => l.trim());
  expect(labels.indexOf("Chat")).toBe(labels.indexOf("Data display") + 1);
  expect(labels.indexOf("Charts")).toBe(labels.indexOf("Chat") + 1);
  await expect(group(page, "Chat")).toBeVisible();
  expect(await names(page, "Chat")).toEqual(CHAT);
  await expect(group(page, "Chat").locator("svg").first()).toBeVisible();
});

test("the chat parts are no longer in Data display, and Data display keeps its own", async ({ page }) => {
  await gotoHydrated(page, PAGE);
  await page.locator('.dx-nav-sort-option[data-nav-sort-option="category"]').click();
  const data = await names(page, "Data display");
  for (const chat of CHAT) expect(data, `${chat} left Data display`).not.toContain(chat);
  for (const kept of ["avatar", "table", "card", "data table"]) expect(data).toContain(kept);
});

test("a chat component's page links work from the Chat group", async ({ page }) => {
  await gotoHydrated(page, PAGE);
  await page.locator('.dx-nav-sort-option[data-nav-sort-option="category"]').click();
  await group(page, "Chat").getByRole("link", { name: "bubble" }).click();
  await expect(page).toHaveURL(/\/component\/bubble/);
});
