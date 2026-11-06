import { test, expect } from "./fixtures";
import { type Page } from "@playwright/test";
import { BASE_URL } from "./base-url";
import { gotoHydrated } from "./hydration";

// A quick re-grab of a draggable control (the Slider thumb here; Resizable and ColorPicker share the code path) must keep the control under the pointer.
// A mouse's `pointerId` is the same on every press, so a per-move frame of the previous drag that is still on its
// way to Rust when the next drag starts must not be applied to the new drag. `__dxBridgeDelayFrames = 8` delays
// every bridge message (the window pointer frames included) by eight animation frames, the way a busy Rust side
// delays them on a desktop webview; the rsx `pointerdown` is not delayed, so the frames of drag 1 land after the
// `pointerdown` of drag 2.

const GOTO = { timeout: 20 * 60 * 1000 };

async function delayBridge(page: Page, frames: number) {
  await page.evaluate((n) => {
    (window as any).__dxBridgeDelayFrames = n;
  }, frames);
}

async function drag(page: Page, from: { x: number; y: number }, to: { x: number; y: number }) {
  await page.mouse.move(from.x, from.y);
  await page.mouse.down();
  await page.mouse.move(to.x, to.y, { steps: 6 });
  await page.mouse.up();
}

test.describe("a quick re-grab at a delayed bridge", () => {
  test("Slider: the second drag's value follows the pointer, not the first drag's last position", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/slider/`, GOTO);
    const thumb = page.getByRole("slider", { name: "Demo Slider" });
    const track = page
      .getByRole("slider", { name: "Demo Slider" })
      .first()
      .locator('xpath=ancestor::*[@role="group" and @data-orientation="horizontal"][1]')
      .locator('div[data-orientation="horizontal"]:has([role="slider"])')
      .first();
    await expect(thumb).toHaveAttribute("aria-valuenow", "50");
    const box = (await track.boundingBox())!;
    const at = (frac: number) => ({ x: box.x + box.width * frac, y: box.y + box.height / 2 });

    await delayBridge(page, 8);
    await drag(page, at(0.2), at(0.85));
    // Re-grab at once: drag 1's frames are still queued.
    await drag(page, at(0.3), at(0.4));
    await delayBridge(page, 0);
    await expect.poll(async () => Number(await thumb.getAttribute("aria-valuenow")), { timeout: 5000 }).toBeGreaterThan(0);
    await page.waitForTimeout(300);
    const value = Number(await thumb.getAttribute("aria-valuenow"));
    expect(value).toBeGreaterThan(36);
    expect(value).toBeLessThan(44);
  });
});
