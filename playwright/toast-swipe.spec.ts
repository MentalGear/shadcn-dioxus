import { test, expect } from "./fixtures";
import { type Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { measureAtRest } from "./animations";
import { fixedViewportBox } from "./viewport-box";

/** Each toast demo owns one provider whose region carries a distinct name (axe landmark-unique). */
const region = (page: Page, name: string) => page.getByRole("region", { name });
const toasts = (page: Page, name: string) => region(page, name).getByRole("alertdialog");
const open = (page: Page) => page.goto(`${BASE_URL}/component/?name=toast&`);

const R = "Swipe toast notifications";

async function show(page: Page) {
  await open(page);
  await page.getByRole("button", { name: "Show swipeable toast" }).click();
  const t = toasts(page, R);
  await expect(t).toHaveCount(1);
  return t;
}

/** Press at the toast's left part (away from its buttons), drag by dx in `steps` moves, `pause` ms apart. */
async function drag(page: Page, t: ReturnType<typeof toasts>, dx: number, steps: number, pause: number) {
  const b = await measureAtRest(t);
  const x = b.x + 24;
  const y = b.y + b.height / 2;
  await page.mouse.move(x, y);
  await page.mouse.down();
  for (let i = 1; i <= steps; i++) {
    await page.mouse.move(x + (dx * i) / steps, y);
    if (pause) await page.waitForTimeout(pause);
  }
  await page.mouse.up();
}

test.describe("toast: swipe to dismiss (row 138 step 4)", () => {
  test("a long drag toward the edge dismisses", async ({ page }) => {
    const t = await show(page);
    await drag(page, t, 120, 6, 30);
    await expect(t).toHaveCount(0);
  });

  test("a short slow drag springs back", async ({ page }) => {
    const t = await show(page);
    const before = await measureAtRest(t);
    await drag(page, t, 20, 10, 80);
    await expect(t).toHaveCount(1);
    await expect(t).not.toHaveAttribute("data-swiping", /.+/);
    const after = await measureAtRest(t);
    expect(Math.abs(after.x - before.x)).toBeLessThan(1);
  });

  test("a short fast flick (velocity) dismisses", async ({ page }) => {
    const t = await show(page);
    await drag(page, t, 25, 1, 0);
    await expect(t).toHaveCount(0);
  });

  test("the toast follows the pointer while dragging", async ({ page }) => {
    const t = await show(page);
    const before = await measureAtRest(t);
    const x = before.x + 24;
    const y = before.y + before.height / 2;
    await page.mouse.move(x, y);
    await page.mouse.down();
    await page.mouse.move(x + 30, y, { steps: 3 });
    await expect(t).toHaveAttribute("data-swiping", "true");
    await expect.poll(async () => (await t.boundingBox())!.x - before.x).toBeGreaterThan(20);
    await page.mouse.move(x, y, { steps: 3 });
    await page.mouse.up();
    await expect(t).toHaveCount(1);
  });

  test("a drag away from the edge never dismisses", async ({ page }) => {
    const t = await show(page);
    await drag(page, t, -150, 6, 30);
    await expect(t).toHaveCount(1);
  });

  test("a press on the close button is still a click, not a swipe", async ({ page }) => {
    const t = await show(page);
    await t.getByRole("button", { name: "close", exact: true }).click();
    await expect(t).toHaveCount(0);
  });

  test("RTL: the toaster sits at the left edge and swipes dismiss leftward", async ({ page }) => {
    await open(page);
    // Direction is inherited: flip the toaster's whole ancestor chain (a demo frame may pin its own dir).
    await region(page, R).evaluate((el) => {
      for (let n: HTMLElement | null = el as HTMLElement; n; n = n.parentElement) n.setAttribute("dir", "rtl");
    });
    await page.getByRole("button", { name: "Show swipeable toast" }).click();
    const t = toasts(page, R);
    await expect(t).toHaveCount(1);
    const b = await measureAtRest(t);
    const vw = (await fixedViewportBox(page)).width;
    expect(b.x + b.width / 2, "RTL toaster is on the left half").toBeLessThan(vw / 2);
    await drag(page, t, 120, 6, 30); // toward the right: away from the edge
    await expect(t).toHaveCount(1);
    await drag(page, t, -120, 6, 30); // toward the left edge
    await expect(t).toHaveCount(0);
  });
});
