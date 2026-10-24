import { test, expect } from "./fixtures";
import { type Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { measureAtRest } from "./animations";
import { gotoHydrated } from "./hydration";

// Touch taps on the Color Picker panel (on iOS a tap on the color
// area, and on other controls in it, must change the color). The color area and the hue slider share
// `MoveInteraction` + the window pointer bridge (`primitives/src/pointer.rs`); the failure mode they
// share is a pointer that stays "active" after its release was never seen, which makes every later
// pointerdown on that control be ignored. Real touch input only (`touchscreen.tap`, CDP touch events).

const PAGE_URL = `${BASE_URL}/component/color_picker/`;

test.use({ hasTouch: true, isMobile: true, viewport: { width: 390, height: 844 }, deviceScaleFactor: 3 });

async function openPicker(page: Page) {
  await gotoHydrated(page, PAGE_URL, { timeout: 20 * 60 * 1000 });
  const button = page.getByRole("button", { name: /Color picker/i }).first();
  await button.tap();
  await expect(button).toHaveAttribute("aria-expanded", "true");
  const area = page.locator(".dx-color-picker-area-container");
  const hue = page.locator(".dx-color-picker-slider").first();
  await expect(area).toBeVisible();
  return { button, area, hue, hueThumb: page.getByRole("slider", { name: "Hue" }) };
}

async function tapAt(page: Page, box: { x: number; y: number; width: number; height: number }, fx: number, fy: number) {
  await page.touchscreen.tap(box.x + box.width * fx, box.y + box.height * fy);
}

test("touch taps on the color area change the color, repeatedly", async ({ page }) => {
  const { button, area } = await openPicker(page);
  const box = await measureAtRest(area);
  const seen = new Set<string>([(await button.getAttribute("aria-label"))!]);
  for (const [fx, fy] of [[0.2, 0.2], [0.8, 0.3], [0.5, 0.8], [0.1, 0.9], [0.9, 0.1]]) {
    const before = await button.getAttribute("aria-label");
    await tapAt(page, box, fx, fy);
    await expect(button).not.toHaveAttribute("aria-label", before!);
    seen.add((await button.getAttribute("aria-label"))!);
  }
  expect(seen.size).toBe(6);
});

test("touch taps on the hue slider change the hue, repeatedly", async ({ page }) => {
  const { hue, hueThumb } = await openPicker(page);
  const box = await measureAtRest(hue);
  for (const [f, expected] of [[0.1, 36], [0.5, 180], [0.9, 324]] as const) {
    await tapAt(page, box, f, 0.5);
    await expect(hueThumb).toHaveAttribute("aria-valuenow", String(expected));
  }
});

test("a touch whose release was never seen does not block the next taps", async ({ page }) => {
  const { button, area, hue, hueThumb } = await openPicker(page);
  // Simulate the lost release: swallow the first pointerup / pointercancel in the capture phase of
  // `window`, ahead of the pointer bridge's own window listeners, so the bridge never learns that the
  // finger lifted. The per-control element handlers of the page must still recover on the next touch.
  await page.evaluate(() => {
    let swallowed = 0;
    for (const type of ["pointerup", "pointercancel"]) {
      window.addEventListener(type, (e) => { if (swallowed < 2) { swallowed++; e.stopImmediatePropagation(); } }, true);
    }
  });
  const areaBox = await measureAtRest(area);
  const hueBox = await measureAtRest(hue);

  await tapAt(page, areaBox, 0.2, 0.2);
  const afterFirst = await button.getAttribute("aria-label");

  for (const [fx, fy] of [[0.8, 0.3], [0.5, 0.8]]) {
    const before = await button.getAttribute("aria-label");
    await tapAt(page, areaBox, fx, fy);
    await expect(button).not.toHaveAttribute("aria-label", before!);
  }
  expect(await button.getAttribute("aria-label")).not.toBe(afterFirst);

  const hueBefore = await hueThumb.getAttribute("aria-valuenow");
  await tapAt(page, hueBox, hueBefore === "324" ? 0.1 : 0.9, 0.5);
  await expect(hueThumb).not.toHaveAttribute("aria-valuenow", hueBefore!);
});
