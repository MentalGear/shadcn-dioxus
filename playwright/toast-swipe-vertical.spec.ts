import { test, expect } from "./fixtures";
import { type Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { measureAtRest } from "./animations";

const TOP = "Top toast notifications"; // toaster at the top of the viewport
const BOTTOM = "Swipe toast notifications"; // toaster at the bottom (the default)
const toasts = (page: Page, name: string) => page.getByRole("region", { name }).getByRole("alertdialog");

async function show(page: Page, button: string, region: string) {
  await page.goto(`${BASE_URL}/component/?name=toast&`);
  await page.getByRole("button", { name: button }).click();
  const t = toasts(page, region);
  await expect(t).toHaveCount(1);
  return t;
}

async function drag(page: Page, t: ReturnType<typeof toasts>, dx: number, dy: number, steps = 6, pause = 30) {
  const b = await measureAtRest(t);
  const x = b.x + 24;
  const y = b.y + b.height / 2;
  await page.mouse.move(x, y);
  await page.mouse.down();
  for (let i = 1; i <= steps; i++) {
    await page.mouse.move(x + (dx * i) / steps, y + (dy * i) / steps);
    if (pause) await page.waitForTimeout(pause);
  }
  await page.mouse.up();
}

test.describe("toast: vertical swipe (row 263 item 2)", () => {
  test("a toaster at the top dismisses by swiping up", async ({ page }) => {
    const t = await show(page, "Show top toast", TOP);
    expect((await measureAtRest(t)).y).toBeLessThan(100); // really at the top
    await drag(page, t, 0, -80);
    await expect(t).toHaveCount(0);
  });

  test("a toaster at the bottom dismisses by swiping down", async ({ page }) => {
    const t = await show(page, "Show swipeable toast", BOTTOM);
    await drag(page, t, 0, 80);
    await expect(t).toHaveCount(0);
  });

  test("the wrong vertical direction is resisted and springs back", async ({ page }) => {
    const t = await show(page, "Show top toast", TOP);
    const before = await measureAtRest(t);
    await drag(page, t, 0, 90);
    await expect(t).toHaveCount(1);
    await expect(t).not.toHaveAttribute("data-swiping", /.+/);
    const after = await measureAtRest(t);
    expect(Math.abs(after.y - before.y)).toBeLessThan(1);
  });

  test("a short slow vertical drag springs back; the toast follows the pointer meanwhile", async ({ page }) => {
    const t = await show(page, "Show top toast", TOP);
    const before = await measureAtRest(t);
    const b = before;
    await page.mouse.move(b.x + 24, b.y + b.height / 2);
    await page.mouse.down();
    for (let i = 1; i <= 10; i++) {
      await page.mouse.move(b.x + 24, b.y + b.height / 2 - i * 2);
      await page.waitForTimeout(80);
    }
    await expect(t).toHaveAttribute("data-swiping", "true");
    const mid = await t.boundingBox();
    expect(before.y - mid!.y).toBeGreaterThan(10);
    await page.mouse.up();
    await expect(t).toHaveCount(1);
    await expect(t).not.toHaveAttribute("data-swiping", /.+/);
    expect(Math.abs((await measureAtRest(t)).y - before.y)).toBeLessThan(1);
  });

  test("a horizontal drag still dismisses a top toast (the axis follows the first movement)", async ({ page }) => {
    const t = await show(page, "Show top toast", TOP);
    await drag(page, t, 120, 0);
    await expect(t).toHaveCount(0);
  });
});
