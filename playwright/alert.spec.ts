import { test, expect } from "./fixtures";
import { expectNoAxeViolations, EXCLUDE_VENDORED_CODE_HIGHLIGHT } from "./axe";
import { BASE_URL } from "./base-url";

test("test", async ({ page }) => {
  await page.goto(`${BASE_URL}/component/?name=alert&`, { timeout: 20 * 60 * 1000 });
  // Scope to the main demo frame: the alert page also renders the other
  // variants (e.g. no_icon) in their own frames, each with its own role="alert".
  const frame = page.locator("#component-preview-frame");
  const alerts = frame.getByRole("alert");
  await expect(alerts).toHaveCount(2);
  await expect(alerts.first()).toContainText("You can add components to your app");
  await expect(alerts.last()).toContainText("Unable to process your payment.");
});

test.describe("Axe automated scan", () => {
  // Alert has no overlay/expand/select interaction -- one state to scan.
  test("loaded has no automatically detectable a11y issues", async ({ page }) => {
    await page.goto(`${BASE_URL}/component/?name=alert&`, { timeout: 20 * 60 * 1000 });
    await expectNoAxeViolations(page, "alert: loaded", { excludeRegions: [EXCLUDE_VENDORED_CODE_HIGHLIGHT] });
  });
});

// `AlertAction` (shadcn's fourth Alert part): pinned to the top-end corner, the alert keeps room on the end side
// so its text never runs under the button, and the layout mirrors in RTL.
test("AlertAction is pinned top-end and the text stays clear of it", async ({ page }) => {
  await page.goto(`${BASE_URL}/component/?name=alert&`, { timeout: 20 * 60 * 1000 });
  const alert = page.locator(".dx-alert:has(> .dx-alert-action)").first();
  await expect(alert).toBeVisible();
  const action = alert.locator("> .dx-alert-action");
  await expect(action.getByRole("button", { name: "Enable" })).toBeVisible();

  const a = (await alert.boundingBox())!;
  const b = (await action.boundingBox())!;
  const rem = await page.evaluate(() => parseFloat(getComputedStyle(document.documentElement).fontSize));
  // top-2 / right-2 (0.5rem), inside the alert's border box.
  expect(Math.abs(b.y - a.y - 0.5 * rem)).toBeLessThan(2);
  expect(Math.abs(a.x + a.width - (b.x + b.width) - 0.5 * rem)).toBeLessThan(2);

  // The description is clipped to the reserved padding, not to the action.
  const desc = (await alert.locator(".dx-alert-description").boundingBox())!;
  expect(desc.x + desc.width).toBeLessThanOrEqual(b.x + 0.5);
  expect(await alert.evaluate((el) => getComputedStyle(el).paddingInlineEnd)).toBe(`${4.5 * rem}px`);

  // RTL: the action moves to the start (left) corner.
  await alert.evaluate((el) => el.setAttribute("dir", "rtl"));
  const ra = (await alert.boundingBox())!;
  const rb = (await action.boundingBox())!;
  expect(Math.abs(rb.x - ra.x - 0.5 * rem)).toBeLessThan(2);
});
