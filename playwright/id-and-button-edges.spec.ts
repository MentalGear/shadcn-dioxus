/**
 * Two edge cases of caller input (code-quality review F5 and F10, backlog row 436):
 *
 *   - a caller-supplied id with CSS and script punctuation (`odd.content:it's`) opens its popover and anchors it
 *     to its trigger: the id reaches `anchor-name`, the lookup selector and the show script escaped, not raw;
 *   - a press with a non-primary button (a middle press, a pen's barrel button) on a slider thumb starts no drag
 *     and is not consumed: its `pointerdown` is not default-prevented, while a primary press is (the control that
 *     proves the probe sees a handler's cancel).
 */
import { test, expect } from "./fixtures";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

const NAV_TIMEOUT = 20 * 60 * 1000;

test("a popover whose ids hold a dot, a colon and an apostrophe opens next to its trigger", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/top_layer/`, { timeout: NAV_TIMEOUT });
  const trigger = page.locator('[id="odd.trigger:it\'s"]');
  const content = page.locator('[id="odd.content:it\'s"]');
  await trigger.click();
  await expect(content).toBeVisible();
  await expect.poll(() => content.evaluate((el) => el.matches(":popover-open"))).toBe(true);
  // Anchored: the panel touches the trigger's column and sits within a panel-height of it, on either side.
  const gap = await page.evaluate(() => {
    const t = document.getElementById("odd.trigger:it's")!.getBoundingClientRect();
    const c = document.getElementById("odd.content:it's")!.getBoundingClientRect();
    return { dx: Math.max(0, Math.max(t.left, c.left) - Math.min(t.right, c.right)), dy: Math.min(Math.abs(c.top - t.bottom), Math.abs(t.top - c.bottom)) };
  });
  expect(gap.dx, "the panel overlaps the trigger horizontally").toBe(0);
  expect(gap.dy, "the panel sits against the trigger").toBeLessThan(24);
});

async function pointerdownProbe(page: import("@playwright/test").Page, button: "left" | "middle") {
  const thumb = page.getByRole("slider", { name: "Demo Slider" });
  const box = await thumb.boundingBox();
  if (!box) throw new Error("the thumb has no bounding box");
  await page.evaluate(() => {
    (window as any).__seen = null;
    window.addEventListener("pointerdown", (e) => ((window as any).__seen = { defaultPrevented: e.defaultPrevented }), { once: true });
  });
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.mouse.down({ button });
  await page.mouse.up({ button });
  return page.evaluate(() => (window as any).__seen as { defaultPrevented: boolean } | null);
}

test("a middle press on a slider thumb is left to the browser, a primary press is consumed", async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/slider/`, { timeout: NAV_TIMEOUT });
  const middle = await pointerdownProbe(page, "middle");
  expect(middle, "the press reaches the page above the slider").not.toBeNull();
  expect(middle!.defaultPrevented, "a middle press is not default-prevented").toBe(false);
  const primary = await pointerdownProbe(page, "left");
  expect(primary?.defaultPrevented, "a primary press starts a drag and is default-prevented").toBe(true);
});
