import { test, expect } from "./fixtures";
import { type Locator, type Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

// Base UI's `thumbCollisionBehavior="push"` and `thumbAlignment="edge"` (row 482), as `ThumbCollision::Push`
// and `ThumbAlignment::Edge` on the primitives. The defaults (clamp, centre) are covered by slider.spec.ts.

const GOTO = { timeout: 20 * 60 * 1000 };
const thumbs = (page: Page, name: string) => page.getByRole("slider", { name });
const values = async (page: Page, name: string) =>
  Promise.all([0, 1].map(async (i) => Number(await thumbs(page, name).nth(i).getAttribute("aria-valuenow"))));

test.describe("ThumbCollision::Push", () => {
  test("a thumb that reaches the other one pushes it along, keeping min_steps_between_thumbs", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/slider/`, GOTO);
    const [start, end] = [thumbs(page, "Pushing Range").nth(0), thumbs(page, "Pushing Range").nth(1)];
    await expect(start).toHaveAttribute("aria-valuenow", "20");
    await expect(end).toHaveAttribute("aria-valuenow", "60");

    await start.focus();
    // Within the gap's reach nothing is pushed.
    await page.keyboard.press("ArrowRight");
    expect(await values(page, "Pushing Range")).toEqual([21, 60]);
    await page.keyboard.press("Shift+ArrowRight");
    await page.keyboard.press("Shift+ArrowRight");
    expect(await values(page, "Pushing Range")).toEqual([41, 60]);
    // 51 would leave 9 steps to the end thumb, so the end thumb moves to 61 and then 71.
    await page.keyboard.press("Shift+ArrowRight");
    expect(await values(page, "Pushing Range")).toEqual([51, 61]);
    await page.keyboard.press("Shift+ArrowRight");
    expect(await values(page, "Pushing Range")).toEqual([61, 71]);
    // End: the start thumb reaches the room the end thumb needs (max - gap), the end thumb is at max.
    await page.keyboard.press("End");
    expect(await values(page, "Pushing Range")).toEqual([90, 100]);

    await end.focus();
    await page.keyboard.press("Home");
    expect(await values(page, "Pushing Range")).toEqual([0, 10]);
  });

  test("dragging a thumb past the other pushes it, and the displayed range follows", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/slider/`, GOTO);
    const start = thumbs(page, "Pushing Range").nth(0);
    await start.scrollIntoViewIfNeeded();
    const box = (await start.boundingBox())!;
    const track = (await start.locator("xpath=..").boundingBox())!;
    const y = box.y + box.height / 2;
    await page.mouse.move(box.x + box.width / 2, y);
    await page.mouse.down();
    await page.mouse.move(track.x + track.width * 0.7, y, { steps: 8 });
    await page.mouse.up();
    const [a, b] = await values(page, "Pushing Range");
    expect(a).toBeGreaterThan(60);
    expect(b - a).toBe(10);
  });
});

test.describe("ThumbAlignment::Edge", () => {
  const rect = async (l: Locator) => (await l.boundingBox())!;

  test("the thumb's edge is on the track's end at min and at max", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/slider/`, GOTO);
    const thumb = thumbs(page, "Edge Slider");
    const track = thumb.locator("xpath=..");
    // The thumb's size is measured after mount, so wait for the position to settle.
    await expect
      .poll(async () => Math.round((await rect(thumb)).x - (await rect(track)).x), { timeout: 5000 })
      .toBe(0);
    await thumb.focus();
    await page.keyboard.press("End");
    await expect
      .poll(async () => {
        const [t, tr] = [await rect(thumb), await rect(track)];
        return Math.round(t.x + t.width - (tr.x + tr.width));
      })
      .toBe(0);
  });

  test("the range fills between the thumb centres", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/slider/`, GOTO);
    const [a, b] = [thumbs(page, "Edge Range").nth(0), thumbs(page, "Edge Range").nth(1)];
    const track = a.locator("xpath=..");
    const range = track.locator(".dx-slider-range");
    await expect
      .poll(async () => {
        const [ra, rb, rr] = [await rect(a), await rect(b), await rect(range)];
        return [Math.round(rr.x - (ra.x + ra.width / 2)), Math.round(rr.x + rr.width - (rb.x + rb.width / 2))];
      }, { timeout: 5000 })
      .toEqual([0, 0]);
  });

  test("a press at the track's end reaches min and max, and the thumb stays under the pointer", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/slider/`, GOTO);
    const thumb = thumbs(page, "Edge Slider");
    const track = thumb.locator("xpath=..");
    await thumb.scrollIntoViewIfNeeded();
    await expect.poll(async () => Math.round((await rect(thumb)).x - (await rect(track)).x)).toBe(0);
    const tr = await rect(track);
    const y = tr.y + tr.height / 2;
    await page.mouse.click(tr.x + tr.width - 1, y);
    await expect(thumb).toHaveAttribute("aria-valuenow", "100");
    await page.mouse.click(tr.x + 1, y);
    await expect(thumb).toHaveAttribute("aria-valuenow", "0");
    // Drag from the thumb's centre by half the travel (track less thumb): the value is 50 and the thumb is
    // centred on the pointer.
    const t = await rect(thumb);
    const startX = t.x + t.width / 2;
    const travel = tr.width - t.width;
    await page.mouse.move(startX, y);
    await page.mouse.down();
    await page.mouse.move(startX + travel / 2, y, { steps: 8 });
    await page.mouse.up();
    await expect(thumb).toHaveAttribute("aria-valuenow", "50");
    const after = await rect(thumb);
    expect(Math.abs(after.x + after.width / 2 - (startX + travel / 2))).toBeLessThan(2);
  });
});
