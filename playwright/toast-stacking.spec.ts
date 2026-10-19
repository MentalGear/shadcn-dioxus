import { test, expect } from "./fixtures";
import { type Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { measureAtRest } from "./animations";

/** Each toast demo owns one provider whose region carries a distinct name (axe landmark-unique). */
const region = (page: Page, name: string) => page.getByRole("region", { name });
const toasts = (page: Page, name: string) => region(page, name).getByRole("alertdialog");
const open = (page: Page) => page.goto(`${BASE_URL}/component/?name=toast&`);

const R = "Stacked toast notifications";

/** Height of the box that contains every toast of the stack, measured at rest. */
async function pileHeight(page: Page) {
  const t = toasts(page, R);
  const n = await t.count();
  let top = Infinity;
  let bottom = -Infinity;
  for (let i = 0; i < n; i++) {
    const b = await measureAtRest(t.nth(i));
    top = Math.min(top, b.y);
    bottom = Math.max(bottom, b.y + b.height);
  }
  return bottom - top;
}

async function addThree(page: Page) {
  const add = page.getByRole("button", { name: "Add to stack" });
  for (let i = 0; i < 3; i++) await add.click();
  await expect(toasts(page, R)).toHaveCount(3);
}

test.describe("toast: stacking (row 138)", () => {
  test("collapsed pile; focus expands it", async ({ page }) => {
    await open(page);
    await addThree(page);
    // Park the pointer away so :hover cannot expand it.
    await page.mouse.move(2, 2);
    const one = (await measureAtRest(toasts(page, R).first())).height;
    const collapsed = await pileHeight(page);
    expect(collapsed, "three toasts collapse into roughly one toast's height").toBeLessThan(one * 1.6);

    await toasts(page, R).first().focus();
    await expect.poll(async () => pileHeight(page)).toBeGreaterThan(one * 2.5);
  });

  test("hover expands the pile", async ({ page }) => {
    await open(page);
    await addThree(page);
    await page.mouse.move(2, 2);
    const one = (await measureAtRest(toasts(page, R).first())).height;
    const box = await measureAtRest(toasts(page, R).first());
    await page.mouse.move(box.x + 20, box.y + box.height / 2);
    await expect.poll(async () => pileHeight(page)).toBeGreaterThan(one * 2.5);
  });

  test("prefers-reduced-motion: no entrance animation, no restack transition", async ({ page }) => {
    await page.emulateMedia({ reducedMotion: "reduce" });
    await open(page);
    await addThree(page);
    const t = toasts(page, R);
    for (let i = 0; i < 3; i++) {
      const style = await t.nth(i).evaluate((el) => {
        const s = getComputedStyle(el);
        return { name: s.animationName, dur: s.transitionDuration };
      });
      expect(style.name).toBe("none");
      for (const d of style.dur.split(",")) expect(parseFloat(d)).toBeLessThanOrEqual(0.001);
    }
  });
});
