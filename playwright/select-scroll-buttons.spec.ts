import { test, expect } from "./fixtures";
import type { Locator, Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";
import { awaitAnimationsSettled, measureAtRest } from "./animations";

// Row 268: Select scroll buttons. The `scroll` variant has 24 options, more than the popup's 18rem max-height shows.

const trigger = (page: Page) => page.getByRole("combobox", { name: "Time zone select" });
const list = (page: Page) => page.getByRole("listbox", { name: "Time zone options" });
const up = (page: Page) => list(page).locator('[data-slot="select-scroll-button"][data-direction="up"]');
const down = (page: Page) => list(page).locator('[data-slot="select-scroll-button"][data-direction="down"]');
const scrollTop = (l: Locator) => l.evaluate((el) => el.scrollTop);

async function open(page: Page) {
  await gotoHydrated(page, `${BASE_URL}/component/select/`);
  await trigger(page).scrollIntoViewIfNeeded();
  await trigger(page).click();
  await expect(list(page)).toHaveAttribute("data-state", "open");
  await awaitAnimationsSettled(list(page));
}

test("the list scrolls inside the popup and only the down button shows at the top", async ({ page }) => {
  await open(page);
  const scrolls = await list(page).evaluate((el) => el.scrollHeight > el.clientHeight + 10);
  expect(scrolls, "24 options do not fit in the popup").toBe(true);
  await expect(down(page)).toHaveAttribute("data-visible", "true");
  await expect(up(page)).toHaveAttribute("data-visible", "false");
  await expect(down(page)).toBeVisible();
  await expect(up(page)).toBeHidden();
  // Hidden from assistive technology, and not a listbox child the options tree has to account for.
  await expect(down(page)).toHaveAttribute("aria-hidden", "true");
});

test("the buttons sit on the list's edges and never move an option", async ({ page }) => {
  await open(page);
  const first = list(page).getByRole("option").first();
  const before = await measureAtRest(first);
  const l = await measureAtRest(list(page));
  const d = await measureAtRest(down(page));
  expect(Math.abs(d.y + d.height - (l.y + l.height - 1)), "down button is at the list's bottom edge").toBeLessThanOrEqual(8);
  // Scroll to the bottom: the up button appears; the options' layout is the list's scroll, not the button's.
  await list(page).evaluate((el) => (el.scrollTop = 40));
  await expect(up(page)).toHaveAttribute("data-visible", "true");
  const u = await measureAtRest(up(page));
  expect(Math.abs(u.y - l.y), "up button is at the list's top edge").toBeLessThanOrEqual(8);
  await list(page).evaluate((el) => (el.scrollTop = 0));
  await expect(up(page)).toHaveAttribute("data-visible", "false");
  const after = await measureAtRest(first);
  expect(after.y).toBeCloseTo(before.y, 0);
});

test("scrolling to the end swaps which button shows", async ({ page }) => {
  await open(page);
  await list(page).evaluate((el) => (el.scrollTop = el.scrollHeight));
  await expect(down(page)).toHaveAttribute("data-visible", "false");
  await expect(up(page)).toHaveAttribute("data-visible", "true");
});

test("hovering the down button scrolls the list, leaving stops it, and it stops at the end", async ({ page }) => {
  await open(page);
  const l = list(page);
  expect(await scrollTop(l)).toBe(0);
  const box = await measureAtRest(down(page));
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await expect.poll(() => scrollTop(l), { timeout: 5000 }).toBeGreaterThan(40);
  // Away from the button: the scroll holds still.
  await page.mouse.move(box.x + box.width / 2, box.y - 120);
  const held = await scrollTop(l);
  await page.waitForTimeout(250);
  expect(Math.abs((await scrollTop(l)) - held), "leaving stops the auto-scroll").toBeLessThanOrEqual(1);
  // Back on it, all the way down: it ends at the bottom and the button hides.
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await expect(down(page)).toHaveAttribute("data-visible", "false", { timeout: 15_000 });
  const max = await l.evaluate((el) => el.scrollHeight - el.clientHeight);
  expect(Math.abs((await scrollTop(l)) - max)).toBeLessThanOrEqual(2);
});

test("hovering the up button scrolls back to the top", async ({ page }) => {
  await open(page);
  const l = list(page);
  await l.evaluate((el) => (el.scrollTop = el.scrollHeight));
  await expect(up(page)).toHaveAttribute("data-visible", "true");
  const box = await measureAtRest(up(page));
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await expect(up(page)).toHaveAttribute("data-visible", "false", { timeout: 15_000 });
  expect(await scrollTop(l)).toBeLessThanOrEqual(2);
});

test("a list that fits shows no scroll button, and a closed select renders none", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/select/`);
  await expect(page.locator('[data-slot="select-scroll-button"]')).toHaveCount(0);
  const sizes = page.getByRole("combobox", { name: "Small size select" });
  await sizes.scrollIntoViewIfNeeded();
  await sizes.click();
  const l = page.getByRole("listbox", { name: "Small size options" });
  await expect(l).toHaveAttribute("data-state", "open");
  await expect(l.locator('[data-slot="select-scroll-button"][data-visible="true"]')).toHaveCount(0);
});
