import { test, expect } from "./fixtures";
import { expectNoAxeViolations, EXCLUDE_VENDORED_CODE_HIGHLIGHT } from './axe';
import { BASE_URL } from './base-url';
import { gotoHydrated } from "./hydration";

test('test', async ({ page }) => {
  await gotoHydrated(page, `${BASE_URL}/component/switch/`, { timeout: 20 * 60 * 1000 }); // Increase timeout to 20 minutes
  await page.waitForLoadState('networkidle');
  let switchElement = page.locator('#component-preview-frame').getByRole('switch', { name: 'Airplane Mode', exact: true });
  await expect(switchElement).toBeVisible();
  // The switch should not be checked initially
  await expect(switchElement).toHaveAttribute('data-state', 'unchecked');
  // Clicking the switch should check it
  await switchElement.click();
  await expect(switchElement).toHaveAttribute('data-state', 'checked');
  // Pressing space should also toggle the switch.
  // Use locator.press so the element is focused before the keystroke —
  // webkit does not always retain focus on a button after a synthetic click.
  await switchElement.press('Space');
  await expect(switchElement).toHaveAttribute('data-state', 'unchecked');
});

test.describe('Axe automated scan', () => {
  test('loaded (unchecked) has no automatically detectable a11y issues', async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/switch/`, { timeout: 20 * 60 * 1000, waitUntil: 'networkidle' });
    await expectNoAxeViolations(page, 'switch: unchecked', { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });

  test('checked has no automatically detectable a11y issues', async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/switch/`, { timeout: 20 * 60 * 1000, waitUntil: 'networkidle' });
    await page.locator('#component-preview-frame').getByRole('switch', { name: 'Airplane Mode', exact: true }).click();
    await expectNoAxeViolations(page, 'switch: checked', { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });
});

// Backlog row 155 group 7: `size` (sm, default) -- shadcn's `data-[size=default]:h-[18.4px] w-[32px]`,
// `data-[size=sm]:h-[14px] w-[24px]`, thumb `size-4` / `size-3`. The sizes are three slot variables on the
// track, so the thumb travel (`width - 1px - thumb`) follows; both are asserted.
test.describe("size (row 155 group 7)", () => {
  const box = (page: import("@playwright/test").Page, id: string) =>
    page.locator("#component-preview-frame-sizes").locator(`#${id}`).evaluate((el) => {
      const track = getComputedStyle(el);
      const thumb = el.querySelector(".dx-switch-thumb") as HTMLElement;
      const ts = getComputedStyle(thumb);
      return {
        size: el.getAttribute("data-size"),
        w: parseFloat(track.width),
        h: parseFloat(track.height),
        tw: parseFloat(ts.width),
        th: parseFloat(ts.height),
      };
    });

  test("sm is 24 x 14 with a 12px thumb, default is 32 x 18.4 with a 16px thumb", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/switch/`, { timeout: 20 * 60 * 1000, waitUntil: "networkidle" });
    expect(await box(page, "switch-size-sm")).toEqual({ size: "sm", w: 24, h: 14, tw: 12, th: 12 });
    const d = await box(page, "switch-size-default");
    expect(d.size).toBe("default");
    expect([d.w, d.tw, d.th]).toEqual([32, 16, 16]);
    expect(d.h).toBeCloseTo(18.4, 1);
  });

  test("the checked thumb ends flush with the track end in both sizes", async ({ page }) => {
    await gotoHydrated(page, `${BASE_URL}/component/switch/`, { timeout: 20 * 60 * 1000, waitUntil: "networkidle" });
    for (const id of ["switch-size-sm", "switch-size-default"]) {
      const sw = page.locator("#component-preview-frame-sizes").locator(`#${id}`);
      await sw.click();
      await expect(sw).toHaveAttribute("data-state", "checked");
      await expect
        .poll(async () =>
          sw.evaluate((el) => {
            const t = el.getBoundingClientRect();
            const h = (el.querySelector(".dx-switch-thumb") as HTMLElement).getBoundingClientRect();
            return Math.round((t.right - h.right) * 10) / 10;
          }),
        )
        .toBe(1);
    }
  });
});
